//! Shared helpers for the tests of rulings batch S12 (`r_s12_*.rs`): morph, multikicker,
//! myriad, nightbound, ninjutsu, offspring, opus, outlast, overload, paradigm, paradox,
//! parley, partner. (The helpers of batches S01–S11 are used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casting a card face down with its morph ability (CR 702.37c).
pub const MORPH: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);

/// `p` casts the real card `name` face down with morph (paying {3} with Wastes) and it
/// resolves. Returns the face-down permanent.
pub fn morph(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, name);
    let spell = t.cast(p, card).method(MORPH).go();
    assert!(t.obj(spell).face_down);
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down && t.on_battlefield(id), "{name} not face down");
    id
}

/// The attacking creatures attacking `target` now.
pub fn attackers_at(t: &TestGame, target: Entity) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| {
            c.attackers
                .iter()
                .filter(|a| a.target == Some(target))
                .map(|a| a.id)
                .collect()
        })
        .unwrap_or_default()
}

/// A game that hasn't begun (no commanders designated, no opening hands), with the given
/// decks and scripted agents.
pub fn pregame(
    config: mtg_engine::game::GameConfig,
    decks: Vec<Vec<std::sync::Arc<mtg_engine::card::CardDef>>>,
) -> TestGame {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    let n = decks.len();
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn mtg_engine::decision::Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn mtg_engine::decision::Agent>
        })
        .collect();
    let mut g = mtg_engine::game::Game::new(config, decks, agents);
    g.logging = true;
    TestGame { g, script }
}

/// What `id` is attacking now, if it's attacking.
pub fn attack_target(t: &TestGame, id: ObjectId) -> Option<Entity> {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()
        .and_then(|c| c.attackers.iter().find(|a| a.id == id))
        .and_then(|a| a.target)
}
