//! Permanents entering with counters (CR 614.1c, 614.12, 122.6): other permanents
//! entering with counters counted "for each [permanent] you already control", with a
//! number from the source's power and a creature type, face-down creatures entering with
//! a keyword counter, and a spell modified by a cast trigger entering with counters.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn giada_counts_the_angels_you_already_control() {
    cr!("614.1c", "614.12");
    ruling!(
        "Giada, Font of Hope",
        "\"Each Angel you already control\" means each Angel you control other than the Angel entering, including Giada."
    );
    supported("Giada, Font of Hope");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Giada, Font of Hope");
    t.battlefield(P0, "Serra Angel");
    let angel = t.enter(P0, "Serra Angel");
    assert_eq!(t.counters(angel, "+1/+1"), 2);
    // Not an Angel: no counters.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn master_biomancer_gives_counters_equal_to_its_power_and_the_mutant_type() {
    cr!("614.1c", "614.12", "205.1b");
    ruling!(
        "Master Biomancer",
        "use Master Biomancer's power as that creature is entering"
    );
    supported("Master Biomancer");
    let mut t = TestGame::new(2);
    let bio = t.battlefield(P0, "Master Biomancer");
    t.g.add_counters(Entity::Object(bio), "+1/+1", 1, None);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    assert!(t.obj_now(bears).chars.has_subtype("Mutant"));
    assert!(t.obj_now(bears).chars.has_subtype("Bear"));
    // Not Master Biomancer itself, nor an opponent's creature.
    assert!(!t.obj_now(bio).chars.has_subtype("Mutant"));
    let theirs = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert!(!t.obj_now(theirs).chars.has_subtype("Mutant"));
}

#[test]
fn veiled_ascension_cloaked_creatures_enter_with_a_flying_counter() {
    cr!("614.1c", "701.58a");
    supported("Veiled Ascension");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Veiled Ascension");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let face_down: Vec<ObjectId> =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .filter(|o| t.g.obj(*o).face_down)
            .collect();
    assert_eq!(face_down.len(), 1);
    assert_eq!(t.counters(face_down[0], "flying"), 1);
    // A face-up creature doesn't.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "flying"), 0);
}

#[test]
fn torgal_counts_dogs_and_wolves_for_the_first_human_spell() {
    cr!("614.1c", "603.2");
    supported("Torgal, A Fine Hound");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Torgal, A Fine Hound");
    t.battlefield(P0, "Timber Wolves");
    t.lands(P0, "Plains", 4);
    let a = t.hand(P0, "Elite Vanguard");
    t.cast(P0, a).go();
    t.resolve_all();
    let first = t.named_on_battlefield("Elite Vanguard")[0];
    // Torgal (a Wolf) and Timber Wolves.
    assert_eq!(t.counters(first, "+1/+1"), 2);
    // Only the first Human creature spell each turn.
    let b = t.hand(P0, "Elite Vanguard");
    t.cast(P0, b).go();
    t.resolve_all();
    let all = t.named_on_battlefield("Elite Vanguard");
    let second = all.into_iter().find(|o| *o != t.g.current(first)).unwrap();
    assert_eq!(t.counters(second, "+1/+1"), 0);
}

#[test]
fn gluttonous_hellkite_enters_with_two_counters_per_creature_sacrificed() {
    cr!("614.1c", "608.2h", "603.2");
    supported("Gluttonous Hellkite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let h = t.hand(P0, "Gluttonous Hellkite");
    t.cast(P0, h).x(1).go();
    t.resolve_all();
    // One creature from each player: four counters.
    let hk = t.named_on_battlefield("Gluttonous Hellkite")[0];
    assert_eq!(t.counters(hk, "+1/+1"), 4);
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 1);
}

#[test]
fn dark_intimations_gives_the_bolas_planeswalker_an_additional_loyalty_counter() {
    cr!("614.1c", "306.5b");
    supported("Dark Intimations");
    let mut t = TestGame::new(2);
    let di = t.graveyard(P0, "Dark Intimations");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let nb = t.hand(P0, "Nicol Bolas, the Deceiver");
    t.cast(P0, nb).go();
    t.resolve_all();
    let pw = t.named_on_battlefield("Nicol Bolas, the Deceiver")[0];
    assert_eq!(t.counters(pw, "loyalty"), 6);
    assert!(t.in_exile("Dark Intimations"));
    let _ = di;
}

#[test]
fn blue_loyal_raptor_gives_entering_dinosaurs_one_counter_of_each_kind_it_has() {
    cr!("614.1c", "122.6");
    supported("Blue, Loyal Raptor");
    let mut t = TestGame::new(2);
    let blue = t.battlefield(P0, "Blue, Loyal Raptor");
    t.g.add_counters(Entity::Object(blue), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(blue), "flying", 1, None);
    let dino = t.enter(P0, "Ripjaw Raptor");
    assert_eq!(t.counters(dino, "+1/+1"), 1);
    assert_eq!(t.counters(dino, "flying"), 1);
    // Not a Dinosaur: nothing.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn naya_soulbeast_enters_with_the_total_mana_value_revealed() {
    cr!("614.1c", "608.2h", "701.20a");
    supported("Naya Soulbeast");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Hill Giant");
    t.library_top(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 8);
    let s = t.hand(P0, "Naya Soulbeast");
    t.cast(P0, s).go();
    t.resolve_all();
    let nb = t.named_on_battlefield("Naya Soulbeast")[0];
    assert_eq!(t.counters(nb, "+1/+1"), 4 + 2);
}
