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

/// The `max` of the target choices asked of `p` since decision `from`.
fn target_maxes(t: &TestGame, p: PlayerId, from: usize) -> Vec<u32> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets { max, .. } if *q == p => Some(*max),
            _ => None,
        })
        .collect()
}

/// P0's library becomes exactly `n` Hill Giants.
fn short_library(t: &mut TestGame, n: usize) {
    t.g.player_mut(P0).library.clear();
    giants(t, P0, n);
}

#[test]
fn celeborn_scries_once_per_attack_and_grows_by_the_cards_looked_at() {
    cr!("508.1", "603.2c", "701.22a", "701.22d");
    ruling!("Celeborn the Wise", "Celeborn the Wise's first ability has you scry 1 just once whenever you attack with one or more Elves, no matter how many Elves you attack with and no matter how many players you attack.");
    supported("Celeborn the Wise");
    // Celeborn and two Llanowar Elves attack two players.
    let mut t = TestGame::new(3);
    giants(&mut t, P0, 3);
    let celeborn = t.battlefield(P0, "Celeborn the Wise");
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Llanowar Elves");
    let from = t.asked().len();
    attack_with(
        &mut t,
        &[
            (celeborn, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
        ],
    );
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    // Its last ability: +1/+1 for the one card looked at.
    assert_eq!(t.pt(celeborn), (4, 4));
    // Attacking with no Elf doesn't trigger it.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    t.battlefield(P0, "Celeborn the Wise");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(scry_sizes(&t, P0, from).is_empty());
}

#[test]
fn scry_triggers_count_the_cards_actually_looked_at() {
    cr!("701.22a", "701.22d", "609.3");
    ruling!("Celeborn the Wise", "Celeborn the Wise's last ability cares about the number of cards you actually looked at. For example, if you were supposed to scry 3 but only had two cards in your library, Celeborn would get +2/+2.");
    ruling!("Elvish Mariner", "Elvish Mariner's last ability cares about the number of cards you actually looked at. For example, if you were supposed to scry 3 but only had two cards in your library, X would be 2.");
    supported("Celeborn the Wise");
    supported("Elvish Mariner");
    supported("Augury Owl");
    for lib in [2usize, 5] {
        let looked = lib.min(3);
        // Celeborn: Augury Owl's "When this creature enters, scry 3."
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        short_library(&mut t, lib);
        let celeborn = t.battlefield(P0, "Celeborn the Wise");
        cast(&mut t, P0, "Augury Owl", &[]);
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(scry_sizes(&t, P0, from), vec![looked]);
        let n = looked as i32;
        assert_eq!(t.pt(celeborn), (3 + n, 3 + n), "library {lib}");
        // Elvish Mariner: tap up to X target nonland permanents.
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        short_library(&mut t, lib);
        t.battlefield(P0, "Elvish Mariner");
        let foes: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        t.battlefield(P1, "Forest");
        let chosen: Vec<Entity> = foes[..looked].iter().map(|o| Entity::Object(*o)).collect();
        t.answer_targets(P0, &chosen);
        cast(&mut t, P0, "Augury Owl", &[]);
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(target_maxes(&t, P0, from), vec![looked as u32], "library {lib}");
        let tapped = foes.iter().filter(|o| t.obj(**o).tapped).count();
        assert_eq!(tapped, looked);
    }
}

#[test]
fn elvish_mariner_attack_scry_taps_one() {
    cr!("508.1m", "701.22d");
    supported("Elvish Mariner");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let mariner = t.battlefield(P0, "Elvish Mariner");
    let foe = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[foe.into()]);
    attack_with(&mut t, &[(mariner, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.obj(foe).tapped);
}

#[test]
fn elrond_puts_counters_on_up_to_x_targets_and_draws_when_they_are_targeted() {
    cr!("701.22d", "603.2", "115.1");
    supported("Elrond, Master of Healing");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    giants(&mut t, P0, 5);
    t.battlefield(P0, "Elrond, Master of Healing");
    let mine: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let chosen: Vec<Entity> = mine[..3].iter().map(|o| Entity::Object(*o)).collect();
    t.answer_targets(P0, &chosen);
    cast(&mut t, P0, "Augury Owl", &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(target_maxes(&t, P0, from), vec![3]);
    let with: Vec<u32> = mine
        .iter()
        .map(|o| t.counters(*o, mtg_engine::types::counters::PLUS1))
        .collect();
    assert_eq!(with, vec![1, 1, 1, 0]);
    // An opponent targets a creature with a counter: P0 may draw. Not one without.
    for (target, draws) in [(mine[0], true), (mine[3], false)] {
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.set_step(P1, Step::PrecombatMain);
        cast(&mut t, P1, "Shock", &[target.into()]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws as usize);
    }
}

#[test]
fn arboreal_alliance_populates_when_you_attack_with_elves() {
    cr!("701.36a", "508.1");
    // Its first ability ("create an X/X green Treefolk creature token") isn't compiled;
    // the attack trigger is.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arboreal Alliance");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    for (attacker, populates) in [(bears, false), (elf, true)] {
        let before = tokens(&t, P0).len();
        attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), before + populates as usize);
        t.advance_to(P0, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
    }
    let copies: Vec<ObjectId> = tokens(&t, P0).into_iter().filter(|o| *o != token).collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).chars.name, t.obj(token).chars.name);
}

#[test]
fn carrot_cake_triggers_on_entering_and_however_it_is_sacrificed() {
    cr!("603.1b", "603.10a", "701.21a");
    ruling!("Carrot Cake", "Carrot Cake's first ability will trigger whether you sacrifice it to pay the cost of its own last ability or due to another cost or effect. For example, if you sacrifice Carrot Cake in order to forage, you'll still create a Rabbit token and scry 1. It's delicious no matter how it's served!");
    supported("Carrot Cake");
    supported("Rusted Slasher");
    let rabbits = |t: &TestGame| crate::r_s01_common::with_subtype(t, P0, "Rabbit").len();
    // Entering: a Rabbit and scry 1.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 5);
    t.set_step(P0, Step::PrecombatMain);
    cast(&mut t, P0, "Carrot Cake", &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(rabbits(&t), 1);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    // Its own ability: gain 3 life, and another Rabbit and scry.
    let cake = t.named_on_battlefield("Carrot Cake")[0];
    t.lands(P0, "Wastes", 2);
    let from = t.asked().len();
    crate::r_s06_common::activate_containing(&mut t, P0, cake, "gain 3 life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(rabbits(&t), 2);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    // Sacrificed for another permanent's cost (Rusted Slasher: "Sacrifice an artifact:
    // Regenerate this creature.").
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 5);
    t.set_step(P0, Step::PrecombatMain);
    let cake = t.battlefield(P0, "Carrot Cake");
    let slasher = t.battlefield(P0, "Rusted Slasher");
    t.answer_choose(P0, &[cake.into()]);
    let from = t.asked().len();
    crate::r_s06_common::activate_containing(&mut t, P0, slasher, "Regenerate").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Carrot Cake"));
    assert_eq!(rabbits(&t), 1);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    // Destroyed (not sacrificed): nothing.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 5);
    let cake = t.battlefield(P0, "Carrot Cake");
    destroy(&mut t, cake);
    t.resolve_all();
    assert_eq!(rabbits(&t), 0);
}

#[test]
fn heaped_harvest_searches_on_entering_and_when_sacrificed() {
    cr!("603.1b", "603.10a", "701.23a");
    supported("Heaped Harvest");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    let forests = |t: &TestGame| t.named_on_battlefield("Forest").len();
    cast(&mut t, P0, "Heaped Harvest", &[]);
    let before = forests(&t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(forests(&t), before + 1);
    let harvest = t.named_on_battlefield("Heaped Harvest")[0];
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    crate::r_s06_common::activate_containing(&mut t, P0, harvest, "gain 3 life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(forests(&t), before + 2);
}
