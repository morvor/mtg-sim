//! Mulligans (CR 103.5): the London mulligan.

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// Whether a player's first mulligan is free: in a multiplayer game and in any Brawl game
/// (CR 103.5c).
pub fn first_mulligan_free(g: &Game) -> bool {
    g.is_multiplayer() || g.is_brawl()
}

/// How many mulligans count toward the cards put on the bottom and the mulligan limit
/// (CR 103.5c).
fn counted(g: &Game, taken: u32) -> u32 {
    if first_mulligan_free(g) {
        taken.saturating_sub(1)
    } else {
        taken
    }
}

pub fn run_mulligans(g: &mut Game) {
    let mut taken: Vec<u32> = vec![0; g.players.len()];
    let mut kept: Vec<bool> = vec![false; g.players.len()];
    loop {
        // CR 103.5: the starting player declares first, then each other player in turn
        // order (the starting player is the active player, CR 101.4e); with shared team
        // turns, the starting team's players first (CR 103.5d).
        let order = g.pregame_order();
        let mut mulling = Vec::new();
        for p in order {
            if kept[p.idx()] {
                continue;
            }
            // A player can take mulligans until their opening hand would be zero cards.
            if counted(g, taken[p.idx()]) >= g.starting_hand_size(p) {
                kept[p.idx()] = true;
                continue;
            }
            // CR 103.5b: actions a player may take any time they could mulligan.
            could_mulligan_actions(g, p);
            let ans = g.ask(
                p,
                Decision::Mulligan {
                    mulligans_taken: taken[p.idx()],
                },
            );
            if matches!(ans, Answer::Bool(true)) {
                mulling.push(p);
            } else {
                // The remaining cards become that player's opening hand.
                kept[p.idx()] = true;
            }
        }
        if mulling.is_empty() {
            break;
        }
        // All players who decided to take mulligans do so at the same time.
        for p in mulling {
            take_mulligan(g, p, &mut taken[p.idx()]);
        }
    }
    for p in g.player_ids() {
        g.players[p.idx()].mulligans = taken[p.idx()];
    }
    g.dirty = true;
}

/// Takes a mulligan (CR 103.5): shuffle the hand into the library, draw a new hand of the
/// starting hand size, then put a card on the bottom of the library for each mulligan
/// taken (the first one is free in multiplayer and Brawl games, CR 103.5c).
fn take_mulligan(g: &mut Game, p: PlayerId, taken: &mut u32) {
    for c in g.player(p).hand.clone() {
        g.players[p.idx()].hand.retain(|x| *x != c);
        let n = g.create_incarnation(c, Zone::Library(p));
        g.objects[n.0 as usize].zone = Zone::Library(p);
        g.players[p.idx()].library.push(n);
    }
    g.shuffle_library(p);
    for _ in 0..g.starting_hand_size(p) {
        g.draw_card_raw(p);
    }
    *taken += 1;
    let n = counted(g, *taken);
    if n == 0 {
        return;
    }
    let hand = g.player(p).hand.clone();
    let n = n.min(hand.len() as u32);
    let chosen = match g.ask(
        p,
        Decision::PutOnBottom {
            cards: hand.clone(),
            n,
        },
    ) {
        Answer::Entities(v)
            if v.len() as u32 == n
                && v.iter()
                    .all(|e| e.object().is_some_and(|o| hand.contains(&o))) =>
        {
            v.into_iter().filter_map(|e| e.object()).collect::<Vec<_>>()
        }
        _ => hand.iter().copied().take(n as usize).collect(),
    };
    for c in chosen {
        g.players[p.idx()].hand.retain(|x| *x != c);
        let nid = g.create_incarnation(c, Zone::Library(p));
        g.objects[nid.0 as usize].zone = Zone::Library(p);
        g.players[p.idx()].library.insert(0, nid);
    }
}

/// CR 103.5b: "Any time you could mulligan and this card is in your hand, you may ...":
/// offered each time the player is about to declare whether they'll mulligan (at most
/// once per card per declaration).
fn could_mulligan_actions(g: &mut Game, p: PlayerId) {
    g.recompute();
    for card in g.player(p).hand.clone() {
        if !g.player(p).hand.contains(&card) {
            continue;
        }
        let effects: Vec<Effect> = g
            .obj(card)
            .chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Static(s) => match &s.effect {
                    StaticEffect::AnyTimeCouldMulligan(e) => Some((**e).clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for e in effects {
            let name = g.obj(card).chars.name.clone();
            if !g.ask_yes_no(
                p,
                Some(card),
                &format!("Use {name} before mulligans?"),
                false,
            ) {
                continue;
            }
            let mut ctx = Ctx::new(Some(card), p);
            g.exec(&e, &mut ctx);
            g.events.clear();
            g.recompute();
        }
    }
}
