//! Shared helpers for the tests of rulings batch S20 (`r_s20_*.rs`): split cards and
//! Rooms, transforming double-faced cards, attacking, and Auras. (The helpers of batches
//! S01–S16 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield under `p`'s control through a real zone
/// change during this turn: `p` hasn't controlled it continuously since the turn began.
pub fn entered_this_turn(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.settle();
    id
}

/// `p` tries to activate the mana ability of `source` whose text contains `needle`;
/// whether it was activated (the mana is added to `p`'s mana pool).
pub fn tap_for_mana(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| {
            matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability)
                && a.text.contains(needle)
        })
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no mana ability containing {needle:?}"));
    let before = t.g.player(p).mana_pool.total();
    let ok = t.g.activate_ability(p, source, uid).is_ok();
    t.g.flush_events();
    ok && t.g.player(p).mana_pool.total() > before
}

/// Whether `p` could take the priority action `pred` accepts now.
pub fn has_action(t: &mut TestGame, p: PlayerId, pred: impl Fn(&Action) -> bool) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p).iter().any(pred)
}

/// Whether the permanent (followed across zone changes) is a creature now.
pub fn creature_now(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).is(CardType::Creature)
}

/// Moves to `p`'s beginning of combat step in a fresh combat phase.
pub fn to_beginning_of_combat(t: &mut TestGame, p: PlayerId) {
    t.g.combat = None;
    t.set_step(p, Step::BeginningOfCombat);
}

/// The permanents `p` controls named `name`.
pub fn controlled_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}
