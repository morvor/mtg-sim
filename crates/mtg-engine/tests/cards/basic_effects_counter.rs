//! Countering spells and abilities described with qualifiers (CR 701.6, 113.7a, 115.9b):
//! sources of abilities, lists of spells and abilities, "unless ... pays", non-targeted
//! "counter all abilities", and the permanent whose countered ability it was.

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 activates Rod of Ruin ({3}, {T}: 1 damage to any target) at P1; returns the ability
/// on the stack and the Rod.
fn rod_ability(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let rod = t.battlefield(P0, "Rod of Ruin");
    t.lands(P0, "Mountain", 3);
    let ab = t
        .activate(P0, rod, 0, &[Entity::Player(P1)])
        .unwrap()
        .expect("on the stack");
    (ab, rod)
}

#[test]
fn rust_counters_an_activated_ability_from_an_artifact_source() {
    cr!("701.6a", "113.7");
    ruling!("Rust", "Can’t be used on mana abilities because they don’t use the stack");
    assert_supported("Rust");
    let mut t = TestGame::new(2);
    let (ab, _) = rod_ability(&mut t);
    t.lands(P1, "Forest", 1);
    let rust = t.hand(P1, "Rust");
    t.cast(P1, rust).target(ab).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);

    // A mana ability of an artifact never is on the stack: nothing to target.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    assert_eq!(t.activate(P0, stone, 0, &[]).unwrap(), None);
    assert!(t.g.stack.is_empty());
    t.lands(P1, "Forest", 1);
    let rust = t.hand(P1, "Rust");
    assert!(t.cast(P1, rust).try_go().is_err());
}

#[test]
fn an_ability_from_a_creature_source_isnt_an_artifact_sources() {
    cr!("113.7");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let ab = t
        .activate(P0, pyro, 0, &[Entity::Player(P1)])
        .unwrap()
        .expect("on the stack");
    t.lands(P1, "Forest", 2);
    let ouphe = t.battlefield(P1, "Brown Ouphe");
    assert_supported("Brown Ouphe");
    // The only ability on the stack isn't from an artifact source: no legal target.
    assert!(t.activate(P1, ouphe, 0, &[Entity::Object(ab)]).is_err());
    // An artifact's ability is.
    let (rod_ab, _) = rod_ability(&mut t);
    assert!(t.activate(P1, ouphe, 0, &[Entity::Object(rod_ab)]).is_ok());
}

#[test]
fn ouphe_vandals_counters_and_destroys_the_artifact() {
    cr!("701.6a", "113.7a");
    ruling!(
        "Ouphe Vandals",
        "The ability targets an ability on the stack, not the artifact"
    );
    assert_supported("Ouphe Vandals");
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    t.lands(P1, "Forest", 1);
    let ouphe = t.battlefield(P1, "Ouphe Vandals");
    t.activate(P1, ouphe, 0, &[Entity::Object(ab)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(rod));
    assert!(t.in_graveyard(P0, "Rod of Ruin"));
}

#[test]
fn green_slime_destroys_the_permanent_whose_ability_it_counters() {
    cr!("701.6a");
    assert_supported("Green Slime");
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    t.answer_targets(P1, &[Entity::Object(ab)]);
    t.enter(P1, "Green Slime");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(rod));
}

#[test]
fn interdict_stops_further_activations_this_turn() {
    cr!("701.6a", "602.5a");
    ruling!("Interdict", "This card targets the ability, and not the permanent itself.");
    assert_supported("Interdict");
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    t.lands(P1, "Island", 2);
    let interdict = t.hand(P1, "Interdict");
    t.cast(P1, interdict).target(ab).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Rod of Ruin is still there, untapped again, but can't be activated this turn.
    t.g.untap(rod);
    t.lands(P0, "Mountain", 3);
    assert!(t.activate(P0, rod, 0, &[Entity::Player(P1)]).is_err());
}

#[test]
fn ayesha_tanaka_counters_unless_that_abilitys_controller_pays() {
    cr!("118.12a");
    assert_supported("Ayesha Tanaka");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let (ab, _) = rod_ability(&mut t);
        t.lands(P0, "Plains", 1);
        let ayesha = t.battlefield(P1, "Ayesha Tanaka");
        t.activate(P1, ayesha, 0, &[Entity::Object(ab)]).unwrap();
        t.answer_yes(P0, pays);
        t.resolve();
        t.resolve_all();
        assert_eq!(t.life(P1), if pays { 19 } else { 20 });
    }
}

#[test]
fn tales_end_counters_a_triggered_ability_or_a_legendary_spell_only() {
    cr!("701.6a");
    ruling!(
        "Tale's End",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\""
    );
    assert_supported("Tale's End");
    // A legendary creature spell is a legal target; a nonlegendary one isn't.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    let isamaru = t.cast(P0, isamaru).go();
    t.lands(P1, "Island", 2);
    let te = t.hand(P1, "Tale's End");
    t.cast(P1, te).target(isamaru).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));

    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let bears = t.cast(P0, bears).go();
    t.lands(P1, "Island", 2);
    let te = t.hand(P1, "Tale's End");
    assert!(t.cast(P1, te).target(bears).try_go().is_err());

    // A triggered ability ("When this creature enters, draw a card") is a legal target.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    t.enter(P0, "Elvish Visionary");
    t.settle();
    let trigger = *t.g.stack.last().expect("the enters trigger");
    t.lands(P1, "Island", 2);
    let te = t.hand(P1, "Tale's End");
    t.cast(P1, te).target(trigger).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn stern_scolding_targets_a_creature_spell_with_power_or_toughness_two_or_less() {
    cr!("701.6a");
    assert_supported("Stern Scolding");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    // Flash creatures: a 2/1 and a 3/3.
    let viper = t.hand(P0, "Ambush Viper");
    let bears = t.cast(P0, viper).go();
    let alpha = t.hand(P0, "Briarpack Alpha");
    let giant = t.cast(P0, alpha).target(bears).go();
    t.lands(P1, "Island", 1);
    let ss = t.hand(P1, "Stern Scolding");
    let _ = t.cast(P1, ss).target(bears).try_go();
    let cands = last_target_candidates(&t, P1);
    assert!(cands.contains(&Entity::Object(bears)));
    assert!(!cands.contains(&Entity::Object(giant)));
}

#[test]
fn second_guess_targets_only_the_second_spell_cast_this_turn() {
    cr!("701.6a");
    ruling!(
        "Second Guess",
        "Second Guess can’t be cast unless the spell it targets is the second spell cast this turn."
    );
    assert_supported("Second Guess");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let a = t.hand(P0, "Shock");
    let first = t.cast(P0, a).target(P1).go();
    let b = t.hand(P0, "Shock");
    let second = t.cast(P0, b).target(P1).go();
    t.lands(P1, "Island", 2);
    let sg = t.hand(P1, "Second Guess");
    t.cast(P1, sg).target(second).go();
    let cands = last_target_candidates(&t, P1);
    assert!(cands.contains(&Entity::Object(second)));
    assert!(!cands.contains(&Entity::Object(first)));
    t.resolve_all();
    // Only the first Shock resolved.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn frontline_medic_counters_a_spell_with_x_unless_its_controller_pays() {
    cr!("118.12a");
    assert_supported("Frontline Medic");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let fb = t.hand(P0, "Fireball");
    let fb = t.cast(P0, fb).target(P1).x(2).go();
    let medic = t.battlefield(P1, "Frontline Medic");
    t.activate(P1, medic, 0, &[Entity::Object(fb)]).unwrap();
    // P0 has no mana left to pay {3}.
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Fireball"));
}

#[test]
fn thassas_intervention_demands_twice_x() {
    cr!("118.12a", "107.3");
    assert_supported("Thassa's Intervention");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Forest", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    let bears = t.cast(P0, bears).go();
    t.lands(P1, "Island", 4);
    let ti = t.hand(P1, "Thassa's Intervention");
    // X = 2: P0 would have to pay {4} but has only three untapped lands.
    t.cast(P1, ti).modes(&[1]).x(2).target(bears).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn summary_dismissal_exiles_other_spells_and_counters_abilities() {
    cr!("701.6a");
    ruling!("Summary Dismissal", "Spells that can’t be countered are exiled by Summary Dismissal.");
    assert_supported("Summary Dismissal");
    let mut t = TestGame::new(2);
    let (_ab, _) = rod_ability(&mut t);
    // A spell that can't be countered is exiled all the same.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let decay = t.hand(P0, "Abrupt Decay");
    t.cast(P0, decay).target(bears).go();
    t.lands(P1, "Island", 4);
    let sd = t.hand(P1, "Summary Dismissal");
    t.cast(P1, sd).go();
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Abrupt Decay"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Summary Dismissal"));
}
