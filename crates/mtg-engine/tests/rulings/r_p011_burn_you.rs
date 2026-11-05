//! Rulings batch P011 — burn that damages you: Madcap Experiment (no actions mid-resolution,
//! CR 608.2; triggers wait, CR 603.3), Demonlord Belzenlok (repeated process, mana value of
//! X costs, cardless costs and split cards, CR 202.3, 709.4b; one damage event), Yusri's
//! free casting (X is 0, additional costs, CR 107.3b, 118.9d), illegal targets (CR 608.2b),
//! die results below 0 (CR 706.2), Grim Feast in multiplayer, and Murmuring Bosk's land type
//! (CR 205.3i, 305.6).

use crate::r_p009_common::{obj, pl};
use crate::r_p012_common::trigger_on_top;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves every card out of `p`'s library (into exile), leaving it empty.
fn empty_library(t: &mut TestGame, p: PlayerId) {
    for id in t.g.player(p).library.clone() {
        t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.flush_events();
    assert_eq!(t.library_size(p), 0);
}

/// Damage events dealt to player `p` since event index `from` this turn: their amounts.
fn damage_to(t: &TestGame, p: PlayerId, from: usize) -> Vec<u32> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage { target, amount, .. } if *target == pl(p) => Some(*amount),
            _ => None,
        })
        .collect()
}

// --- Madcap Experiment --------------------------------------------------------------------

#[test]
fn madcap_experiment_counts_the_artifact_and_triggers_wait() {
    cr!("608.2", "603.3");
    ruling!(
        "Madcap Experiment",
        "The number of cards revealed this way includes the artifact card you put onto the battlefield."
    );
    ruling!(
        "Madcap Experiment",
        "Any abilities that trigger as the artifact enters the battlefield aren't put onto the stack until Madcap Experiment is done resolving."
    );
    ruling!(
        "Madcap Experiment",
        "Once Madcap Experiment begins resolving, no player may take any actions other than those specified by Madcap Experiment until it's done resolving."
    );
    supported("Madcap Experiment");
    supported("Ichor Wellspring");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Opt", "Opt", "Ichor Wellspring"]);
    cast_new(&mut t, P0, "Madcap Experiment", &[]);
    let hand = t.hand_size(P0);
    let asked = t.asked().len();
    t.g.resolve_top();
    // Two Opts and the Wellspring: 3 damage, all dealt before the trigger is put on the
    // stack; nobody got priority meanwhile.
    assert_eq!(t.life(P0), 17);
    assert!(t.asked()[asked..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    assert_eq!(t.stack_len(), 0, "the trigger isn't on the stack yet");
    t.settle();
    assert!(trigger_on_top(&t));
    assert_eq!(t.hand_size(P0), hand);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.named_on_battlefield("Ichor Wellspring").len(), 1);
}

#[test]
fn madcap_experiment_without_an_artifact_reveals_the_whole_library() {
    cr!("701.20a", "608.2");
    ruling!(
        "Madcap Experiment",
        "If you don't reveal an artifact card, you'll randomize your library and be dealt damage equal to the number of cards in your library."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].life = 50;
    let lib = t.library_size(P0);
    cast_new(&mut t, P0, "Madcap Experiment", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 50 - lib as i32);
    assert_eq!(t.library_size(P0), lib);
}

// --- Demonlord Belzenlok --------------------------------------------------------------------

#[test]
fn demonlord_belzenlok_repeats_and_deals_damage_once() {
    cr!("608.2c");
    ruling!(
        "Demonlord Belzenlok",
        "Once the triggered ability resolves, the ability will continue until you either exile a nonland card with mana value 3 or less or fail to exile any nonland cards while performing the process. You can't choose to stop receiving the blessings of Demonlord Belzenlok any sooner."
    );
    ruling!(
        "Demonlord Belzenlok",
        "Demonlord Belzenlok's ability causes it to deal an amount of damage to you all at once; it doesn't deal 1 damage multiple times."
    );
    supported("Demonlord Belzenlok");
    // Two big cards then the Bears: three cards to hand, 3 damage in one event, no choices.
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &[
            "Forest",
            "Craw Wurm",
            "Colossal Dreadmaw",
            "Grizzly Bears",
            "Hill Giant",
        ],
    );
    let hand = t.hand_size(P0);
    t.enter(P0, "Demonlord Belzenlok");
    t.settle();
    let asked = t.asked().len();
    let ev = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert_eq!(damage_to(&t, P0, ev), vec![3]);
    assert_eq!(t.life(P0), 17);
    assert!(t.asked()[asked..]
        .iter()
        .all(|(p, d)| *p != P0 || matches!(d, Decision::Priority { .. })));
    // Running out of nonland cards ends it: the Wurm, then only a land.
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Craw Wurm", "Forest"]);
    t.enter(P0, "Demonlord Belzenlok");
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn demonlord_belzenlok_mana_values() {
    cr!("202.3a", "202.3d", "202.3e", "709.4b");
    ruling!(
        "Demonlord Belzenlok",
        "If the mana cost of the nonland card includes {X}, X is considered to be 0."
    );
    ruling!(
        "Demonlord Belzenlok",
        "If the nonland card doesn't have a mana cost, its mana value is 0."
    );
    ruling!(
        "Demonlord Belzenlok",
        "The mana value of a split card, such as a card with aftermath from the Amonkhet block, is equal to the combined mana cost of its two halves."
    );
    supported("Fireball");
    supported("Ancestral Vision");
    supported("Fire // Ice");
    // (top card, cards put into hand): Fireball ({X}{R}) is 1, Ancestral Vision 0 — both
    // stop it; Fire // Ice is 4 — it repeats and the Bears follow.
    for (top, n) in [("Fireball", 1), ("Ancestral Vision", 1), ("Fire // Ice", 2)] {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &[top, "Grizzly Bears", "Hill Giant"]);
        let hand = t.hand_size(P0);
        t.enter(P0, "Demonlord Belzenlok");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + n, "{top}");
        assert_eq!(t.life(P0), 20 - n as i32, "{top}");
    }
}

// --- Yusri, Fortune's Flame -------------------------------------------------------------------

/// Yusri attacks and P0 wins five flips (choosing 5); the attack ends and P0 is in the
/// postcombat main phase with free casting from hand.
fn yusri_wins_five(t: &mut TestGame) {
    supported("Yusri, Fortune's Flame");
    let y = t.battlefield(P0, "Yusri, Fortune's Flame");
    t.g.dice.loaded_coins.extend([true; 5]);
    t.answer(P0, DecisionKind::Number, Answer::Number(5));
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.attack(&[(y, pl(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.clear_answers();
}

#[test]
fn yusri_free_spells_have_x_0_and_keep_additional_costs() {
    cr!("107.3b", "118.9d", "118.8a");
    ruling!(
        "Yusri, Fortune's Flame",
        "If an effect allows you to cast a spell with {X} in its cost without paying its mana cost, you must choose 0 for the value of X."
    );
    ruling!(
        "Yusri, Fortune's Flame",
        "When you cast a spell without paying its mana cost, you may still pay additional costs. If any additional costs are required, you must pay them."
    );
    supported("Blaze");
    // Before winning, a free cast isn't allowed.
    let mut t = TestGame::new(2);
    let blaze = t.hand(P0, "Blaze");
    assert!(t
        .cast(P0, blaze)
        .method(CastMethod::Free)
        .target(pl(P1))
        .try_go()
        .is_err());
    // Blaze cast free with X "5" asked: X is 0.
    let mut t = TestGame::new(2);
    yusri_wins_five(&mut t);
    let life = t.life(P1);
    let blaze = t.hand(P0, "Blaze");
    let spell = t
        .cast(P0, blaze)
        .method(CastMethod::Free)
        .x(5)
        .target(pl(P1))
        .go();
    assert_eq!(crate::r_s25_common::x_of(&t, spell).unwrap_or(0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), life);
    // Shrapnel Blast free: the artifact must still be sacrificed.
    let mut t = TestGame::new(2);
    yusri_wins_five(&mut t);
    let blast = t.hand(P0, "Shrapnel Blast");
    assert!(t
        .cast(P0, blast)
        .method(CastMethod::Free)
        .target(pl(P1))
        .try_go()
        .is_err());
    let stone = t.battlefield(P0, "Mind Stone");
    let life = t.life(P1);
    t.cast(P0, blast)
        .method(CastMethod::Free)
        .target(pl(P1))
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    assert_eq!(t.life(P1), life - 5);
    // Fight with Fire free, kicked by paying {5}{R}: 10 damage divided (all to P1).
    let mut t = TestGame::new(2);
    yusri_wins_five(&mut t);
    t.lands(P0, "Mountain", 6);
    let life = t.life(P1);
    let fwf = t.hand(P0, "Fight with Fire");
    t.cast(P0, fwf)
        .method(CastMethod::Free)
        .kicked(true)
        .targets(&[pl(P1)])
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), life - 10);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 6);
}

// --- Illegal targets ------------------------------------------------------------------------

#[test]
fn orcish_burn_to_you_needs_a_legal_target() {
    cr!("608.2b");
    ruling!(
        "Orcish Cannonade",
        "If the chosen target is an illegal target by the time Orcish Cannonade tries to resolve, the spell doesn't resolve. You don't draw a card, but you also don't take 3 damage."
    );
    ruling!(
        "Orcish Artillery",
        "You do not take the 3 points of damage if the ability doesn't resolve because the target is illegal."
    );
    supported("Orcish Cannonade");
    supported("Orcish Artillery");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Orcish Cannonade", &[obj(giant)]);
        if !legal {
            crate::r_p127_common::bounce(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if legal { 17 } else { 20 });
        assert_eq!(t.hand_size(P0), hand + legal as usize);
        assert_eq!(
            crate::r_p009_common::dmg(&t, giant),
            if legal { 2 } else { 0 }
        );

        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let art = t.battlefield(P0, "Orcish Artillery");
        t.activate(P0, art, 0, &[obj(giant)]).unwrap();
        if !legal {
            crate::r_p127_common::bounce(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if legal { 17 } else { 20 }, "Artillery");
    }
}

// --- Dice results below 0 -------------------------------------------------------------------

#[test]
fn non_human_cannonball_negative_result_deals_no_damage() {
    cr!("706.2", "107.1b");
    ruling!(
        "Non-Human Cannonball",
        "If the result is somehow negative because of effects that modify the die roll, Non-Human Cannonball deals 0 damage to you."
    );
    supported("Non-Human Cannonball");
    supported("Night Shift of the Living Dead");
    supported("Xenosquirrels");
    // Natural 1, decreased by Night Shift (1 life) and by Xenosquirrels (a counter): -1.
    // Natural 3, decreased the same way: 1 damage (positive control).
    for (natural, life) in [(1, 19), (3, 18)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Night Shift of the Living Dead");
        let xs = t.battlefield(P0, "Xenosquirrels");
        t.g.objects[xs.0 as usize]
            .counters
            .insert(counters::PLUS1.into(), 2);
        let cb = t.battlefield(P0, "Non-Human Cannonball");
        for _ in 0..4 {
            t.answer_yes(P0, true);
        }
        for _ in 0..4 {
            t.answer(P0, DecisionKind::Option, Answer::Index(1));
        }
        t.g.dice.loaded.push_back(natural);
        crate::r_p009_common::kill(&mut t, cb);
        t.resolve_all();
        assert_eq!(t.counters(xs, counters::PLUS1), 1, "natural {natural}");
        assert_eq!(t.life(P0), life, "natural {natural}");
    }
}

// --- Grim Feast in multiplayer --------------------------------------------------------------

#[test]
fn grim_feast_watches_every_opponent() {
    cr!("102.3");
    ruling!(
        "Grim Feast",
        "It affects all opposing players in a multiplayer game."
    );
    supported("Grim Feast");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grim Feast");
    let giant = t.battlefield(P2, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Craw Wurm");
    crate::r_p009_common::kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    crate::r_p009_common::kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    crate::r_p009_common::kill(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.life(P0), 25, "not P0's own creature");
}

// --- Murmuring Bosk -------------------------------------------------------------------------

#[test]
fn murmuring_bosk_is_a_forest_but_not_basic() {
    cr!("205.3i", "205.4c", "305.6");
    ruling!(
        "Murmuring Bosk",
        "Murmuring Bosk is a Forest, but it isn't a basic land. Things that affect basic lands don't affect it. Things that affect basic land types do. For example, you can't find Murmuring Bosk with Fertilid's ability (\"searches their library for a basic land card\"), but you can find Murmuring Bosk with Everbark Shaman's ability (\"search your library for two Forest cards\")."
    );
    supported("Murmuring Bosk");
    supported("Fertilid");
    supported("Everbark Shaman");
    let searched = |t: &TestGame, from: usize| -> Vec<Entity> {
        t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    };
    // Fertilid: basic land card — only the Forest card is offered.
    let mut t = TestGame::new(2);
    let bosk = t.library_top(P0, "Murmuring Bosk");
    let forest = t.library_top(P0, "Forest");
    let f = t.battlefield(P0, "Fertilid");
    t.g.objects[f.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 2);
    t.lands(P0, "Forest", 2);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bosk)]);
    t.activate(P0, f, 0, &[pl(P0)]).unwrap();
    t.resolve_all();
    let offered = searched(&t, from);
    assert!(offered.contains(&obj(forest)));
    assert!(!offered.contains(&obj(bosk)));
    assert_eq!(t.named_on_battlefield("Murmuring Bosk").len(), 0);
    // Everbark Shaman: two Forest cards — the Bosk is found.
    let mut t = TestGame::new(2);
    let bosk = t.library_top(P0, "Murmuring Bosk");
    let forest = t.library_top(P0, "Forest");
    let es = t.battlefield(P0, "Everbark Shaman");
    t.graveyard(P0, "Treefolk Seedlings");
    t.answer_choose(P0, &[obj(bosk), obj(forest)]);
    t.activate(P0, es, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Murmuring Bosk").len(), 1);
    assert!(t.obj_now(bosk).chars.has_subtype("Forest"));
    assert!(!t.obj_now(bosk).chars.supertypes.contains(Supertype::Basic));
}
