//! Structure-coverage seed tests: activated-ability and mana-ability structures shared by
//! many cards (see `docs/STRUCTURE_COVERAGE.md`). Each test checks a real card's ability
//! does what its text says.

use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn checkpoint_officer_taps_target_creature() {
    cr!("602.2", "701.26a");
    // "{1}{W}, {T}: Tap target creature."
    supported("Checkpoint Officer");
    let mut t = TestGame::new(2);
    let officer = t.battlefield(P0, "Checkpoint Officer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, officer, 0, &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(officer).tapped, "{{T}} is part of the cost");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn soothsayer_adept_loots() {
    cr!("602.2", "121.1", "701.9a");
    // "{1}{U}, {T}: Draw a card, then discard a card."
    supported("Soothsayer Adept");
    let mut t = TestGame::new(2);
    let adept = t.battlefield(P0, "Soothsayer Adept");
    t.lands(P0, "Island", 2);
    t.hand(P0, "Grizzly Bears");
    t.activate(P0, adept, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn stone_kavu_pumps_its_toughness() {
    cr!("602.2", "611.2a", "613.4c");
    // "{W}: This creature gets +0/+1 until end of turn."
    supported("Stone Kavu");
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Stone Kavu");
    t.lands(P0, "Plains", 1);
    t.activate(P0, kavu, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(kavu), (3, 4));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(kavu), (3, 3), "until end of turn");
}

#[test]
fn saltcrusted_steppe_stores_a_counter() {
    cr!("602.2", "122.6");
    // "{1}, {T}: Put a storage counter on this land."
    supported("Saltcrusted Steppe");
    let mut t = TestGame::new(2);
    let steppe = t.battlefield(P0, "Saltcrusted Steppe");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, steppe, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(steppe, "storage"), 1);
    assert!(t.obj_now(steppe).tapped);
}

#[test]
fn the_sound_of_drums_returns_from_the_graveyard_to_hand() {
    cr!("602.2", "113.6");
    // "{2}{R}: Return this card from your graveyard to your hand."
    supported("The Sound of Drums");
    let mut t = TestGame::new(2);
    let aura = t.graveyard(P0, "The Sound of Drums");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, aura, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "The Sound of Drums"));
}

#[test]
fn slaughter_drone_gains_deathtouch_for_colorless_mana() {
    cr!("602.2", "611.2a", "106.1b");
    // "{C}: This creature gains deathtouch until end of turn."
    supported("Slaughter Drone");
    let mut t = TestGame::new(2);
    let drone = t.battlefield(P0, "Slaughter Drone");
    t.lands(P0, "Swamp", 1);
    assert!(
        t.activate(P0, drone, 0, &[]).is_err(),
        "{{C}} can't be paid with colored mana"
    );
    t.lands(P0, "Wastes", 1);
    t.activate(P0, drone, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(drone).has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn unyielding_krumar_gains_first_strike() {
    cr!("602.2", "611.2a");
    // "{1}{W}: This creature gains first strike until end of turn."
    supported("Unyielding Krumar");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Unyielding Krumar");
    t.lands(P0, "Plains", 2);
    t.activate(P0, k, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(k).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn savage_mansion_surveils_1() {
    cr!("602.2", "701.25a");
    // "{4}, {T}: Surveil 1."
    supported("Savage Mansion");
    let mut t = TestGame::new(2);
    let mansion = t.battlefield(P0, "Savage Mansion");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, mansion, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(surveils(&t), vec![1]);
}

#[test]
fn quandrix_campus_scries_1() {
    cr!("602.2", "701.22a");
    // "{4}, {T}: Scry 1."
    supported("Quandrix Campus");
    let mut t = TestGame::new(2);
    let campus = t.battlefield(P0, "Quandrix Campus");
    t.lands(P0, "Forest", 4);
    t.activate(P0, campus, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(scries(&t), vec![1]);
}

#[test]
fn glimmerbell_untaps_itself() {
    cr!("602.2", "701.26b");
    // "{1}{U}: Untap this creature."
    supported("Glimmerbell");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Glimmerbell");
    t.g.tap(g);
    t.lands(P0, "Island", 2);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(g).tapped);
}

#[test]
fn water_tribe_captain_pumps_creatures_you_control() {
    cr!("602.2", "611.2c");
    // "{5}: Creatures you control get +1/+1 until end of turn."
    supported("Water Tribe Captain");
    let mut t = TestGame::new(2);
    let cap = t.battlefield(P0, "Water Tribe Captain");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    t.activate(P0, cap, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(cap), (4, 4));
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn lightning_spear_sacrifices_itself_to_deal_damage() {
    cr!("602.2", "701.21a");
    // "{2}{R}, Sacrifice this Equipment: It deals 3 damage to any target."
    supported("Lightning Spear");
    let mut t = TestGame::new(2);
    let spear = t.battlefield(P0, "Lightning Spear");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, spear, 0, &[Entity::Player(P1)]).unwrap();
    assert!(!t.on_battlefield(spear), "sacrificed as a cost");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn qasali_pridemage_destroys_an_artifact() {
    cr!("602.2", "701.8a");
    // "{1}, Sacrifice this creature: Destroy target artifact or enchantment."
    supported("Qasali Pridemage");
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Qasali Pridemage");
    let obelisk = t.battlefield(P1, "Obelisk of Naya");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.activate(P0, cat, 0, &[Entity::Object(obelisk)]).unwrap();
    assert!(!t.on_battlefield(cat), "sacrificed as a cost");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(obelisk));
    assert!(t.in_graveyard(P1, "Obelisk of Naya"));
}

#[test]
fn harbor_bandit_cant_be_blocked_this_turn() {
    cr!("602.2", "509.1b");
    // "{1}{U}: This creature can't be blocked this turn."
    supported("Harbor Bandit");
    let mut t = TestGame::new(2);
    let bandit = t.battlefield(P0, "Harbor Bandit");
    let wall = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.activate(P0, bandit, 0, &[]).unwrap();
    t.resolve_all();
    // +1/+1 as long as you control an Island.
    assert_eq!(t.pt(bandit), (3, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bandit, Entity::Player(P1))], &[(wall, bandit)]);
    assert_eq!(t.life(P1), 17, "the block was illegal");
}

#[test]
fn thallid_germinator_removes_spore_counters_for_a_saproling() {
    cr!("602.2", "122.8");
    // "Remove three spore counters from this creature: Create a 1/1 green Saproling
    // creature token."
    supported("Thallid Germinator");
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Thallid Germinator");
    assert!(t.activate(P0, th, 0, &[]).is_err(), "no counters to remove");
    t.g.add_counters(Entity::Object(th), "spore", 3, None);
    t.activate(P0, th, 0, &[]).unwrap();
    assert_eq!(t.counters(th, "spore"), 0);
    t.resolve_all();
    let s = subtype_ids(&t, P0, "Saproling");
    assert_eq!(s.len(), 1);
    assert_eq!(t.pt(s[0]), (1, 1));
}

#[test]
fn dimir_locket_draws_two_paid_with_hybrid_mana() {
    cr!("602.2", "107.4e");
    // "{U/B}{U/B}{U/B}{U/B}, {T}, Sacrifice this artifact: Draw two cards."
    supported("Dimir Locket");
    let mut t = TestGame::new(2);
    let locket = t.battlefield(P0, "Dimir Locket");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    t.activate(P0, locket, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert!(!t.on_battlefield(locket));
}

#[test]
fn obelisk_of_naya_adds_one_of_three_colors() {
    cr!("605.1a", "605.3a", "106.4");
    // "{T}: Add {R}, {G}, or {W}."
    supported("Obelisk of Naya");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Obelisk of Naya");
    t.activate(P0, o, 0, &[]).unwrap();
    let pool = &t.g.player(P0).mana_pool;
    let (r, g, w) = (
        pool.count(ManaType::R),
        pool.count(ManaType::G),
        pool.count(ManaType::W),
    );
    assert_eq!(r + g + w, 1);
    assert_eq!(pool.mana.len(), 1);
    assert!(t.obj_now(o).tapped);
}

#[test]
fn selesnya_sanctuary_adds_two_mana() {
    cr!("605.1a", "605.3a");
    // "{T}: Add {G}{W}."
    supported("Selesnya Sanctuary");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Selesnya Sanctuary");
    t.activate(P0, s, 0, &[]).unwrap();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.count(ManaType::G), 1);
    assert_eq!(pool.count(ManaType::W), 1);
}
