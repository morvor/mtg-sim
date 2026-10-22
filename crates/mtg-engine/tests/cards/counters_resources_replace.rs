//! Replacement effects on amounts of life and counters (CR 614.1a, 614.16): "If you would
//! gain life, you gain twice that much life instead", "... that many plus one +1/+1
//! counters are put on it instead", "If you would get one or more counters, ...".

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
}

#[test]
fn rhox_faithmender_doubles_life_gained() {
    cr!("614.1a", "119.3");
    ruling!(
        "Rhox Faithmender",
        "If you control two Rhox Faithmenders, life you gain will be multiplied by four."
    );
    assert_supported(&["Rhox Faithmender"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhox Faithmender");
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P0), 26);
    // Opponents' life gain isn't affected.
    t.g.gain_life(P1, 3);
    assert_eq!(t.life(P1), 23);
    t.battlefield(P0, "Rhox Faithmender");
    t.g.gain_life(P0, 1);
    assert_eq!(t.life(P0), 30);
}

#[test]
fn heron_of_hope_adds_one_to_life_gained() {
    cr!("614.1a");
    assert_supported(&["Heron of Hope"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heron of Hope");
    t.g.gain_life(P0, 2);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn sulfuric_vortex_stops_all_life_gain() {
    cr!("614.1a", "119.3");
    assert_supported(&["Sulfuric Vortex"]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sulfuric Vortex");
    t.g.gain_life(P0, 5);
    t.g.gain_life(P1, 5);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn hardened_scales_adds_a_counter_to_each_placement() {
    cr!("614.16", "122.6");
    ruling!(
        "Hardened Scales",
        "If a creature you control would enter the battlefield with a number of +1/+1 counters on it, it enters with that many plus one instead."
    );
    assert_supported(&["Hardened Scales"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let tp = t.hand(P0, "Travel Preparations");
    t.cast(P0, tp)
        .targets(&[Entity::Object(bear), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    // Only creatures you control.
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    // Entering with counters counts as putting them on it.
    let hydra = t.enter(P0, "Kalonian Hydra");
    assert_eq!(t.counters(hydra, "+1/+1"), 5);
}

#[test]
fn two_corpsejack_menaces_quadruple_counters() {
    cr!("614.16", "616.1");
    ruling!(
        "Corpsejack Menace",
        "If you control two Corpsejack Menaces, the number of +1/+1 counters put on a creature is four times the original number."
    );
    assert_supported(&["Corpsejack Menace"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corpsejack Menace");
    t.battlefield(P0, "Corpsejack Menace");
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, "+1/+1", 1);
    assert_eq!(t.counters(bear, "+1/+1"), 4);
    // Other kinds of counters aren't affected.
    add(&mut t, bear, "flying", 1);
    assert_eq!(t.counters(bear, "flying"), 1);
}

#[test]
fn winding_constrictor_adds_to_each_kind_and_to_player_counters() {
    cr!("614.16", "107.14", "701.34a");
    ruling!(
        "Winding Constrictor",
        "If you would get counters of multiple kinds at the same time, Winding Constrictor increases the number of each of those kinds of counters by one."
    );
    assert_supported(&["Winding Constrictor", "Thriving Rats"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Winding Constrictor");
    // "When this creature enters, you get {E}{E}": three instead.
    t.enter(P0, "Thriving Rats");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 3);
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, "+1/+1", 1);
    add(&mut t, bear, "flying", 1);
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    assert_eq!(t.counters(bear, "flying"), 2);
    // Not an opponent's creature, nor an opponent.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, theirs, "+1/+1", 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    t.g.add_counters(Entity::Player(P1), "poison", 1, None);
    assert_eq!(t.g.player(P1).counter("poison"), 1);
    // Several kinds at the same time (proliferate): one more of each kind, on the
    // creature and on the player.
    t.g.add_counters(Entity::Player(P0), "experience", 1, None);
    assert_eq!(t.g.player(P0).counter("experience"), 2);
    t.lands(P0, "Island", 3);
    let sp = t.hand(P0, "Steady Progress");
    t.answer_choose(P0, &[Entity::Object(bear), Entity::Player(P0)]);
    t.cast(P0, sp).go();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 4);
    assert_eq!(t.counters(bear, "flying"), 4);
    assert_eq!(t.g.player(P0).counter("energy"), 5);
    assert_eq!(t.g.player(P0).counter("experience"), 4);
}

#[test]
fn vizier_of_remedies_puts_one_fewer_minus_counter() {
    cr!("614.16");
    ruling!(
        "Vizier of Remedies",
        "Each additional Vizier of Remedies you control will decrease the number of -1/-1 counters put on a creature by one."
    );
    assert_supported(&["Vizier of Remedies"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of Remedies");
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, "-1/-1", 2);
    assert_eq!(t.counters(bear, "-1/-1"), 1);
    // A single counter is reduced to none.
    add(&mut t, bear, "-1/-1", 1);
    assert_eq!(t.counters(bear, "-1/-1"), 1);
    t.battlefield(P0, "Vizier of Remedies");
    add(&mut t, bear, "-1/-1", 3);
    assert_eq!(t.counters(bear, "-1/-1"), 2);
}
