//! Rulings batch P011 — spells and abilities: dividing damage as the spell is cast (CR
//! 601.2d), choices made only as a spell resolves (CR 608.2c, 118.12), exactly-one sacrifice
//! costs (CR 601.2h), cantrips with an illegal target (CR 608.2b), intervening "if" (CR
//! 603.4), maximum hand size (CR 514.1, 402.2), drawing from a too-small library (CR 704.5b)
//! and an ability whose source left the battlefield (CR 113.7a).

use crate::r_p009_common::{dmg, kill, obj, pl};
use crate::r_p011_common::*;
use crate::r_p012_common::{cast_card, trigger_on_top};
use crate::r_p127_common::{bounce, exile_now};
use crate::r_s01_common::{supported, tapped_lands};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The `Divide` decisions asked since decision `from`: (total, recipients, minimum each).
fn divides_since(t: &TestGame, from: usize) -> Vec<(u32, usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect()
}

// --- Dividing damage as the spell is cast ---------------------------------------------------

#[test]
fn division_is_chosen_as_the_spell_is_cast_with_at_least_1_each() {
    cr!("601.2d");
    ruling!(
        "Fight with Fire",
        "You choose how many targets Fight with Fire has and how the damage is divided as you put the spell onto the stack. Each target must receive at least 1 damage if Fight with Fire is kicked."
    );
    ruling!(
        "Ghired's Belligerence",
        "You choose how the damage will be divided as you cast Ghired's Belligerence. Each target creature must be assigned at least 1 damage."
    );
    supported("Fight with Fire");
    supported("Ghired's Belligerence");
    // Kicked Fight with Fire: three targets, 8 / 1 / 1.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 7);
    let fwf = t.hand(P0, "Fight with Fire");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![8, 1, 1]));
    t.cast(P0, fwf)
        .kicked(true)
        .targets(&[obj(wurm), obj(bears), pl(P1)])
        .go();
    // Asked while casting (the spell is on the stack, unresolved), at least 1 each.
    assert_eq!(divides_since(&t, from), vec![(10, 3, 1)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(dmg(&t, bears), 1);
    assert_eq!(t.life(P1), 19);
    // A division giving a target 0 isn't accepted: each still gets at least 1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 7);
    let fwf = t.hand(P0, "Fight with Fire");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![10, 0]));
    t.cast(P0, fwf)
        .kicked(true)
        .targets(&[obj(bears), pl(P1)])
        .go();
    t.resolve_all();
    assert!(t.life(P1) < 20, "P1 got at least 1");
    assert!(!t.on_battlefield(bears));

    // Ghired's Belligerence, X = 4 among two creatures: asked as cast, 1 minimum.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 4);
    let gb = t.hand(P0, "Ghired's Belligerence");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    t.cast(P0, gb).x(4).targets(&[obj(giant), obj(wurm)]).go();
    assert_eq!(divides_since(&t, from), vec![(4, 2, 1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(dmg(&t, wurm), 1);
}

#[test]
fn harnessed_lightning_energy_is_chosen_as_it_resolves() {
    cr!("608.2c", "107.14");
    ruling!(
        "Harnessed Lightning",
        "You choose the target creature as you cast Harnessed Lightning. You don't choose how much {E} to pay until Harnessed Lightning is resolving. No player may take actions between the time you choose how much {E} to pay and the time the creature takes damage equal to the {E} paid this way."
    );
    supported("Harnessed Lightning");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.players[0].counters.insert(counters::ENERGY.into(), 1);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Harnessed Lightning", &[obj(wurm)]);
    let numbers = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseNumber { .. } | Decision::ChooseX { .. }))
            .count()
    };
    assert_eq!(numbers(&t), 0, "nothing chosen while casting");
    t.answer(P0, DecisionKind::Number, Answer::Number(4));
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    let res = t.asked().len();
    t.g.resolve_top();
    assert_eq!(numbers(&t), 1);
    // The amount was paid and the damage dealt in the same resolution, with no priority.
    assert!(t.asked()[res..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    assert_eq!(dmg(&t, wurm), 4, "3 energy gained + 1 already there");
    assert_eq!(t.g.player(P0).counters.iter().filter(|(k, _)| k.as_str() == counters::ENERGY).map(|(_, n)| *n).sum::<u32>(), 0);
}

#[test]
fn inferno_trap_targets_any_creature() {
    cr!("115.1a");
    ruling!(
        "Inferno Trap",
        "You may target any creature, not just one that dealt damage to you this turn."
    );
    // Two P1 creatures dealt damage to P0; Inferno Trap (for {R}) targets a third one.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Prodigal Sorcerer");
    let b = t.battlefield(P1, "Prodigal Sorcerer");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.deal_damage(a, pl(P0), 1, false);
    t.g.deal_damage(b, pl(P0), 1, false);
    t.settle();
    t.lands(P0, "Mountain", 1);
    let trap = t.hand(P0, "Inferno Trap");
    let uid = t
        .obj(trap)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("rather than pay"))
        .map(|a| a.uid)
        .unwrap();
    t.cast(P0, trap)
        .method(mtg_engine::object::CastMethod::Alternative(uid))
        .target(wurm)
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

// --- Sacrificing exactly one permanent as a cost --------------------------------------------

#[test]
fn sacrifice_costs_take_exactly_one() {
    cr!("601.2h", "118.3");
    ruling!(
        "Shrapnel Blast",
        "You must sacrifice exactly one artifact to cast this spell; you can’t cast it without sacrificing an artifact, and you can’t sacrifice additional artifacts."
    );
    ruling!(
        "Final Flare",
        "You must sacrifice exactly one creature or enchantment to cast Final Flare; you can’t cast it without sacrificing a permanent, and you can’t sacrifice additional permanents."
    );
    ruling!(
        "Kazuul's Fury // Kazuul's Cliffs",
        "You must sacrifice exactly one creature to cast Kazuul's Fury; you can't cast it without sacrificing a creature, and you can't sacrifice additional creatures."
    );
    // (card, two permanents of the kind, a permanent of another kind, target)
    let cases = [
        ("Shrapnel Blast", ["Mind Stone", "Ornithopter"], "Grizzly Bears"),
        ("Final Flare", ["Grizzly Bears", "Pacifism"], "Mind Stone"),
        ("Kazuul's Fury // Kazuul's Cliffs", ["Grizzly Bears", "Hill Giant"], "Mind Stone"),
    ];
    for (card, kinds, other) in cases {
        supported(card);
        // Only a permanent of another kind: can't be cast.
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.battlefield(P0, other);
        crate::r_s25_common::lands_for_cost(&mut t, P0, card);
        let c = t.hand(P0, card);
        let target = if card == "Shrapnel Blast" || card.starts_with("Kazuul") {
            pl(P1)
        } else {
            obj(wurm)
        };
        assert!(t.cast_with(P0, c, &[target]).is_err(), "{card}");
        t.clear_answers();
        // Two of the kind, both offered: answering both still sacrifices exactly one.
        let a = t.battlefield(P0, kinds[0]);
        let b = t.battlefield(P0, kinds[1]);
        if kinds[1] == "Pacifism" {
            t.g.attach(b, obj(wurm));
            t.g.recompute();
        }
        let from = t.asked().len();
        t.answer_choose(P0, &[obj(a), obj(b)]);
        t.cast_with(P0, c, &[target]).unwrap_or_else(|e| panic!("{card}: {e:?}"));
        let sacrificed = [a, b].iter().filter(|x| !t.on_battlefield(**x)).count();
        assert_eq!(sacrificed, 1, "{card}");
        let asked = t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseEntities {
                    candidates,
                    min,
                    max,
                    ..
                } => Some((candidates.len(), *min, *max)),
                _ => None,
            })
            .expect("asked which to sacrifice");
        assert_eq!(asked, (2, 1, 1), "{card}");
    }
}

// --- Maximum hand size ----------------------------------------------------------------------

#[test]
fn maximum_hand_size_is_checked_only_in_cleanup() {
    cr!("514.1", "402.2");
    ruling!(
        "Roaring Furnace // Steaming Sauna",
        "Your maximum hand size is checked only during the cleanup step of your turn. At other times, you may have more cards in hand than your maximum hand size."
    );
    supported("Roaring Furnace // Steaming Sauna");
    for sauna in [false, true] {
        let mut t = TestGame::new(2);
        let room = t.battlefield(P0, "Roaring Furnace // Steaming Sauna");
        if sauna {
            assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
            t.g.flush_events();
            t.resolve_all();
        }
        let base = t.hand_size(P0);
        for _ in 0..(10 - base) {
            t.hand(P0, "Opt");
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        // Steaming Sauna's end step draw (when unlocked) — still more than seven.
        assert!(t.hand_size(P0) >= 10, "end step: nothing discarded yet");
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.hand_size(P0), if sauna { 11 } else { 7 }, "Sauna {sauna}");
    }
}

// --- Choosing X as the ability resolves; drawing too many ----------------------------------

#[test]
fn vigil_for_the_lost_chooses_x_as_it_resolves() {
    cr!("118.12", "107.3");
    ruling!(
        "Vigil for the Lost",
        "As the ability resolves, you choose a value for X and decide whether to pay {X}. If you do decide to pay {X}, it’s too late for any player to respond since the ability is already in the midst of resolving."
    );
    supported("Vigil for the Lost");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vigil for the Lost");
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    kill(&mut t, bears);
    assert!(trigger_on_top(&t));
    let chose_x = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseX { .. } | Decision::ChooseNumber { .. }))
    };
    assert!(!chose_x(&t), "nothing chosen as it's put on the stack");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.resolve_all();
    assert!(chose_x(&t));
    assert_eq!(t.life(P0), 23);
    assert_eq!(tapped_lands(&t, P0), 3);
}

#[test]
fn sphinxs_revelation_drawing_past_the_library_loses_at_the_next_check() {
    cr!("704.5b", "121.4");
    ruling!(
        "Sphinx's Revelation",
        "If the value chosen for X is larger than the number of cards left in your library, you'll lose the game the next time state-based actions are performed."
    );
    supported("Sphinx's Revelation");
    let mut t = TestGame::new(2);
    let lib = t.library_size(P0);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", lib + 1);
    let sr = t.hand(P0, "Sphinx's Revelation");
    t.cast(P0, sr).x(lib as i64 + 1).go();
    t.g.resolve_top();
    // The spell finished: life gained, library empty, not lost until SBAs are checked.
    assert_eq!(t.life(P0), 20 + lib as i32 + 1);
    assert_eq!(t.library_size(P0), 0);
    assert!(!t.has_lost(P0));
    t.settle();
    assert!(t.has_lost(P0));
}

// --- Cantrips -------------------------------------------------------------------------------

#[test]
fn cantrips_with_an_illegal_target_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Honor",
        "If the target creature is an illegal target as Honor tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    ruling!(
        "Oracle's Restoration",
        "If the target creature is an illegal target as Oracle's Restoration tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card or gain 1 life."
    );
    ruling!(
        "Defiant Strike",
        "If the target creature is an illegal target by the time Defiant Strike tries to resolve, the spell won't resolve. You won't draw a card."
    );
    ruling!(
        "Playful Shove",
        "If the target is illegal as Playful Shove tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    // (card, P0's creature is the target, life gained when it resolves)
    for (card, mine, life) in [
        ("Honor", true, 0),
        ("Oracle's Restoration", true, 1),
        ("Defiant Strike", true, 0),
        ("Playful Shove", false, 0),
    ] {
        supported(card);
        for legal in [true, false] {
            let mut t = TestGame::new(2);
            let target = t.battlefield(if mine { P0 } else { P1 }, "Hill Giant");
            let hand = t.hand_size(P0);
            cast_card(&mut t, P0, card, &[obj(target)]);
            if !legal {
                exile_now(&mut t, target);
            }
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + legal as usize, "{card} {legal}");
            assert_eq!(t.life(P0), 20 + if legal { life } else { 0 }, "{card} {legal}");
        }
    }
}

#[test]
fn sheltering_boughs_with_an_illegal_target_doesnt_enter() {
    cr!("608.3b");
    ruling!(
        "Sheltering Boughs",
        "If the creature the Aura spell targets isn't a legal target as it tries to resolve, Sheltering Boughs is put into its owner's graveyard from the stack. It doesn't enter the battlefield and you won't draw a card."
    );
    supported("Sheltering Boughs");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let hand = t.hand_size(P0);
        cast_card(&mut t, P0, "Sheltering Boughs", &[obj(bears)]);
        if !legal {
            bounce(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Sheltering Boughs").len(), legal as usize);
        assert_eq!(t.in_graveyard(P0, "Sheltering Boughs"), !legal);
        // The hand lost the Bears when bounced (+1) but no card was drawn.
        assert_eq!(t.hand_size(P0), hand + 1, "legal {legal}");
        if legal {
            assert_eq!(t.pt(bears), (3, 5));
        }
    }
}

#[test]
fn steady_progress_has_no_targets() {
    cr!("701.34a", "115.1");
    ruling!(
        "Steady Progress",
        "This spell has no targets. You can cast it even if there are no players or permanents with counters on them. If you do, you'll simply draw a card as it resolves."
    );
    supported("Steady Progress");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    cast_card(&mut t, P0, "Steady Progress", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
}

#[test]
fn donatello_checks_for_an_artifact_twice() {
    cr!("603.4");
    ruling!(
        "Donatello, Turtle Techie",
        "When Donatello enters the battlefield, his triggered ability will check to see if you control an artifact. If you don't, the ability won't trigger at all. If the ability does trigger, it will check again as it tries to resolve. If you no longer control any artifacts, the ability won't do anything."
    );
    supported("Donatello, Turtle Techie");
    // (artifact, removed in response, triggers, draws)
    for (artifact, remove, triggers, draws) in [
        (false, false, false, false),
        (true, true, true, false),
        (true, false, true, true),
    ] {
        let mut t = TestGame::new(2);
        let stone = artifact.then(|| t.battlefield(P0, "Mind Stone"));
        let hand = t.hand_size(P0);
        t.enter(P0, "Donatello, Turtle Techie");
        t.settle();
        assert_eq!(trigger_on_top(&t), triggers, "{artifact} {remove}");
        if remove {
            exile_now(&mut t, stone.unwrap());
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws as usize, "{artifact} {remove}");
    }
}

// --- Wicked Guardian ------------------------------------------------------------------------

#[test]
fn wicked_guardian_still_deals_damage_and_draws_after_leaving() {
    cr!("113.7a", "608.2");
    ruling!(
        "Wicked Guardian",
        "If Wicked Guardian leaves the battlefield before its triggered ability resolves, you may still have it deal 2 damage to another creature you control and draw a card."
    );
    ruling!(
        "Wicked Guardian",
        "No player may take actions between the time you choose which creature (if any) is dealt damage and the time you draw a card if you chose one."
    );
    supported("Wicked Guardian");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wg = t.enter(P0, "Wicked Guardian");
    t.settle();
    assert!(trigger_on_top(&t));
    bounce(&mut t, wg);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(giant)]);
    let from = t.asked().len();
    t.g.resolve_top();
    assert_eq!(dmg(&t, giant), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
}

#[test]
fn wicked_guardian_draws_even_if_the_damage_is_prevented() {
    cr!("615.1", "608.2c");
    ruling!(
        "Wicked Guardian",
        "If the damage Wicked Guardian deals is prevented (perhaps because you chose for it to deal 2 damage to a creature with protection from black), you still draw a card."
    );
    supported("White Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(knight)]);
    t.answer_targets(P0, &[obj(knight)]);
    t.enter(P0, "Wicked Guardian");
    t.resolve_all();
    assert_eq!(dmg(&t, knight), 0, "protection from black");
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn obsessive_skinner_can_target_itself() {
    cr!("603.3d");
    ruling!(
        "Obsessive Skinner",
        "Both of Obsessive Skinner's abilities can target Obsessive Skinner itself."
    );
    supported("Obsessive Skinner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let os = t.enter(P0, "Obsessive Skinner");
    t.answer_targets(P0, &[obj(os)]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(os, counters::PLUS1), 1);
    // The delirium upkeep trigger, at the beginning of P1's upkeep.
    delirium(&mut t, P0);
    t.answer_targets(P0, &[obj(os)]);
    crate::r_p057_common::into_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.counters(os, counters::PLUS1), 2);
}
