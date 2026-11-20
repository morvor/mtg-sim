//! Item `rulings-residue-misc-2`, pass C — triggered abilities: what a trigger sees as it
//! triggers and as it resolves (Vito, Grishnákh's reflexive trigger, Tilonalli's Summoner's
//! delayed trigger, Optimus Prime's bolster, Goliath Hatchery, Adrestia, Herald of Leshrac,
//! Palantír of Orthanc, Paranormal Analyst, Anowon, Quartzwood Crasher, Avatar Aang, Grist,
//! Sword of Hours).

use crate::r_residue_misc_2c_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Vito, Fanatic of Aclazotz ---------------------------------------------------------------

#[test]
fn vito_sacrificed_with_others_triggers_for_each_of_them() {
    cr!("603.10a", "701.21a", "603.2c");
    ruling!(
        "Vito, Fanatic of Aclazotz",
        "If you sacrifice Vito at the same time you sacrifice other permanents, Vito's ability will trigger for each of those other permanents."
    );
    supported("Vito, Fanatic of Aclazotz");
    let mut t = TestGame::new(2);
    let vito = t.battlefield(P0, "Vito, Fanatic of Aclazotz");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    t.g.sacrifice_simultaneously(&[(vito, P0), (a, P0), (b, P0)]);
    t.g.flush_events();
    t.settle();
    // Two triggers (not one for Vito itself), resolving as the first and second times.
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn vito_does_nothing_beyond_the_third_resolution_in_a_turn() {
    cr!("603.2", "608.2c");
    ruling!(
        "Vito, Fanatic of Aclazotz",
        "Vito's triggered ability has no effect each time beyond the third it resolves in a turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vito, Fanatic of Aclazotz");
    let fodder: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let mut seen = vec![];
    for f in fodder {
        t.g.sacrifice(f, P0);
        t.g.flush_events();
        t.resolve_all();
        seen.push((t.life(P0), t.life(P1), tokens_with(&t, P0, "Vampire").len()));
    }
    assert_eq!(
        seen,
        vec![(22, 20, 0), (22, 18, 0), (22, 18, 1), (22, 18, 1)],
        "{}",
        t.dump_log()
    );
}

// --- Grishnákh, Brash Instigator -------------------------------------------------------------

#[test]
fn grishnakh_targets_with_a_reflexive_trigger_after_amassing() {
    cr!("603.12", "701.47a", "115.1");
    ruling!(
        "Grishnákh, Brash Instigator",
        "You don't choose a target for Grishnákh, Brash Instigator's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you amass Orcs this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let armies = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| {
            g.permanents()
                .filter(|o| o.chars.has_subtype("Army"))
                .count()
        },
    );
    t.enter(P0, "Grishnákh, Brash Instigator");
    t.settle();
    // The enters trigger has no target.
    assert_eq!(t.stack_len(), 1);
    assert!(armies.lock().unwrap().is_empty());
    t.resolve();
    // The Army exists when the target is chosen, for the reflexive trigger on the stack.
    assert_eq!(*armies.lock().unwrap(), vec![1]);
    assert_eq!(t.stack_len(), 1);
    // P1 responds: the Bears gets bigger than the Army (power 3): the target is illegal.
    plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1, "{}", t.dump_log());
}

// --- Tilonalli's Summoner --------------------------------------------------------------------

/// P0 attacks with Tilonalli's Summoner, paying {X}{R} with X = `x`, with `others` more
/// permanents (Wastes) beside the Summoner and the lands for the payment; then the game
/// advances to the end step, where the delayed trigger waits on the stack.
fn summoner_to_end_step(x: i64, others: usize) -> TestGame {
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Tilonalli's Summoner");
    t.lands(P0, "Mountain", x as usize + 1);
    t.lands(P0, "Wastes", others);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    attack_with(&mut t, &[(s, pl(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    t
}

#[test]
fn tilonalli_summoner_blessing_is_checked_as_the_delayed_trigger_resolves() {
    cr!("702.131b", "702.131c", "603.7c");
    ruling!(
        "Tilonalli's Summoner",
        "Whether you have the city’s blessing is checked only as the delayed triggered ability resolves during the end step. The tokens you create can help you ascend."
    );
    supported("Tilonalli's Summoner");
    // 1 + 3 + 4 = 8 permanents, and the two tokens make 10: the city's blessing.
    let mut t = summoner_to_end_step(2, 4);
    assert!(t.g.players[0].has_citys_blessing, "{}", t.dump_log());
    t.resolve_all();

    assert_eq!(tokens_with(&t, P0, "Elemental").len(), 2);
    // Nine permanents: the tokens are exiled...
    let mut t = summoner_to_end_step(2, 3);
    assert!(!t.g.players[0].has_citys_blessing);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(tokens_with(&t, P0, "Elemental").is_empty());
    // ... unless P0 gets the city's blessing before the trigger resolves.
    let mut t = summoner_to_end_step(2, 3);
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Wastes");
    t.settle();
    assert!(t.g.players[0].has_citys_blessing);
    t.resolve_all();
    assert_eq!(
        tokens_with(&t, P0, "Elemental").len(),
        2,
        "{}",
        t.dump_log()
    );
}

#[test]
fn berserk_destroys_the_creature_if_it_attacked_by_the_end_step() {
    // Regression: "At the beginning of the next end step, destroy that creature if it
    // attacked this turn." checks the condition as the delayed trigger resolves (CR 603.7,
    // 608.2c), not as Berserk resolves: cast before combat, the creature attacks later.
    cr!("603.7a", "608.2c");
    supported("Berserk");
    for attack in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_new(&mut t, P0, "Berserk", &[obj(bears)]);
        t.resolve_all();
        if attack {
            attack_with(&mut t, &[(bears, pl(P1))]);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(
            t.in_graveyard(P0, "Grizzly Bears"),
            attack,
            "attack {attack}: {}",
            t.dump_log()
        );
    }
}

// --- Optimus Prime, Autobot Leader -----------------------------------------------------------

const OPTIMUS: &str = "Optimus Prime, Hero // Optimus Prime, Autobot Leader";

#[test]
fn optimus_prime_bolster_chooses_the_least_toughness_as_it_resolves() {
    cr!("701.39a", "115.1", "608.2c");
    ruling!(
        "Optimus Prime, Hero // Optimus Prime, Autobot Leader",
        "Bolster doesn't target; you choose the creature to put the counters on as the attack triggered ability resolves."
    );
    ruling!(
        "Optimus Prime, Hero // Optimus Prime, Autobot Leader",
        "To bolster 2, put two +1/+1 counters on the creature with the least toughness among creatures you control. If more than one creature is tied for the least toughness, choose one of them."
    );
    supported(OPTIMUS);
    let mut t = TestGame::new(2);
    let leader = back_face(&mut t, P0, OPTIMUS);
    t.g.objects[leader.0 as usize].summoning_sick = false;
    let giant = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    attack_with(&mut t, &[(giant, pl(P1))]);
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    // A 2/2 and a 2/1 arrive before it resolves: the 2/1 gets the counters.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.resolve();
    assert_eq!(t.counters(lions, counters::PLUS1), 2, "{}", t.dump_log());
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
    // Tied for the least toughness: P0 chooses between them, and only them.
    let mut t = TestGame::new(2);
    let leader = back_face(&mut t, P0, OPTIMUS);
    t.g.objects[leader.0 as usize].summoning_sick = false;
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(b)]);
    let from = t.asked().len();
    attack_with(&mut t, &[(a, pl(P1))]);
    t.resolve();
    assert_eq!(t.counters(b, counters::PLUS1), 2, "{}", t.dump_log());
    assert_eq!(t.counters(a, counters::PLUS1), 0);
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1, "{offered:?}");
    assert_eq!(offered[0].len(), 2, "{offered:?}");
    assert!(offered[0].contains(&obj(a)) && offered[0].contains(&obj(b)));
}

// --- Goliath Hatchery -------------------------------------------------------------------------

#[test]
fn goliath_hatchery_chooses_the_creature_as_it_resolves() {
    cr!("603.4", "608.2c", "702.164a");
    ruling!(
        "Goliath Hatchery",
        "The creature isn't chosen until the corrupted ability resolves. Notably, this means you may respond to that ability by giving a creature toxic (or more toxic) in order to draw more cards."
    );
    supported("Skrelv, Defector Mite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goliath Hatchery");
    let beast = t.battlefield(P0, "Plague Nurse"); // toxic 2
    let skrelv = t.battlefield(P0, "Skrelv, Defector Mite");
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, Skrelv gives Plague Nurse toxic 1 more.
    t.lands(P0, "Plains", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, skrelv, 0, &[obj(beast)]).expect("Skrelv");
    t.resolve();
    let before = t.hand_size(P0);
    t.answer_choose(P0, &[obj(beast)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 3, "{}", t.dump_log());
}

// --- Adrestia ---------------------------------------------------------------------------------

#[test]
fn adrestia_crewed_by_an_assassin_after_it_attacks_doesnt_trigger() {
    cr!("603.4", "702.122a");
    ruling!(
        "Adrestia",
        "When Adrestia attacks, if it hasn’t already been crewed by an Assassin this turn, its second ability won’t trigger at all. It’s not possible to crew it with an Assassin after it attacks in time to have the ability trigger."
    );
    let mut t = TestGame::new(2);
    let adrestia = t.battlefield(P0, "Adrestia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let assassin = t.battlefield(P0, "Etrata, Deadly Fugitive"); // a Vampire Assassin
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, adrestia, "Crew").expect("crew");
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(adrestia, pl(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    // Not crewed by an Assassin: no trigger. Crewing it with Etrata now doesn't help.
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
    t.answer_choose(P0, &[obj(assassin)]);
    activate_containing(&mut t, P0, adrestia, "Crew").expect("crew again");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(!t.obj_now(adrestia).chars.has_subtype("Assassin"));
}

// --- Herald of Leshrac ------------------------------------------------------------------------

#[test]
fn herald_of_leshrac_chooses_the_land_as_its_upkeep_trigger_resolves() {
    cr!("702.24a", "115.1", "608.2c");
    ruling!(
        "Herald of Leshrac",
        "The cumulative upkeep trigger doesn't target what you gain control of, and you don't choose lands until the ability resolves. Your opponents can not tap lands in response to the choice, but they could respond to the ability by tapping any number of lands before you've made any choices."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Leshrac");
    t.advance_to(P0, Step::Upkeep);
    let from = t.asked().len();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, P1 taps the land for mana (and a land arrives after the trigger).
    let forest = t.battlefield(P1, "Forest");
    t.g.objects[forest.0 as usize].tapped = true;
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.resolve_all();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(controller(&t, forest), P0, "{}", t.dump_log());
    assert!(t.obj_now(forest).tapped);
}

// --- Palantír of Orthanc ----------------------------------------------------------------------

#[test]
fn palantir_of_orthanc_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Palantír of Orthanc",
        "You choose a target for Palantír of Orthanc's ability when it goes on the stack. If that target is illegal as the ability tries to resolve, the ability is removed from the stack. You won't put an influence counter on Palantír of Orthanc, and you won't scry, draw, or mill."
    );
    supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    let pal = t.battlefield(P0, "Palantír of Orthanc");
    t.answer_targets(P0, &[pl(P1)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P1 gets hexproof in response.
    t.battlefield(P1, "Leyline of Sanctity");
    let (hand, lib, from) = (t.hand_size(P0), t.library_size(P0), t.asked().len());
    t.resolve_all();
    assert_eq!(t.counters(pal, "influence"), 0, "{}", t.dump_log());
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, lib));
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Scry { .. })));
}

// --- Paranormal Analyst -----------------------------------------------------------------------

#[test]
fn paranormal_analyst_returns_nothing_if_no_card_reached_the_graveyard_or_it_left() {
    cr!("701.62a", "603.2", "608.2b");
    ruling!(
        "Paranormal Analyst",
        "If you didn't put any cards into your graveyard when you manifested dread (perhaps because your opponent controls Leyline of the Void), Paranormal Analyst's ability will still trigger, but you won't be able to put any cards into your hand when it resolves. Similarly, if a card put into your graveyard when you manifested dread leaves your graveyard before Paranormal Analyst's ability resolves, you won't be able to put that card in your hand."
    );
    supported("Innocuous Rat");
    supported("Leyline of the Void");
    for leyline in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Paranormal Analyst");
        let bears = t.library_top(P0, "Grizzly Bears");
        let giant = t.library_top(P0, "Hill Giant");
        let rat = t.battlefield(P0, "Innocuous Rat");
        kill(&mut t, rat);
        if leyline {
            // (Arriving after the Rat died, so that the Rat isn't exiled instead.)
            t.battlefield(P1, "Leyline of the Void");
        }
        t.answer_choose(P0, &[obj(bears)]);
        t.resolve(); // the Rat's trigger: manifest the Bears, the Giant "to the graveyard"
        let analyst_triggers = triggers_of(&t, "Paranormal Analyst");
        assert_eq!(analyst_triggers.len(), 1, "{}", t.dump_log());
        if !leyline {
            // The Giant leaves the graveyard before the Analyst's ability resolves.
            assert!(t.in_graveyard(P0, "Hill Giant"));
            mv(&mut t, giant, Zone::Exile);
        } else {
            assert!(t.in_exile("Hill Giant"));
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "leyline {leyline}: {}", t.dump_log());
        assert!(t.in_exile("Hill Giant"));
    }
}

// --- Anowon, the Ruin Thief -------------------------------------------------------------------

#[test]
fn anowon_mills_for_the_rogues_damage_only_and_draws_one_card() {
    cr!("510.2", "603.2c", "701.17a");
    ruling!(
        "Anowon, the Ruin Thief",
        "The player dealt combat damage mills cards only for the damage dealt by your Rogues, not for any other combat damage dealt to them at the same time."
    );
    ruling!(
        "Anowon, the Ruin Thief",
        "You draw just one card, no matter how many creature cards the player mills beyond the first."
    );
    supported("Anowon, the Ruin Thief");
    let mut t = TestGame::new(2);
    let anowon = t.battlefield(P0, "Anowon, the Ruin Thief"); // a 2/4 Vampire Rogue
    let giant = t.battlefield(P0, "Hill Giant"); // 3/3, not a Rogue
    stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Forest"]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(anowon, pl(P1)), (giant, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Two cards milled (Anowon's 2 damage), both creature cards: one card drawn.
    assert_eq!(t.graveyard_size(P1), 2, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 1);
}

// --- Quartzwood Crasher -----------------------------------------------------------------------

#[test]
fn quartzwood_crasher_counts_all_damage_of_creatures_with_trample_including_itself() {
    cr!("702.19b", "510.2", "603.2c");
    ruling!(
        "Quartzwood Crasher",
        "If a creature you control with trample is unblocked, the full amount of combat damage it deals to the defending player is counted by Quartzwood Crasher's last ability."
    );
    ruling!(
        "Quartzwood Crasher",
        "Quartzwood Crasher's second ability includes damage dealt by Quartzwood Crasher itself as long as it still has trample."
    );
    supported("Quartzwood Crasher");
    let mut t = TestGame::new(2);
    let crasher = t.battlefield(P0, "Quartzwood Crasher"); // 6/6 trample
    let rhino = t.battlefield(P0, "Colossal Dreadmaw"); // 6/6 trample
    let bears = t.battlefield(P0, "Grizzly Bears"); // no trample: not counted
    attack_with(
        &mut t,
        &[(crasher, pl(P1)), (rhino, pl(P1)), (bears, pl(P1))],
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 6);
    let dinos: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(dinos.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(dinos[0]), (12, 12));
}

// --- Avatar Aang ------------------------------------------------------------------------------

#[test]
fn avatar_aang_counts_the_bending_done_earlier_in_the_turn() {
    cr!("701.65b", "701.66b", "701.67c", "702.189b", "701.27a");
    ruling!(
        "Avatar Aang // Aang, Master of Elements",
        "Avatar Aang's last ability looks at the whole turn when determining whether or not Avatar Aang should transform, not just actions you've taken since he entered or came under your control. For example, say you airbend, waterbend, and earthbend, then play Avatar Aang. If you firebend later in the turn, then when Avatar Aang's ability resolves, he'll transform since you've done all four this turn."
    );
    supported("Geyser Leaper");
    supported("Airbending Lesson");
    supported("Earthbending Lesson");
    let mut t = TestGame::new(2);
    // Waterbend (Geyser Leaper's waterbend {4}, paid with mana), airbend, earthbend.
    let leaper = t.battlefield(P0, "Geyser Leaper");
    t.lands(P0, "Island", 4);
    t.answer_choose(P0, &[]);
    t.activate(P0, leaper, 0, &[]).expect("waterbend");
    t.resolve_all();
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Airbending Lesson", &[obj(giant)]);
    t.resolve_all();
    let land = t.battlefield(P0, "Wastes");
    cast_new(&mut t, P0, "Earthbending Lesson", &[obj(land)]);
    t.resolve_all();
    // Then Aang arrives and attacks: firebending 2.
    let aang = t.battlefield(P0, "Avatar Aang // Aang, Master of Elements");
    attack_with(&mut t, &[(aang, pl(P1))]);
    t.resolve_all();
    assert_eq!(
        t.obj_now(aang).chars.name,
        "Aang, Master of Elements",
        "{}",
        t.dump_log()
    );
}

// --- Grist, Voracious Larva -------------------------------------------------------------------

#[test]
fn grist_that_left_the_battlefield_isnt_returned_transformed() {
    cr!("712.14a", "400.7", "608.2c");
    ruling!(
        "Grist, Voracious Larva // Grist, the Plague Swarm",
        "If Grist, Voracious Larva leaves the battlefield while its triggered ability is on the stack, you can't exile it from the zone it's put into, so you won't return it to the battlefield transformed under its owner's control."
    );
    let mut t = TestGame::new(2);
    let grist = t.battlefield(P0, "Grist, Voracious Larva // Grist, the Plague Swarm");
    // A creature enters from P0's graveyard.
    let bears = t.graveyard(P0, "Grizzly Bears");
    mv(&mut t, bears, Zone::Battlefield);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    // Grist dies in response.
    kill(&mut t, grist);
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(
        t.in_graveyard(P0, "Grist, Voracious Larva"),
        "{}",
        t.dump_log()
    );
    assert!(t.named_on_battlefield("Grist, the Plague Swarm").is_empty());
}

// --- Sword of Hours ---------------------------------------------------------------------------

#[test]
fn sword_of_hours_still_rolls_for_a_creature_that_died_dealing_damage() {
    cr!("510.2", "603.10", "706.2");
    ruling!(
        "Sword of Hours",
        "If the equipped creature takes lethal damage at the same time that it deals combat damage, it will die before the counters on it can be doubled. However, you will still roll a d12, which may cause abilities of other permanents to trigger."
    );
    supported("Sword of Hours");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Sword of Hours");
    t.g.attach(sword, obj(bears));
    t.g.recompute();
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all(); // the attack trigger: a +1/+1 counter (3/3)
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    let rolled =
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, mtg_engine::events::Event::DieRolled { .. }))
            .count();
    assert_eq!(rolled, 1, "{}", t.dump_log());
}
