//! Mulligans (CR 103.5): the London mulligan.

use crate::decision::{Answer, Decision};
use crate::game::{Game, Variant};
use crate::object::Zone;

pub fn run_mulligans(g: &mut Game) {
    let multiplayer = g.players.len() > 2 || matches!(g.config.variant, Variant::Commander);
    let hand_size = g.config.starting_hand_size;
    let mut taken: Vec<u32> = vec![0; g.players.len()];
    let mut kept: Vec<bool> = vec![false; g.players.len()];
    loop {
        // CR 103.5: starting player declares first, then others in turn order.
        let order = g.apnap();
        let mut mulling = Vec::new();
        for p in order {
            if kept[p.idx()] {
                continue;
            }
            let ans = g.ask(
                p,
                Decision::Mulligan {
                    mulligans_taken: taken[p.idx()],
                },
            );
            let mull = matches!(ans, Answer::Bool(true)) && taken[p.idx()] < hand_size + 1;
            if mull {
                mulling.push(p);
            } else {
                kept[p.idx()] = true;
            }
        }
        if mulling.is_empty() {
            break;
        }
        for p in mulling {
            // Shuffle hand into library, draw a new hand.
            for c in g.player(p).hand.clone() {
                g.players[p.idx()].hand.retain(|x| *x != c);
                let n = g.create_incarnation(c, Zone::Library(p));
                g.objects[n.0 as usize].zone = Zone::Library(p);
                g.players[p.idx()].library.push(n);
            }
            g.shuffle_library(p);
            for _ in 0..hand_size {
                g.draw_card_raw(p);
            }
            taken[p.idx()] += 1;
        }
    }
    // Put cards on the bottom (CR 103.5; 103.5c: first mulligan free in multiplayer).
    for p in g.apnap() {
        let mut n = taken[p.idx()];
        if multiplayer && n > 0 {
            n -= 1;
        }
        g.players[p.idx()].mulligans = taken[p.idx()];
        if n == 0 {
            continue;
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
    g.dirty = true;
}
