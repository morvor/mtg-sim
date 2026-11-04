//! Rulings batch P096 — "whenever you gain life" (and "whenever an opponent gains life")
//! triggered abilities: each life-gaining event triggers them once, however much life is
//! gained; life gained "for each" of something is one event (CR 119.9, 603.2c); each
//! creature with lifelink dealing combat damage is its own event (CR 702.15b); and in
//! Two-Headed Giant, life gained by a teammate isn't life you gained (CR 810.9).

use crate::r_p096_common::*;
use crate::r_s01_common::{supported, triggers_on_stack, with_subtype};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::{set_life, two_headed_giant};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The number of yes/no questions `p` was asked since index `from` of the asked list.
fn yes_no_asked(t: &TestGame, from: usize, p: PlayerId) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

// ---------------------------------------------------------------------------------------
// One trigger per life-gaining event
// ---------------------------------------------------------------------------------------

#[test]
fn kavu_predator_triggers_once_per_life_gaining_event_of_an_opponent() {
    cr!("119.9", "603.2c");
    ruling!(
        "Kavu Predator",
        "If an opponent gains an amount of life \"for each\" of something (such as from Congregate), that life is gained as one event and Kavu Predator's ability triggers only once."
    );
    ruling!(
        "Kavu Predator",
        "Kavu Predator's last ability triggers just once for each life-gaining event, whether it's 1 life from Kor Firewalker or 6 life from Renewed Faith."
    );
    supported("Kavu Predator");
    // "Whenever an opponent gains life, put that many +1/+1 counters on this creature."
    // Congregate with Kavu and three Bears: P1 gains 8 life at once.
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Kavu Predator");
    congregate(&mut t, P1);
    assert_eq!(triggers_from(&t, kavu), 1);
    t.resolve_all();
    assert_eq!(t.counters(kavu, "+1/+1"), 8);
    // Kor Firewalker ("Whenever a player casts a red spell, you may gain 1 life.") under
    // P1's control: P0 casts Shock, P1 gains 1. Then P1 casts Renewed Faith ("You gain 6
    // life."). One trigger each.
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Kavu Predator");
    t.battlefield(P1, "Kor Firewalker");
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_from(&t, kavu), 1);
    t.resolve_all();
    assert_eq!(t.counters(kavu, "+1/+1"), 1);
    cast_new(&mut t, P1, "Renewed Faith", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_from(&t, kavu), 1);
    t.resolve_all();
    assert_eq!(t.counters(kavu, "+1/+1"), 7);
}

#[test]
fn dawn_of_hope_and_trudge_garden_pay_once_per_life_gaining_event() {
    cr!("119.9", "603.2c", "603.5");
    ruling!(
        "Dawn of Hope",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and Dawn of Hope's first ability triggers only once."
    );
    ruling!(
        "Dawn of Hope",
        "You can't pay {2} multiple times to draw more than one card each time you resolve Dawn of Hope's triggered ability."
    );
    ruling!(
        "Trudge Garden",
        "You may only pay {2} once for each life gain event."
    );
    supported("Dawn of Hope");
    // Dawn of Hope: "Whenever you gain life, you may pay {2}. If you do, draw a card."
    // Trudge Garden: "... you may pay {2}. If you do, create a 4/4 green Fungus Beast
    // creature token with trample." With plenty of mana and every "yes" P0 could want,
    // Congregate's one event gives one card and one token, each paid for once.
    let mut t = TestGame::new(2);
    let dawn = t.battlefield(P0, "Dawn of Hope");
    let garden = t.battlefield(P0, "Trudge Garden");
    t.lands(P0, "Wastes", 12);
    for _ in 0..6 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    congregate(&mut t, P0);
    assert_eq!((triggers_from(&t, dawn), triggers_from(&t, garden)), (1, 1));
    let tapped = crate::r_s01_common::tapped_lands(&t, P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, from, P0), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(with_subtype(&t, P0, "Fungus").len(), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), tapped + 4);
}

#[test]
fn wax_wane_witness_triggers_once_for_life_gained_or_lost_for_each() {
    cr!("119.9", "603.2c");
    ruling!(
        "Wax-Wane Witness",
        "If you gain or lose an amount of life “for each” of something, that life is gained or lost as one event and Wax-Wane Witness’s last ability triggers only once."
    );
    supported("Wax-Wane Witness");
    // "Whenever you gain or lose life during your turn, this creature gets +1/+0 until end
    // of turn."
    let mut t = TestGame::new(2);
    let witness = t.battlefield(P0, "Wax-Wane Witness");
    congregate(&mut t, P0);
    assert_eq!(triggers_from(&t, witness), 1);
    t.resolve_all();
    assert_eq!(t.pt(witness), (3, 4));
    // Stronghold Discipline: "Each player loses 1 life for each creature they control."
    // P0 controls four creatures: 4 life lost at once.
    let from = t.g.turn_events.len();
    cast_new(&mut t, P0, "Stronghold Discipline", &[]);
    t.resolve();
    t.settle();
    assert_eq!(losses_since(&t, from, P0), vec![4]);
    assert_eq!(triggers_from(&t, witness), 1);
    t.resolve_all();
    assert_eq!(t.pt(witness), (4, 4));
}

#[test]
fn wax_wane_witness_two_headed_giant_teammate_gaining_life_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Wax-Wane Witness",
        "In a Two-Headed Giant game, life gained by your teammate won’t cause Wax-Wane Witness’s last ability to trigger even though it caused your team’s life total to increase."
    );
    let mut t = two_headed_giant();
    let witness = t.battlefield(P0, "Wax-Wane Witness");
    cast_new(&mut t, P1, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.life(P0), 34);
    assert_eq!(triggers_from(&t, witness), 0);
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_from(&t, witness), 1);
}

#[test]
fn vanguard_seraph_triggers_once_for_the_first_life_gaining_event_each_turn() {
    cr!("603.2c", "119.9");
    ruling!(
        "Vanguard Seraph",
        "Vanguard Seraph's last ability triggers just once for your first life-gaining event on each turn, no matter how much life you gain."
    );
    supported("Vanguard Seraph");
    // "Whenever you gain life for the first time each turn, surveil 1."
    let mut t = TestGame::new(2);
    let seraph = t.battlefield(P0, "Vanguard Seraph");
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_from(&t, seraph), 1);
    t.resolve_all();
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_from(&t, seraph), 0);
    assert_eq!(t.life(P0), 28);
}

#[test]
fn whenever_you_gain_life_once_per_event_however_much() {
    cr!("119.9", "603.2c");
    ruling!(
        "Cliffhaven Vampire",
        "The ability triggers just once for each life-gaining event, no matter how much life is gained."
    );
    ruling!(
        "Marauding Blight-Priest",
        "Marauding Blight-Priest's ability triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    ruling!(
        "Marauding Blight-Priest",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Marauding Blight-Priest's ability will trigger only once."
    );
    ruling!(
        "Epicure of Blood",
        "The ability of Epicure of Blood triggers just once for each life-gaining event, whether it's 1 life from Impassioned Orator or 4 life from Dawning Angel. If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Epicure of Blood triggers only once."
    );
    ruling!(
        "Sanguine Bond",
        "Sanguine Bond's ability triggers just once for each life-gaining event, whether it's 1 life from Soul Warden or 8 life from Tendrils of Corruption."
    );
    ruling!(
        "Sanguine Bond",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and Sanguine Bond's ability triggers only once."
    );
    ruling!(
        "Oloro, Ageless Ascetic",
        "Oloro's second ability triggers just once for each life-gaining event, no matter how much life is gained."
    );
    ruling!(
        "Searing Meditation",
        "Searing Meditation doesn’t care how much life you gain. It triggers just once for each life-gaining event, whether it’s 1 life from Firemane Angel or 15 life from Brightflame."
    );
    // Each card, with: 1 life (Soul Warden), 4 life (Sacred Nectar), 15 life (Blessed
    // Wind: "Target player's life total becomes 20." at 5 life), life "for each"
    // (Congregate) and "equal to the number" (Ondu Cleric). P1's life shows what Sanguine
    // Bond's "target opponent loses that much life" did.
    for name in [
        "Cliffhaven Vampire",
        "Marauding Blight-Priest",
        "Epicure of Blood",
        "Sanguine Bond",
        "Oloro, Ageless Ascetic",
        "Searing Meditation",
    ] {
        supported(name);
        type Gain = fn(&mut TestGame) -> u32;
        let gains: [(&str, Gain); 4] = [
            ("1 life", |t| {
                t.battlefield(P0, "Soul Warden");
                crate::r_s05_common::enter(t, P0, "Grizzly Bears");
                t.resolve();
                t.settle();
                1
            }),
            ("4 life", |t| {
                cast_new(t, P0, "Sacred Nectar", &[]);
                t.resolve();
                t.settle();
                4
            }),
            ("15 life", |t| {
                set_life(t, P0, 5);
                cast_new(t, P0, "Blessed Wind", &[Entity::Player(P0)]);
                t.resolve();
                t.settle();
                15
            }),
            ("for each", |t| {
                congregate(t, P0);
                2 * t.g.permanents().filter(|o| o.is_creature()).count() as u32
            }),
        ];
        for (what, gain) in gains {
            let mut t = TestGame::new(2);
            let source = t.battlefield(P0, name);
            let from = t.g.turn_events.len();
            let n = gain(&mut t);
            assert_eq!(gains_since(&t, from, P0), vec![n], "{name}: {what}");
            assert_eq!(triggers_from(&t, source), 1, "{name}: {what}");
            if name == "Sanguine Bond" {
                t.resolve_all();
                assert_eq!(t.life(P1), 20 - n as i32, "{name}: {what}");
            }
        }
        // "Equal to the number": Ondu Cleric ("Whenever this creature or another Ally you
        // control enters, you may gain life equal to the number of Allies you control.")
        // entering with two other Allies.
        let mut t = TestGame::new(2);
        let source = t.battlefield(P0, name);
        t.battlefield(P0, "Ondu Cleric");
        t.battlefield(P0, "Ondu Cleric");
        crate::r_s05_common::enter(&mut t, P0, "Ondu Cleric");
        t.answer_yes(P0, true);
        let from = t.g.turn_events.len();
        // The top Cleric trigger resolves: P0 gains 3 at once (4 with Cliffhaven Vampire,
        // an Ally too).
        let allies = with_subtype(&t, P0, "Ally").len() as u32;
        t.resolve();
        assert_eq!(gains_since(&t, from, P0), vec![allies], "{name}");
        assert_eq!(triggers_from(&t, source), 1, "{name}: equal to the number");
    }
}

// ---------------------------------------------------------------------------------------
// Lifelink
// ---------------------------------------------------------------------------------------

#[test]
fn each_lifelinking_creature_is_a_separate_life_gaining_event() {
    cr!("702.15b", "119.9", "510.2");
    ruling!(
        "Cliffhaven Vampire",
        "A creature with lifelink dealing combat damage causes a single life-gaining event. For example, if a single creature with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), Cliffhaven Vampire’s triggered ability will trigger only once. However, if two creatures you control with lifelink deal combat damage at the same time, the ability will trigger twice."
    );
    ruling!(
        "Epicure of Blood",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Epicure of Blood's ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    ruling!(
        "Marauding Blight-Priest",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Marauding Blight-Priest's ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    ruling!(
        "Oloro, Ageless Ascetic",
        "If two creatures you control with lifelink deal combat damage at the same time, Oloro's ability will trigger twice. However, if a single creature with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    // "Whenever you gain life, each opponent loses 1 life." P1 loses 4 combat damage plus
    // 2, or 2 combat damage plus 1.
    for name in [
        "Cliffhaven Vampire",
        "Epicure of Blood",
        "Marauding Blight-Priest",
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        two_lifelinkers(&mut t);
        assert_eq!(t.life(P1), 20 - 4 - 2, "{name}");
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        one_trampling_lifelinker(&mut t);
        assert_eq!(t.life(P1), 20 - 2 - 1, "{name}");
    }
    // Oloro: "Whenever you gain life, you may pay {1}. If you do, draw a card and each
    // opponent loses 1 life." Asked (and paying) once per trigger.
    for (two, triggers) in [(true, 2), (false, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Oloro, Ageless Ascetic");
        t.lands(P0, "Wastes", 4);
        for _ in 0..4 {
            t.answer_yes(P0, true);
        }
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        if two {
            two_lifelinkers(&mut t);
        } else {
            one_trampling_lifelinker(&mut t);
        }
        assert_eq!(yes_no_asked(&t, from, P0), triggers);
        assert_eq!(t.hand_size(P0), hand + triggers);
    }
}

#[test]
fn sanguine_bond_two_lifelinkers_can_target_different_opponents() {
    cr!("702.15b", "119.9", "603.3d");
    ruling!(
        "Sanguine Bond",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Sanguine Bond's ability will trigger twice and you may choose a different opponent for each trigger. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    // "Whenever you gain life, target opponent loses that much life." Three players: the
    // two triggers target P1 and P2.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sanguine Bond");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    two_lifelinkers(&mut t);
    assert_eq!((t.life(P1), t.life(P2)), (20 - 4 - 2, 20 - 2));
    // One creature dealing damage to a creature and a player: one trigger, for 3.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sanguine Bond");
    t.answer_targets(P0, &[Entity::Player(P2)]);
    one_trampling_lifelinker(&mut t);
    assert_eq!((t.life(P1), t.life(P2)), (20 - 2, 20 - 3));
}

#[test]
fn oloro_triggers_when_you_gain_life_even_if_your_life_total_goes_down() {
    cr!("119.9", "510.2", "702.15b");
    ruling!(
        "Oloro, Ageless Ascetic",
        "In some unusual cases, you can gain life even though your life total actually decreases. For example, if you are being attacked by two 3/3 creatures and you block one with a 2/2 creature with lifelink, your life total will decrease by 1 even though you've gained 2 life. Oloro's second ability would trigger."
    );
    ruling!(
        "Oloro, Ageless Ascetic",
        "You decide whether to pay {1} as the second ability resolves. If you do, you'll draw a card and each opponent will lose 1 life."
    );
    // P1 attacks P0 with two Centaur Coursers (3/3); P0 blocks one with Grizzly Bears
    // enchanted with Lifelink. P0 is dealt 3 and gains 2 at once: 19.
    let mut t = TestGame::new(3);
    let oloro = t.battlefield(P0, "Oloro, Ageless Ascetic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Lifelink");
    t.g.attach(aura, Entity::Object(bears));
    t.lands(P0, "Wastes", 1);
    let a = t.battlefield(P1, "Centaur Courser");
    let b = t.battlefield(P1, "Centaur Courser");
    t.g.recompute();
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P0)), (b, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, a)]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::CombatDamage && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(t.life(P0), 19);
    assert_eq!(triggers_from(&t, oloro), 1);
    // Nothing was asked as the ability triggered: the payment is decided on resolution.
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(yes_no_asked(&t, from, P0), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Each opponent lost 1 life.
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
    // Declining: no card and no life loss. (Renewed Faith: "You gain 6 life.")
    cast_new(&mut t, P0, "Renewed Faith", &[]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
}

#[test]
fn oloros_first_ability_works_only_on_the_battlefield() {
    cr!("113.6", "113.6k", "603.6");
    ruling!(
        "Oloro, Ageless Ascetic",
        "Oloro's first ability works only if Oloro is on the battlefield."
    );
    // "At the beginning of your upkeep, you gain 2 life. ... At the beginning of your
    // upkeep, if Oloro is in the command zone, you gain 2 life." On the battlefield or in
    // the command zone, P0 gains 2 life at upkeep, not 4; in the hand, none.
    for (zone, gained) in [("battlefield", 2), ("command", 2), ("hand", 0)] {
        let mut t = TestGame::new(2);
        match zone {
            "battlefield" => t.battlefield(P0, "Oloro, Ageless Ascetic"),
            "command" => t.command(P0, "Oloro, Ageless Ascetic"),
            _ => t.hand(P0, "Oloro, Ageless Ascetic"),
        };
        t.set_step(P0, Step::Untap);
        t.advance_to(P0, Step::Upkeep);
        t.answer_yes(P0, false);
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gained, "{zone}");
    }
}

// ---------------------------------------------------------------------------------------
// Two-Headed Giant
// ---------------------------------------------------------------------------------------

#[test]
fn opponents_losing_life_two_headed_giant() {
    cr!("810.9", "810.9a");
    ruling!(
        "Epicure of Blood",
        "In a Two-Headed Giant game, Epicure of Blood's ability causes the opposing team to lose 1 life twice."
    );
    // "Whenever you gain life, each opponent loses 1 life." Each opponent loses 1: two
    // separate losses of 1 from the opposing team's life total.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Epicure of Blood");
    let from = t.g.turn_events.len();
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    assert_eq!(losses_since(&t, from, P2), vec![1]);
    assert_eq!(losses_since(&t, from, P3), vec![1]);
    assert_eq!((t.life(P2), t.life(P3)), (28, 28));
}

#[test]
fn life_gained_by_a_teammate_doesnt_trigger_whenever_you_gain_life() {
    cr!("810.9", "119.9");
    ruling!(
        "Marauding Blight-Priest",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Marauding Blight-Priest's ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "Sanguine Bond",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Sanguine Bond's ability to trigger, even though it caused your team's life total to increase."
    );
    for name in ["Marauding Blight-Priest", "Sanguine Bond"] {
        let mut t = two_headed_giant();
        let source = t.battlefield(P0, name);
        cast_new(&mut t, P1, "Sacred Nectar", &[]);
        t.resolve();
        t.settle();
        assert_eq!((t.life(P0), t.life(P1)), (34, 34), "{name}");
        assert_eq!(triggers_from(&t, source), 0, "{name}");
        cast_new(&mut t, P0, "Sacred Nectar", &[]);
        t.resolve();
        t.settle();
        assert_eq!(triggers_from(&t, source), 1, "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Sanguine Bond and Exquisite Blood
// ---------------------------------------------------------------------------------------

#[test]
fn sanguine_bond_and_exquisite_blood_loop_until_a_player_wins() {
    cr!("104.4b", "104.3b", "603.2");
    interp!("growing-loops-run-to-the-limit");
    ruling!(
        "Sanguine Bond",
        "If an ability triggers whenever an opponent loses life and causes you to gain life, such as the ability of Exquisite Blood, this will loop until either you win the game or a player takes an action to break the loop. If neither of these happen, the game ends in a draw."
    );
    supported("Exquisite Blood");
    // Exquisite Blood: "Whenever an opponent loses life, you gain that much life." P1 at 5
    // life: gaining 1 life starts the loop, which ends when P1 is at 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanguine Bond");
    t.battlefield(P0, "Exquisite Blood");
    set_life(&mut t, P1, 5);
    gain_one(&mut t, P0);
    assert!(t.has_lost(P1));
    assert!(t.g.player(P0).has_won);
    // (P1 lost the game before the last Exquisite Blood trigger resolved.)
    assert_eq!(t.life(P0), 25);
    // A player can break it: P1 destroys Exquisite Blood with Disenchant in response to
    // Sanguine Bond's first trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanguine Bond");
    let blood = t.battlefield(P0, "Exquisite Blood");
    t.battlefield(P0, "Soul Warden");
    crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "target opponent loses"), 1);
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(blood)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    assert!(!t.has_lost(P1));
    // With Platinum Angel ("You can't lose the game and your opponents can't win the
    // game.") under P1's control, no one can win and no one breaks the loop: the game is a
    // draw (the life totals keep changing, so the engine reaches the draw at its action
    // limit).
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            max_actions: 300,
            ..GameConfig::default()
        },
    );
    t.battlefield(P0, "Sanguine Bond");
    t.battlefield(P0, "Exquisite Blood");
    t.battlefield(P1, "Platinum Angel");
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    assert_eq!(t.g.run(), GameResult::Draw);
    assert!(t.life(P1) < -20);
}
