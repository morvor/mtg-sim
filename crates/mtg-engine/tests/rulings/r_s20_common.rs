//! Shared helpers for the tests of rulings batch S20 (`r_s20_*.rs`): split cards and
//! Rooms, transforming double-faced cards, attacking, and Auras. (The helpers of batches
//! S01–S16 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
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

/// The options offered by the last "cast a spell ... from your hand" choice asked of P0.
pub fn expertise_options(t: &TestGame) -> Vec<String> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if *p == P0 && prompt.contains("from your hand") => Some(options.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// P0 casts Sram's Expertise ("Create three 1/1 colorless Servo artifact creature tokens.
/// You may cast a spell with mana value 3 or less from your hand without paying its mana
/// cost.") and it resolves, P0 answering the choice of a spell with option `pick` (0 is
/// declining). The spell cast this way (if any) is left on the stack. Returns the options
/// offered.
pub fn sram_expertise(t: &mut TestGame, pick: usize) -> Vec<String> {
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::W, 4);
    let expertise = t.hand(P0, "Sram's Expertise");
    let spell = t.cast(P0, expertise).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(pick));
    t.settle();
    t.g.resolve_top();
    t.settle();
    assert!(!t.g.stack.contains(&spell));
    expertise_options(t)
}
