//! Library manipulation: scry (CR 701.22), surveil (701.25), searching (701.23),
//! looking at the top N cards, and revealing until.

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::types::*;

/// Top `n` cards of a library, top first.
pub fn top_cards(g: &Game, p: PlayerId, n: u32) -> Vec<ObjectId> {
    g.player(p)
        .library
        .iter()
        .rev()
        .take(n as usize)
        .copied()
        .collect()
}

/// Puts the cards named in [`crate::game::GameConfig::top_of_library`] on top of each
/// player's library (top first), after the starting shuffle. A name not found in the
/// library is looked for among the player's face-down cards in the command zone — their
/// supplementary decks (planar, scheme and Attraction decks, CR 901.4, 904.4, 717.2),
/// which run in command-zone order — and that card is put on top of its deck.
pub fn stack_starting_libraries(g: &mut Game) {
    for (i, names) in g.config.top_of_library.clone().iter().enumerate() {
        let p = PlayerId(i as u8);
        if i >= g.players.len() {
            break;
        }
        let named = |g: &Game, c: ObjectId, name: &str| {
            g.obj(c)
                .card
                .as_ref()
                .is_some_and(|d| d.name.eq_ignore_ascii_case(name))
        };
        let mut chosen: Vec<ObjectId> = Vec::new();
        let mut supplementary: Vec<ObjectId> = Vec::new();
        for name in names {
            let found = g
                .player(p)
                .library
                .iter()
                .rev()
                .copied()
                .find(|c| !chosen.contains(c) && named(g, *c, name));
            if let Some(c) = found {
                chosen.push(c);
                continue;
            }
            let found = g.command.iter().copied().find(|c| {
                let o = g.obj(*c);
                o.face_down && o.owner == p && !supplementary.contains(c) && named(g, *c, name)
            });
            supplementary.extend(found);
        }
        set_top(g, p, &chosen);
        g.command.retain(|c| !supplementary.contains(c));
        for (k, c) in supplementary.into_iter().enumerate() {
            g.command.insert(k, c);
        }
    }
}

/// Reorders the library so `top_first` are on top in that order.
fn set_top(g: &mut Game, p: PlayerId, top_first: &[ObjectId]) {
    let lib = &mut g.players[p.idx()].library;
    lib.retain(|c| !top_first.contains(c));
    for c in top_first.iter().rev() {
        lib.push(*c);
    }
}

/// Puts cards already in `p`'s library on top of it, in the given order (first on top).
/// Cards not in that library are ignored.
pub fn put_on_top(g: &mut Game, p: PlayerId, top_first: &[ObjectId]) {
    let cards: Vec<ObjectId> = top_first
        .iter()
        .copied()
        .filter(|c| g.player(p).library.contains(c))
        .collect();
    set_top(g, p, &cards);
}

fn to_bottom(g: &mut Game, p: PlayerId, cards: &[ObjectId]) {
    let lib = &mut g.players[p.idx()].library;
    lib.retain(|c| !cards.contains(c));
    for (i, c) in cards.iter().enumerate() {
        lib.insert(i, *c);
    }
}

/// CR 701.22a: look at the top N cards, put any number on the bottom in any order and
/// the rest on top in any order. See [`crate::scry_rules`].
pub fn scry(g: &mut Game, p: PlayerId, n: u32) {
    crate::scry_rules::perform(g, &[p], n, crate::scry_rules::Look::Scry, None);
}

/// CR 701.25a: look at the top N cards, put any number into the graveyard and the rest
/// on top in any order. See [`crate::scry_rules`].
pub fn surveil(g: &mut Game, p: PlayerId, n: u32) {
    crate::scry_rules::perform(g, &[p], n, crate::scry_rules::Look::Surveil, None);
}

/// Puts cards already in `p`'s library on the bottom of it, the first one lowest. Cards
/// not in that library are ignored.
pub fn put_on_bottom(g: &mut Game, p: PlayerId, cards: &[ObjectId]) {
    let cards: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| g.player(p).library.contains(c))
        .collect();
    to_bottom(g, p, &cards);
}

/// Searches `owner`'s library for up to `n` cards matching `filter` (CR 701.23). The
/// searcher may fail to find cards in a hidden zone (CR 701.23b... 'find' is optional).
pub fn search(
    g: &mut Game,
    searcher: PlayerId,
    owner: PlayerId,
    filter: &Filter,
    n: u32,
    ctx: &Ctx,
) -> Vec<ObjectId> {
    if g.player_restricted(searcher, |r| matches!(r, Restriction::CantSearch(_))) {
        return vec![];
    }
    // CR 701.23f: an effect may replace searching the library with searching its top
    // cards.
    let portion = crate::search_rules::portion(g, searcher, owner).unwrap_or(usize::MAX);
    let cands: Vec<ObjectId> = g
        .player(owner)
        .library
        .iter()
        .rev()
        .take(portion)
        .copied()
        .filter(|c| g.matches(*c, filter, ctx))
        .collect();
    let n = n.min(cands.len() as u32);
    // CR 701.23b, 701.23d: cards with a stated quality needn't be found; a quantity of
    // cards must be.
    let min = if crate::search_rules::quantity_only(filter) {
        n
    } else {
        0
    };
    let found: Vec<ObjectId> = if cands.is_empty() || n == 0 {
        vec![]
    } else {
        let decision = Decision::ChooseEntities {
            source: ctx.source,
            prompt: "Search: choose cards".into(),
            candidates: cands.iter().map(|c| Entity::Object(*c)).collect(),
            min,
            max: n,
        };
        // "While they're searching their libraries" (CR 723.2).
        let ans = if searcher == owner {
            crate::player_control::while_searching(g, searcher, |g| g.ask(searcher, decision))
        } else {
            g.ask(searcher, decision)
        };
        let chosen: Option<Vec<ObjectId>> = match ans {
            Answer::Entities(v) => {
                let objs: Vec<ObjectId> = v.iter().filter_map(|e| e.object()).collect();
                let mut uniq = objs.clone();
                uniq.sort();
                uniq.dedup();
                (objs.len() == v.len()
                    && uniq.len() == objs.len()
                    && objs.len() as u32 >= min
                    && objs.len() as u32 <= n
                    && objs.iter().all(|o| cands.contains(o)))
                .then_some(objs)
            }
            _ => None,
        };
        let found = match chosen {
            Some(v) => v,
            // Default (or invalid) answers: automated agents prefer finding cards.
            None if g.search_finds_by_default => cands.iter().copied().take(n as usize).collect(),
            None => cands.iter().copied().take(min as usize).collect(),
        };
        // "Up to three artifact cards with different names": the cards found must have
        // the relationship (see `target_groups::fit_together`).
        crate::target_groups::fit_together(g, filter, found, &cands, min as usize, ctx)
    };
    // CR 701.23h: searching a library again before it's shuffled is the same search.
    if crate::search_rules::begin(g, searcher, owner, ctx) {
        g.emit(Event::Searched { player: searcher });
    }
    found
}

/// Look at (or reveal) the top N cards, take some matching `filter`, put the rest
/// somewhere else.
#[allow(clippy::too_many_arguments)]
pub fn dig(
    g: &mut Game,
    p: PlayerId,
    n: u32,
    reveal: bool,
    filter: &Filter,
    take: u32,
    up_to: bool,
    take_to: &Destination,
    rest_to: &Destination,
    ctx: &mut Ctx,
) {
    let cards = top_cards(g, p, n);
    if reveal {
        // CR 701.20a: revealed while the effect needs them.
        crate::reveal::reveal_in(g, p, &cards, Some(ctx));
    }
    ctx.set_var(
        vars::REVEALED,
        cards.iter().map(|o| Entity::Object(*o)).collect(),
    );
    // "From among them", "the rest" (`dig_steps.rs`).
    crate::dig_steps::set_dug(ctx, cards.iter().map(|o| Entity::Object(*o)).collect());
    let cands: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| g.matches(*c, filter, ctx))
        .collect();
    let k = take.min(cands.len() as u32);
    let min = if up_to { 0 } else { k };
    // ("For each card type, ... a card of that type": the cards taken must be chosen
    // together, see `target_groups::choose_together`.) They're chosen by the player
    // performing the instruction, whose library it may not be ("look at the top four
    // cards of target opponent's library, exile one of them").
    let taken = crate::target_groups::choose_together(
        g,
        ctx.controller,
        ctx.source,
        "Choose cards to take",
        filter,
        cands,
        min,
        k,
        ctx,
    );
    let rest: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| !taken.contains(c))
        .collect();
    let moved = g.move_to_destination(taken, take_to, ctx);
    // "Look at the top four cards ..., exile one of them face down".
    crate::zones::looked_then_exiled(g, ctx.controller, &moved);
    ctx.set_var(vars::IT, moved.iter().map(|o| Entity::Object(*o)).collect());
    ctx.prev_affected = moved.iter().map(|o| Entity::Object(*o)).collect();
    if take == 0 {
        // "Look at the top N cards": the looked-at cards are "them" for what follows.
        ctx.set_var(vars::IT, cards.iter().map(|o| Entity::Object(*o)).collect());
    }
    place_rest(g, p, rest, rest_to, ctx);
}

/// Puts the cards left over from looking at or revealing cards from the top of `p`'s
/// library where `rest_to` says. In the library: `Top` — back on top in the order `p`
/// chooses; `FromTop(_)` — left where they are; `Bottom` — on the bottom in the order `p`
/// chooses; `BottomRandom` — on the bottom in a random order; `Shuffled` — shuffled in.
pub(crate) fn place_rest(
    g: &mut Game,
    p: PlayerId,
    rest: Vec<ObjectId>,
    rest_to: &Destination,
    ctx: &mut Ctx,
) {
    if rest_to.zone != ZoneKind::Library {
        g.move_to_destination(rest, rest_to, ctx);
        return;
    }
    match rest_to.position {
        LibraryPosition::Top => {
            let order = choose_order(g, p, &rest, "Order the cards to put on top (top first)");
            set_top(g, p, &order);
        }
        LibraryPosition::FromTop(_) => {}
        LibraryPosition::Bottom => {
            // Listed top first; the last one ends up at the very bottom.
            let mut order = choose_order(
                g,
                p,
                &rest,
                "Order the cards to put on the bottom (top first)",
            );
            order.reverse();
            to_bottom(g, p, &order);
        }
        LibraryPosition::BottomRandom => {
            use rand::seq::SliceRandom;
            let mut order = rest;
            order.shuffle(&mut g.rng);
            to_bottom(g, p, &order);
        }
        LibraryPosition::Shuffled => g.shuffle_library(p),
    }
}

/// Asks `p` to order `cards` (known to them); returns them in the chosen order.
fn choose_order(g: &mut Game, p: PlayerId, cards: &[ObjectId], prompt: &str) -> Vec<ObjectId> {
    let names = cards
        .iter()
        .map(|c| g.obj(*c).chars.name.to_string())
        .collect();
    g.ask_order(p, prompt, names)
        .into_iter()
        .map(|i| cards[i])
        .collect()
}

/// Reveal cards from the top until one matches; that card goes to `found_to`, the rest
/// to `rest_to`.
pub fn reveal_until(
    g: &mut Game,
    p: PlayerId,
    filter: &Filter,
    found_to: &Destination,
    rest_to: &Destination,
    ctx: &mut Ctx,
) {
    let lib: Vec<ObjectId> = g.player(p).library.iter().rev().copied().collect();
    let mut revealed = Vec::new();
    let mut found = None;
    for c in lib {
        if g.matches(c, filter, ctx) {
            found = Some(c);
            break;
        }
        revealed.push(c);
    }
    if let Some(f) = found {
        // A library position "from the top" leaves the card where it is: a later
        // instruction says what happens to "that card".
        let in_place = found_to.zone == ZoneKind::Library
            && matches!(found_to.position, LibraryPosition::FromTop(_));
        let moved = if in_place {
            vec![f]
        } else {
            g.move_to_destination(vec![f], found_to, ctx)
        };
        ctx.set_var(vars::IT, moved.iter().map(|o| Entity::Object(*o)).collect());
    } else {
        // No card was found: "that card" doesn't exist.
        ctx.set_var(vars::IT, vec![]);
    }
    let mut dug: Vec<Entity> = ctx.vars.get(&vars::IT).cloned().unwrap_or_default();
    ctx.set_var(vars::DUG_FOUND, dug.clone());
    if rest_to.zone == ZoneKind::Library {
        dug.extend(revealed.iter().map(|o| Entity::Object(*o)));
        place_rest(g, p, revealed, rest_to, ctx);
    } else {
        // A later instruction can find the other cards ("put the rest on the bottom of
        // your library in a random order" after exiling them).
        let moved = g.move_to_destination(revealed, rest_to, ctx);
        dug.extend(moved.iter().map(|o| Entity::Object(*o)));
        ctx.set_var(
            vars::REVEALED,
            moved.into_iter().map(Entity::Object).collect(),
        );
    }
    // All the cards revealed, for "the rest" (`dig_steps.rs`).
    crate::dig_steps::set_dug(ctx, dug);
}
