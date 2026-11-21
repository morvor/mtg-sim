//! Effect-created triggers (cards5-game-rules-dice, `effect-triggers`): "Until the end of
//! your next turn, whenever you cast a spell, ..." (Season of the Bold) and an optional
//! "you may choose to have it deal damage ... If you do, it assigns no combat damage this
//! turn." in an "until end of turn, whenever ..." trigger (Gaze of Pain).

use crate::combat2_common::{declare, supported, to_end_of_combat};
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn ready(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

/// Casts Opt (an instant) for `p`, targeting `creature` with Season of the Bold's
/// trigger if it triggers, and resolves everything.
fn cast_opt(t: &mut TestGame, p: PlayerId, creature: ObjectId) {
    let opt = t.hand(p, "Opt");
    mana(t, p);
    t.answer_targets(p, &[Entity::Object(creature)]);
    t.cast(p, opt).go();
    t.resolve_all();
}

fn season_triggers(t: &TestGame) -> usize {
    t.g.delayed_triggers.len()
}

#[test]
fn season_of_the_bold_lasts_until_the_end_of_your_next_turn() {
    cr!("603.7b", "603.7c", "611.2a");
    supported("Season of the Bold");
    ruling!(
        "Season of the Bold",
        "You choose a target for that ability when it goes on the stack."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let season = t.hand(P0, "Season of the Bold");
    mana(&mut t, P0);
    t.cast(P0, season).modes(&[2]).go();
    t.resolve_all();
    assert_eq!(season_triggers(&t), 1);
    // This turn: casting a spell triggers it; its target is chosen as it's put on the
    // stack, and it deals 2 damage.
    let giant = t.battlefield(P1, "Hill Giant");
    cast_opt(&mut t, P0, giant);
    assert_eq!(t.obj_now(giant).damage, 2);
    // The opponent's turn (the damage was removed in the cleanup step).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(giant).damage, 0);
    cast_opt(&mut t, P0, giant);
    assert_eq!(t.obj_now(giant).damage, 2);
    // Your next turn, through its end step.
    t.advance_to(P0, Step::End);
    cast_opt(&mut t, P0, giant);
    assert_eq!(t.obj_now(giant).damage, 2);
    // The opponent casting a spell doesn't trigger it.
    let other = t.battlefield(P0, "Hill Giant");
    cast_opt(&mut t, P1, other);
    assert_eq!(t.obj_now(other).damage, 0);
    // After your next turn: gone.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(season_triggers(&t), 0);
    cast_opt(&mut t, P0, giant);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn season_of_the_bold_trigger_resolves_before_the_spell() {
    cr!("603.7b", "603.3");
    ruling!(
        "Season of the Bold",
        "resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let season = t.hand(P0, "Season of the Bold");
    mana(&mut t, P0);
    t.cast(P0, season).modes(&[2]).go();
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, opt).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "Opt and the trigger above it");
    // The trigger resolves first: the Bears die while Opt is still on the stack.
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.g.stack.contains(&spell));
}

/// Casts Gaze of Pain, then attacks with a 3/3 Hill Giant that isn't blocked; the trigger
/// targets the opponent's Grizzly Bears, and `yes` is the choice to have it deal damage.
fn gaze_attack(yes: bool) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    ready(&mut t, giant);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gaze = t.hand(P0, "Gaze of Pain");
    mana(&mut t, P0);
    t.cast(P0, gaze).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, yes);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    t.resolve_all();
    (t, giant, bears)
}

#[test]
fn gaze_of_pain_the_unblocked_creature_deals_damage_instead_of_combat_damage() {
    cr!("603.7b", "608.2d", "510.1a");
    supported("Gaze of Pain");
    let (t, _giant, bears) = gaze_attack(true);
    assert!(!t.on_battlefield(bears), "3 damage to the Bears");
    assert_eq!(t.life(P1), 20, "the Giant assigned no combat damage");
}

#[test]
fn gaze_of_pain_declining_leaves_combat_damage_as_usual() {
    cr!("603.7b", "608.2d");
    let (t, _giant, bears) = gaze_attack(false);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn gaze_of_pain_lasts_only_this_turn() {
    cr!("603.7b");
    let (mut t, giant, bears) = gaze_attack(false);
    assert_eq!(t.g.delayed_triggers.len(), 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.delayed_triggers.is_empty());
    t.advance_to(P0, Step::PrecombatMain);
    ready(&mut t, giant);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn gaze_of_pain_doesnt_trigger_for_a_blocked_creature() {
    cr!("603.7b", "509.1h");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    ready(&mut t, giant);
    let wall = t.battlefield(P1, "Wall of Wood");
    ready(&mut t, wall);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gaze = t.hand(P0, "Gaze of Pain");
    mana(&mut t, P0);
    t.cast(P0, gaze).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    crate::combat2_common::declare_blocks(&mut t, P1, &[(wall, giant)]);
    assert!(t.g.pending_triggers.is_empty() && t.stack_len() == 0);
    to_end_of_combat(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}
