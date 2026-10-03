//! CR 701.23 Search.
//!
//! * Searching a hidden zone for cards with a stated quality, a player needn't find them
//!   (CR 701.23b); searching for a quantity of cards ("a card"), they must find that many
//!   if they can (CR 701.23d). A quality that's undefined matches no card (CR 701.23c).
//! * Found cards aren't revealed unless the effect says so (CR 701.23e).
//! * "If an opponent would search a library, that player searches the top four cards of
//!   that library instead." (Aven Mindcensor): the search is of that portion, and the
//!   rest of the effect still refers to the library (CR 701.23f).
//! * Searching a library several times before it's shuffled is a single search: it's
//!   reported once (CR 701.23h).
//! * Several players searching at once look at the cards at the same time, choose in
//!   APNAP order, and then the found cards move (CR 701.23i).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// `StaticEffect::Custom` prefix: "search portion:[N]:[who]" — a player searching a
/// library searches only its top N cards (who: "opponents", "you", or "each").
pub const SEARCH_PORTION: &str = "search portion:";

/// Searches of a library in progress: (searcher, library owner, the resolving spell or
/// ability). A search ends when that library is shuffled.
#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub open: Vec<(PlayerId, PlayerId, Option<ObjectId>)>,
}

/// How many cards from the top of `owner`'s library `searcher` searches, if an effect
/// limits the search to a portion of it (CR 701.23f).
pub fn portion(g: &Game, searcher: PlayerId, _owner: PlayerId) -> Option<usize> {
    let mut out: Option<usize> = None;
    for (_, ctl, name) in &g.statics.customs {
        let Some(spec) = name.strip_prefix(SEARCH_PORTION) else {
            continue;
        };
        let Some((k, who)) = spec.split_once(':') else {
            continue;
        };
        let Ok(k) = k.parse::<usize>() else {
            continue;
        };
        let applies = match who {
            "opponents" => g.are_opponents(*ctl, searcher),
            "you" => *ctl == searcher,
            "each" => true,
            _ => false,
        };
        if applies {
            out = Some(out.map_or(k, |o| o.min(k)));
        }
    }
    out
}

/// Whether a search is simply for a quantity of cards ("a card", "three cards"): the
/// filter states no quality (CR 701.23d).
pub fn quantity_only(f: &Filter) -> bool {
    match f {
        Filter::Any | Filter::Card | Filter::InZone(_) | Filter::OwnedBy(_) => true,
        Filter::And(v) => v.iter().all(quantity_only),
        _ => false,
    }
}

/// Records that `searcher` searches `owner`'s library for the resolving spell or ability
/// in `ctx`. Returns false if that's a further search of a library already being searched
/// by it (the same search, CR 701.23h).
pub fn begin(g: &mut Game, searcher: PlayerId, owner: PlayerId, ctx: &Ctx) -> bool {
    let key = (searcher, owner, ctx.stack_obj.or(ctx.source));
    // Only one spell or ability resolves at a time: searches for any other are over.
    g.searches.open.retain(|(_, _, s)| *s == key.2);
    if g.searches.open.contains(&key) {
        return false;
    }
    g.searches.open.push(key);
    true
}

/// A library was shuffled: searches of it are over.
pub fn library_shuffled(g: &mut Game, owner: PlayerId) {
    g.searches.open.retain(|(_, o, _)| *o != owner);
}

/// `Condition::Custom`: the player of the current "for each player" iteration can search
/// libraries. One who can't can't choose to search ("each player may search their
/// library ..."), so they don't shuffle either (CR 701.23, 118.12b).
pub const ITERATED_CAN_SEARCH: &str = "search: the player can search libraries";

/// `Condition::Custom`: "if you search your library this way" — the resolving spell or
/// ability had you search your library, and it hasn't been shuffled since.
pub const YOU_SEARCHED_THIS_WAY: &str = "search: you searched your library this way";

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    if name == YOU_SEARCHED_THIS_WAY {
        let p = ctx.controller;
        let key = ctx.stack_obj.or(ctx.source);
        return Some(g.searches.open.contains(&(p, p, key)));
    }
    (name == ITERATED_CAN_SEARCH).then(|| {
        ctx.iter_player
            .is_none_or(|p| !g.player_restricted(p, |r| matches!(r, Restriction::CantSearch(_))))
    })
}

/// Whether `spec` can be performed by at least one of its searchers ("you may search
/// ..."): one who can't search libraries can search other zones only.
pub fn possible(g: &Game, spec: &SearchSpec, ctx: &Ctx) -> bool {
    if spec.zones.iter().any(|z| *z != ZoneKind::Library) {
        return true;
    }
    let mut searchers = g.eval_players(&spec.who, ctx);
    if searchers.is_empty() {
        searchers.push(ctx.controller);
    }
    searchers
        .into_iter()
        .any(|p| !g.player_restricted(p, |r| matches!(r, Restriction::CantSearch(_))))
}

/// The found cards that a search put somewhere from a hand ("draws a card for each card
/// exiled from their hand this way").
pub const FROM_HAND: Var = vars::USER + 7023;

/// The cards a search found, kept for instructions after others that change "it"
/// ("Search ... for five cards and exile the rest. Put the chosen cards on top ...").
pub const FOUND: Var = vars::USER + 7024;

/// What one searcher found.
struct Found {
    searcher: PlayerId,
    owner: PlayerId,
    cards: Vec<ObjectId>,
    library_searched: bool,
}

/// Performs [`Effect::SearchCards`] (CR 701.23).
pub fn perform(g: &mut Game, spec: &SearchSpec, ctx: &mut Ctx) {
    let mut searchers = g.eval_players(&spec.who, ctx);
    // The players who chose to search ("each opponent may search") or were chosen ("any
    // number of target players") can be no one.
    if searchers.is_empty() && !matches!(spec.who, PlayerRef::Var(_) | PlayerRef::Target(_)) {
        searchers.push(ctx.controller);
    }
    // CR 701.23i: players searching at once look at the cards at the same time and
    // choose in APNAP order (the order players are listed in); then the found cards move.
    let mut founds: Vec<Found> = Vec::new();
    for p in searchers {
        let mut c = ctx.clone();
        c.iter_player = Some(p);
        let owner = g.eval_player(&spec.whose, &c).unwrap_or(p);
        if let Some(f) = search_one(g, spec, p, owner, &c) {
            founds.push(f);
        }
    }
    let searched = !founds.is_empty();
    let mut all: Vec<ObjectId> = Vec::new();
    let mut moves = Vec::new();
    let mut shuffles: Vec<PlayerId> = Vec::new();
    // Found cards that stay in a library that's shuffled are revealed after the shuffle:
    // they aren't part of it (CR 701.24b), so they don't become new objects (CR 701.20d).
    let mut late_reveals: Vec<(PlayerId, Vec<ObjectId>, Ctx)> = Vec::new();
    for f in &founds {
        let mut c = ctx.clone();
        c.iter_player = Some(f.searcher);
        let mut cards = f.cards.clone();
        let stays = spec.shuffle == SearchShuffle::Before
            || (spec.dests.is_empty() && spec.shuffle != SearchShuffle::No);
        // CR 701.23e: revealed only if the effect says so.
        if spec.reveal && !cards.is_empty() {
            if stays && f.library_searched {
                late_reveals.push((f.searcher, cards.clone(), c.clone()));
            } else {
                crate::reveal::reveal_in(g, f.searcher, &cards, Some(&c));
            }
        }
        // A rule that deals with cards found in a library instead ("they exile each card
        // they find"): the rest of the effect still applies.
        let from_library: Vec<ObjectId> = cards
            .iter()
            .copied()
            .filter(|o| g.obj(*o).zone == Zone::Library(f.owner))
            .collect();
        if !from_library.is_empty()
            && crate::kw::search_found(g, f.searcher, f.owner, &from_library)
        {
            cards.retain(|o| !from_library.contains(o));
        }
        let shuffle = f.library_searched && spec.shuffle != SearchShuffle::No;
        if spec.shuffle == SearchShuffle::Before {
            // CR 701.24b, 701.24g: the library is shuffled except the found cards, which
            // are then put where the effect says (they don't change zones).
            if shuffle {
                g.shuffle_library(f.owner);
            }
            place_in_library(g, f.searcher, f.owner, &cards, spec);
            all.extend(cards);
            continue;
        }
        if spec.dests.is_empty() {
            all.extend(cards.iter().copied());
        } else {
            for (objs, to) in split(g, f.searcher, &cards, spec, &c) {
                moves.extend(g.destination_moves(objs, &to, &mut c));
            }
        }
        if shuffle && !shuffles.contains(&f.owner) {
            shuffles.push(f.owner);
        }
    }
    // The found cards are put where they go at the same time.
    let mut from_hand: Vec<Entity> = Vec::new();
    if !moves.is_empty() {
        let in_hand: Vec<bool> = moves
            .iter()
            .map(|m| matches!(g.obj(m.obj).zone, Zone::Hand(_)))
            .collect();
        for (moved, hand) in g.move_objects(moves).into_iter().zip(in_hand) {
            if let Some(o) = moved {
                if hand {
                    from_hand.push(Entity::Object(o));
                }
                all.push(o);
            }
        }
    }
    ctx.set_var(FROM_HAND, from_hand);
    for o in shuffles {
        g.shuffle_library(o);
    }
    for (p, cards, c) in late_reveals {
        let live: Vec<ObjectId> = cards.into_iter().filter(|o| g.is_live(*o)).collect();
        crate::reveal::reveal_in(g, p, &live, Some(&c));
    }
    ctx.prev_affected = all.iter().map(|o| Entity::Object(*o)).collect();
    // CR 118.12b: "if you do" after a search checks whether the player searched.
    ctx.prev_happened = searched;
    ctx.set_var(vars::IT, all.into_iter().map(Entity::Object).collect());
}

/// One searcher's search: `None` if they don't search at all.
fn search_one(
    g: &mut Game,
    spec: &SearchSpec,
    p: PlayerId,
    owner: PlayerId,
    ctx: &Ctx,
) -> Option<Found> {
    let src = ctx.source;
    let cant_search = g.player_restricted(p, |r| matches!(r, Restriction::CantSearch(_)));
    let mut zones: Vec<ZoneKind> = spec.zones.clone();
    if cant_search {
        zones.retain(|z| *z != ZoneKind::Library);
    }
    if zones.is_empty() {
        return None;
    }
    if spec.optional && !g.ask_yes_no(p, src, "Search?", true) {
        return None;
    }
    // "Search your library and/or graveyard": the searcher chooses which zones; searching
    // the library matters (it's shuffled, and searching it can trigger).
    if spec.zones_optional
        && zones.len() > 1
        && zones.contains(&ZoneKind::Library)
        && !g.ask_yes_no(p, src, "Search the library too?", true)
    {
        zones.retain(|z| *z != ZoneKind::Library);
    }
    let library_searched = zones.contains(&ZoneKind::Library);
    let mut cands: Vec<ObjectId> = Vec::new();
    for z in &zones {
        match z {
            ZoneKind::Library => {
                // CR 701.23f: an effect may replace searching the library with searching
                // its top cards.
                let portion = portion(g, p, owner).unwrap_or(usize::MAX);
                cands.extend(g.player(owner).library.iter().rev().take(portion).copied());
            }
            ZoneKind::Graveyard => cands.extend(g.player(owner).graveyard.iter().copied()),
            ZoneKind::Hand => cands.extend(g.player(owner).hand.iter().copied()),
            _ => {}
        }
    }
    let cards = if library_searched && p == owner {
        // "While they're searching their libraries" (CR 723.2).
        crate::player_control::while_searching(g, p, |g| choose(g, spec, p, &cands, ctx))
    } else {
        choose(g, spec, p, &cands, ctx)
    };
    // CR 701.23h: searching a library again before it's shuffled is the same search.
    if library_searched && begin(g, p, owner, ctx) {
        g.emit(crate::events::Event::Searched { player: p });
    }
    Some(Found {
        searcher: p,
        owner,
        cards,
        library_searched,
    })
}

/// Chooses the cards found for each part of the search, in order.
fn choose(
    g: &mut Game,
    spec: &SearchSpec,
    p: PlayerId,
    cands: &[ObjectId],
    ctx: &Ctx,
) -> Vec<ObjectId> {
    let mut chosen: Vec<ObjectId> = Vec::new();
    for part in &spec.parts {
        let n = g.eval_value(&part.count, ctx).max(0) as usize;
        let named_ok = |g: &Game, o: ObjectId, chosen: &[ObjectId]| {
            !spec.distinct_names
                || chosen
                    .iter()
                    .all(|x| !g.obj(o).chars.shares_name_with(&g.obj(*x).chars))
        };
        let pc: Vec<ObjectId> = cands
            .iter()
            .copied()
            .filter(|o| !chosen.contains(o) && g.matches(*o, &part.filter, ctx))
            .filter(|o| named_ok(g, *o, &chosen))
            .collect();
        if pc.is_empty() || n == 0 {
            continue;
        }
        // "All cards with that name": every one in a public zone is found (CR 701.23b
        // lets the searcher leave only those in hidden zones).
        let forced: Vec<ObjectId> = if part.all {
            pc.iter()
                .copied()
                .filter(|o| matches!(g.obj(*o).zone, Zone::Graveyard(_)))
                .collect()
        } else {
            vec![]
        };
        let n = if part.all { pc.len() } else { n.min(pc.len()) };
        // As many as can be found together (with different names, if required).
        let possible = if spec.distinct_names {
            let mut picked: Vec<ObjectId> = Vec::new();
            for o in &pc {
                if named_ok(g, *o, &picked) {
                    picked.push(*o);
                }
            }
            picked.len().min(n)
        } else {
            n
        };
        // CR 701.23b, 701.23d: cards with a stated quality needn't be found; a quantity of
        // cards must be.
        let min = if !part.up_to && quantity_only(&part.filter) {
            possible
        } else {
            forced.len()
        };
        // The forced cards are found without being offered; the searcher chooses among
        // the rest (if any).
        let optional: Vec<ObjectId> = pc.iter().copied().filter(|o| !forced.contains(o)).collect();
        let valid = |g: &Game, v: &[ObjectId]| {
            let mut seen: Vec<ObjectId> = Vec::new();
            v.len() >= min
                && v.len() <= n
                && forced.iter().all(|f| v.contains(f))
                && v.iter().all(|o| {
                    let ok = pc.contains(o) && !seen.contains(o) && named_ok(g, *o, &seen);
                    seen.push(*o);
                    ok
                })
        };
        let picked: Option<Vec<ObjectId>> = if optional.is_empty() || n <= forced.len() {
            Some(forced.clone())
        } else {
            let decision = crate::decision::Decision::ChooseEntities {
                source: ctx.source,
                prompt: "Search: choose cards".into(),
                candidates: optional.iter().map(|c| Entity::Object(*c)).collect(),
                min: min.saturating_sub(forced.len()) as u32,
                max: (n - forced.len()) as u32,
            };
            match g.ask(p, decision) {
                crate::decision::Answer::Entities(v) => {
                    let mut objs: Vec<ObjectId> = forced.clone();
                    objs.extend(v.iter().filter_map(|e| e.object()));
                    (objs.len() == v.len() + forced.len() && valid(g, &objs)).then_some(objs)
                }
                _ => None,
            }
        };
        let picked = picked.unwrap_or_else(|| {
            // Default (or invalid) answers: the forced cards, then (automated agents
            // prefer finding cards) others in order.
            let want = if g.search_finds_by_default { n } else { min };
            let mut v = forced.clone();
            for o in &pc {
                if v.len() >= want.max(forced.len()) {
                    break;
                }
                if !v.contains(o) && named_ok(g, *o, &v) {
                    v.push(*o);
                }
            }
            v
        });
        chosen.extend(picked);
    }
    chosen
}

/// Splits the found cards among the destinations: each takes its count (as many as
/// possible) of the cards, chosen by the searcher, and the last the rest.
fn split(
    g: &mut Game,
    p: PlayerId,
    cards: &[ObjectId],
    spec: &SearchSpec,
    ctx: &Ctx,
) -> Vec<(Vec<ObjectId>, Destination)> {
    let mut rest: Vec<ObjectId> = cards.to_vec();
    let mut out = Vec::new();
    for (i, d) in spec.dests.iter().enumerate() {
        let last = i + 1 == spec.dests.len();
        let take: Vec<ObjectId> = match &d.count {
            Some(n) if !last || rest.len() as i64 > g.eval_value(n, ctx) => {
                let n = (g.eval_value(n, ctx).max(0) as usize).min(rest.len());
                if n == rest.len() {
                    rest.clone()
                } else {
                    g.ask_objects(
                        p,
                        ctx.source,
                        "Choose cards to put there",
                        rest.clone(),
                        n as u32,
                        n as u32,
                    )
                }
            }
            _ => rest.clone(),
        };
        rest.retain(|o| !take.contains(o));
        // "Into your hand or graveyard": the searcher chooses where these go.
        let to = if d.or.is_empty() || take.is_empty() {
            d.to.clone()
        } else {
            let all: Vec<&Destination> = std::iter::once(&d.to).chain(d.or.iter()).collect();
            let labels = all.iter().map(|d| destination_label(d)).collect();
            let i = g.ask_option(p, ctx.source, "Choose where to put the found cards", labels);
            all.get(i).copied().unwrap_or(&d.to).clone()
        };
        out.push((take, to));
    }
    out
}

fn destination_label(d: &Destination) -> String {
    match d.zone {
        ZoneKind::Battlefield => "Put onto the battlefield".into(),
        ZoneKind::Hand => "Put into hand".into(),
        ZoneKind::Graveyard => "Put into graveyard".into(),
        ZoneKind::Exile => "Exile".into(),
        ZoneKind::Library => "Put into library".into(),
        z => format!("Put into {z:?}"),
    }
}

/// "Then shuffle and put those cards on top in any order", "... third from the top": the
/// found cards (still in the library) go to their position (CR 701.24b).
fn place_in_library(
    g: &mut Game,
    p: PlayerId,
    owner: PlayerId,
    cards: &[ObjectId],
    spec: &SearchSpec,
) {
    let cards: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| g.player(owner).library.contains(c))
        .collect();
    if cards.is_empty() {
        return;
    }
    let pos = spec
        .dests
        .first()
        .map_or(LibraryPosition::Top, |d| d.to.position);
    // The searcher chooses the order (the first chosen ends up on top).
    let order = if cards.len() > 1 {
        let names = cards
            .iter()
            .map(|c| g.obj(*c).chars.name.to_string())
            .collect();
        g.ask_order(p, "Order the cards (first on top)", names)
    } else {
        vec![0]
    };
    let ordered: Vec<ObjectId> = order.into_iter().map(|i| cards[i]).collect();
    match pos {
        LibraryPosition::FromTop(n) => {
            let lib = &mut g.players[owner.idx()].library;
            lib.retain(|c| !ordered.contains(c));
            // Nth from the top (0-based), or the bottom if the library is too small.
            let idx = lib.len().checked_sub(n as usize).unwrap_or(0);
            for c in ordered.iter() {
                lib.insert(idx, *c);
            }
        }
        _ => crate::library::put_on_top(g, owner, &ordered),
    }
}

/// `Filter::Custom`: "with a mana ability" (CR 605.1a: an activated ability that could add
/// mana, doesn't target, and isn't a loyalty ability).
pub const HAS_MANA_ABILITY: &str = "search: has a mana ability";
/// `Filter::Custom`: "an Aura card with enchant creature" (CR 702.5).
pub const ENCHANT_CREATURE: &str = "search: has enchant creature";

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
    let c = &g.obj(id).chars;
    match name {
        HAS_MANA_ABILITY => Some(
            c.abilities
                .iter()
                .any(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability)),
        ),
        ENCHANT_CREATURE => Some(
            c.keywords_of(crate::keywords::KeywordKind::Enchant)
                .any(|k| matches!(&k.filter, Some(Filter::Type(CardType::Creature)))),
        ),
        _ => None,
    }
}

/// `Effect::Custom`: "Put the chosen cards on top of your library in any order."
/// (Doomsday): the cards the search found ([`FOUND`]) go on top of their owners'
/// libraries in the order each owner chooses (CR 401.4). Cards already in that library
/// are rearranged, not moved (it isn't a zone change, CR 400.7).
pub const FOUND_ON_TOP_ANY_ORDER: &str = "search: put the found cards on top in any order";

pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    if name != FOUND_ON_TOP_ANY_ORDER {
        return false;
    }
    let objs: Vec<ObjectId> = g
        .resolve_objects(&Sel::Var(FOUND), ctx)
        .into_iter()
        .filter(|o| g.is_live(*o))
        .collect();
    let (in_library, elsewhere): (Vec<ObjectId>, Vec<ObjectId>) = objs
        .into_iter()
        .partition(|o| g.obj(*o).zone == Zone::Library(g.obj(*o).owner));
    let moved = g.move_to_destination(elsewhere, &Destination::library_top(), ctx);
    let all: Vec<ObjectId> = in_library.into_iter().chain(moved).collect();
    let mut owners: Vec<PlayerId> = all.iter().map(|o| g.obj(*o).owner).collect();
    owners.dedup();
    owners.sort_unstable_by_key(|p| p.idx());
    owners.dedup();
    for owner in owners {
        let cards: Vec<ObjectId> = all
            .iter()
            .copied()
            .filter(|o| g.obj(*o).owner == owner && g.player(owner).library.contains(o))
            .collect();
        let names = cards
            .iter()
            .map(|c| g.obj(*c).chars.name.to_string())
            .collect();
        let order = g.ask_order(owner, "Order the cards (first on top)", names);
        let ordered: Vec<ObjectId> = order.into_iter().map(|i| cards[i]).collect();
        crate::library::put_on_top(g, owner, &ordered);
    }
    ctx.set_var(vars::IT, all.into_iter().map(Entity::Object).collect());
    true
}
