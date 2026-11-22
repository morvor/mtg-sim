//! Rulings batch S27 — a spell with {X} in its mana cost cast without paying its mana
//! cost has X = 0 (CR 107.3b), while one cast by paying its costs has X chosen as normal
//! (CR 107.3a, 707.12).

use crate::r_s01_common::*;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s07_common::damage_on;
use crate::r_s22_common::attack_p1_unblocked;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Queues P0's answers for casting Blaze ("Blaze deals X damage to any target.") through
/// an effect: choose the card, X = 5 (which isn't allowed), and the target.
fn queue_blaze(t: &mut TestGame, blaze: ObjectId, target: ObjectId) {
    t.answer_choose(P0, &[Entity::Object(blaze)]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Object(target)]);
}

/// Blaze was cast without paying its mana cost, with X = 0: it resolved and dealt no
/// damage to the Hill Giant, and no land was tapped for it.
fn blaze_had_x_0(t: &TestGame, giant: ObjectId) {
    assert!(t.in_graveyard(P0, "Blaze"), "Blaze was cast");
    assert!(t.on_battlefield(giant));
    assert_eq!(damage_on(t, giant), 0);
    assert_eq!(tapped_lands(t, P0), 0);
}

#[test]
fn oracle_of_bones_casts_a_spell_with_x_0() {
    cr!("107.3b", "118.9", "702.104a");
    ruling!(
        "Oracle of Bones",
        "If the card has {X} in its mana cost, you must choose 0 as its value."
    );
    supported("Oracle of Bones");
    // "Tribute 2. When this creature enters, if tribute wasn't paid, you may cast an
    // instant or sorcery spell from your hand without paying its mana cost."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    t.lands(P0, "Mountain", 6);
    // P0 chooses P1 for the tribute; P1 doesn't pay it; P0 casts Blaze.
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.answer_yes(P0, true);
    queue_blaze(&mut t, blaze, giant);
    t.enter(P0, "Oracle of Bones");
    t.resolve_all();
    blaze_had_x_0(&t, giant);
}

#[test]
fn glamdring_casts_a_spell_with_x_0() {
    cr!("107.3b", "118.9");
    ruling!(
        "Glamdring",
        "If the spell has {X} in its mana cost, you must choose 0 as the value of X."
    );
    supported("Glamdring");
    // "Whenever equipped creature deals combat damage to a player, you may cast an
    // instant or sorcery spell from your hand with mana value less than or equal to that
    // damage without paying its mana cost."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Glamdring", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    t.lands(P0, "Mountain", 6);
    queue_blaze(&mut t, blaze, giant);
    attack_p1_unblocked(&mut t, bears);
    blaze_had_x_0(&t, giant);
}

#[test]
fn evercoat_ursine_plays_a_spell_with_x_0() {
    cr!("107.3b", "118.9", "702.75a");
    ruling!(
        "Evercoat Ursine",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when playing it without paying its mana cost."
    );
    supported("Evercoat Ursine");
    // "Hideaway 3, hideaway 3. Whenever this creature deals combat damage to a player, if
    // there are cards exiled with it, you may play one of them without paying its mana
    // cost."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Blaze", "Forest", "Forest", "Island"]);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    let ursine = t.enter(P0, "Evercoat Ursine");
    t.resolve_all();
    t.clear_answers();
    t.g.objects[ursine.0 as usize].summoning_sick = false;
    let blaze = t.g.current(cards[0]);
    assert!(t.obj(blaze).face_down);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 6);
    queue_blaze(&mut t, blaze, giant);
    attack_p1_unblocked(&mut t, ursine);
    blaze_had_x_0(&t, giant);
}

#[test]
fn brain_in_a_jar_casts_a_spell_with_x_0() {
    cr!("107.3b", "118.9", "202.3e");
    ruling!(
        "Brain in a Jar",
        "If the card has {X} in its mana cost, you must choose 0 as the value of X."
    );
    supported("Brain in a Jar");
    // "{1}, {T}: Put a charge counter on this artifact, then you may cast an instant or
    // sorcery spell with mana value equal to the number of charge counters on this
    // artifact from your hand without paying its mana cost." Blaze in hand has mana
    // value 1.
    let mut t = TestGame::new(2);
    let jar = t.battlefield(P0, "Brain in a Jar");
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
    t.lands(P0, "Mountain", 6);
    queue_blaze(&mut t, blaze, giant);
    activate_containing(&mut t, P0, jar, "Put a charge counter").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(jar, "charge"), 1);
    blaze_had_x_0(&t, giant);
}
