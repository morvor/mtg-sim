//! Structure-coverage tests, round 2 (item 3): static- and replacement-ability structures
//! shared by many cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).
//! Each test checks a real card's ability does what its text says, including when it
//! doesn't apply.

use super::structure_activated_2::target_choices;
use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Plays the land `name` from P0's hand and returns it.
fn play(t: &mut TestGame, name: &str) -> ObjectId {
    let land = t.hand(P0, name);
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    t.g.current(land)
}

/// Casts the Aura `name` from P0's hand onto `target` (paying with `land`s) and resolves it.
fn enchant(t: &mut TestGame, name: &str, land: &str, n: usize, target: ObjectId) -> ObjectId {
    t.lands(P0, land, n);
    let aura = t.hand(P0, name);
    t.cast(P0, aura).target(target).go();
    t.resolve_all();
    assert!(t.on_battlefield(aura), "{name} resolved");
    t.g.current(aura)
}

// ---------------------------------------------------------------------------
// "Enters tapped" replacement effects
// ---------------------------------------------------------------------------

#[test]
fn hall_of_storm_giants_enters_tapped_with_two_other_lands() {
    cr!("614.1d", "305.1");
    // "If you control two or more other lands, this land enters tapped."
    supported("Hall of Storm Giants");
    for (others, tapped) in [(0, false), (1, false), (2, true), (3, true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", others);
        // An opponent's lands don't count.
        t.lands(P1, "Island", 3);
        let hall = play(&mut t, "Hall of Storm Giants");
        assert_eq!(t.obj_now(hall).tapped, tapped, "{others} other lands");
    }
}

#[test]
fn reef_roads_enters_tapped_unless_you_control_a_mount_or_vehicle() {
    cr!("614.1d", "305.1");
    // "This land enters tapped unless you control a Mount or Vehicle."
    supported("Reef Roads");
    // (your permanent, opponent's permanent, tapped)
    let cases: [(Option<&str>, Option<&str>, bool); 5] = [
        (None, None, true),
        (Some("Grizzly Bears"), None, true),
        (None, Some("Caustic Bronco"), true),
        (Some("Caustic Bronco"), None, false),
        (Some("Smuggler's Copter"), None, false),
    ];
    for (mine, theirs, tapped) in cases {
        let mut t = TestGame::new(2);
        if let Some(m) = mine {
            t.battlefield(P0, m);
        }
        if let Some(o) = theirs {
            t.battlefield(P1, o);
        }
        let roads = play(&mut t, "Reef Roads");
        assert_eq!(t.obj_now(roads).tapped, tapped, "{mine:?} / {theirs:?}");
    }
}

#[test]
fn sleep_cursed_faerie_enters_tapped_with_three_stun_counters() {
    cr!("614.1c", "122.1d");
    // "This creature enters tapped with three stun counters on it."
    supported("Sleep-Cursed Faerie");
    let mut t = TestGame::new(2);
    let f = t.enter(P0, "Sleep-Cursed Faerie");
    t.resolve_all();
    assert!(t.obj_now(f).tapped);
    assert_eq!(t.counters(f, "stun"), 3);
    // Each untap step removes a stun counter instead of untapping it.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(f).tapped);
    assert_eq!(t.counters(f, "stun"), 2);
}

#[test]
fn myojin_of_towering_might_gets_an_indestructible_counter_only_if_cast_from_hand() {
    cr!("614.1c", "122.1b", "702.12b");
    // "Myojin of Towering Might enters with an indestructible counter on it if you cast it
    // from your hand."
    supported("Myojin of Towering Might");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 8);
    let m = t.hand(P0, "Myojin of Towering Might");
    t.cast(P0, m).go();
    t.resolve_all();
    assert!(t.on_battlefield(m));
    assert_eq!(t.counters(m, "indestructible"), 1);
    assert!(t.obj_now(m).has_keyword(KeywordKind::Indestructible));
    t.g.destroy(t.g.current(m), None);
    t.resolve_all();
    assert!(t.on_battlefield(m), "indestructible");
    // Put onto the battlefield without being cast: no counter.
    let mut t = TestGame::new(2);
    let m = t.enter(P0, "Myojin of Towering Might");
    t.resolve_all();
    assert_eq!(t.counters(m, "indestructible"), 0);
    assert!(!t.obj_now(m).has_keyword(KeywordKind::Indestructible));
}

// ---------------------------------------------------------------------------
// Untap restrictions
// ---------------------------------------------------------------------------

#[test]
fn veldt_doesnt_untap_while_it_has_a_depletion_counter() {
    cr!("502.3");
    // "This land doesn't untap during your untap step if it has a depletion counter on
    // it. / At the beginning of your upkeep, remove a depletion counter from this land."
    supported("Veldt");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Veldt");
    // Tapping it for mana to cast a spell puts the depletion counter on it.
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    assert!(t.on_battlefield(elves));
    assert!(t.obj_now(v).tapped);
    assert_eq!(t.counters(v, "depletion"), 1);
    // A tapped Veldt without a depletion counter untaps normally.
    let plain = t.battlefield(P0, "Veldt");
    t.g.tap(plain);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(
        t.obj_now(v).tapped,
        "had a depletion counter during the untap step"
    );
    assert!(!t.obj_now(plain).tapped);
    t.resolve_all();
    assert_eq!(t.counters(v, "depletion"), 0, "removed in the upkeep");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(v).tapped, "no counter now");
}

// ---------------------------------------------------------------------------
// Characteristic-defining abilities
// ---------------------------------------------------------------------------

#[test]
fn horde_of_boggarts_counts_red_permanents_you_control() {
    cr!("604.3", "613.4a");
    // "Horde of Boggarts's power and toughness are each equal to the number of red
    // permanents you control."
    supported("Horde of Boggarts");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Horde of Boggarts");
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Raging Goblin");
    assert_eq!(t.pt(h), (1, 1), "only itself");
    let gob = t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Hematite Talisman");
    assert_eq!(t.pt(h), (2, 2), "the Goblin; the Talisman is colorless");
    t.g.destroy(gob, None);
    t.resolve_all();
    assert_eq!(t.pt(h), (1, 1));
}

// ---------------------------------------------------------------------------
// Anthems
// ---------------------------------------------------------------------------

#[test]
fn waker_of_waves_gives_opponents_creatures_minus_1_power() {
    cr!("611.3a", "613.4c");
    // "Creatures your opponents control get -1/-0."
    supported("Waker of Waves");
    let mut t = TestGame::new(3);
    let w = t.battlefield(P0, "Waker of Waves");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    assert_eq!(t.pt(w), (7, 7));
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(a), (1, 2));
    assert_eq!(t.pt(b), (2, 3));
    // A creature entering later is affected too.
    let late = t.battlefield(P1, "Llanowar Elves");
    assert_eq!(t.pt(late), (0, 1));
    t.g.destroy(w, None);
    t.resolve_all();
    assert_eq!(t.pt(a), (2, 2));
}

#[test]
fn balthor_the_stout_pumps_every_other_barbarian() {
    cr!("611.3a", "613.4c");
    // "Other Barbarian creatures get +1/+1."
    supported("Balthor the Stout");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Balthor the Stout");
    let mine = t.battlefield(P0, "Keldon Champion");
    let theirs = t.battlefield(P1, "Keldon Champion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(b), (2, 2), "other");
    assert_eq!(t.pt(mine), (4, 3));
    assert_eq!(t.pt(theirs), (4, 3), "not just yours");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn spirit_lords_pump_only_other_spirits_you_control() {
    cr!("611.3a", "613.4c");
    supported("Battleground Geist");
    supported("Gallows Warden");
    // Battleground Geist: "Other Spirit creatures you control get +1/+0."
    // Gallows Warden: "Other Spirit creatures you control get +0/+1."
    for (lord, base, bonus) in [
        ("Battleground Geist", (3, 3), (1, 0)),
        ("Gallows Warden", (3, 3), (0, 1)),
    ] {
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let spirit = t.battlefield(P0, "Mausoleum Wanderer");
        let theirs = t.battlefield(P1, "Mausoleum Wanderer");
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(t.pt(l), base, "{lord}: other");
        assert_eq!(t.pt(spirit), (1 + bonus.0, 1 + bonus.1), "{lord}");
        assert_eq!(t.pt(theirs), (1, 1), "{lord}: an opponent's Spirit");
        assert_eq!(t.pt(bears), (2, 2), "{lord}: not a Spirit");
    }
}

#[test]
fn escarpment_fortress_gives_your_other_creatures_plus_1_power() {
    cr!("611.3a", "613.4c");
    // "Other creatures you control get +1/+0."
    supported("Escarpment Fortress");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Escarpment Fortress");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(f), (3, 5));
    assert_eq!(t.pt(mine), (3, 2));
    assert_eq!(t.pt(theirs), (2, 2));
}

// ---------------------------------------------------------------------------
// Auras granting abilities
// ---------------------------------------------------------------------------

#[test]
fn sleepers_robe_gives_the_enchanted_creature_fear() {
    cr!("702.36b", "613.1f");
    // "Enchanted creature has fear."
    supported("Sleeper's Robe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let green = t.battlefield(P1, "Grizzly Bears");
    let black = t.battlefield(P1, "Walking Corpse");
    let artifact = t.battlefield(P1, "Ornithopter");
    // {U}{B}: an Island plus the Swamp `enchant` adds.
    t.lands(P0, "Island", 1);
    enchant(&mut t, "Sleeper's Robe", "Swamp", 1, bears);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Fear));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Fear));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(green, bears));
    assert!(t.g.can_block(black, bears));
    assert!(t.g.can_block(artifact, bears));
    assert!(t.g.can_block(green, other));
}

#[test]
fn crystal_carapace_gives_plus_3_and_ward_2() {
    cr!("613.4c", "613.1f", "702.21a");
    // "Enchanted creature gets +3/+3 and has ward {2}."
    supported("Crystal Carapace");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, "Crystal Carapace", "Forest", 4, bears);
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(other), (2, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Ward));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Ward));
    // An opponent who can't pay {2} has their spell countered.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // Targeting the unenchanted creature works.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(other).go();
    t.resolve_all();
    assert!(!t.on_battlefield(other));
}

#[test]
fn muzzle_prevents_all_damage_dealt_by_the_enchanted_creature() {
    cr!("615.1a", "615.6");
    // "Prevent all damage that would be dealt by enchanted creature."
    supported("Muzzle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    enchant(&mut t, "Muzzle", "Plains", 2, bears);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P0)), (other, Entity::Player(P0))],
        &[],
    );
    assert_eq!(t.life(P0), 18, "only the unenchanted Bears dealt damage");
    // Blocked: it deals no damage but is dealt damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blocker = t.battlefield(P0, "Hill Giant");
    enchant(&mut t, "Muzzle", "Plains", 2, bears);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[(blocker, bears)]);
    assert_eq!(t.obj_now(blocker).damage, 0);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

// ---------------------------------------------------------------------------
// Conditional keywords
// ---------------------------------------------------------------------------

#[test]
fn dragonlord_ojutai_has_hexproof_only_while_untapped() {
    cr!("702.11b", "611.3a");
    // "Dragonlord Ojutai has hexproof as long as it's untapped."
    supported("Dragonlord Ojutai");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Dragonlord Ojutai");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj_now(o).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    // An opponent's Bolt isn't offered it as a target while it's untapped.
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    let offered = &target_choices(&t, P1)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(o)), "hexproof");
    // Tapped, it loses hexproof and can be targeted.
    t.g.tap(o);
    t.g.recompute();
    assert!(!t.obj_now(o).has_keyword(KeywordKind::Hexproof));
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(o).go();
    t.resolve_all();
    assert!(target_choices(&t, P1)[1].contains(&obj(o)));
    assert_eq!(t.obj_now(o).damage, 3);
}

// ---------------------------------------------------------------------------
// Cost reductions
// ---------------------------------------------------------------------------

#[test]
fn depower_costs_2_less_targeting_an_attacking_creature() {
    cr!("601.2f");
    // "This spell costs {2} less to cast if it targets an attacking creature."
    supported("Depower");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Hill Giant");
    let home = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareAttackers);
    // Targeting the creature that isn't attacking costs the full {2}{U}.
    let d = t.hand(P0, "Depower");
    assert!(t.cast(P0, d).target(home).try_go().is_err());
    t.clear_answers();
    assert!(t.in_hand(P0, "Depower"));
    // Targeting the attacker costs just {U}.
    t.cast(P0, d).target(attacker).go();
    t.resolve_all();
    assert_eq!(t.pt(attacker), (-1, 3));
    assert_eq!(t.pt(home), (3, 3));
    assert_eq!(t.hand_size(P0), 1, "drew a card");
}
