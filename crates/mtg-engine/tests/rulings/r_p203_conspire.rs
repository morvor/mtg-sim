//! Rulings batch P203 — conspire (CR 702.78), tested with Rally the Galadhrim ({2}{G}{U}
//! sorcery: "Create a token that's a copy of target creature you control. Conspire") and
//! Traitor's Roar ({4}{B/R} sorcery: "Tap target untapped creature. It deals damage equal
//! to its power to its controller. Conspire").

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::is_spell_copy;
use crate::r_s07_common::resolved;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

const RALLY: &str = "Rally the Galadhrim";

/// Queues paying (or declining) the conspire cost by tapping `tap`.
fn pay_conspire(t: &mut TestGame, p: PlayerId, tap: Option<&[ObjectId]>) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(tap.is_some()));
    if let Some(tap) = tap {
        let es: Vec<Entity> = tap.iter().map(|o| Entity::Object(*o)).collect();
        t.answer_choose(p, &es);
    }
}

/// Tokens named `name` controlled by `p`.
fn token_copies(t: &TestGame, p: PlayerId, name: &str) -> usize {
    tokens(t, p)
        .into_iter()
        .filter(|id| t.obj(*id).chars.name == name)
        .count()
}

/// Spell copies on the stack.
fn copies_on_stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| is_spell_copy(t, *id))
        .collect()
}

/// P0 with two Grizzly Bears (green, to tap for conspire), a Hill Giant to copy, and
/// Rally the Galadhrim in hand with its mana. Returns (bears, giant, rally).
fn rally_setup(t: &mut TestGame) -> ([ObjectId; 2], ObjectId, ObjectId) {
    supported(RALLY);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let rally = in_hand_with_mana(t, P0, RALLY);
    ([a, b], giant, rally)
}

#[test]
fn conspire_is_an_additional_cost_and_a_cast_trigger_if_it_was_paid() {
    cr!("702.78a", "601.2b", "601.2f", "603.2");
    ruling!(
        "Rally the Galadhrim",
        "Conspire represents both an additional cost and a triggered ability that triggers when you cast the spell if you paid that cost."
    );
    // Paid: the two Bears are tapped as the spell is cast, and the conspire ability
    // triggers.
    let mut t = TestGame::new(2);
    let (bears, giant, rally) = rally_setup(&mut t);
    pay_conspire(&mut t, P0, Some(&bears));
    t.cast(P0, rally).target(giant).go();
    assert!(bears.iter().all(|b| t.obj(*b).tapped));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Conspire"), 1);
    t.resolve();
    assert_eq!(copies_on_stack(&t).len(), 1);
    t.resolve_all();
    assert_eq!(token_copies(&t, P0, "Hill Giant"), 2);
    // Not paid: nothing is tapped and nothing triggers.
    let mut t = TestGame::new(2);
    let (bears, giant, rally) = rally_setup(&mut t);
    pay_conspire(&mut t, P0, None);
    t.cast(P0, rally).target(giant).go();
    t.settle();
    assert!(bears.iter().all(|b| !t.obj(*b).tapped));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(token_copies(&t, P0, "Hill Giant"), 1);
}

#[test]
fn each_creature_tapped_for_conspire_must_share_a_color_with_the_spell() {
    cr!("702.78a", "105.2");
    ruling!(
        "Rally the Galadhrim",
        "Each creature you tap for conspire must share a color with Rally the Galadhrim For example, you could tap two green creatures, two blue creatures, or one of each. For either or both of the creatures, you may tap a multicolored creature that's green and/or blue, and possibly other colors as well."
    );
    // A green creature and a green-blue one; two blue ones; a red-and-green creature
    // (Ghor-Clan Rampager) with a blue one: all work.
    for pair in [
        ["Grizzly Bears", "Kiora's Follower"],
        ["Merfolk of the Pearl Trident", "Merfolk of the Pearl Trident"],
        ["Ghor-Clan Rampager", "Merfolk of the Pearl Trident"],
    ] {
        supported(pair[0]);
        supported(pair[1]);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, pair[0]);
        let b = t.battlefield(P0, pair[1]);
        let giant = t.battlefield(P0, "Hill Giant");
        let rally = in_hand_with_mana(&mut t, P0, RALLY);
        pay_conspire(&mut t, P0, Some(&[a, b]));
        t.cast(P0, rally).target(giant).go();
        assert!(t.obj(a).tapped && t.obj(b).tapped, "{pair:?}");
        t.resolve_all();
        assert_eq!(token_copies(&t, P0, "Hill Giant"), 2, "{pair:?}");
    }
    // A red creature (Raging Goblin) shares no color with it: with only one green
    // creature, conspire can't be paid.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Raging Goblin");
    let giant = t.battlefield(P0, "Hill Giant");
    let rally = in_hand_with_mana(&mut t, P0, RALLY);
    pay_conspire(&mut t, P0, Some(&[a, b]));
    t.cast(P0, rally).target(giant).go();
    assert!(!t.obj(b).tapped);
    t.resolve_all();
    assert_eq!(token_copies(&t, P0, "Hill Giant"), 1);
}

#[test]
fn the_conspire_copy_may_have_new_targets() {
    cr!("702.78a", "707.10c");
    ruling!(
        "Rally the Galadhrim",
        "If a spell with conspire has targets, you may choose new targets for the copy."
    );
    let mut t = TestGame::new(2);
    let (bears, giant, rally) = rally_setup(&mut t);
    let elves = t.battlefield(P0, "Llanowar Elves");
    pay_conspire(&mut t, P0, Some(&bears));
    t.cast(P0, rally).target(giant).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.resolve_all();
    assert_eq!(token_copies(&t, P0, "Hill Giant"), 1);
    assert_eq!(token_copies(&t, P0, "Llanowar Elves"), 1);
}

#[test]
fn the_conspire_copy_isnt_cast() {
    cr!("702.78a", "707.10", "603.2");
    ruling!(
        "Rally the Galadhrim",
        "The copy that conspire creates is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Young Pyromancer");
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let (bears, giant, rally) = rally_setup(&mut t);
    pay_conspire(&mut t, P0, Some(&bears));
    t.cast(P0, rally).target(giant).go();
    t.resolve_all();
    assert_eq!(token_copies(&t, P0, "Hill Giant"), 2);
    assert_eq!(tokens(&t, P0).len(), 3); // two Giants and one Elemental
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn the_original_and_the_copy_are_countered_individually() {
    cr!("702.78a", "701.6a", "707.10");
    ruling!(
        "Rally the Galadhrim",
        "A copy of a spell can be countered like any other spell, but it must be countered individually. Countering a spell with conspire won't affect the copy, and vice versa."
    );
    for counter_copy in [false, true] {
        let mut t = TestGame::new(2);
        let (bears, giant, rally) = rally_setup(&mut t);
        pay_conspire(&mut t, P0, Some(&bears));
        let spell = t.cast(P0, rally).target(giant).go();
        t.resolve(); // the conspire trigger
        let copy = copies_on_stack(&t)[0];
        let victim = if counter_copy { copy } else { spell };
        let cs = in_hand_with_mana(&mut t, P1, "Counterspell");
        t.cast(P1, cs).target(victim).go();
        t.resolve_all();
        assert_eq!(token_copies(&t, P0, "Hill Giant"), 1);
        assert_eq!(resolved(&t, spell), counter_copy);
        assert_eq!(resolved(&t, copy), !counter_copy);
    }
}

#[test]
fn a_traitors_roar_copy_with_the_same_target_makes_the_original_fizzle() {
    cr!("702.78a", "608.2b", "405.5");
    ruling!(
        "Traitor's Roar",
        "If you use conspire to copy Traitor’s Roar, but you don’t change its target, the copy will resolve just fine but the original doesn’t resolve. That’s because the copy will resolve first, and as part of its resolution, it will tap the targeted creature. Then, when the original Traitor’s Roar tries to resolve, it will have an illegal target (since it must target an untapped creature)."
    );
    supported("Traitor's Roar");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Raging Goblin");
    let b = t.battlefield(P0, "Raging Goblin");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let roar = t.hand(P0, "Traitor's Roar");
    pay_conspire(&mut t, P0, Some(&[a, b]));
    let spell = t.cast(P0, roar).target(giant).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.obj(giant).tapped);
    // Only the copy dealt damage: the Giant's 3 power, once.
    assert_eq!(t.life(P1), 17);
    assert!(!resolved(&t, spell));
    assert!(t.in_graveyard(P0, "Traitor's Roar"));
}
