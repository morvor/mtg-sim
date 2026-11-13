//! Rulings batch P060 — modal draw effects (CR 700.2): the chosen modes are performed in
//! the order printed, each in its entirety, with abilities that trigger meanwhile waiting
//! until the whole effect has resolved (CR 608.2c, 603.3); repeated modes, zero modes and
//! copies of modal spells (CR 707.10c); Two-Headed Giant teams losing or gaining life
//! once for each player (CR 810.9); Happily Ever After's upkeep check (CR 603.4).

use crate::r_p050_common::{team_lives, two_headed_giant};
use crate::r_p060_common::*;
use crate::r_s01_common::{attack_with, triggers_on_stack, watch};
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::mana::ManaType;
use mtg_engine::*;

#[test]
fn depth_defiler_modes_in_order_with_triggers_waiting() {
    cr!("700.2", "608.2c", "603.3");
    ruling!(
        "Depth Defiler",
        "Each chosen mode is performed in the order specified. Abilities that trigger while one mode is being performed won't be put onto the stack until Depth Defiler's triggered ability has finished resolving."
    );
    supported("Depth Defiler");
    // Kicked: both modes. P1's Grizzly Bears returns to P1's hand first; then P1 draws two
    // and discards a card: the Bears can be discarded. P0's Spiteful Visions triggers on
    // each draw, but its triggers wait until the whole ability has resolved.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spiteful Visions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 4);
    let dd = t.hand(P0, "Depth Defiler");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, dd).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the spell and its cast trigger");
    t.answer_choose_moved(P1, &[obj(bears)]);
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            (
                g.find_in_zone(Zone::Hand(PlayerId(1)), "Grizzly Bears").len(),
                g.player(PlayerId(1)).hand.len(),
                g.stack.len(),
            )
        },
    );
    let before = libraries(&t);
    t.g.resolve_top();
    // As P1 discards: the Bears are in hand with the two cards drawn; nothing new on the
    // stack (only the Depth Defiler spell, and the resolving ability if it's still there).
    let s = seen.lock().unwrap().clone();
    assert_eq!(s.len(), 1);
    assert_eq!((s[0].0, s[0].1), (1, 3));
    assert!(s[0].2 <= 2);
    assert_eq!(drawn_since(&t, &before), vec![0, 2]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the returned Bears discarded");
    assert_eq!(triggers_on_stack(&t, "damage to that player"), 0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "damage to that player"), 2);
}

/// P0's Rankle, Master of Pranks attacks `opponent` unblocked with the modes `modes`
/// chosen for its trigger; the game stops with the trigger resolved.
fn rankle(t: &mut TestGame, opponent: PlayerId, modes: &[usize]) {
    supported("Rankle, Master of Pranks");
    let r = t.battlefield(P0, "Rankle, Master of Pranks");
    attack_with(t, &[(r, pl(opponent))]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(modes.to_vec()));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    // With no mode chosen, the ability is removed from the stack (CR 603.3c).
    let on_stack = usize::from(!modes.is_empty());
    assert_eq!(triggers_on_stack(t, "choose any number"), on_stack);
    t.resolve_all();
}

#[test]
fn rankle_performs_each_mode_in_full_in_printed_order() {
    cr!("700.2", "608.2c", "101.4");
    ruling!(
        "Rankle, Master of Pranks",
        "For each chosen mode, perform that mode in its entirety before continuing on to the next chosen mode in the order printed."
    );
    // Discard, then lose life and draw (chosen in the other order): every discard comes
    // before every draw, so each player discards the Bears they held.
    let mut t = TestGame::new(2);
    let b0 = bears_in_hand(&mut t, P0, 1);
    let b1 = bears_in_hand(&mut t, P1, 1);
    let from = t.g.turn_events.len();
    rankle(&mut t, P1, &[1, 0]);
    let order: Vec<&str> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Discarded { .. } => Some("discard"),
            Event::Drew { .. } => Some("draw"),
            Event::LifeLost { .. } => Some("life"),
            _ => None,
        })
        .collect();
    // (P1's combat damage loss of 3 life comes first.)
    assert_eq!(
        order,
        vec!["life", "discard", "discard", "life", "life", "draw", "draw"]
    );
    assert_eq!(t.zone(b0[0]), Zone::Graveyard(P0));
    assert_eq!(t.zone(b1[0]), Zone::Graveyard(P1));
    assert_eq!((t.life(P0), t.life(P1)), (19, 16));
    assert_eq!(hands(&t), vec![1, 1]);
}

#[test]
fn rankle_may_choose_no_modes() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Rankle, Master of Pranks",
        "If you really want, you can choose zero modes for Rankle's triggered ability, but carefully consider the hidden costs in not entertaining someone titled Master of Pranks."
    );
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    rankle(&mut t, P1, &[]);
    // The choice offered zero as the least number of modes.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { min: 0, .. })));
    // Nothing happened beyond the combat damage.
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
    assert_eq!(hands(&t), vec![0, 1]);
    assert!(t.on_battlefield(bears));
}

#[test]
fn two_headed_giant_teams_lose_and_gain_once_for_each_player() {
    cr!("810.9");
    ruling!(
        "Rankle, Master of Pranks",
        "In a Two-Headed Giant game, Rankle's second mode causes each team to lose 1 life twice."
    );
    ruling!(
        "Happily Ever After",
        "In a Two-Headed Giant game, Happily Ever After's first ability causes each team to gain 5 life twice."
    );
    ruling!(
        "Stormfist Crusader",
        "In a Two-Headed Giant game, Stormfist Crusader's triggered ability causes each team to lose 1 life twice after each player on that team has drawn a card."
    );
    // Rankle (P0) deals 3 combat damage to P2's team; then each player loses 1 life and
    // draws a card: each team loses 2.
    let mut t = two_headed_giant();
    let before = libraries(&t);
    rankle(&mut t, P2, &[1]);
    assert_eq!(team_lives(&t), (28, 25));
    assert_eq!(drawn_since(&t, &before), vec![1, 1, 1, 1]);
    // Happily Ever After: each team gains 10.
    supported("Happily Ever After");
    let mut t = two_headed_giant();
    t.enter(P0, "Happily Ever After");
    t.resolve_all();
    assert_eq!(team_lives(&t), (40, 40));
    // Stormfist Crusader at P0's team's upkeep: each player draws, then each team loses
    // 2, after all the draws.
    supported("Stormfist Crusader");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Stormfist Crusader");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let from = t.g.turn_events.len();
    let before = libraries(&t);
    t.settle();
    t.resolve_all();
    assert_eq!(team_lives(&t), (28, 28));
    assert_eq!(drawn_since(&t, &before), vec![1, 1, 1, 1]);
    let order: Vec<bool> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Drew { .. } => Some(true),
            Event::LifeLost { .. } => Some(false),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec![true, true, true, true, false, false, false, false]);
}

/// WUBRG and {2} in P0's pool for Unite the Coalition.
fn unite_mana(t: &mut TestGame) {
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        add_mana(t, P0, ty, 1);
    }
    add_mana(t, P0, ManaType::C, 2);
}

/// Casts Unite the Coalition choosing "Target player draws a card" five times, for the
/// players `targets` in that order. Returns the spell.
fn unite_draws(t: &mut TestGame, targets: [PlayerId; 5]) -> ObjectId {
    supported("Unite the Coalition");
    unite_mana(t);
    let u = t.hand(P0, "Unite the Coalition");
    let mut c = t.cast(P0, u).modes(&[1, 1, 1, 1, 1]);
    for p in targets {
        c = c.targets(&[pl(p)]);
    }
    c.go()
}

/// The players who drew since event `from`, in order.
fn drawers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Drew { player, .. } => Some(*player),
            _ => None,
        })
        .collect()
}

#[test]
fn unite_the_coalition_repeated_modes_in_the_chosen_order() {
    cr!("700.2d", "608.2c");
    ruling!(
        "Unite the Coalition",
        "If you choose any mode more than once, you choose the relative order of each of those modes."
    );
    for order in [[P1, P0, P0, P1, P0], [P0, P0, P1, P1, P1]] {
        let mut t = TestGame::new(2);
        unite_draws(&mut t, order);
        let from = t.g.turn_events.len();
        t.resolve_all();
        assert_eq!(drawers(&t, from), order.to_vec());
    }
}

#[test]
fn unite_the_coalition_triggers_wait_until_it_has_resolved() {
    cr!("608.2c", "603.3", "117.3b");
    ruling!(
        "Unite the Coalition",
        "No player can cast spells or activate abilities in between the modes of a resolving spell. Any abilities that trigger won’t be put onto the stack until Unite the Coalition is done resolving."
    );
    // Spiteful Visions triggers on each of the five draws; all five go on the stack only
    // after Unite the Coalition is in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spiteful Visions");
    unite_draws(&mut t, [P1, P0, P1, P0, P1]);
    t.settle();
    t.g.resolve_top();
    assert!(t.in_graveyard(P0, "Unite the Coalition"));
    assert_eq!(triggers_on_stack(&t, "damage to that player"), 0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "damage to that player"), 5);
    // The players pass priority as the game runs: at each priority, whether the spell is on
    // the stack and the cards drawn so far are recorded. No player had priority while it
    // was on the stack with some of its draws done.
    let mut t = TestGame::new(2);
    unite_draws(&mut t, [P1, P0, P1, P0, P1]);
    let look = |g: &mtg_engine::game::Game| {
        (
            g.find_in_zone(Zone::Stack, "Unite the Coalition").len(),
            g.player(PlayerId(0)).hand.len() + g.player(PlayerId(1)).hand.len(),
        )
    };
    let priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let seen: Vec<_> = [P0, P1].map(|p| watch(&mut t, p, priority, look)).into();
    assert!(t.g.run_until(100, |g| g.stack.is_empty()));
    t.g.advance();
    let all: Vec<(usize, usize)> = seen
        .iter()
        .flat_map(|s| s.lock().unwrap().clone())
        .collect();
    assert!(all.contains(&(1, 0)) && all.contains(&(0, 5)), "{all:?}");
    assert!(all.iter().all(|(on_stack, drawn)| *on_stack == 0 || *drawn == 0), "{all:?}");
}

#[test]
fn a_copy_of_unite_the_coalition_keeps_its_modes() {
    cr!("707.10", "707.10c", "700.2g");
    ruling!(
        "Unite the Coalition",
        "If Unite the Coalition is copied, the effect that creates the copy may allow you to choose new targets, but you can’t choose new modes."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    unite_draws(&mut t, [P1, P1, P1, P1, P1]);
    t.lands(P0, "Island", 2);
    let tc = t.hand(P0, "Twincast");
    let u = t.g.stack[0];
    t.cast(P0, tc).target(u).go();
    // New targets for the copy: P0 for each instance.
    t.answer_yes(P0, true);
    for _ in 0..5 {
        t.answer_targets(P0, &[pl(P0)]);
    }
    let from = t.asked().len();
    let before = libraries(&t);
    t.resolve_all();
    // No modes were chosen for the copy: it drew five for P0 with the same modes.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    assert_eq!(drawn_since(&t, &before), vec![5, 5]);
}

#[test]
fn unite_the_coalition_phased_out_permanents_phase_in_untapped_with_counters() {
    cr!("502.1", "702.26a", "702.26c", "702.26d");
    ruling!(
        "Unite the Coalition",
        "Permanents phase back in during their controller’s untap step, immediately before that player untaps their permanents. Creatures that phase in this way are able to attack and pay a cost of {T} during that turn. If a permanent had counters on it when it phased out, it will have those counters when it phases back in."
    );
    supported("Unite the Coalition");
    // P1's tapped Llanowar Elves and Glen Elendra Guardian (with its -1/-1 counter) phase
    // out during P0's turn.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(elves);
    let glen = t.enter(P1, "Glen Elendra Guardian");
    t.g.tap(glen);
    assert_eq!(t.counters(glen, "-1/-1"), 1);
    unite_mana(&mut t);
    let u = t.hand(P0, "Unite the Coalition");
    t.cast(P0, u)
        .modes(&[0, 0, 1, 1, 1])
        .targets(&[obj(elves)])
        .targets(&[obj(glen)])
        .targets(&[pl(P0)])
        .targets(&[pl(P0)])
        .targets(&[pl(P0)])
        .go();
    t.resolve_all();
    assert!(t.obj_now(elves).phased_out && t.obj_now(glen).phased_out);
    // P1's turn: they phase in, untapped, and can attack and pay {T}.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    for c in [elves, glen] {
        assert!(!t.obj_now(c).phased_out);
        assert!(!t.obj_now(c).tapped);
    }
    assert_eq!(t.counters(glen, "-1/-1"), 1, "the counter stays");
    t.activate(P1, elves, 0, &[]).unwrap();
    assert_eq!(t.g.player(P1).mana_pool.total(), 1, "tapped for {{G}}");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, glen));
}

/// P0 has five colors and six card types among permanents (Happily Ever After, Sliver
/// Queen, Sol Ring, Forest) and graveyard (Divination and `instant`); returns the game
/// at the start of P0's next upkeep (triggers not yet on the stack).
fn happily_setup(instant: &str) -> TestGame {
    supported("Happily Ever After");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Happily Ever After");
    t.battlefield(P0, "Sliver Queen");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Forest");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, instant);
    t.set_step(P1, Step::End);
    t
}

#[test]
fn happily_ever_after_checks_before_anyone_can_act() {
    cr!("603.4", "502.4", "503.1a", "104.2b");
    ruling!(
        "Happily Ever After",
        "Players can't take any actions during your turn before your upkeep begins."
    );
    let mut t = happily_setup("Lightning Bolt");
    // P1 holds a Lightning Bolt with mana, but no player gets priority in P0's turn
    // before the ability triggers.
    t.lands(P1, "Mountain", 1);
    t.hand(P1, "Lightning Bolt");
    let turn = t.g.turn.number;
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| (g.turn.number, g.stack.len()),
    );
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you win the game"), 1);
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .all(|(n, _)| *n == turn), "no priority in P0's turn before the trigger");
    t.resolve_all();
    assert!(t.g.player(P0).has_won);
}

#[test]
fn happily_ever_after_ignores_an_adventurers_alternative_card_type() {
    cr!("715.4", "603.4");
    ruling!(
        "Happily Ever After",
        "Ignore the alternative characteristics of an adventurer card in your graveyard. Its alternative card type won't be counted among the card types of cards in your graveyard."
    );
    // Bonecrusher Giant // Stomp is only a creature card in the graveyard: five card
    // types, no trigger. A Lightning Bolt instead: six, and P0 wins.
    let mut t = happily_setup("Bonecrusher Giant // Stomp");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you win the game"), 0);
    let mut t = happily_setup("Lightning Bolt");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(t.g.player(P0).has_won);
}
