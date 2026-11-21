//! Rulings batch P094 — "if an opponent lost life this turn": these abilities care
//! whether an opponent lost any life (damage and paying life included), not whether their
//! life total went down; the card needn't have been around when the life was lost.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::can_activate;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` loses `lose` life and gains `gain` life (the gain last), as resolving effects
/// would.
fn lose_then_gain(t: &mut TestGame, p: PlayerId, lose: u32, gain: u32) {
    t.g.lose_life(p, lose);
    t.g.gain_life(p, gain);
    t.g.flush_events();
    t.settle();
}

#[test]
fn enters_with_a_counter_if_an_opponent_lost_life_even_if_they_gained_more() {
    cr!("119.3", "614.1c");
    ruling!(
        "Cindering Cutthroat",
        "Cindering Cutthroat's first ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    ruling!(
        "Frilled Sparkshooter",
        "Frilled Sparkshooter's last ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    ruling!(
        "Mounted Dreadknight",
        "Mounted Dreadknight's last ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    for name in [
        "Cindering Cutthroat",
        "Frilled Sparkshooter",
        "Mounted Dreadknight",
    ] {
        supported(name);
        // No life lost: no counter.
        let mut t = TestGame::new(2);
        t.g.gain_life(P1, 2);
        let c = t.enter(P0, name);
        assert_eq!(t.counters(c, "+1/+1"), 0, "{name}");
        // P1 lost 1 and gained 2: they're above 20, but still lost life.
        let mut t = TestGame::new(2);
        lose_then_gain(&mut t, P1, 1, 2);
        assert_eq!(t.life(P1), 21);
        let c = t.enter(P0, name);
        assert_eq!(t.counters(c, "+1/+1"), 1, "{name}");
        // Only an opponent's loss counts.
        let mut t = TestGame::new(2);
        lose_then_gain(&mut t, P0, 1, 0);
        let c = t.enter(P0, name);
        assert_eq!(t.counters(c, "+1/+1"), 0, "{name}");
    }
}

#[test]
fn bloodtithe_collector_opponent_who_gained_more_still_lost_life() {
    cr!("119.3", "603.4");
    ruling!(
        "Bloodtithe Collector",
        "Bloodtithe Collector's triggered ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    supported("Bloodtithe Collector");
    for lost in [false, true] {
        let mut t = TestGame::new(2);
        t.hand(P1, "Grizzly Bears");
        if lost {
            lose_then_gain(&mut t, P1, 1, 2);
        } else {
            t.g.gain_life(P1, 2);
        }
        t.enter(P0, "Bloodtithe Collector");
        t.resolve_all();
        assert_eq!(t.hand_size(P1), if lost { 0 } else { 1 }, "lost: {lost}");
    }
}

#[test]
fn hired_claw_can_be_activated_after_an_opponent_lost_and_gained_life() {
    cr!("602.5", "119.3");
    ruling!(
        "Hired Claw",
        "Hired Claw’s activated ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    supported("Hired Claw");
    let mut t = TestGame::new(2);
    let claw = t.battlefield(P0, "Hired Claw");
    t.lands(P0, "Mountain", 2);
    t.g.gain_life(P1, 2);
    assert!(!can_activate(&mut t, P0, claw));
    lose_then_gain(&mut t, P1, 1, 0);
    assert_eq!(t.life(P1), 21);
    assert!(can_activate(&mut t, P0, claw));
    t.activate(P0, claw, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(claw, "+1/+1"), 1);
}

/// Runs to P0's end step; returns the number of objects then on the stack and resolves
/// them.
fn end_step_triggers(t: &mut TestGame) -> usize {
    t.advance_to(P0, Step::End);
    t.settle();
    let n = t.stack_len();
    t.resolve_all();
    n
}

#[test]
fn end_step_counters_care_whether_an_opponent_lost_life_not_their_total() {
    cr!("603.4", "119.3");
    ruling!(
        "Stromkirk Bloodthief",
        "Stromkirk Bloodthief's ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    ruling!(
        "Savage Gorger",
        "It doesn’t matter whether an opponent has less life than they started the turn with. If a player lost 4 life but also gained 6 life, that player still lost 4 life."
    );
    for (name, lose, gain) in [("Stromkirk Bloodthief", 1, 2), ("Savage Gorger", 4, 6)] {
        supported(name);
        for lost in [false, true] {
            let mut t = TestGame::new(2);
            let c = t.battlefield(P0, name);
            lose_then_gain(&mut t, P1, if lost { lose } else { 0 }, gain);
            t.answer_targets(P0, &[Entity::Object(c)]);
            assert_eq!(end_step_triggers(&mut t), lost as usize, "{name}");
            assert_eq!(t.counters(c, "+1/+1"), lost as u32, "{name}");
        }
    }
}

#[test]
fn savage_gorger_any_opponent_any_amount_even_before_it_was_there() {
    cr!("603.4", "119.3", "120.3a");
    ruling!(
        "Savage Gorger",
        "Savage Gorger looks at your entire turn to determine whether an opponent lost any life. It doesn’t matter how many opponents lost life or how much life they lost, only whether any of them did. It also doesn’t matter whether Savage Gorger was on the battlefield when they lost life."
    );
    // Three players: both opponents lose life (one by damage) before Gorger arrives.
    let mut t = TestGame::new(3);
    let bolt_src = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(bolt_src, Entity::Player(P1), 1, false);
    lose_then_gain(&mut t, P2, 5, 0);
    let gorger = t.battlefield(P0, "Savage Gorger");
    assert_eq!(end_step_triggers(&mut t), 1);
    assert_eq!(t.counters(gorger, "+1/+1"), 1);
}

#[test]
fn vampire_socialite_opponent_who_gained_more_still_lost_life() {
    cr!("603.4", "614.1c", "119.3");
    ruling!(
        "Vampire Socialite",
        "Vampire Socialite's last two abilities care whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    supported("Vampire Socialite");
    for lost in [false, true] {
        let mut t = TestGame::new(2);
        let knight = t.battlefield(P0, "Vampire Nighthawk");
        lose_then_gain(&mut t, P1, lost as u32, 2);
        t.enter(P0, "Vampire Socialite");
        t.resolve_all();
        assert_eq!(t.counters(knight, "+1/+1"), lost as u32, "lost: {lost}");
        // Another Vampire entering gets the additional counter.
        let later = t.enter(P0, "Vampire Nighthawk");
        assert_eq!(t.counters(later, "+1/+1"), lost as u32, "lost: {lost}");
        // A non-Vampire doesn't.
        let bears = t.enter(P0, "Grizzly Bears");
        assert_eq!(t.counters(bears, "+1/+1"), 0);
    }
}

/// Sygg on P0's battlefield; P0 always draws when asked.
fn sygg(t: &mut TestGame) -> ObjectId {
    supported("Sygg, River Cutthroat");
    t.answer_yes(P0, true);
    t.battlefield(P0, "Sygg, River Cutthroat")
}

#[test]
fn sygg_checks_only_life_lost_not_gained() {
    cr!("603.4", "119.3");
    ruling!(
        "Sygg, River Cutthroat",
        "Sygg’s ability checks only whether life was lost. It doesn’t care whether life was also gained. For example, if an opponent lost 4 life and gained 6 life during the turn, that player will have a higher life total than they started the turn with — but Sygg’s ability will trigger anyway."
    );
    ruling!(
        "Sygg, River Cutthroat",
        "It doesn’t matter how the opponent lost life or who caused it, as long as the total loss of life is 3 or more."
    );
    // Lost 4, gained 6.
    let mut t = TestGame::new(2);
    sygg(&mut t);
    lose_then_gain(&mut t, P1, 4, 6);
    assert_eq!(t.life(P1), 22);
    let hand = t.hand_size(P0);
    assert_eq!(end_step_triggers(&mut t), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // 2 life lost: no trigger.
    let mut t = TestGame::new(2);
    sygg(&mut t);
    lose_then_gain(&mut t, P1, 2, 0);
    assert_eq!(end_step_triggers(&mut t), 0);
    // 1 damage from P1's own creature, 1 from P0's, and 1 life paid: 3 in total.
    let mut t = TestGame::new(2);
    sygg(&mut t);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let ours = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(theirs, Entity::Player(P1), 1, false);
    t.g.deal_damage(ours, Entity::Player(P1), 1, false);
    assert!(t.g.pay_life(P1, 1));
    t.g.flush_events();
    assert_eq!(t.life(P1), 17);
    assert_eq!(end_step_triggers(&mut t), 1);
}

#[test]
fn sygg_triggers_on_any_turn_and_even_if_it_arrived_later() {
    cr!("603.4", "513.1");
    ruling!(
        "Sygg, River Cutthroat",
        "Sygg’s ability checks to see if it triggers at the end of every turn. It doesn’t have to be your turn, and it doesn’t have to be the turn of the opponent that lost life."
    );
    ruling!(
        "Sygg, River Cutthroat",
        "Sygg’s ability will trigger even if Sygg wasn’t on the battlefield at the time some or all of the life was lost."
    );
    // P1's turn in a three-player game: P2 loses 3 life, then Sygg arrives.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    lose_then_gain(&mut t, P2, 3, 0);
    sygg(&mut t);
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn sygg_triggers_once_however_many_opponents_lost_life() {
    cr!("603.4");
    ruling!(
        "Sygg, River Cutthroat",
        "You don’t specify an opponent when the ability triggers. If multiple opponents have lost 3 or more life over the course of the turn, the ability will trigger only once."
    );
    let mut t = TestGame::new(3);
    sygg(&mut t);
    lose_then_gain(&mut t, P1, 3, 0);
    lose_then_gain(&mut t, P2, 4, 0);
    let hand = t.hand_size(P0);
    assert_eq!(end_step_triggers(&mut t), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// Feast on the Fallen on P0's battlefield with a Grizzly Bears; P1 loses life as set up
/// by `during` in P0's turn; returns the Bears' +1/+1 counters after P1's upkeep.
fn feast(during: impl FnOnce(&mut TestGame)) -> u32 {
    supported("Feast on the Fallen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feast on the Fallen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    during(&mut t);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t.counters(bears, "+1/+1")
}

#[test]
fn feast_on_the_fallen_checks_only_whether_life_was_lost_last_turn() {
    cr!("603.4", "119.3", "120.3a");
    ruling!(
        "Feast on the Fallen",
        "Feast on the Fallen checks only if an opponent lost life during the turn, not whether that player’s life total decreased over the course of the turn. For example, if an opponent lost 2 life and then gained 8 life last turn, Feast on the Fallen’s ability will trigger."
    );
    ruling!(
        "Feast on the Fallen",
        "Only one +1/+1 counter will be put on the target creature no matter how much life was lost."
    );
    ruling!(
        "Feast on the Fallen",
        "Damage dealt to an opponent causes that player to lose that much life."
    );
    assert_eq!(feast(|_| {}), 0);
    assert_eq!(feast(|t| lose_then_gain(t, P1, 2, 8)), 1);
    assert_eq!(feast(|t| lose_then_gain(t, P1, 10, 0)), 1);
    // Damage: P0's creature deals 3 damage to P1.
    assert_eq!(
        feast(|t| {
            let src = t.battlefield(P0, "Hill Giant");
            t.g.deal_damage(src, Entity::Player(P1), 3, false);
            t.g.flush_events();
            assert_eq!(t.life(P1), 17);
        }),
        1
    );
}

#[test]
fn gev_doesnt_give_counters_to_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Gev, Scaled Scorch",
        "In the case where Gev and one or more other creatures enter at the same time, those creatures won't enter with additional +1/+1 counters, even if one or more opponents lost life this turn."
    );
    supported("Gev, Scaled Scorch");
    supported("Living Death");
    // Gev already on the battlefield: a creature entering gets one counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gev, Scaled Scorch");
    lose_then_gain(&mut t, P1, 1, 0);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // Gev and Grizzly Bears return together with Living Death: no counters.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Gev, Scaled Scorch");
    t.graveyard(P0, "Grizzly Bears");
    lose_then_gain(&mut t, P1, 1, 0);
    t.lands(P0, "Swamp", 5);
    let ld = t.hand(P0, "Living Death");
    t.cast(P0, ld).go();
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.named_on_battlefield("Gev, Scaled Scorch").len(), 1);
    assert_eq!(t.counters(bears[0], "+1/+1"), 0);
}
