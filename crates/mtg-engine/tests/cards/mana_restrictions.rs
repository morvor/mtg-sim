//! Mana with spending restrictions (CR 106.6): "Spend this mana only to cast a
//! multicolored spell", "... Dragon spells or activate abilities of Dragons", "... creature
//! spells or activate abilities of creatures". The spell purpose is checked against the
//! spell being cast, the ability purpose against the source of the ability being
//! activated; automatic payment doesn't tap a source whose mana can't pay for the cost.

use mtg_engine::mana::ManaType;
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

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

#[test]
fn restricted_lands_compile() {
    assert_supported(&[
        "Pillar of the Paruns",
        "Haven of the Spirit Dragon",
        "Base Camp",
        "Gnarlroot Trapper",
        "Great Hall of the Citadel",
        "Orb of Dragonkind",
        "Castle Garenbrig",
        "Geosurge",
        "Master of Dark Rites",
        "Untaidake, the Cloud Keeper",
    ]);
}

#[test]
fn multicolored_only_mana_pays_for_a_multicolored_spell_only() {
    cr!("106.6", "601.2h");
    let mut t = TestGame::new(2);
    let pillar = t.battlefield(P0, "Pillar of the Paruns");
    // Its mana can't pay for a monocolored spell.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.obj_now(pillar).tapped);
    // It can pay the red part of Lightning Helix.
    t.battlefield(P0, "Plains");
    let helix = t.hand(P0, "Lightning Helix");
    t.cast(P0, helix).target(P1).go();
    assert!(t.obj_now(pillar).tapped);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn automatic_payment_skips_a_source_whose_mana_cant_be_spent() {
    cr!("106.6", "601.2g");
    let mut t = TestGame::new(2);
    let pillar = t.battlefield(P0, "Pillar of the Paruns");
    let mountain = t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.obj_now(mountain).tapped);
    assert!(!t.obj_now(pillar).tapped);
}

#[test]
fn restricted_mana_in_the_pool_stays_for_a_legal_spend() {
    cr!("106.6", "106.4");
    let mut t = TestGame::new(2);
    let pillar = t.battlefield(P0, "Pillar of the Paruns");
    t.answer(P0, DecisionKind::Option, Answer::Index(3)); // red
    t.activate(P0, pillar, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // The red mana can't pay for Lightning Bolt; the Mountain does.
    t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    t.resolve();
    // It pays for the multicolored spell.
    t.battlefield(P0, "Plains");
    let helix = t.hand(P0, "Lightning Helix");
    t.cast(P0, helix).target(P1).go();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn creature_spells_or_abilities_of_creatures() {
    cr!("106.6", "602.2b");
    let mut t = TestGame::new(2);
    let castle = t.battlefield(P0, "Castle Garenbrig");
    t.lands(P0, "Forest", 4);
    t.activate(P0, castle, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G; 6]);
    // A sorcery can't be paid for with it.
    let growth = t.hand(P0, "Rampant Growth");
    assert!(t.cast(P0, growth).try_go().is_err());
    // A creature spell can.
    let boa = t.hand(P0, "River Boa");
    t.cast(P0, boa).go();
    t.resolve();
    assert_eq!(pool(&t, P0), vec![ManaType::G; 4]);
    // So can an ability of a creature ("{G}: Regenerate River Boa")...
    let boa = t.named_on_battlefield("River Boa")[0];
    t.activate(P0, boa, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G; 3]);
    // ... but not an ability of an artifact.
    let stone = t.battlefield(P0, "Mind Stone");
    assert!(t.activate(P0, stone, 1, &[]).is_err());
    assert!(t.on_battlefield(stone));
    assert_eq!(pool(&t, P0), vec![ManaType::G; 3]);
}

#[test]
fn dragon_spells_or_abilities_of_dragons() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Orb of Dragonkind");
    t.battlefield(P0, "Wastes");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, orb, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::R]);
    // Not for a non-Dragon creature spell.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // For an ability of a Dragon.
    let dragon = t.battlefield(P0, "Shivan Dragon");
    t.activate(P0, dragon, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(dragon), (6, 5));
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}

#[test]
fn a_dragon_creature_spell() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let haven = t.battlefield(P0, "Haven of the Spirit Dragon");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, haven, 1, &[]).unwrap();
    t.lands(P0, "Mountain", 5);
    // A Dragon creature spell can use it (with five Mountains, six mana in all).
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    assert!(pool(&t, P0).is_empty());
    // Lightning Bolt couldn't have.
    let mut t = TestGame::new(2);
    let haven = t.battlefield(P0, "Haven of the Spirit Dragon");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, haven, 1, &[]).unwrap();
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
}

#[test]
fn a_spells_restriction_applies_to_all_its_mana() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let geosurge = t.hand(P0, "Geosurge");
    t.cast(P0, geosurge).go();
    t.resolve();
    assert_eq!(pool(&t, P0), vec![ManaType::R; 7]);
    // Seven red mana, only for artifact or creature spells.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}

#[test]
fn creature_type_lists_name_any_of_the_types() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Dark Rites");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, master, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::B; 3]);
    // A Vampire spell ("Vampire, Cleric, and/or Demon spells").
    let vampire = t.hand(P0, "Vampire Nighthawk");
    t.cast(P0, vampire).go();
    assert!(pool(&t, P0).is_empty());
    // Not a Zombie spell.
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Dark Rites");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, master, 0, &[]).unwrap();
    let zombie = t.hand(P0, "Gravedigger");
    assert!(t.cast(P0, zombie).try_go().is_err());
}

#[test]
fn abilities_of_creatures_are_abilities_of_creature_permanents() {
    cr!("106.6", "109.2");
    ruling!(
        "Castle Garenbrig",
        "can't be spent to activate abilities of creature cards that aren't on the battlefield"
    );
    let mut t = TestGame::new(2);
    let castle = t.battlefield(P0, "Castle Garenbrig");
    t.lands(P0, "Forest", 4);
    t.activate(P0, castle, 1, &[]).unwrap();
    // Krosan Tusker's cycling ({2}{G}) is an ability of a creature card in a hand.
    let tusker = t.hand(P0, "Krosan Tusker");
    assert!(t.activate(P0, tusker, 0, &[]).is_err());
    assert!(t.in_hand(P0, "Krosan Tusker"));
    assert_eq!(pool(&t, P0), vec![ManaType::G; 6]);
}

#[test]
fn abilities_of_sources_include_cards() {
    cr!("106.6", "109.2");
    // "... or activate an ability of a Dinosaur source": a Dinosaur card's cycling.
    let mut t = TestGame::new(2);
    let lorekeeper = t.battlefield(P0, "Ixalli's Lorekeeper");
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.activate(P0, lorekeeper, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    t.battlefield(P0, "Wastes");
    let rex = t.hand(P0, "Titanoth Rex");
    t.activate(P0, rex, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Titanoth Rex"));
    assert!(pool(&t, P0).is_empty());
}
