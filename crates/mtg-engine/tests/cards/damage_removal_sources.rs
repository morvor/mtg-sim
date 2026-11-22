//! "A source of your choice" prevention and redirection (CR 609.7, 615), compiled by
//! `src/oracle/patterns/damage_removal_sources.rs`.

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The candidates offered by the last "choose a source" decision.
fn source_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if q == p && prompt.contains("source") => Some(candidates),
            _ => None,
        })
        .last()
        .expect("no source choice")
}

#[test]
fn source_of_your_choice_cards_compile() {
    assert_compiles(&[
        "Circle of Protection: Red",
        "Circle of Protection: Artifacts",
        "Rune of Protection: Lands",
        "Greater Realm of Preservation",
        "Story Circle",
        "Cho-Arrim Alchemist",
        "Reverse Damage",
        "Beacon of Destiny",
        "Charm Peddler",
        "Pay No Heed",
        "Auriok Replica",
        "Burrenton Forge-Tender",
        "General's Regalia",
    ]);
}

#[test]
fn circle_of_protection_red_prevents_the_next_damage_from_the_chosen_red_source() {
    cr!("609.7a", "615.1a", "615.7");
    let mut t = TestGame::new(2);
    let cop = t.battlefield(P0, "Circle of Protection: Red");
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    // In response, choose the Bolt as the red source.
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, cop, 0, &[]).unwrap();
    t.resolve();
    // Only red sources could be chosen: the Bolt, not the green Bears.
    assert_eq!(source_candidates(&t, P0), vec![Entity::Object(bolt)]);
    t.resolve();
    assert_eq!(t.life(P0), 20, "prevented");
    // The shield is used up; the next red source isn't prevented.
    let bolt2 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt2).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn circle_of_protection_applies_only_to_the_chosen_source() {
    cr!("609.7b");
    let mut t = TestGame::new(2);
    let cop = t.battlefield(P0, "Circle of Protection: Red");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Mountain", 2);
    let b1 = t.hand(P1, "Lightning Bolt");
    let b2 = t.hand(P1, "Lightning Bolt");
    let b1 = t.cast(P1, b1).target(P0).go();
    t.cast(P1, b2).target(P0).go();
    // Choose the first Bolt (the one lower on the stack).
    t.answer_choose(P0, &[Entity::Object(b1)]);
    t.activate(P0, cop, 0, &[]).unwrap();
    t.resolve();
    // The second Bolt resolves first and isn't prevented.
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve();
    assert_eq!(t.life(P0), 17, "the chosen Bolt was prevented");
}

#[test]
fn cho_arrim_alchemist_gains_life_equal_to_the_damage_prevented() {
    cr!("615.5", "609.7a");
    let mut t = TestGame::new(2);
    let alchemist = t.battlefield(P0, "Cho-Arrim Alchemist");
    t.lands(P0, "Plains", 3);
    let discard = t.hand(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, alchemist, 0, &[]).unwrap();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P0), 23, "3 prevented, then 3 life gained");
}

#[test]
fn beacon_of_destiny_redirects_the_damage_to_itself() {
    cr!("614.9", "609.7a");
    let mut t = TestGame::new(2);
    let beacon = t.battlefield(P0, "Beacon of Destiny");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, beacon, 0, &[]).unwrap();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    // Beacon of Destiny is a 1/3: 3 damage destroys it.
    assert!(!t.on_battlefield(beacon));
}

#[test]
fn charm_peddler_protects_the_target_creature_from_the_chosen_source() {
    cr!("609.7a", "615.1a");
    let mut t = TestGame::new(2);
    let peddler = t.battlefield(P0, "Charm Peddler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let discard = t.hand(P0, "Island");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(bears).go();
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, peddler, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn pay_no_heed_prevents_all_damage_the_chosen_source_would_deal() {
    cr!("609.7a", "615.1a");
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Mountain", 2);
    let pyroclasm = t.hand(P1, "Pyroclasm");
    let pyroclasm = t.cast(P1, pyroclasm).go();
    let heed = t.hand(P0, "Pay No Heed");
    t.answer_choose(P0, &[Entity::Object(pyroclasm)]);
    t.cast(P0, heed).go();
    t.resolve();
    t.resolve();
    // All of the chosen source's damage is prevented, to every creature.
    assert!(t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn generals_regalia_redirects_the_damage_to_the_target_creature() {
    cr!("614.9", "609.7a");
    let mut t = TestGame::new(2);
    let regalia = t.battlefield(P0, "General's Regalia");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.lands(P0, "Plains", 3);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, regalia, 0, &[Entity::Object(wall)]).unwrap();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(wall).damage, 3);
}
