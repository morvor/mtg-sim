//! Digging through the top of a library (CR 401, 701.20): the steps after cards were
//! looked at, revealed, milled or exiled from the top of a library ([`vars::DUG`]) —
//! choosing among them and putting the chosen cards somewhere ("You may put a creature
//! card and/or a land card from among them into your hand."), putting the rest somewhere
//! ("Put the rest on the bottom of your library in a random order."), and revealing cards
//! until several of a kind are revealed ("Reveal cards from the top of your library until
//! you reveal two land cards."). See [`DigStep`] and `oracle/patterns/dig_grammar.rs`.
//!
//! Cards that stay in a library while they're rearranged don't change zones (CR 400.7),
//! so they keep their objects; cards put into a library position at the same time are
//! arranged by their owner unless the text says "in a random order" (CR 401.4).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// Resolves a dig step.
pub fn resolve(g: &mut Game, step: &DigStep, ctx: &mut Ctx) {
    match step {
        DigStep::Take {
            from,
            chooser,
            filter,
            each_of,
            count,
            up_to,
            random,
            reveal,
            to,
        } => take(
            g,
            from,
            chooser,
            filter,
            each_of,
            count.as_ref(),
            *up_to,
            *random,
            *reveal,
            to,
            ctx,
        ),
        DigStep::Rest { from, to } => {
            // What the text says next is about the cards chosen ("If you didn't put a card
            // into your hand this way"), not the rest.
            let taken = ctx.var_objects(vars::DUG_TAKEN);
            let cards: Vec<ObjectId> = remaining(g, from, ctx)
                .into_iter()
                .filter(|o| !taken.contains(o))
                .collect();
            let placed = place(g, cards, to, ctx);
            // "for each card put into your graveyard this way".
            ctx.nums.insert(vars::DUG, placed.len() as i64);
        }
        DigStep::Until {
            who,
            filter,
            count,
            exile,
        } => {
            let n = g.eval_value(count, ctx).max(0) as usize;
            let (mut all, mut found) = (Vec::new(), Vec::new());
            for p in g.eval_players(who, ctx) {
                let (a, f) = until(g, p, filter, n, *exile, ctx);
                all.extend(a);
                found.extend(f);
            }
            ctx.prev_value = found.len() as i64;
            ctx.prev_happened = !all.is_empty();
            set_dug(ctx, entities(&all));
            ctx.set_var(vars::REVEALED, entities(&all));
            ctx.set_var(vars::DUG_FOUND, entities(&found));
            // "Put those land cards onto the battlefield."
            ctx.set_var(vars::DUG_CHOSEN, entities(&found));
            ctx.set_var(vars::IT, entities(&found));
        }
    }
}

/// Records the cards a dig looked at, revealed, milled or exiled ([`vars::DUG`]); none of
/// them has been taken yet.
pub fn set_dug(ctx: &mut Ctx, cards: Vec<Entity>) {
    ctx.set_var(vars::DUG, cards);
    ctx.set_var(vars::DUG_TAKEN, vec![]);
}

fn entities(v: &[ObjectId]) -> Vec<Entity> {
    v.iter().map(|o| Entity::Object(*o)).collect()
}

/// The cards of `from` still where the dig left them (a card that moved is a new
/// object, CR 400.7).
fn remaining(g: &Game, from: &Sel, ctx: &Ctx) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = Vec::new();
    for o in g.eval_sel_objects(from, ctx) {
        if g.is_live(o) && !out.contains(&o) {
            out.push(o);
        }
    }
    out
}

/// Whether `to` leaves the cards where they are.
pub fn in_place(to: &Destination) -> bool {
    to.zone == ZoneKind::Library && matches!(to.position, LibraryPosition::FromTop(0))
}

/// Puts `cards` `to`; returns the objects they are afterwards. Cards already in their
/// owner's library that go to a position in it are rearranged there (no zone change).
fn place(g: &mut Game, cards: Vec<ObjectId>, to: &Destination, ctx: &mut Ctx) -> Vec<ObjectId> {
    if cards.is_empty() || in_place(to) {
        return cards;
    }
    if to.zone != ZoneKind::Library || !to.position_choice.is_empty() {
        return g.move_to_destination(cards, to, ctx);
    }
    let (in_library, elsewhere): (Vec<ObjectId>, Vec<ObjectId>) = cards
        .into_iter()
        .partition(|o| g.obj(*o).zone == Zone::Library(g.obj(*o).owner));
    let mut out = g.move_to_destination(elsewhere, to, ctx);
    let mut owners: Vec<PlayerId> = in_library.iter().map(|o| g.obj(*o).owner).collect();
    owners.sort_unstable_by_key(|p| p.idx());
    owners.dedup();
    for owner in owners {
        let mine: Vec<ObjectId> = in_library
            .iter()
            .copied()
            .filter(|o| g.obj(*o).owner == owner)
            .collect();
        out.extend(mine.iter().copied());
        rearrange(g, owner, mine, to.position, ctx.controller);
    }
    out
}

/// Puts cards that are in `owner`'s library at `pos` in it, in the order `orderer` (the
/// player the text instructs to put them there, as Sealed Fate's ruling says) chooses, or
/// in a random order.
fn rearrange(
    g: &mut Game,
    owner: PlayerId,
    cards: Vec<ObjectId>,
    pos: LibraryPosition,
    orderer: PlayerId,
) {
    let ask = |g: &mut Game, prompt: &str| -> Vec<ObjectId> {
        if cards.len() < 2 {
            return cards.clone();
        }
        let names = cards.iter().map(|c| g.describe(*c)).collect();
        g.ask_order(orderer, prompt, names)
            .into_iter()
            .map(|i| cards[i])
            .collect()
    };
    match pos {
        LibraryPosition::Top => {
            let order = ask(g, "Order the cards to put on top (top first)");
            crate::library::put_on_top(g, owner, &order);
        }
        LibraryPosition::Bottom => {
            // Listed top first; the last one ends up at the very bottom.
            let mut order = ask(g, "Order the cards to put on the bottom (top first)");
            order.reverse();
            crate::library::put_on_bottom(g, owner, &order);
        }
        LibraryPosition::BottomRandom => {
            use rand::seq::SliceRandom;
            let mut order = cards.clone();
            order.shuffle(&mut g.rng);
            crate::library::put_on_bottom(g, owner, &order);
        }
        LibraryPosition::FromTop(_) => {}
        LibraryPosition::Shuffled => g.shuffle_library(owner),
    }
}

/// Whether each of `cards` can stand for a different one of `each_of` (a matching).
fn one_each(g: &Game, cards: &[ObjectId], each_of: &[Filter], ctx: &Ctx) -> bool {
    fn assign(
        k: usize,
        fits: &[Vec<usize>],
        owner: &mut Vec<Option<usize>>,
        seen: &mut Vec<bool>,
    ) -> bool {
        for &d in &fits[k] {
            if seen[d] {
                continue;
            }
            seen[d] = true;
            if owner[d].is_none_or(|other| assign(other, fits, owner, seen)) {
                owner[d] = Some(k);
                return true;
            }
        }
        false
    }
    if cards.len() > each_of.len() {
        return false;
    }
    let fits: Vec<Vec<usize>> = cards
        .iter()
        .map(|c| {
            (0..each_of.len())
                .filter(|d| g.matches(*c, &each_of[*d], ctx))
                .collect()
        })
        .collect();
    let mut owner = vec![None; each_of.len()];
    (0..cards.len()).all(|k| assign(k, &fits, &mut owner, &mut vec![false; each_of.len()]))
}

#[allow(clippy::too_many_arguments)]
fn take(
    g: &mut Game,
    from: &Sel,
    chooser: &PlayerRef,
    filter: &Filter,
    each_of: &[Filter],
    count: Option<&Value>,
    up_to: bool,
    random: bool,
    reveal: bool,
    to: &Destination,
    ctx: &mut Ctx,
) {
    let p = g.eval_player(chooser, ctx).unwrap_or(ctx.controller);
    // Cards an earlier selection took (and left in the library) aren't among them any
    // more, unless the selection is of exactly those ("put the revealed cards into your
    // hand").
    let taken = if matches!(from, Sel::Var(vars::DUG)) {
        ctx.var_objects(vars::DUG_TAKEN)
    } else {
        vec![]
    };
    let cands: Vec<ObjectId> = remaining(g, from, ctx)
        .into_iter()
        .filter(|o| !taken.contains(o))
        .filter(|o| {
            g.matches(*o, filter, ctx)
                && (each_of.is_empty() || each_of.iter().any(|f| g.matches(*o, f, ctx)))
        })
        .collect();
    let chosen: Vec<ObjectId> = match count {
        None => cands.clone(),
        Some(n) => {
            let mut n = g.eval_value(n, ctx).max(0) as usize;
            if !each_of.is_empty() {
                n = n.min(each_of.len());
            }
            let n = n.min(cands.len());
            if random {
                use rand::seq::SliceRandom;
                let mut v = cands.clone();
                v.shuffle(&mut g.rng);
                v.truncate(n);
                v
            } else {
                // Without "up to"/"may", as many as possible must be chosen; an "and/or"
                // list asks for one of each description that can be found.
                let min = if up_to {
                    0
                } else if each_of.is_empty() {
                    n
                } else {
                    max_matching(g, &cands, each_of, ctx)
                };
                let picked = crate::target_groups::choose_together(
                    g,
                    p,
                    ctx.source,
                    "Choose cards",
                    filter,
                    cands.clone(),
                    min as u32,
                    n as u32,
                    ctx,
                );
                fit_each(g, picked, &cands, each_of, min, ctx)
            }
        }
    };
    if reveal {
        if let Some(c) = chosen.first() {
            let owner = g.obj(*c).owner;
            crate::reveal::reveal_in(g, owner, &chosen, Some(ctx));
        }
        // "If an instant or sorcery card is revealed this way" (CR 701.20a).
        ctx.set_var(vars::REVEALED, entities(&chosen));
    }
    ctx.set_var(vars::DUG_CHOSEN, entities(&chosen));
    let n = chosen.len();
    let placed = place(g, chosen, to, ctx);
    // Not part of "the rest" (`DigStep::Rest`), even if they stay in the library.
    let mut taken = ctx.vars.get(&vars::DUG_TAKEN).cloned().unwrap_or_default();
    taken.extend(entities(&placed));
    ctx.set_var(vars::DUG_TAKEN, taken);
    ctx.prev_value = n as i64;
    ctx.prev_happened = n > 0;
    ctx.prev_affected = entities(&placed);
    // "for each card you put into your hand this way".
    ctx.nums.insert(vars::DUG_CHOSEN, n as i64);
    ctx.set_var(vars::IT, entities(&placed));
}

/// The most cards of `cands` that can stand for different descriptions.
fn max_matching(g: &Game, cands: &[ObjectId], each_of: &[Filter], ctx: &Ctx) -> usize {
    let mut kept: Vec<ObjectId> = Vec::new();
    // Greedy with augmenting paths: adding a card keeps a matching if one exists.
    for c in cands {
        kept.push(*c);
        if !one_each(g, &kept, each_of, ctx) {
            kept.pop();
        }
    }
    kept.len()
}

/// Keeps the chosen cards that together stand for different descriptions; completed to
/// `min` from the candidates if needed.
fn fit_each(
    g: &Game,
    picked: Vec<ObjectId>,
    cands: &[ObjectId],
    each_of: &[Filter],
    min: usize,
    ctx: &Ctx,
) -> Vec<ObjectId> {
    if each_of.is_empty() {
        return picked;
    }
    let mut kept: Vec<ObjectId> = Vec::new();
    for c in picked.iter().chain(cands.iter()) {
        if kept.contains(c) || (kept.len() >= min && !picked.contains(c)) {
            continue;
        }
        kept.push(*c);
        if !one_each(g, &kept, each_of, ctx) {
            kept.pop();
        }
    }
    kept
}

/// Reveals (or exiles) cards from the top of `p`'s library until `n` cards matching
/// `filter` are revealed or the library runs out. Returns all the cards (the objects they
/// are afterwards) and the matching ones.
fn until(
    g: &mut Game,
    p: PlayerId,
    filter: &Filter,
    n: usize,
    exile: bool,
    ctx: &mut Ctx,
) -> (Vec<ObjectId>, Vec<ObjectId>) {
    let lib: Vec<ObjectId> = g.player(p).library.iter().rev().copied().collect();
    let mut revealed = Vec::new();
    let mut found = Vec::new();
    if n > 0 {
        for c in lib {
            revealed.push(c);
            if g.matches(c, filter, ctx) {
                found.push(c);
                if found.len() >= n {
                    break;
                }
            }
        }
    }
    if !exile {
        crate::reveal::reveal_in(g, p, &revealed, Some(ctx));
        return (revealed, found);
    }
    g.move_to_destination(revealed.clone(), &Destination::zone(ZoneKind::Exile), ctx);
    // The objects the cards became (a card a replacement effect put elsewhere is followed
    // there).
    let all: Vec<ObjectId> = revealed.iter().map(|o| g.current(*o)).collect();
    let hits: Vec<ObjectId> = found.iter().map(|o| g.current(*o)).collect();
    (all, hits)
}
