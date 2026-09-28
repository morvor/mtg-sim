//! Shared helpers for the tests of rulings batch S21 (`r_s21_*.rs`): blocking (evasion,
//! block requirements, "becomes blocked" triggers, "blocking creature", combat timing),
//! lands entering at the same time, and split-card mana values. (The helpers of batches
//! S01–S20 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::stack_library;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Whether the defending player `dp` declaring `blocks` would be legal now (CR 509.1a–c),
/// not counting costs.
pub fn legal_blocks(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let options = block_options(&t.g, &[dp]);
    block_declaration_legal(&t.g, &options, blocks)
}

/// The blocks declared in the current combat, as (blocker, attacker) pairs.
pub fn blocks_now(t: &TestGame) -> Vec<(ObjectId, ObjectId)> {
    t.g.combat
        .as_ref()
        .map(|c| {
            c.blockers
                .iter()
                .flat_map(|b| b.blocking.iter().map(move |a| (b.id, *a)))
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `p` could cast `card` now (it's among `p`'s legal actions).
pub fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    let ok =
        t.g.legal_actions(p)
            .iter()
            .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card));
    t.g.turn.priority = saved;
    ok
}

/// Runs the game (everyone passing) until `step` of the current turn begins and the active
/// player has priority.
pub fn go_to(t: &mut TestGame, step: Step) {
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?}");
}

/// P0 casts Genesis Wave ("Reveal the top X cards of your library. You may put any number
/// of permanent cards with mana value X or less revealed this way onto the battlefield.
/// ...") with the real cards `top_first` on top of their library and X equal to their
/// number, putting all of them onto the battlefield at the same time. `yes` answers the
/// yes/no questions asked as they enter, in order. Returns the cards.
pub fn genesis_wave(t: &mut TestGame, top_first: &[&str], yes: &[bool]) -> Vec<ObjectId> {
    let cards = stack_library(t, P0, top_first);
    let wave = t.hand(P0, "Genesis Wave");
    t.lands(P0, "Forest", 3 + top_first.len());
    let chosen: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P0, &chosen);
    for y in yes {
        t.answer_yes(P0, *y);
    }
    t.cast(P0, wave).x(top_first.len() as i64).go();
    t.resolve_all();
    t.clear_answers();
    cards
}

/// Puts the real card `name` onto the battlefield under `p`'s control blocking `attacker`
/// (as "create a token that's blocking target creature" would, CR 509.4).
pub fn enter_blocking(t: &mut TestGame, p: PlayerId, name: &str, attacker: ObjectId) -> ObjectId {
    let id = t.custom(
        p,
        (*mtg_engine::card::card(name)).clone(),
        mtg_engine::object::Zone::Exile,
    );
    let new =
        t.g.move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: mtg_engine::object::Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                blocking: Some(attacker),
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
    t.g.flush_events();
    t.settle();
    new
}
