//! Library manipulation: scry (CR 701.22), surveil (701.25), searching (701.23),
//! looking at the top N cards, and revealing until.

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::object::*;
use crate::replacement::*;
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

/// Reorders the library so `top_first` are on top in that order.
fn set_top(g: &mut Game, p: PlayerId, top_first: &[ObjectId]) {
    let lib = &mut g.players[p.idx()].library;
    lib.retain(|c| !top_first.contains(c));
    for c in top_first.iter().rev() {
        lib.push(*c);
    }
}

fn to_bottom(g: &mut Game, p: PlayerId, cards: &[ObjectId]) {
    let lib = &mut g.players[p.idx()].library;
    lib.retain(|c| !cards.contains(c));
    for (i, c) in cards.iter().enumerate() {
        lib.insert(i, *c);
    }
}

/// CR 701.22a: look at the top N cards, put any number on the bottom in any order and
/// the rest on top in any order.
pub fn scry(g: &mut Game, p: PlayerId, n: u32) {
    if n == 0 {
        return;
    }
    let cards = top_cards(g, p, n);
    if cards.is_empty() {
        return;
    }
    let (top, bottom) = match g.ask(
        p,
        Decision::Scry {
            cards: cards.clone(),
        },
    ) {
        Answer::Split(t, b) if split_ok(&cards, &t, &b) => (t, b),
        _ => (cards.clone(), vec![]),
    };
    set_top(g, p, &top);
    to_bottom(g, p, &bottom);
    g.emit(Event::Custom {
        name: "scry".into(),
        player: Some(p),
        obj: None,
        amount: n as i32,
    });
}

/// CR 701.25a: look at the top N cards, put any number into the graveyard and the rest
/// on top in any order.
pub fn surveil(g: &mut Game, p: PlayerId, n: u32) {
    if n == 0 {
        return;
    }
    let cards = top_cards(g, p, n);
    if cards.is_empty() {
        return;
    }
    let (top, gy) = match g.ask(
        p,
        Decision::Surveil {
            cards: cards.clone(),
        },
    ) {
        Answer::Split(t, b) if split_ok(&cards, &t, &b) => (t, b),
        _ => (cards.clone(), vec![]),
    };
    set_top(g, p, &top);
    let moves = gy
        .iter()
        .map(|c| MoveEv {
            obj: *c,
            to: Zone::Graveyard(p),
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo::default(),
            source: None,
        })
        .collect();
    g.move_objects(moves);
    g.emit(Event::Custom {
        name: "surveil".into(),
        player: Some(p),
        obj: None,
        amount: n as i32,
    });
}

fn split_ok(all: &[ObjectId], a: &[ObjectId], b: &[ObjectId]) -> bool {
    let mut v: Vec<ObjectId> = a.iter().chain(b.iter()).copied().collect();
    v.sort();
    let mut w = all.to_vec();
    w.sort();
    v == w
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
    let cands: Vec<ObjectId> = g
        .player(owner)
        .library
        .iter()
        .rev()
        .copied()
        .filter(|c| g.matches(*c, filter, ctx))
        .collect();
    let n = n.min(cands.len() as u32);
    let found = g.ask_objects(searcher, ctx.source, "Search: choose cards", cands, 0, n);
    // Default answers choose none; for automated agents prefer finding cards.
    let found = if found.is_empty() && n > 0 && g.search_finds_by_default {
        let cands: Vec<ObjectId> = g
            .player(owner)
            .library
            .iter()
            .rev()
            .copied()
            .filter(|c| g.matches(*c, filter, ctx))
            .collect();
        cands.into_iter().take(n as usize).collect()
    } else {
        found
    };
    g.emit(Event::Searched { player: searcher });
    found
}

/// Look at (or reveal) the top N cards, take some matching `filter`, put the rest
/// somewhere else.
#[allow(clippy::too_many_arguments)]
pub fn dig(
    g: &mut Game,
    p: PlayerId,
    n: u32,
    _reveal: bool,
    filter: &Filter,
    take: u32,
    up_to: bool,
    take_to: &Destination,
    rest_to: &Destination,
    ctx: &mut Ctx,
) {
    let cards = top_cards(g, p, n);
    let cands: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| g.matches(*c, filter, ctx))
        .collect();
    let k = take.min(cands.len() as u32);
    let min = if up_to { 0 } else { k };
    let taken = g.ask_objects(p, ctx.source, "Choose cards to take", cands, min, k);
    let rest: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| !taken.contains(c))
        .collect();
    let moved = g.move_to_destination(taken, take_to, ctx);
    ctx.set_var(vars::IT, moved.iter().map(|o| Entity::Object(*o)).collect());
    ctx.prev_affected = moved.iter().map(|o| Entity::Object(*o)).collect();
    if rest_to.zone == ZoneKind::Library {
        match rest_to.position {
            LibraryPosition::Bottom => to_bottom(g, p, &rest),
            LibraryPosition::Shuffled => g.shuffle_library(p),
            _ => set_top(g, p, &rest),
        }
    } else {
        g.move_to_destination(rest, rest_to, ctx);
    }
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
        let moved = g.move_to_destination(vec![f], found_to, ctx);
        ctx.set_var(vars::IT, moved.iter().map(|o| Entity::Object(*o)).collect());
    }
    if rest_to.zone == ZoneKind::Library {
        match rest_to.position {
            LibraryPosition::Bottom => to_bottom(g, p, &revealed),
            LibraryPosition::Shuffled => g.shuffle_library(p),
            _ => set_top(g, p, &revealed),
        }
    } else {
        g.move_to_destination(revealed, rest_to, ctx);
    }
}
