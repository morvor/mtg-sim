//! Shared helpers for the tests of rulings batch P100 (`r_p100_*.rs`): abilities that
//! care about mana values (CR 202.3): X as 0 off the stack and as chosen on it, tokens and
//! copies, free casts, cost reductions, and the triggers, layers and targets around them.
//! (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p003_common::{activate_resolve, cast, cast_resolve, damage, dmg, kill, mana, obj, pl, put};
pub use crate::r_p046_common::{cost_to_cast, others_asked_since, pool, tap_lands};
pub use crate::r_p108_common::{end_step, resolved, two_headed_giant};
pub use crate::r_p117_common::{exile, gain_control, modify};
pub use crate::r_p151_common::act;
pub use crate::r_p152_common::{bounce, count_named, sacrifice, set_life};
pub use crate::r_p153_common::{answer_x, counter, mana_for, mv, set_library, trigger_sources, x_asked};
pub use crate::r_p154_common::{
    asked_of, endless_one, entity_candidates, last_offered_names, mana_value, target_candidate_names,
    was_offered,
};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, give_mana_for, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_cast, create_token, destroy, target_candidates};
pub use crate::r_s04_common::{next_upkeep, stack_items};
pub use crate::r_s06_common::{activate_containing, give_control, has_kw};
pub use crate::r_s25_common::{cast_new, targets_of, x_of};

/// A real permanent on `p`'s battlefield with `n` +1/+1 counters (as one cast with X = n
/// would have; its mana value doesn't change).
pub fn with_counters(t: &mut TestGame, p: PlayerId, name: &str, n: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    put(t, id, counters::PLUS1, n);
    id
}

/// A Walking Ballista ({X}{X}) with `n` +1/+1 counters on `p`'s battlefield: an artifact
/// creature whose X was n as it was cast and is 0 now (mana value 0).
pub fn ballista(t: &mut TestGame, p: PlayerId, n: u32) -> ObjectId {
    with_counters(t, p, "Walking Ballista", n)
}

/// `p` casts the real card `name` from their hand with the given X and targets, paying
/// from a pool filled for it (the spell stays on the stack); the spell's id.
pub fn cast_x(t: &mut TestGame, p: PlayerId, name: &str, x: i64, targets: &[Entity]) -> ObjectId {
    let c = t.hand(p, name);
    mana_for(t, p, name);
    let xs = format!("{}", mtg_engine::card::card(name).front().chars.mana_cost.clone().unwrap_or_default())
        .matches("{X}")
        .count() as u32;
    t.g.players[p.idx()].mana_pool.add_type(mtg_engine::mana::ManaType::C, x as u32 * xs);
    t.g.turn.priority = Some(p);
    t.cast(p, c).x(x).targets(targets).go()
}

/// As [`cast_x`], then everything resolves.
pub fn cast_x_resolve(t: &mut TestGame, p: PlayerId, name: &str, x: i64, targets: &[Entity]) {
    cast_x(t, p, name, x, targets);
    t.resolve_all();
}

/// Queues `p`'s next number answer.
pub fn answer_number(t: &mut TestGame, p: PlayerId, n: i64) {
    t.answer(p, DecisionKind::Number, Answer::Number(n));
}

/// Whether the object (followed across zone changes) has protection from `source` now
/// (CR 702.16).
pub fn protected_from(t: &TestGame, id: ObjectId, source: ObjectId) -> bool {
    t.g.protected_from(t.g.current(id), t.g.current(source))
}

/// The names of the permanents on the battlefield (sorted).
pub fn battlefield_names(t: &TestGame) -> Vec<String> {
    let mut v: Vec<String> = t.g.permanents().map(|o| o.chars.name.to_string()).collect();
    v.sort();
    v
}

/// Whether P0 may reveal `companion` as their companion with a 60-card starting deck of
/// the given cards (CR 702.139a): the game is started with the deck and the companion in
/// the sideboard, and P0 asked to reveal it.
pub fn companion_revealable(deck: &[&str], companion: &str) -> bool {
    use mtg_engine::card::card;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    let decks: Vec<Vec<Arc<mtg_engine::card::CardDef>>> =
        vec![deck.iter().map(|n| card(n)).collect(), (0..60).map(|_| card("Forest")).collect()];
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); 2],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn mtg_engine::decision::Agent>> = (0..2)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn mtg_engine::decision::Agent>
        })
        .collect();
    let config = mtg_engine::game::GameConfig {
        starting_player: Some(P0),
        ..Default::default()
    };
    let g = mtg_engine::game::Game::new(config, decks, agents);
    let mut t = TestGame { g, script };
    let side = t.g.add_to_sideboard(P0, vec![card(companion)]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    t.g.companion_of(P0) == Some(side[0])
}
