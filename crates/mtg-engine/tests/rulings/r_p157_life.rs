//! Rulings batch P157 — life lost and gained: "if you lost life last turn" (First
//! Response, Paladin of Atonement), "the first time each turn" (Vengeful Warchief),
//! triggers on paying life as a cost (CR 601.2i, 602.2b), last known toughness (CR
//! 603.10a), damage to each opponent in Two-Headed Giant (CR 810.7), and intervening "if"
//! clauses checked twice (Aerith, CR 603.4).

use crate::r_p157_common::*;
use crate::r_p180_common::plus;
use crate::r_s01_common::watch;
use crate::r_s02_common::can_attack;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 Shocks `p` (2 damage from a red source).
fn shocked(t: &mut TestGame, p: PlayerId) {
    cast_at(t, P1, "Shock", &[pl(p)]);
    t.resolve_all();
}

#[test]
fn first_response_lost_life_last_turn_one_soldier() {
    cr!("120.3a", "119.3", "603.4", "503.1a");
    ruling!(
        "First Response",
        "Damage dealt to you causes you to lose that much life."
    );
    ruling!(
        "First Response",
        "First Response checks only if you lost life during the turn, not whether your life total decreased over the course of the turn. For example, if you lost 2 life and then gained 8 life last turn, First Response's ability will trigger."
    );
    ruling!(
        "First Response",
        "First Response looks at the entire previous turn to determine whether its ability triggers or not. It doesn't matter whether First Response was on the battlefield when you lost life."
    );
    ruling!(
        "First Response",
        "Only one Soldier will be created no matter how much life was lost."
    );
    supported("First Response");
    let mut t = TestGame::new(2);
    // P0 is dealt 2 damage, gains 8 life, and loses 3 more, all before First Response
    // is on the battlefield.
    shocked(&mut t, P0);
    gain(&mut t, P0, 8);
    lose(&mut t, P0, 3);
    assert_eq!(t.life(P0), 23);
    t.battlefield(P0, "First Response");
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
    // In P1's turn P0 lost no life: nothing at P0's upkeep.
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
}

#[test]
fn first_response_the_soldier_cant_attack_that_turn() {
    cr!("302.6");
    ruling!(
        "First Response",
        "The Soldier won't be able to attack the turn it's created (unless something gives it haste)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "First Response");
    t.set_step(P1, Step::PrecombatMain);
    lose(&mut t, P0, 1);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    let s = with_subtype(&t, P0, "Soldier");
    assert_eq!(s.len(), 1);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, s[0]));
}

#[test]
fn paladin_of_atonement_cares_only_whether_you_lost_life() {
    cr!("603.4", "119.3");
    ruling!(
        "Paladin of Atonement",
        "Paladin of Atonement’s first ability cares only whether you lost life last turn, even if Paladin of Atonement wasn’t on the battlefield when that happened. It doesn’t care how much you lost, whether you also gained life, or even whether you gained more life than you lost."
    );
    supported("Paladin of Atonement");
    let mut t = TestGame::new(2);
    lose(&mut t, P0, 1);
    gain(&mut t, P0, 10);
    let p = t.battlefield(P0, "Paladin of Atonement");
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(plus(&t, p), 1);
}

#[test]
fn paladin_of_atonement_gains_its_last_known_toughness() {
    cr!("603.10a", "119.3");
    ruling!(
        "Paladin of Atonement",
        "To determine how much life you gain for the last ability, use Paladin of Atonement’s toughness as it last existed on the battlefield. If its toughness was less than 0, you won’t gain life. (You also won’t lose life.)."
    );
    // Giant Growth: toughness 4 when it's destroyed.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Paladin of Atonement");
    cast_at(&mut t, P0, "Giant Growth", &[obj(p)]);
    t.resolve_all();
    let toughness = t.pt(p).1;
    kill(&mut t, p);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 + toughness);
    // Negative toughness: no life gained or lost.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Paladin of Atonement");
    cast_at(&mut t, P1, "Grasp of Darkness", &[obj(p)]);
    t.resolve_all();
    assert!(!t.on_battlefield(p));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn vengeful_warchief_not_if_you_already_lost_life_this_turn_and_one_counter() {
    cr!("603.2", "603.10");
    ruling!(
        "Vengeful Warchief",
        "If Vengeful Warchief comes under your control after you’ve already lost life in a turn, its ability can’t trigger during that turn."
    );
    ruling!(
        "Vengeful Warchief",
        "You put only one +1/+1 counter on Vengeful Warchief, no matter how much life you lost."
    );
    supported("Vengeful Warchief");
    let mut t = TestGame::new(2);
    lose(&mut t, P0, 1);
    let w = t.battlefield(P0, "Vengeful Warchief");
    lose(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus(&t, w), 0);
    // Next turn: losing 5 gives one counter.
    pass_turn(&mut t);
    lose(&mut t, P0, 5);
    t.resolve_all();
    assert_eq!(plus(&t, w), 1);
    lose(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus(&t, w), 1);
}

#[test]
fn paying_life_triggers_after_activation_before_resolution() {
    cr!("602.2b", "601.2i", "603.3", "118.3");
    ruling!(
        "Vengeful Warchief",
        "If you pay life to cast a spell or activate an ability, you don’t put a +1/+1 counter on Vengeful Warchief until after you’ve finished casting that spell or activating that ability. You put the counter on Vengeful Warchief before that spell or ability resolves."
    );
    ruling!(
        "Vilis, Broker of Blood",
        "If you pay life to cast a spell or activate an ability, you don't draw cards until after you've finished casting that spell or activating that ability. You draw those cards before that spell or ability resolves, but after targets are chosen for it."
    );
    supported("Vilis, Broker of Blood");
    let mut t = TestGame::new(2);
    let vilis = t.battlefield(P0, "Vilis, Broker of Blood");
    let w = t.battlefield(P0, "Vengeful Warchief");
    let giant = t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    // At the target choice, no card has been drawn yet.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.player(P0).hand.len(),
    );
    act_only(&mut t, P0, vilis, 0, &[obj(giant)]);
    assert_eq!(*seen.lock().unwrap(), vec![hand]);
    assert_eq!(t.life(P0), 18);
    // Before anything resolves: nothing drawn and no counter yet; then both triggers are
    // on the stack above the ability.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(plus(&t, w), 0);
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(plus(&t, w), 1);
    assert_eq!(t.stack_len(), 1, "the ability itself hasn't resolved");
    assert_eq!(t.pt(giant), (3, 3));
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn syr_konrad_deals_one_damage_twice_in_two_headed_giant() {
    cr!("810.9", "120.3a");
    ruling!(
        "Syr Konrad, the Grim",
        "In a Two-Headed Giant game, Syr Konrad's first ability causes it to deal 1 damage twice."
    );
    supported("Syr Konrad, the Grim");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Syr Konrad, the Grim");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = t.life(P2);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 2);
    assert_eq!(t.life(P3), t.life(P2));
    let n = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::Damage { .. }))
        .count();
    assert_eq!(n, 2, "1 damage to each of the two opponents");
}

#[test]
fn aerith_checks_life_gained_as_it_triggers_and_amount_as_it_resolves() {
    cr!("603.4", "608.2c");
    ruling!(
        "Aerith, Last Ancient",
        "Aerith's last ability checks at the moment it would trigger to see if you gained life this turn. If you didn't, the ability won't trigger at all. Once your end step begins, it's too late to gain life in order to cause this ability to trigger. However, the last part of the ability will check how much life you've gained as the ability resolves, so if you've only gained 5 life this turn by the time the end step comes around, you could still gain 2 more life while this ability is on the stack in order to return the target creature card to the battlefield instead of your hand."
    );
    supported("Aerith, Last Ancient");
    // No life gained before the end step: no trigger, even if life is gained in it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aerith, Last Ancient");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    gain(&mut t, P0, 7);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Five life gained before; two more while the ability is on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aerith, Last Ancient");
    let bears = t.graveyard(P0, "Grizzly Bears");
    gain(&mut t, P0, 5);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    gain(&mut t, P0, 2);
    t.resolve_all();
    assert!(named_here(&t, "Grizzly Bears"));
    // Only five: to the hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aerith, Last Ancient");
    let bears = t.graveyard(P0, "Grizzly Bears");
    gain(&mut t, P0, 5);
    t.answer_targets(P0, &[obj(bears)]);
    end_step(&mut t);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
