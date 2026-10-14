//! Structure-coverage tests, round 3 (item 5): static-ability structures shared by many
//! cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks
//! a real card's ability does what its text says, including when it doesn't apply.

use super::structure_activated_2::{act, target_choices};
use super::structure_seed_common::*;
use super::structure_triggered_5::{next_turn, obj};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// Mills `n` cards from `p`'s library (a real zone change).
fn mill(t: &mut TestGame, p: PlayerId, n: usize) {
    t.g.mill(p, n as u32);
    t.resolve_all();
    t.g.recompute();
}

/// Declares `attackers` attacking P1 and advances to the declare attackers step.
fn attack_with(t: &mut TestGame, attackers: &[ObjectId]) {
    t.set_step(P0, Step::BeginningOfCombat);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, &decl);
    go_to(t, Step::DeclareAttackers);
}

#[test]
fn mass_hysteria_gives_all_creatures_haste() {
    cr!("611.3a", "613.1f", "702.10b");
    // "All creatures have haste."
    supported("Mass Hysteria");
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    let theirs = t.battlefield_sick(P1, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    assert!(!t.g.can_attack(mine));
    let mh = t.battlefield(P0, "Mass Hysteria");
    assert!(t.g.can_attack(mine));
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Haste), "every player's");
    assert!(!t.obj_now(land).has_keyword(KeywordKind::Haste), "creatures only");
    // A summoning-sick creature with haste attacks.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mine, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    // Gone when the enchantment leaves.
    t.g.move_object(mh, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.g.recompute();
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Haste));
}

#[test]
fn possessed_centaur_threshold_pumps_turns_black_and_grants_an_ability() {
    cr!("611.3a", "105.3", "613.1e", "613.1f", "613.4c");
    // "Threshold — As long as there are seven or more cards in your graveyard, this creature
    // gets +1/+1, is black, and has "{2}{B}, {T}: Destroy target green creature.""
    supported("Possessed Centaur");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Possessed Centaur");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let red = t.battlefield(P1, "Raging Goblin");
    mill(&mut t, P1, 7);
    mill(&mut t, P0, 6);
    assert_eq!(t.pt(c), (3, 3), "six cards, and the opponent's graveyard doesn't count");
    assert!(!t.obj_now(c).chars.colors.contains(Color::Black));
    let n_abilities = t.obj_now(c).chars.abilities.len();
    mill(&mut t, P0, 1);
    assert_eq!(t.pt(c), (4, 4));
    let col = t.obj_now(c).chars.colors;
    // "Is black" replaces its other colors (CR 105.3).
    assert!(col.contains(Color::Black) && !col.contains(Color::Green));
    assert_eq!(t.obj_now(c).chars.abilities.len(), n_abilities + 1);
    t.lands(P0, "Swamp", 3);
    act(&mut t, P0, c, "destroy target green", &[obj(elf)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    let offered = &target_choices(&t, P0)[0];
    assert!(!offered.contains(&obj(c)), "itself: no longer green");
    assert!(!offered.contains(&obj(red)), "not a green creature");
    assert!(t.obj_now(c).tapped);
}

#[test]
fn pterodon_knight_has_flying_while_you_control_a_dinosaur() {
    cr!("611.3a", "613.1f");
    // "This creature has flying as long as you control a Dinosaur."
    supported("Pterodon Knight");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Pterodon Knight");
    t.battlefield(P1, "Colossal Dreadmaw");
    t.g.recompute();
    assert!(!t.obj_now(k).has_keyword(KeywordKind::Flying), "an opponent's Dinosaur");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    assert!(t.obj_now(k).has_keyword(KeywordKind::Flying));
    t.g.move_object(dino, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.g.recompute();
    assert!(!t.obj_now(k).has_keyword(KeywordKind::Flying));
}

#[test]
fn bone_picker_costs_3_less_if_a_creature_died_this_turn() {
    cr!("601.2f", "118.7", "700.4");
    // "This spell costs {3} less to cast if a creature died this turn."
    supported("Bone Picker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bp = t.hand(P0, "Bone Picker");
    assert!(t.cast(P0, bp).try_go().is_err(), "{{3}}{{B}} with one Swamp");
    assert!(t.in_hand(P0, "Bone Picker"));
    // An opponent's creature dying counts too.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.resolve_all();
    t.cast(P0, bp).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bone Picker").len(), 1);
    // A land being destroyed doesn't count.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let land = t.battlefield(P1, "Forest");
    t.g.destroy(land, None);
    t.resolve_all();
    let bp = t.hand(P0, "Bone Picker");
    assert!(t.cast(P0, bp).try_go().is_err());
    // A creature that died last turn doesn't count either.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    next_turn(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    let bp = t.hand(P0, "Bone Picker");
    assert!(t.cast(P0, bp).try_go().is_err());
}

#[test]
fn privileged_position_gives_your_other_permanents_hexproof() {
    cr!("702.11b", "611.3a", "115.1");
    // "Other permanents you control have hexproof."
    supported("Privileged Position");
    let mut t = TestGame::new(2);
    let pp = t.battlefield(P0, "Privileged Position");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    let other_ench = t.battlefield(P0, "Mass Hysteria");
    let their_ench = t.battlefield(P1, "Mass Hysteria");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    assert!(t.obj_now(land).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(pp).has_keyword(KeywordKind::Hexproof), "other permanents");
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Hexproof));
    // An opponent can't target them, but can target the enchantment itself.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let dis = t.hand(P1, "Naturalize");
    t.cast(P1, dis).target(pp).go();
    let offered = &target_choices(&t, P1)[0];
    assert!(offered.contains(&obj(pp)));
    assert!(!offered.contains(&obj(other_ench)), "hexproof");
    assert!(offered.contains(&obj(their_ench)));
    t.resolve_all();
    assert!(!t.on_battlefield(pp));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    // You can still target your own permanents.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Privileged Position");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    let mine_offered = &target_choices(&t, P0)[0];
    assert!(mine_offered.contains(&obj(bears)) && mine_offered.contains(&obj(theirs)));
    // The opponent's Bolt offers neither of P0's hexproof creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Privileged Position");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let offered = &target_choices(&t, P1)[0];
    assert!(!offered.contains(&obj(bears)));
    assert!(offered.contains(&obj(theirs)) && offered.contains(&Entity::Player(P0)));
}

#[test]
fn singer_of_swift_rivers_lets_you_cast_merfolk_as_though_they_had_flash() {
    cr!("601.3", "702.8a", "609.4");
    // "You may cast Merfolk spells as though they had flash."
    supported("Singer of Swift Rivers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Singer of Swift Rivers");
    t.lands(P0, "Island", 2);
    t.set_step(P1, Step::Upkeep);
    let merfolk = t.hand(P0, "Merfolk of the Pearl Trident");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err(), "not a Merfolk");
    t.cast(P0, merfolk).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Merfolk of the Pearl Trident").len(), 1);
    // The opponent's Merfolk don't get flash.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Singer of Swift Rivers");
    t.lands(P1, "Island", 1);
    let theirs = t.hand(P1, "Merfolk of the Pearl Trident");
    assert!(t.cast(P1, theirs).try_go().is_err());
    // Without the Singer, a Merfolk can't be cast at instant speed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.set_step(P1, Step::Upkeep);
    let merfolk = t.hand(P0, "Merfolk of the Pearl Trident");
    assert!(t.cast(P0, merfolk).try_go().is_err());
}

#[test]
fn stamina_gives_the_enchanted_creature_vigilance() {
    cr!("303.4", "611.3a", "702.20b");
    // "Enchanted creature has vigilance."
    supported("Stamina");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let s = t.battlefield(P0, "Stamina");
    t.g.attach(s, obj(bears));
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[],
    );
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(other).tapped);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn expedition_healer_has_lifelink_with_another_cleric() {
    cr!("611.3a", "613.1f", "702.15b");
    // "This creature has lifelink as long as you control another Cleric."
    supported("Expedition Healer");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Expedition Healer");
    t.battlefield(P1, "Expedition Healer");
    t.g.recompute();
    assert!(!t.obj_now(h).has_keyword(KeywordKind::Lifelink), "itself and an opponent's");
    t.battlefield(P0, "Expedition Healer");
    assert!(t.obj_now(h).has_keyword(KeywordKind::Lifelink));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(h, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn blade_historian_gives_your_attacking_creatures_double_strike() {
    cr!("611.3a", "702.4b");
    // "Attacking creatures you control have double strike."
    supported("Blade Historian");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Blade Historian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    attack_with(&mut t, &[bears]);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(home).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(h).has_keyword(KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16, "2 + 2");
    // An opponent's attacking creatures don't get it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blade Historian");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(theirs, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn out_of_the_way_costs_2_less_targeting_a_green_permanent() {
    cr!("601.2f", "118.7");
    // "This spell costs {2} less to cast if it targets a green permanent. / Return target
    // nonland permanent an opponent controls to its owner's hand. / Draw a card."
    supported("Out of the Way");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P1, "Llanowar Elves");
    let red = t.battlefield(P1, "Raging Goblin");
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Out of the Way");
    assert!(t.cast(P0, s).target(red).try_go().is_err(), "full cost for a red one");
    t.clear_answers();
    t.cast(P0, s).target(elf).go();
    t.resolve_all();
    assert_eq!(t.zone(elf), Zone::Hand(P1));
    assert!(t.on_battlefield(red));
    assert_eq!(t.hand_size(P0), 1, "drew a card");
    // Paying the full {3}{U} for a nongreen target.
    let mut t = TestGame::new(2);
    let red = t.battlefield(P1, "Raging Goblin");
    t.lands(P0, "Island", 4);
    let s = t.hand(P0, "Out of the Way");
    t.cast(P0, s).target(red).go();
    t.resolve_all();
    assert_eq!(t.zone(red), Zone::Hand(P1));
}

#[test]
fn neurok_spy_cant_be_blocked_while_the_defender_controls_an_artifact() {
    cr!("509.1b", "611.3a");
    // "This creature can't be blocked as long as defending player controls an artifact."
    supported("Neurok Spy");
    let mut t = TestGame::new(2);
    let spy = t.battlefield(P0, "Neurok Spy");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    attack_with(&mut t, &[spy]);
    assert!(t.g.can_block(blocker, spy), "only the attacker controls an artifact");
    let mut t = TestGame::new(2);
    let spy = t.battlefield(P0, "Neurok Spy");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Sol Ring");
    attack_with(&mut t, &[spy]);
    assert!(!t.g.can_block(blocker, spy));
    block(&mut t, P1, &[(blocker, spy)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18, "unblocked");
}

#[test]
fn call_for_unity_pumps_your_creatures_per_unity_counter() {
    cr!("611.3a", "613.4c", "122.1");
    // "Creatures you control get +1/+1 for each unity counter on this enchantment."
    supported("Call for Unity");
    let mut t = TestGame::new(2);
    let cfu = t.battlefield(P0, "Call for Unity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2), "no counters");
    t.g.add_counters(obj(cfu), "unity", 2, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
    t.g.add_counters(obj(cfu), "+1/+1", 3, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4), "only unity counters count");
}

#[test]
fn frightcrawler_threshold_pumps_and_stops_it_blocking() {
    cr!("611.3a", "509.1b");
    // "Threshold — As long as there are seven or more cards in your graveyard, this
    // creature gets +2/+2 and can't block."
    supported("Frightcrawler");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Frightcrawler");
    mill(&mut t, P0, 6);
    assert_eq!(t.pt(f), (1, 1));
    assert!(t.g.can_block_at_all(f));
    mill(&mut t, P0, 1);
    assert_eq!(t.pt(f), (3, 3));
    assert!(!t.g.can_block_at_all(f));
}

#[test]
fn divinity_of_pride_gets_plus_4_at_25_life() {
    cr!("611.3a", "613.4c");
    // "This creature gets +4/+4 as long as you have 25 or more life."
    supported("Divinity of Pride");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Divinity of Pride");
    t.g.gain_life(P1, 10);
    t.g.gain_life(P0, 4);
    t.g.recompute();
    assert_eq!(t.pt(d), (4, 4), "24 life, and the opponent's life doesn't count");
    t.g.gain_life(P0, 1);
    t.g.recompute();
    assert_eq!(t.pt(d), (8, 8));
    t.g.lose_life(P0, 1);
    t.g.recompute();
    assert_eq!(t.pt(d), (4, 4));
}

#[test]
fn dragon_mantle_grants_a_firebreathing_ability() {
    cr!("303.4", "613.1f");
    // "Enchanted creature has "{R}: This creature gets +1/+0 until end of turn.""
    supported("Dragon Mantle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let m = t.battlefield(P0, "Dragon Mantle");
    t.g.attach(m, obj(bears));
    t.g.recompute();
    t.lands(P0, "Mountain", 2);
    act(&mut t, P0, bears, "+1/+0", &[]).unwrap();
    act(&mut t, P0, bears, "+1/+0", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2), "the creature, not the Aura");
    assert_eq!(t.pt(other), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn dungrove_elder_counts_forests_you_control() {
    cr!("604.3", "613.4a");
    // "Dungrove Elder's power and toughness are each equal to the number of Forests you
    // control."
    supported("Dungrove Elder");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    t.battlefield(P0, "Mountain");
    let e = t.battlefield(P0, "Dungrove Elder");
    assert_eq!(t.pt(e), (0, 0), "no Forests of yours");
    t.resolve_all();
    assert!(!t.on_battlefield(e));
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.battlefield(P0, "Breeding Pool");
    t.battlefield(P0, "Mountain");
    t.battlefield(P1, "Forest");
    let e = t.battlefield(P0, "Dungrove Elder");
    assert_eq!(t.pt(e), (3, 3), "two Forests plus a nonbasic Forest");
}

#[test]
fn earthbending_student_gives_land_creatures_you_control_vigilance() {
    cr!("611.3a", "702.20b");
    // "Land creatures you control have vigilance."
    supported("Earthbending Student");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Earthbending Student");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Dryad Arbor");
    assert!(t.obj_now(arbor).has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(s).has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(land).has_keyword(KeywordKind::Vigilance), "not a creature");
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Vigilance));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(arbor, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[],
    );
    assert!(!t.obj_now(arbor).tapped);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn elesh_norn_shrinks_your_opponents_creatures() {
    cr!("611.3a", "613.4c", "704.5f");
    // "Other creatures you control get +2/+2. / Creatures your opponents control get
    // -2/-2."
    supported("Elesh Norn, Grand Cenobite");
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Grizzly Bears");
    let n = t.battlefield(P0, "Elesh Norn, Grand Cenobite");
    t.resolve_all();
    assert_eq!(t.pt(a), (1, 1));
    assert!(!t.on_battlefield(b), "a 0/0 for the other opponent too");
    assert_eq!(t.pt(mine), (4, 4));
    assert_eq!(t.pt(n), (4, 7));
}

#[test]
fn favorable_winds_pumps_your_fliers() {
    cr!("611.3a", "613.4c");
    // "Creatures you control with flying get +1/+1."
    supported("Favorable Winds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Favorable Winds");
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Wind Drake");
    assert_eq!(t.pt(drake), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
}
