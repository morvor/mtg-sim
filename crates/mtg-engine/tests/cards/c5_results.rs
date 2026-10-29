//! cards5-referents-statics, family `results`: instructions reading the results of earlier
//! steps of the same ability (CR 608.2c, 608.2h).

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn river_song_deals_damage_equal_to_its_power_after_the_counter() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let song = t.battlefield(P0, "River Song");
    t.lands(P1, "Island", 1);
    t.library_top(P1, "Island");
    t.library_top(P1, "Island");
    let opt = t.hand(P1, "Opt");
    t.cast(P1, opt).go();
    t.resolve_all();
    // The counter is put on first: 3 damage, not 2.
    assert_eq!(t.counters(song, "+1/+1"), 1);
    assert_eq!(t.life(P1), 17);
}

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn genesis_of_the_daleks_counts_the_power_of_every_dalek_that_died_this_turn() {
    cr!("714.2b", "608.2h", "700.4");
    supported("Genesis of the Daleks");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let saga = t.battlefield(P0, "Genesis of the Daleks");
    // Chapters I-III at once: three 3/3 Daleks each.
    t.g.add_counters(Entity::Object(saga), "lore", 3, None);
    t.resolve_all();
    let daleks: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Dalek"))
        .map(|o| o.id)
        .collect();
    assert_eq!(daleks.len(), 9);
    // One died earlier this turn, pumped to 5 power: it counts as it last existed.
    t.g.add_counters(Entity::Object(daleks[0]), "+1/+1", 2, None);
    t.g.destroy(daleks[0], None);
    t.settle();
    // Chapter IV: the opponent chooses to destroy the Daleks.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    assert!(t.g.permanents().all(|o| !o.chars.has_subtype("Dalek")));
    assert_eq!(t.life(P1), 20 - (5 + 8 * 3));
}

fn danse(natural: u32, chosen_back: usize) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Danse Macabre");
    t.g.dice.loaded.push_back(natural);
    let back: Vec<Entity> = [giant, bears][..chosen_back]
        .iter()
        .map(|o| Entity::Object(*o))
        .collect();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.answer_choose_moved(P0, &back);
    t.cast(P0, spell).go();
    t.resolve_all();
    (t, bears, giant)
}

#[test]
fn danse_macabre_adds_the_toughness_of_the_creature_you_sacrificed() {
    cr!("706.2", "608.2h");
    ruling!(
        "Danse Macabre",
        "You add only the toughness of the creature you sacrificed this way"
    );
    supported("Danse Macabre");
    // 12 + 2 (the Bears; not the Giant's 3) = 14: one card returns.
    let (t, _, giant) = danse(12, 1);
    assert!(t.on_battlefield(t.g.current(giant)));
    assert_eq!(t.g.obj(t.g.current(giant)).controller, P0);
    // 13 + 2 = 15: up to two.
    let (t, bears, giant) = danse(13, 2);
    assert!(t.on_battlefield(t.g.current(giant)));
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn magma_pummeler_deals_the_prevented_damage_when_counters_are_removed() {
    cr!("603.12", "615.1");
    ruling!(
        "Magma Pummeler",
        "Its reflexive triggered ability still triggers and it will still deal that much damage to any target"
    );
    supported("Magma Pummeler");
    for (counters, bolt, left) in [(3, "Shock", 1), (2, "Lightning Bolt", 0)] {
        let mut t = TestGame::new(2);
        let pummeler = t.battlefield(P0, "Magma Pummeler");
        t.g.add_counters(Entity::Object(pummeler), "+1/+1", counters, None);
        t.lands(P1, "Mountain", 1);
        let spell = t.hand(P1, bolt);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P1, spell).target(Entity::Object(pummeler)).go();
        t.resolve_all();
        let dealt = if bolt == "Shock" { 2 } else { 3 };
        assert_eq!(t.life(P1), 20 - dealt, "{bolt}");
        if left > 0 {
            assert_eq!(t.counters(pummeler, "+1/+1"), left);
        } else {
            assert!(!t.on_battlefield(pummeler));
        }
    }
}

#[test]
fn semesters_end_returns_each_with_the_counter_for_its_type() {
    cr!("614.1c", "122.6", "603.7a");
    ruling!(
        "Semester's End",
        "Use the characteristics of each permanent as it returns to the battlefield"
    );
    supported("Semester's End");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    t.lands(P0, "Plains", 4);
    let spell = t.hand(P0, "Semester's End");
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(ajani)])
        .go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let bears = t.g.current(bears);
    let ajani = t.g.current(ajani);
    assert!(t.on_battlefield(bears) && t.on_battlefield(ajani));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "loyalty"), 0);
    assert_eq!(t.counters(ajani, "loyalty"), 5);
    assert_eq!(t.counters(ajani, "+1/+1"), 0);
}

#[test]
fn ensnared_by_the_mara_deals_the_total_mana_value_of_the_exiled_cards() {
    cr!("701.55a", "608.2c");
    supported("Ensnared by the Mara");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    for c in ["Island", "Lightning Bolt", "Island", "Grizzly Bears", "Hill Giant"] {
        t.library_top(P1, c);
    }
    t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Ensnared by the Mara");
    // The second option: exile the top four (4 + 2 + 0 + 1).
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 7);
    assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
}

#[test]
fn saruman_of_many_colors_reflexive_trigger_needs_a_card_milled() {
    cr!("603.12", "701.17a", "707.12");
    ruling!(
        "Saruman of Many Colors",
        "ability triggers when one or more cards are milled this way"
    );
    supported("Saruman of Many Colors");
    let run = |library: &[&str]| {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.battlefield(P0, "Saruman of Many Colors");
        for c in library {
            t.library_top(P1, c);
        }
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Mountain", 4);
        for _ in 0..3 {
            t.library_top(P0, "Island");
        }
        let elves = t.hand(P0, "Llanowar Elves");
        t.cast(P0, elves).go();
        t.resolve_all();
        let giant = t.hand(P0, "Hill Giant");
        t.answer_yes(P0, true);
        t.cast(P0, giant).go();
        t.resolve_all();
        t
    };
    // Divination (3) milled by the second spell, Hill Giant (4): exiled, copied and cast.
    let t = run(&["Divination", "Island"]);
    assert!(t.in_exile("Divination"));
    assert_eq!(t.hand_size(P0), 2);
    // Nothing to mill: no reflexive trigger, nothing exiled.
    let t = run(&[]);
    assert_eq!(t.hand_size(P0), 0);
}
