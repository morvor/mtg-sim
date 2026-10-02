//! Rulings batch P222 — scry (CR 701.22): scrying after counterspells and removal, once for
//! each trigger, several players scrying at once, and scry 0.

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn giants(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.library_top(p, "Hill Giant");
    }
}

fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"))
}

#[test]
fn no_escape_and_anticognition_scry_even_if_the_spell_cant_be_countered() {
    cr!("701.6b", "608.2c", "701.22a");
    ruling!("No Escape", "A creature or planeswalker spell that can’t be countered is a legal target for No Escape. The spell won’t be countered when No Escape resolves, but you’ll still scry 1.");
    ruling!("Anticognition", "Anticognition can target a spell that can’t be countered. You’ll still scry 2 if an opponent has eight or more cards in their graveyard.");
    supported("No Escape");
    supported("Anticognition");
    supported("Carnage Tyrant");
    for (name, n) in [("No Escape", 1), ("Anticognition", 2)] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        for _ in 0..8 {
            t.graveyard(P1, "Forest");
        }
        t.set_step(P1, Step::PrecombatMain);
        let tyrant = cast(&mut t, P1, "Carnage Tyrant", &[]);
        cast(&mut t, P0, name, &[tyrant.into()]);
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Carnage Tyrant").len(), 1, "{name}");
        assert_eq!(scry_sizes(&t, P0, from), vec![n], "{name}");
    }
}

#[test]
fn anticognition_looks_at_any_opponents_graveyard() {
    cr!("608.2c", "701.22a");
    ruling!("Anticognition", "Anticognition will counter the spell and let you scry 2 if any opponent has eight or more cards in their graveyard, not just that spell’s controller.");
    supported("Anticognition");
    for full in [true, false] {
        let mut t = TestGame::new(3);
        giants(&mut t, P0, 3);
        // P2 (not the spell's controller) has eight cards in their graveyard.
        let n = if full { 8 } else { 7 };
        for _ in 0..n {
            t.graveyard(P2, "Forest");
        }
        t.set_step(P1, Step::PrecombatMain);
        let bears = cast(&mut t, P1, "Grizzly Bears", &[]);
        t.lands(P1, "Wastes", 2);
        t.answer_yes(P1, true);
        cast(&mut t, P0, "Anticognition", &[bears.into()]);
        let from = t.asked().len();
        t.resolve_all();
        if full {
            // Countered outright: P1 isn't offered to pay.
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert_eq!(scry_sizes(&t, P0, from), vec![2]);
        } else {
            // P1 pays {2}: not countered, no scry.
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
            assert!(scry_sizes(&t, P0, from).is_empty());
        }
    }
}

#[test]
fn stymied_hopes_scries_whether_or_not_the_controller_pays() {
    cr!("608.2c", "701.22a", "118.12");
    ruling!("Stymied Hopes", "If Stymied Hopes resolves, you'll scry whether the controller of the target spell pays {1} or not.");
    supported("Stymied Hopes");
    for pays in [true, false] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        t.set_step(P1, Step::PrecombatMain);
        let bears = cast(&mut t, P1, "Grizzly Bears", &[]);
        t.lands(P1, "Wastes", 1);
        t.answer_yes(P1, pays);
        cast(&mut t, P0, "Stymied Hopes", &[bears.into()]);
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), !pays);
        assert_eq!(scry_sizes(&t, P0, from), vec![1], "pays {pays}");
    }
}

#[test]
fn myr_custodian_you_scry_then_each_opponent_may_scry() {
    cr!("701.22c", "101.4");
    ruling!("Myr Custodian", "As the enters-the battlefield ability resolves, first you scry 2. Then each opponent in turn order chooses whether or not to scry 1. Those who do (likely all of them) look at the top card of their library at the same time, then they decide in turn order where their card goes. Each opponent will know the choices of previous players in turn order before making their own choices.");
    supported("Myr Custodian");
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        giants(&mut t, p, 3);
    }
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    cast(&mut t, P0, "Myr Custodian", &[]);
    t.resolve();
    let from = t.asked().len();
    t.resolve_all();
    let seq: Vec<(PlayerId, &str, usize)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards } => Some((*p, "scry", cards.len())),
            Decision::YesNo { .. } => Some((*p, "yes/no", 0)),
            _ => None,
        })
        .collect();
    assert_eq!(
        seq,
        vec![
            (P0, "scry", 2),
            (P1, "yes/no", 0),
            (P2, "yes/no", 0),
            (P1, "scry", 1),
            (P2, "scry", 1),
        ]
    );
}

#[test]
fn veteran_guardmouse_scries_even_if_it_left_the_battlefield() {
    cr!("113.7a", "608.2b", "701.22a");
    ruling!("Veteran Guardmouse", "If Veteran Guardmouse leaves the battlefield before its ability resolves, you’ll still scry 1.");
    supported("Veteran Guardmouse");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    t.set_step(P0, Step::PrecombatMain);
    let mouse = t.battlefield(P0, "Veteran Guardmouse");
    cast(&mut t, P0, "Giant Growth", &[mouse.into()]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "Giant Growth and the valiant trigger");
    destroy(&mut t, mouse);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Veteran Guardmouse"));
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn ugins_insight_with_x_0_doesnt_scry() {
    cr!("701.22b", "107.3");
    ruling!("Ugin's Insight", "If X is 0, you won't scry at all. Any abilities that trigger whenever you scry won't trigger.");
    supported("Ugin's Insight");
    for giant in [false, true] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 8);
        t.set_step(P0, Step::PrecombatMain);
        if giant {
            t.battlefield(P0, "Hill Giant");
        }
        let hand = t.hand_size(P0);
        cast(&mut t, P0, "Ugin's Insight", &[]);
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 3);
        if giant {
            assert_eq!(scry_sizes(&t, P0, from), vec![4]);
            assert_eq!(scry_events(&t, P0), 1);
        } else {
            assert!(scry_sizes(&t, P0, from).is_empty());
            assert_eq!(scry_events(&t, P0), 0);
        }
    }
}

#[test]
fn inscription_of_insight_with_every_target_illegal_does_nothing() {
    cr!("608.2b", "700.2", "702.33a");
    ruling!("Inscription of Insight", "If any targets become illegal, the other targets will still be affected as appropriate. If any targets are chosen and every target becomes illegal, Inscription of Insight doesn't resolve. You won't scry 2 or draw two cards if the middle mode was chosen.");
    supported("Inscription of Insight");
    for spoil_both in [true, false] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 5);
        t.set_step(P0, Step::PrecombatMain);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = in_hand_with_mana(&mut t, P0, "Inscription of Insight");
        t.lands(P0, "Island", 2);
        t.lands(P0, "Wastes", 2);
        t.g.turn.priority = Some(P0);
        t.cast(P0, c)
            .kicked(true)
            .modes(&[0, 1])
            .targets(&[a.into(), b.into()])
            .go();
        destroy(&mut t, a);
        if spoil_both {
            destroy(&mut t, b);
        }
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        t.resolve_all();
        if spoil_both {
            assert!(scry_sizes(&t, P0, from).is_empty());
            assert_eq!(t.hand_size(P1), 0);
            assert_eq!(t.hand_size(P0), hand);
        } else {
            // The remaining target returns, and the middle mode happens.
            assert!(t.in_hand(P1, "Hill Giant"));
            assert_eq!(scry_sizes(&t, P0, from), vec![2]);
            assert_eq!(t.hand_size(P0), hand + 2);
        }
    }
}

#[test]
fn weatherlight_compleated_triggers_for_each_creature_that_dies() {
    cr!("603.2c", "603.10a", "608.2c");
    ruling!("Weatherlight Compleated", "If more than one creature dies at the same time, Weatherlight Compleated’s triggered ability will trigger once for each of those creatures. As each of those resolve, its controller will take the instructed actions. For example, if it has no phyresis counters on it and seven creatures die, its controller will put a counter on it and scry 1 as each of the first six instances of the ability resolve, then put a counter on it and draw a card as the last instance resolves.");
    supported("Weatherlight Compleated");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 8);
    t.set_step(P0, Step::PrecombatMain);
    let wl = t.battlefield(P0, "Weatherlight Compleated");
    for _ in 0..7 {
        t.battlefield(P0, "Grizzly Bears");
    }
    cast(&mut t, P0, "Day of Judgment", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "phyresis"), 7);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(wl, "phyresis"), 7);
    assert_eq!(scry_sizes(&t, P0, from), vec![1; 6]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn simultaneous_events_scry_one_card_at_a_time() {
    cr!("603.2c", "701.22a");
    ruling!("Contraband Kingpin", "If multiple artifacts enter the battlefield simultaneously, you'll scry 1 that many times. You won't look at more than one card from your library at once.");
    ruling!("Season of Growth", "If multiple creatures enter the battlefield under your control simultaneously, you’ll scry 1 for each of those creatures. You won’t look at more than one card from your library at once.");
    ruling!("Warteye Witch", "If multiple creatures you control die simultaneously, you’ll scry 1 that many times. You won’t look at more than one card from your library at once.");
    supported("Contraband Kingpin");
    supported("Season of Growth");
    supported("Warteye Witch");
    supported("Whirler Rogue");
    // Contraband Kingpin: Whirler Rogue creates two Thopter artifact tokens at once.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 4);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Contraband Kingpin");
    cast(&mut t, P0, "Whirler Rogue", &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1, 1]);
    // Season of Growth: Raise the Alarm.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 4);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Season of Growth");
    cast(&mut t, P0, "Raise the Alarm", &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1, 1]);
    // Warteye Witch: it and two other creatures die to Day of Judgment.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 4);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Warteye Witch");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Day of Judgment", &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1, 1, 1]);
}

#[test]
fn removal_that_scries_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b", "702.12b", "701.22a");
    ruling!("Expose to Daylight", "If the chosen target becomes an illegal target for Expose to Daylight, the spell doesn’t resolve. You don’t scry 1. If the target is legal but not destroyed (most likely because it has indestructible), you do scry 1.");
    ruling!("Get the Point", "If the chosen target becomes an illegal target for Get the Point, the spell doesn’t resolve. You don’t scry 1. If the target is legal but not destroyed (most likely because it has indestructible), you do scry 1.");
    ruling!("Fateful End", "If the chosen target is an illegal target by the time Fateful End tries to resolve, the spell won’t resolve. You won’t scry 1.");
    for name in ["Expose to Daylight", "Get the Point", "Fateful End"] {
        supported(name);
        for illegal in [true, false] {
            let mut t = TestGame::new(2);
            giants(&mut t, P0, 3);
            t.set_step(P0, Step::PrecombatMain);
            // Darksteel Myr: an indestructible artifact creature.
            let target = if illegal {
                t.battlefield(P1, "Ornithopter")
            } else {
                t.battlefield(P1, "Darksteel Myr")
            };
            cast(&mut t, P0, name, &[target.into()]);
            if illegal {
                t.g.exile_object(target, None);
            }
            let from = t.asked().len();
            t.resolve_all();
            let n: Vec<usize> = if illegal { vec![] } else { vec![1] };
            assert_eq!(scry_sizes(&t, P0, from), n, "{name} illegal {illegal}");
            if !illegal {
                assert!(t.on_battlefield(target), "{name}");
            }
        }
    }
}
