//! Noted mana (CR 607.2e, `src/noted_mana.rs`): Jeweled Amulet notes the type of mana
//! spent to activate it and adds one mana of that type; Ice Cauldron notes the type and
//! amount.

use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

#[test]
fn jeweled_amulet_stores_mana_of_the_type_spent_to_charge_it() {
    cr!("607.2e");
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Jeweled Amulet");
    t.lands(P0, "Forest", 1);
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(amulet, "charge"), 1);
    // A second charge isn't allowed while it has a charge counter.
    t.g.untap(amulet);
    assert!(t.activate(P0, amulet, 0, &[]).is_err());
    t.activate(P0, amulet, 1, &[]).unwrap();
    assert_eq!(t.counters(amulet, "charge"), 0);
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
}

#[test]
fn jeweled_amulet_uses_the_last_noted_type_and_colorless_can_be_noted() {
    cr!("607.2e");
    ruling!("Jeweled Amulet", "The types of mana are white, blue, black, red, green, and colorless.");
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Jeweled Amulet");
    t.lands(P0, "Island", 1);
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    t.g.untap(amulet);
    t.activate(P0, amulet, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::U]);
    t.g.players[P0.idx()].mana_pool.mana.clear();
    t.g.untap(amulet);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    t.g.untap(amulet);
    t.activate(P0, amulet, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::C]);
}

#[test]
fn jeweled_amulet_with_no_type_noted_adds_no_mana() {
    cr!("607.5a");
    ruling!("Jeweled Amulet", "the second ability will produce no mana");
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Jeweled Amulet");
    t.g.add_counters(Entity::Object(amulet), "charge", 1, None);
    t.activate(P0, amulet, 1, &[]).unwrap();
    assert_eq!(t.counters(amulet, "charge"), 0);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn a_new_object_has_no_note() {
    cr!("400.7", "607.5a");
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Jeweled Amulet");
    t.lands(P0, "Forest", 1);
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    // It's flickered: the new object keeps no note, even with a charge counter.
    let ex = t.g.exile_object(amulet, None).unwrap();
    let new = t
        .g
        .move_object(ex, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.add_counters(Entity::Object(new), "charge", 1, None);
    t.g.untap(new);
    t.activate(P0, new, 1, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn with_several_types_spent_its_controller_chooses_the_one_noted() {
    cr!("607.2e");
    interp!("jeweled-amulet-several-types");
    let mut t = TestGame::new(2);
    // "Activated abilities cost {2} more to activate unless they're mana abilities."
    t.battlefield(P1, "Suppression Field");
    let amulet = t.battlefield(P0, "Jeweled Amulet");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 2);
    // Types are offered in WUBRGC order: {U}, then {G}.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    t.g.untap(amulet);
    t.activate(P0, amulet, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
}
