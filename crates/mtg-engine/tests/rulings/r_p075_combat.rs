//! Rulings batch P075 — +1/+1 counters in combat: combat damage triggers and creatures that
//! died in that damage step (Borborygmos, Drana), attack triggers (Gleam of Battle,
//! Wilderland Scrounger, Finneas, Spider-Man), tokens created tapped and attacking (Elspeth),
//! "can't be blocked by more than one" checked only as blockers are declared (Flopsie),
//! and "becomes tapped" triggers (Zar Ojanen).

use crate::r_p075_common::*;
use crate::r_s02_common::create_token;
use crate::r_s10_common::attacking;
use crate::r_s21_common::blocks_now;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn borborygmos_creatures_dealt_lethal_combat_damage_die_first() {
    cr!("510.2", "704.5g", "603.3");
    ruling!(
        "Borborygmos",
        "Borborygmos's ability won't save your creatures that were dealt lethal damage in combat."
    );
    supported("Borborygmos");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Borborygmos");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    let giant = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[b, bears]));
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!((plus(&t, b), plus(&t, home)), (1, 1));
}

#[test]
fn drana_counters_arrive_before_regular_combat_damage() {
    cr!("510.4", "702.7b", "603.3");
    ruling!(
        "Drana, Liberator of Malakir",
        "only attacking creatures that survive the combat damage step in which Drana deals combat damage will have +1/+1 counters put on them"
    );
    supported("Drana, Liberator of Malakir");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Drana, Liberator of Malakir");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let striker = t.battlefield(P0, "Elite Vanguard");
    grant(
        &mut t,
        striker,
        mtg_engine::keywords::KeywordKind::FirstStrike,
    );
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let killer = t.battlefield(P1, "Hill Giant");
    grant(
        &mut t,
        killer,
        mtg_engine::keywords::KeywordKind::FirstStrike,
    );
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[d, bears, striker]));
    block_and_finish(&mut t, P1, &[(blocker, bears), (killer, striker)]);
    // The Elite Vanguard died in the first-strike damage step with Drana's damage.
    assert!(!t.on_battlefield(striker));
    // The Bears got the counter before regular damage: a 3/3 survives the 2/2 blocker.
    assert_eq!(plus(&t, bears), 1);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(blocker));
    assert_eq!(plus(&t, d), 1);
}

#[test]
fn gleam_of_battle_attackers_only_and_before_blocks() {
    cr!("508.3a", "508.4", "509.1");
    ruling!(
        "Gleam of Battle",
        "If a creature enters the battlefield attacking, it won’t cause Gleam of Battle to trigger."
    );
    ruling!(
        "Gleam of Battle",
        "The +1/+1 counter is put on the creature before blockers are declared and combat damage is dealt."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gleam of Battle");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[hero]));
    t.resolve_all();
    assert_eq!(
        t.g.turn.step,
        Step::DeclareAttackers,
        "no blockers declared yet"
    );
    assert_eq!(plus(&t, hero), 1);
    let soldiers = tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    for s in soldiers {
        assert!(attacking(&t, s));
        assert_eq!(plus(&t, s), 0, "entered attacking: no trigger");
    }
}

#[test]
fn elspeth_storm_slayer_extra_tokens_are_tapped_and_attacking_too() {
    cr!("614.1a", "508.4", "111.1");
    ruling!(
        "Elspeth, Storm Slayer",
        "if an effect tells you to create a token “tapped and attacking,” the additional tokens will also be tapped and attacking"
    );
    supported("Elspeth, Storm Slayer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elspeth, Storm Slayer");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[hero]));
    t.resolve_all();
    let soldiers = tokens(&t, P0);
    assert_eq!(soldiers.len(), 4);
    for s in soldiers {
        assert!(attacking(&t, s) && tapped(&t, s));
    }
}

#[test]
fn wilderland_scrounger_checks_only_as_it_attacks() {
    cr!("603.4");
    ruling!(
        "Wilderland Scrounger",
        "It will not check again when it resolves, so the trigger will resolve even if you no longer control a creature with power 4 or greater."
    );
    supported("Wilderland Scrounger");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wilderland Scrounger");
    let wurm = t.battlefield(P0, "Craw Wurm");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[w]));
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(plus(&t, w), 1);
    // Without a creature with power 4 or greater, it doesn't trigger.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wilderland Scrounger");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[w]));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn finneas_checks_total_power_right_after_the_counters() {
    cr!("608.2c");
    ruling!(
        "Finneas, Ace Archer",
        "Players can't take actions in between the time you put counters on creatures with Finneas's last ability and the point at which that ability checks the total power"
    );
    supported("Finneas, Ace Archer");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Finneas, Ace Archer");
    t.battlefield(P0, "Craw Wurm");
    let tok = create_token(&mut t, P0, "Soldier");
    let hand = t.hand_size(P0);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[f]));
    t.resolve_all();
    assert_eq!(plus(&t, tok), 1);
    assert_eq!(
        t.hand_size(P0),
        hand + 1,
        "9 power became 10 within the resolution"
    );
}

#[test]
fn spider_man_triggers_only_for_himself() {
    cr!("201.5", "603.6a");
    ruling!(
        "Spider-Man, Miles Morales",
        "Spider-Man, Miles Morales's last ability triggers only when he enters or attacks, not when any other creature named Spider-Man enters or attacks."
    );
    supported("Spider-Man, Miles Morales");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spider-Man, Miles Morales");
    let other = t.enter(P0, "Spider-Man, Web-Slinger");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[other]));
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let m2 = t.battlefield(P0, "Spider-Man, Miles Morales");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[m2]));
    t.resolve_all();
    assert_eq!(plus(&t, bears), 1);
}

#[test]
fn flopsie_power_increase_after_blocks_doesnt_undo_them() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Flopsie, Bumi's Buddy",
        "If a creature you control with power less than 4 becomes blocked by more than one creature, increasing its power to 4 or greater won't change or undo the block."
    );
    supported("Flopsie, Bumi's Buddy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flopsie, Bumi's Buddy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(a, bears), (b, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocks_now(&t).len(), 2);
    boost(&mut t, bears, 2, 0);
    assert_eq!(t.pt(bears).0, 4);
    assert_eq!(blocks_now(&t).len(), 2, "still blocked by both");
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn zar_ojanen_trigger_from_a_cost_resolves_first() {
    cr!("603.3", "602.2b");
    ruling!(
        "Zar Ojanen, Scion of Efrava",
        "If Zar Ojanen becomes tapped while casting a spell or activating an ability, that spell or ability resolves after Zar Ojanen's triggered ability has resolved."
    );
    ruling!(
        "Zar Ojanen, Scion of Efrava",
        "Zar Ojanen, Scion of Efrava's ability is a triggered ability, not an activated ability."
    );
    supported("Zar Ojanen, Scion of Efrava");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zar Ojanen, Scion of Efrava");
    assert!(!can_activate(&mut t, P0, z));
    for land in ["Plains", "Island", "Swamp"] {
        t.battlefield(P0, land);
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    let opp = t.battlefield(P0, "Opposition");
    let target = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[obj(z)]);
    t.activate(P0, opp, 0, &[obj(target)]).expect("activates");
    assert!(tapped(&t, z));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(plus(&t, bears), 1, "toughness 2 < 3 basic land types");
    assert!(
        !tapped(&t, target),
        "Opposition's ability hasn't resolved yet"
    );
    t.resolve_all();
    assert!(tapped(&t, target));
    // Attacking taps it too.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zar Ojanen, Scion of Efrava");
    t.battlefield(P0, "Plains");
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Swamp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[z]));
    t.resolve_all();
    assert_eq!(plus(&t, bears), 1);
}
