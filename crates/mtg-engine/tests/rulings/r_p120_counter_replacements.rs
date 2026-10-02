//! Rulings batch P120 — replacement effects that change how many +1/+1 counters are put on
//! a permanent (CR 614.1a, 614.16): "that many plus one" (Hardened Scales style), "twice
//! that many" (doublers), and "enters with an additional +1/+1 counter". They apply to
//! counters a permanent enters with as well (CR 122.6, 614.1c); several of them each apply
//! once to the same event (CR 614.5, 616.1).

use crate::r_p120_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// "If you somehow control two copies" of a legendary permanent: P0 gains control of the
/// copy P1 controls (the only permanent P1 controls), and nothing settles before the
/// counters are put (see [`plus1_without_sbas`]), so the legend rule hasn't applied yet.
fn second_copy(t: &mut TestGame) {
    let b = t.g.permanents().find(|o| o.controller == P1).unwrap().id;
    t.g.objects[b.0 as usize].controller = P0;
    t.g.objects[b.0 as usize].base_controller = P0;
    t.g.recompute();
}

/// Puts `n` +1/+1 counters on the object as one event without checking state-based
/// actions.
fn plus1_without_sbas(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(id), PLUS1, n, None);
    t.g.recompute();
}

// --- "that many plus one" ------------------------------------------------------------------

#[test]
fn solid_ground_adds_one_to_counters_on_any_permanent_you_control() {
    cr!("614.1a", "122.6");
    ruling!(
        "Solid Ground",
        "If a permanent you control would enter with a number of +1/+1 counters on it, it enters with that many plus one instead."
    );
    supported("Solid Ground");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solid Ground");
    // A creature entering with three +1/+1 counters gets four.
    let grakmaw = t.enter(P0, "Grakmaw, Skyclave Ravager");
    assert_eq!(plus1(&t, grakmaw), 4);
    // Any permanent you control, not only creatures: a land.
    let forest = t.battlefield(P0, "Forest");
    give_plus1(&mut t, forest, 1);
    assert_eq!(plus1(&t, forest), 2);
    // Not an opponent's permanent.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give_plus1(&mut t, theirs, 1);
    assert_eq!(plus1(&t, theirs), 1);
}

#[test]
fn solid_ground_applies_to_its_own_earthbend() {
    cr!("614.1a", "701.66a");
    supported("Solid Ground");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[obj(forest)]);
    t.enter(P0, "Solid Ground");
    t.resolve_all();
    // Earthbend 3 puts three counters; Solid Ground is already on the battlefield.
    assert_eq!(plus1(&t, forest), 4);
    assert_eq!(t.pt(forest), (4, 4));
}

#[test]
fn two_solid_grounds_each_add_one() {
    cr!("614.5", "616.1");
    ruling!(
        "Solid Ground",
        "If you control multiple copies of Solid Ground, they each increase the number of +1/+1 counters put on a permanent you control by one."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solid Ground");
    t.battlefield(P0, "Solid Ground");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn kami_of_whispered_hopes_adds_one_including_entering_counters() {
    cr!("614.1a", "122.6", "614.5");
    ruling!(
        "Kami of Whispered Hopes",
        "If another permanent you control would enter the battlefield with a number of +1/+1 counters on it, it enters with that many plus one instead."
    );
    ruling!(
        "Kami of Whispered Hopes",
        "If you control two Kamis of Whispered Hopes, the number of +1/+1 counters put on a permanent is two plus the original number."
    );
    supported("Kami of Whispered Hopes");
    let mut t = TestGame::new(2);
    let kami = t.battlefield(P0, "Kami of Whispered Hopes");
    let grakmaw = t.enter(P0, "Grakmaw, Skyclave Ravager");
    assert_eq!(plus1(&t, grakmaw), 4);
    // It applies to the Kami itself; its mana ability then adds mana equal to its power.
    give_plus1(&mut t, kami, 1);
    assert_eq!(plus1(&t, kami), 2);
    assert_eq!(t.pt(kami), (3, 3));
    // Two Kamis: two plus the original number.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kami of Whispered Hopes");
    t.battlefield(P0, "Kami of Whispered Hopes");
    let forest = t.battlefield(P0, "Forest");
    give_plus1(&mut t, forest, 1);
    assert_eq!(plus1(&t, forest), 3);
}

#[test]
fn mauhur_adds_one_to_an_army_when_you_amass() {
    cr!("614.1a", "701.47a");
    ruling!(
        "Mauhúr, Uruk-hai Captain",
        "Mauhúr's last ability applies to counters placed on Army tokens when you amass Orcs (or Zombies, or anything else.)"
    );
    supported("Mauhúr, Uruk-hai Captain");
    supported("Mordor Muster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mauhúr, Uruk-hai Captain");
    cast_new(&mut t, P0, "Mordor Muster", &[]);
    t.resolve_all();
    let armies: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Army"))
        .map(|o| o.id)
        .collect();
    assert_eq!(armies.len(), 1);
    // Amass Orcs 1 put one counter; Mauhúr made it two.
    assert_eq!(plus1(&t, armies[0]), 2);
    // A Goblin gets the extra counter too; a non-Army, non-Goblin, non-Orc doesn't.
    let goblin = t.battlefield(P0, "Goblin Piker");
    give_plus1(&mut t, goblin, 1);
    assert_eq!(plus1(&t, goblin), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 1);
}

#[test]
fn loading_zone_doubles_each_kind_of_counter_on_your_creatures() {
    cr!("614.1a");
    supported("Loading Zone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loading Zone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 2);
    put_counters(&mut t, bears, "oil", 2);
    assert_eq!(t.counters(bears, "oil"), 4);
    // Not a noncreature permanent that isn't a Spacecraft or Planet, nor an opponent's.
    let forest = t.battlefield(P0, "Forest");
    give_plus1(&mut t, forest, 1);
    assert_eq!(plus1(&t, forest), 1);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give_plus1(&mut t, theirs, 1);
    assert_eq!(plus1(&t, theirs), 1);
}

#[test]
fn pir_adds_one_of_each_kind_to_permanents_its_team_controls() {
    cr!("614.1a", "102.4");
    supported("Pir, Imaginative Rascal");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Pir, Imaginative Rascal");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let teammate = t.battlefield(P1, "Grizzly Bears");
    let opponent = t.battlefield(P2, "Grizzly Bears");
    give_plus1(&mut t, mine, 1);
    put_counters(&mut t, teammate, "oil", 2);
    give_plus1(&mut t, opponent, 1);
    assert_eq!(plus1(&t, mine), 2);
    assert_eq!(t.counters(teammate, "oil"), 3);
    assert_eq!(plus1(&t, opponent), 1);
}

#[test]
fn caradora_adds_one_to_creatures_and_vehicles() {
    cr!("614.1a", "122.6", "614.5", "616.1");
    ruling!(
        "Caradora, Heart of Alacria",
        "If a creature or Vehicle you control would enter the battlefield with a number of +1/+1 counters on it, it enters with that many plus one instead."
    );
    ruling!(
        "Caradora, Heart of Alacria",
        "If you somehow control multiple copies of Caradora, they would each increase the number of +1/+1 counters put on a creature or Vehicle you control by one."
    );
    supported("Caradora, Heart of Alacria");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Caradora, Heart of Alacria");
    let grakmaw = t.enter(P0, "Grakmaw, Skyclave Ravager");
    assert_eq!(plus1(&t, grakmaw), 4);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    give_plus1(&mut t, copter, 1);
    assert_eq!(plus1(&t, copter), 2);
    // A second Caradora adds another one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Caradora, Heart of Alacria");
    t.battlefield(P1, "Caradora, Heart of Alacria");
    second_copy(&mut t);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    plus1_without_sbas(&mut t, copter, 1);
    assert_eq!(plus1(&t, copter), 3);
}

#[test]
fn ozolith_the_shattered_spire_adds_one_and_two_add_two() {
    cr!("614.1a", "122.6", "614.5", "616.1");
    ruling!(
        "Ozolith, the Shattered Spire",
        "If another artifact or creature you control would enter the battlefield with a number of +1/+1 counters on it, it enters with that many plus one instead."
    );
    ruling!(
        "Ozolith, the Shattered Spire",
        "If you somehow control two copies of Ozolith, the Shattered Spire, the number of +1/+1 counters put on an artifact or creature you control is two plus the original number."
    );
    supported("Ozolith, the Shattered Spire");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ozolith, the Shattered Spire");
    let grakmaw = t.enter(P0, "Grakmaw, Skyclave Ravager");
    assert_eq!(plus1(&t, grakmaw), 4);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ozolith, the Shattered Spire");
    t.battlefield(P1, "Ozolith, the Shattered Spire");
    second_copy(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1_without_sbas(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn two_conclave_mentors_add_two() {
    cr!("614.5", "616.1");
    ruling!(
        "Conclave Mentor",
        "If you control two Conclave Mentors, the number of +1/+1 counters put on a creature is two plus the original number."
    );
    supported("Conclave Mentor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conclave Mentor");
    t.battlefield(P0, "Conclave Mentor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
    // Three add three.
    t.battlefield(P0, "Conclave Mentor");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3 + 4);
}

#[test]
fn several_michelangelos_each_add_one() {
    cr!("614.5", "616.1");
    ruling!(
        "Michelangelo, Weirdness to 11",
        "In the unusual case where you control more than one Michelangelo, Weirdness to 11, each one you control will increase the number of +1/+1 counters placed on a creature you control by one."
    );
    supported("Michelangelo, Weirdness to 11");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Michelangelo, Weirdness to 11");
    t.battlefield(P1, "Michelangelo, Weirdness to 11");
    second_copy(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1_without_sbas(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
    // One alone (the legend rule keeps one) adds one.
    t.settle();
    assert_eq!(t.named_on_battlefield("Michelangelo, Weirdness to 11").len(), 1);
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 5);
}

// --- doublers ------------------------------------------------------------------------------

#[test]
fn branching_evolutions_multiply() {
    cr!("614.5", "616.1");
    ruling!(
        "Branching Evolution",
        "If you control two Branching Evolutions, the number of +1/+1 counters put on a creature is four times the original number."
    );
    supported("Branching Evolution");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Branching Evolution");
    t.battlefield(P0, "Branching Evolution");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 4);
    t.battlefield(P0, "Branching Evolution");
    let giant = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, giant, 1);
    assert_eq!(plus1(&t, giant), 8);
}

#[test]
fn two_earth_crystals_quadruple() {
    cr!("614.5", "616.1");
    ruling!(
        "The Earth Crystal",
        "If the rare case where you control two of The Earth Crystal, the number of +1/+1 counters put on a creature is four times the original number."
    );
    supported("The Earth Crystal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Earth Crystal");
    t.battlefield(P1, "The Earth Crystal");
    second_copy(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1_without_sbas(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 4);
}

#[test]
fn primal_vigors_multiply_tokens_and_counters() {
    cr!("614.5", "616.1", "614.16");
    ruling!(
        "Primal Vigor",
        "If there are two Primal Vigors on the battlefield, the number of tokens or +1/+1 counters is four times the original number."
    );
    supported("Primal Vigor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Vigor");
    t.battlefield(P1, "Primal Vigor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 4);
    create_token(&mut t, P0, "Soldier");
    assert_eq!(creature_tokens(&t, P0), 4);
    // Three: eight times.
    t.battlefield(P1, "Primal Vigor");
    let giant = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, giant, 1);
    assert_eq!(plus1(&t, giant), 8);
}

#[test]
fn primal_vigor_affects_everyones_tokens_and_creatures() {
    cr!("614.1a", "614.16");
    ruling!(
        "Primal Vigor",
        "It doesn't matter who controls the tokens or the creature that the +1/+1 counters are being placed on."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Vigor");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give_plus1(&mut t, theirs, 1);
    assert_eq!(plus1(&t, theirs), 2);
    create_token(&mut t, P1, "Soldier");
    assert_eq!(creature_tokens(&t, P1), 2);
}

#[test]
fn primal_vigor_doubles_counters_a_creature_enters_with() {
    cr!("614.1a", "122.6", "614.1c");
    ruling!(
        "Primal Vigor",
        "Primal Vigor affects permanents that \"enter the battlefield with\" a certain number of counters. For example, if a creature would normally enter the battlefield with three +1/+1 counters on it, it will enter with six +1/+1 counters on it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Primal Vigor");
    let grakmaw = t.enter(P0, "Grakmaw, Skyclave Ravager");
    assert_eq!(plus1(&t, grakmaw), 6);
}

// --- "enters with an additional +1/+1 counter" -----------------------------------------------

#[test]
fn oonas_blackguard_gives_rogues_an_additional_counter() {
    cr!("614.1c", "122.6");
    ruling!(
        "Oona's Blackguard",
        "If a Rogue would normally enter with a certain number of +1/+1 counters on it, it enters with that many +1/+1 counters plus one on it instead. If a Rogue would normally enter with no +1/+1 counters on it, it enters with one +1/+1 counter on it instead."
    );
    supported("Oona's Blackguard");
    supported("Burning-Tree Vandal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oona's Blackguard");
    // Riot: it chooses to enter with a +1/+1 counter, and gets one more.
    let vandal = t.enter(P0, "Burning-Tree Vandal");
    assert_eq!(plus1(&t, vandal), 2);
    // A Rogue that would enter with none enters with one.
    let other = t.enter(P0, "Oona's Blackguard");
    assert_eq!(plus1(&t, other), 1);
    // A non-Rogue gets nothing.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn bramblewood_paragon_gives_warriors_an_additional_counter() {
    cr!("614.1c", "122.6");
    ruling!(
        "Bramblewood Paragon",
        "If a Warrior would normally enter with a certain number of +1/+1 counters on it, it enters with that many +1/+1 counters plus one on it instead. If a Warrior would normally enter with no +1/+1 counters on it, it enters with one +1/+1 counter on it instead."
    );
    supported("Bramblewood Paragon");
    supported("Daghatar the Adamant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bramblewood Paragon");
    let daghatar = t.enter(P0, "Daghatar the Adamant");
    assert_eq!(plus1(&t, daghatar), 5);
    let other = t.enter(P0, "Bramblewood Paragon");
    assert_eq!(plus1(&t, other), 1);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(plus1(&t, bears), 0);
}
