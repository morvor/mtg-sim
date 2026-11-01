//! Rulings batch P072 — combat abilities that give evasion or strike keywords: when they
//! apply relative to the declarations (CR 508–509), blocking requirements (CR 509.1c),
//! additional combat phases without main phases (CR 500.8), and timing restrictions tied
//! to combat (CR 506.8).

use crate::r_p007_common::loyalty;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::crew;
use crate::r_s06_common::{activate_containing, attach_new, has_kw};
use crate::r_s10_common::attacking;
use crate::r_s21_common::{blocks_now, castable, legal_blocks};
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::counters;
use mtg_engine::*;

/// From the end of combat step, runs to the next step and returns it.
fn step_after_end_of_combat(t: &mut TestGame) -> Step {
    t.advance_to(P0, Step::EndOfCombat);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step != Step::EndOfCombat && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    t.g.turn.step
}

fn no_blocks(t: &mut TestGame) {
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
}

#[test]
fn smogsteed_rider_gives_other_attackers_fear_before_blocks() {
    cr!("508.1m", "702.36b", "509.1a");
    ruling!(
        "Smogsteed Rider",
        "Smogsteed Rider doesn't give itself fear, but if two are attacking, each will give the other one fear."
    );
    ruling!(
        "Smogsteed Rider",
        "The creatures gain fear before blockers can be declared."
    );
    supported("Smogsteed Rider");
    // One Rider and Grizzly Bears: the Bears gain fear, the Rider doesn't.
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Smogsteed Rider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(rider, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Fear));
    assert!(!has_kw(&t, rider, KeywordKind::Fear));
    assert!(!legal_blocks(&mut t, P1, &[(blocker, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(blocker, rider)]));
    // Two Riders: each gives the other fear.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Smogsteed Rider");
    let b = t.battlefield(P0, "Smogsteed Rider");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    assert!(has_kw(&t, a, KeywordKind::Fear));
    assert!(has_kw(&t, b, KeywordKind::Fear));
}

#[test]
fn chieftain_en_dal_gives_itself_first_strike() {
    cr!("508.1m", "603.2");
    ruling!(
        "Chieftain en-Dal",
        "Chieftain en-Dal does give itself first strike when attacking."
    );
    supported("Chieftain en-Dal");
    let mut t = TestGame::new(2);
    let chief = t.battlefield(P0, "Chieftain en-Dal");
    attack_with(&mut t, &[(chief, Entity::Player(P1))]);
    t.resolve_all();
    assert!(has_kw(&t, chief, KeywordKind::FirstStrike));
}

#[test]
fn siege_modification_falling_off_an_attacking_vehicle() {
    cr!("506.4", "301.7");
    ruling!(
        "Siege Modification",
        "If Siege Modification becomes unattached from a Vehicle that’s attacking or blocking, that Vehicle will be removed from combat unless another effect (such as its crew ability) is also making it a creature."
    );
    supported("Siege Modification");
    for crewed in [false, true] {
        let mut t = TestGame::new(2);
        let copter = t.battlefield(P0, "Smuggler's Copter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let aura = attach_new(&mut t, P0, "Siege Modification", copter);
        assert_eq!(t.pt(copter), (6, 3));
        assert!(has_kw(&t, copter, KeywordKind::FirstStrike));
        if crewed {
            assert!(crew(&mut t, P0, copter, &[bears]));
            t.resolve_all();
        }
        t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
        attack_with(&mut t, &[(copter, Entity::Player(P1))]);
        t.resolve_all();
        assert!(attacking(&t, copter));
        destroy(&mut t, aura);
        assert_eq!(attacking(&t, copter), crewed, "crewed {crewed}");
    }
}

#[test]
fn daring_leap_after_blocks_doesnt_unblock() {
    cr!("509.1h", "702.9b");
    ruling!(
        "Daring Leap",
        "If used after a blocker is assigned your creature is still blocked."
    );
    supported("Daring Leap");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(giant, bears)]);
    cast_new(&mut t, P0, "Daring Leap", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // First strike: the 3/3 Bears kill the Giant and survive.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn dragon_grip_cast_after_blocks_still_gives_first_strike_in_time() {
    cr!("702.7b", "510.4", "601.3b");
    ruling!(
        "Dragon Grip",
        "You must cast Dragon Grip at the latest during the declare blockers step after blockers are declared if you want first strike to matter during that combat."
    );
    supported("Dragon Grip");
    // Ferocious: P0 controls Craw Wurm (6/4).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    cast_new(&mut t, P0, "Dragon Grip", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn rapid_fire_only_before_blockers_of_the_first_combat() {
    cr!("506.8b", "506.8d");
    ruling!(
        "Rapid Fire",
        "If a turn has multiple combat phases, Rapid Fire can only be cast before the beginning of the Declare Blockers Step of the first combat phase in that turn."
    );
    supported("Rapid Fire");
    ruling!(
        "Savage Beating",
        "The additional combat phase is directly after the combat phase in which Savage Beating is cast. There isn't a main phase between the two combat phases."
    );
    supported("Savage Beating");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fire = t.hand(P0, "Rapid Fire");
    lands_for_cost(&mut t, P0, "Rapid Fire");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(castable(&mut t, P0, fire));
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    assert!(!castable(&mut t, P0, fire));
    // Savage Beating's second mode: an additional combat phase right after this one.
    lands_for_cost(&mut t, P0, "Savage Beating");
    let beating = t.hand(P0, "Savage Beating");
    t.cast(P0, beating).modes(&[1]).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(step_after_end_of_combat(&mut t), Step::BeginningOfCombat);
    // In the second combat, before blockers: still not castable.
    assert!(!castable(&mut t, P0, fire));
}

#[test]
fn extra_combat_phases_come_without_main_phases() {
    cr!("500.8", "506.1");
    ruling!(
        "Karlach, Fury of Avernus",
        "Notably, the triggered ability of Karlach doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    ruling!(
        "Karlach, Fury of Avernus",
        "Karlach doesn't have to be among the attacking creatures."
    );
    ruling!(
        "Great Train Heist",
        "Notably, Great Train Heist’s first mode doesn’t give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    // Karlach stays home; Grizzly Bears attacks.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karlach, Fury of Avernus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    no_blocks(&mut t);
    assert_eq!(step_after_end_of_combat(&mut t), Step::BeginningOfCombat);
    // Great Train Heist's first mode, cast in the declare attackers step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let heist = t.hand(P0, "Great Train Heist");
    t.cast(P0, heist).modes(&[0]).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    no_blocks(&mut t);
    assert_eq!(step_after_end_of_combat(&mut t), Step::BeginningOfCombat);
}

#[test]
fn grappling_hook_block_requirement() {
    cr!("509.1c", "603.5");
    ruling!(
        "Grappling Hook",
        "If you control the triggered ability, you decide whether to have the targeted creature block the attacking creature at the time the ability resolves, not at the time blockers are declared."
    );
    ruling!(
        "Grappling Hook",
        "Tapped creatures, creatures that can’t block as the result of an effect, creatures with unpaid costs to block (such as those from War Cadence), and creatures that aren’t controlled by the defending player are exempt from effects that would require them to block. Such creatures can be targeted by the ability, but the requirement to block does nothing."
    );
    supported("Grappling Hook");
    // (choose to have it block, the target is tapped) → blocks
    for (yes, tapped) in [(true, false), (false, false), (true, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Grappling Hook", bears);
        let giant = t.battlefield(P1, "Hill Giant");
        if tapped {
            t.g.tap(giant);
        }
        t.answer_targets(P0, &[Entity::Object(giant)]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        // The choice is made as the ability resolves.
        let from = t.asked().len();
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert!(
            t.asked()[from..]
                .iter()
                .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })),
            "{yes} {tapped}"
        );
        no_blocks(&mut t);
        t.advance_to(P0, Step::DeclareBlockers);
        let blocked = blocks_now(&t) == vec![(giant, bears)];
        assert_eq!(blocked, yes && !tapped, "{yes} {tapped}");
    }
}

#[test]
fn indrik_umbra_tapped_creatures_dont_block() {
    cr!("509.1c");
    ruling!(
        "Indrik Umbra",
        "If, during a player's declare blockers step, a creature is tapped or it's affected by a spell or ability that says it can't block, then it doesn't block. If there's a cost associated with having a creature block, its controller isn't forced to pay that cost. The creature doesn't have to block."
    );
    supported("Indrik Umbra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Indrik Umbra", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(tapped);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    no_blocks(&mut t);
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocks_now(&t), vec![(giant, bears)]);
}

#[test]
fn tidal_flats_payment_on_resolution_per_activation() {
    cr!("118.12", "602.1");
    ruling!(
        "Tidal Flats",
        "Can be used more than once in a turn. Each time, the opponent can pay to prevent giving first strike to a creature, but this payment only stops the current activation and not future activations of Tidal Flats."
    );
    ruling!(
        "Tidal Flats",
        "First Strike is granted to your creatures currently blocking non-Flyers when this effect resolves."
    );
    ruling!("Tidal Flats", "The payment is made when the Tidal Flats effect resolves.");
    supported("Tidal Flats");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let flats = t.battlefield(P1, "Tidal Flats");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Island", 4);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    // First activation: P0 pays {1} as it resolves.
    let from = t.asked().len();
    activate_containing(&mut t, P1, flats, "first strike").unwrap();
    assert!(!t.asked()[from..].iter().any(|(p, _)| *p == P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!has_kw(&t, blocker, KeywordKind::FirstStrike));
    // Second activation: P0 can't pay any more.
    activate_containing(&mut t, P1, flats, "first strike").unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(has_kw(&t, blocker, KeywordKind::FirstStrike));
    assert!(!has_kw(&t, other, KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(blocker));
}

#[test]
fn rowan_fearless_sparkmage_abilities() {
    cr!("115.3", "615.1", "608.2c");
    ruling!(
        "Rowan, Fearless Sparkmage",
        "If a target of Rowan's second ability is a legal target but an effect prevents the damage that would be dealt to it, that creature still can't block this turn."
    );
    ruling!(
        "Rowan, Fearless Sparkmage",
        "Rowan's second ability can't target the same creature twice to deal 2 damage to it."
    );
    ruling!(
        "Rowan, Fearless Sparkmage",
        "Rowan's third ability can affect creatures you already control or ones that are already untapped. It will untap them if applicable and give them haste."
    );
    supported("Rowan, Fearless Sparkmage");
    // −2 on a creature whose damage is prevented (Shielded Passage in response).
    let mut t = TestGame::new(2);
    let rowan = t.battlefield(P0, "Rowan, Fearless Sparkmage");
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let bears = t.battlefield(P0, "Hill Giant");
    loyalty(&mut t, P0, rowan, 1, &[Entity::Object(attacker)]);
    cast_new(&mut t, P1, "Shielded Passage", &[Entity::Object(attacker)]);
    t.resolve_all();
    assert_eq!(t.obj_now(attacker).damage, 0);
    crate::r_s09_common::to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(attacker, bears)]));
    // −2 can't target the same creature twice.
    let mut t = TestGame::new(2);
    let rowan = t.battlefield(P0, "Rowan, Fearless Sparkmage");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant), Entity::Object(giant)]);
    let _ = activate_containing(&mut t, P0, rowan, "1 damage");
    t.resolve_all();
    assert!(t.obj_now(giant).damage <= 1);
    // −9 affects P0's own tapped creature: untapped, with haste.
    let mut t = TestGame::new(2);
    let rowan = t.battlefield(P0, "Rowan, Fearless Sparkmage");
    t.g.objects[rowan.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 9);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.tap(mine);
    let theirs = t.battlefield(P1, "Hill Giant");
    loyalty(&mut t, P0, rowan, 2, &[]);
    t.resolve_all();
    assert!(!t.obj_now(mine).tapped);
    assert!(has_kw(&t, mine, KeywordKind::Haste));
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert!(has_kw(&t, theirs, KeywordKind::Haste));
}

#[test]
fn ballroom_brawlers_one_choice_for_both_creatures() {
    cr!("603.3c", "608.2d");
    ruling!(
        "Ballroom Brawlers",
        "You make the choice of first strike or lifelink only once, as Ballroom Brawlers's triggered ability resolves. Both creatures gain the chosen ability."
    );
    supported("Ballroom Brawlers");
    for (choice, kw, other) in [
        (0, KeywordKind::FirstStrike, KeywordKind::Lifelink),
        (1, KeywordKind::Lifelink, KeywordKind::FirstStrike),
    ] {
        let mut t = TestGame::new(2);
        let brawlers = t.battlefield(P0, "Ballroom Brawlers");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        attack_with(&mut t, &[(brawlers, Entity::Player(P1))]);
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.resolve_all();
        let options = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .count();
        assert_eq!(options, 1);
        for c in [brawlers, bears] {
            assert!(has_kw(&t, c, kw));
            assert!(!has_kw(&t, c, other));
        }
    }
}

#[test]
fn glamdring_casts_during_resolution_ignoring_timing() {
    cr!("608.2g", "601.2", "117.1a");
    ruling!(
        "Glamdring",
        "If you cast a spell using Glamdring's triggered ability, you do so as part of the resolution of the ability. You can't wait to cast the spell later in the turn. Timing permissions based on the card's type are ignored."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Glamdring", giant);
    let div = t.hand(P0, "Divination");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    no_blocks(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(div)]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Divination, a sorcery, was cast in the combat damage step.
    assert!(t.in_graveyard(P0, "Divination"));
}

#[test]
fn embercleave_cost_is_locked_in_while_casting() {
    cr!("601.2f", "601.2h", "117.1");
    ruling!(
        "Embercleave",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to increase Embercleave's cost by removing attacking creatures you control."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.lands(P0, "Mountain", 4);
    let cleave = t.hand(P0, "Embercleave");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.cast(P0, cleave).go();
    assert!(!t.asked()[from..].iter().any(|(p, _)| *p == P1));
    // {4}{R}{R} less {2}: four Mountains.
    assert!(t.g.stack.contains(&t.g.current(cleave)) || t.stack_len() == 1);
    destroy(&mut t, b);
    t.resolve_all();
    let cleave = t.named_on_battlefield("Embercleave")[0];
    assert_eq!(
        crate::r_s06_common::attached_to(&t, cleave),
        Some(Entity::Object(a))
    );
}
