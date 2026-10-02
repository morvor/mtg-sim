//! "This spell can't be copied." / "This ability can't be copied." (CR 113.6g, 707.10;
//! `rule_statics::cant_be_copied`).

use mtg_engine::testing::*;
use mtg_engine::*;

/// P1 casts Twincast ("Copy target instant or sorcery spell. You may choose new targets
/// for the copy.") targeting `spell` and resolves it.
fn twincast(t: &mut TestGame, spell: ObjectId) {
    t.lands(P1, "Island", 2);
    let tc = t.hand(P1, "Twincast");
    t.answer_yes(P1, false);
    t.cast(P1, tc).target(spell).go();
    t.resolve();
}

#[test]
fn a_spell_that_cant_be_copied_isnt() {
    cr!("707.10", "113.6g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let sd = t.hand(P0, "See Double");
    // "Create a token that's a copy of target creature."
    let spell = t.cast(P0, sd).modes(&[1]).target(bears).go();
    twincast(&mut t, spell);
    // No copy: See Double alone is on the stack.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn other_spells_are_copied_as_usual() {
    cr!("707.10");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    twincast(&mut t, spell);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn display_of_power_copies_each_target_spell_once_and_cant_be_copied() {
    cr!("707.10", "113.6g");
    ruling!("Display of Power", "you can, at most, make one copy of each instant and sorcery spell on the stack");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    let b = t.cast(P0, bolt).target(P1).go();
    let s = t.cast(P0, shock).target(P1).go();
    t.lands(P0, "Mountain", 3);
    let dop = t.hand(P0, "Display of Power");
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    let d = t
        .cast(P0, dop)
        .targets(&[Entity::Object(b), Entity::Object(s)])
        .go();
    // Display of Power itself can't be copied.
    twincast(&mut t, d);
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    // Bolt and Shock and one copy of each.
    assert_eq!(t.life(P1), 20 - 2 * (3 + 2));
}

#[test]
fn gogos_ability_copies_an_ability_x_times_and_cant_be_copied() {
    cr!("707.10", "107.3a");
    ruling!("Gogo, Master of Mimicry", "creates one or more additional instances of that ability on the stack");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let gogo_a = t.battlefield(P0, "Gogo, Master of Mimicry");
    let gogo_b = t.battlefield(P0, "Gogo, Master of Mimicry");
    t.lands(P0, "Island", 4);
    let ping = t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap().unwrap();
    // Gogo A copies the ping once (X = 1).
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_yes(P0, false);
    let a = t
        .activate(P0, gogo_a, 0, &[Entity::Object(ping)])
        .unwrap()
        .unwrap();
    // Gogo B targets Gogo A's ability, which can't be copied: nothing happens.
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_yes(P0, false);
    t.activate(P0, gogo_b, 0, &[Entity::Object(a)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn x_copies_with_x_of_two() {
    cr!("707.10");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let gogo = t.battlefield(P0, "Gogo, Master of Mimicry");
    t.lands(P0, "Island", 4);
    let ping = t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap().unwrap();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.activate(P0, gogo, 0, &[Entity::Object(ping)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
