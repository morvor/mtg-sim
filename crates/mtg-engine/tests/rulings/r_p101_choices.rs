//! Rulings batch P101 — choices made as a spell or ability resolves (CR 608.2d–e): numbers,
//! odd or even, and the permanents with the greatest or lowest mana value (Void,
//! Extinction Event, Blot Out, Culling Scales, Summon: Valefor, Hollow Marauder, Polygraph
//! Orb); mana values compared exactly and continuously (Wave of Terror, Favor of the
//! Mighty); choices made as a spell is cast (Talion's number, Nahiri's Sacrifice's X and
//! division, CR 601.2b–d); a spell with no target versus an illegal one (Black Sun's
//! Twilight, CR 608.2b); and dice (Pair o' Dice Lost).

use crate::r_p003_common::divide;
use crate::r_p101_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The index in the decision log of the first decision since `from` matching `f`.
fn first_asked(t: &TestGame, from: usize, f: impl Fn(&Decision) -> bool) -> Option<usize> {
    t.asked()[from..]
        .iter()
        .position(|(_, d)| f(d))
        .map(|i| i + from)
}

#[test]
fn void_targets_as_it_is_cast_and_chooses_its_number_as_it_resolves() {
    cr!("601.2c", "608.2d", "608.2c");
    ruling!(
        "Void",
        "The target player is chosen as you announce Void, but the number isn’t chosen until it’s resolving. No player may choose to take actions between you choosing a number and the rest of Void’s effects."
    );
    supported("Void");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    let from = t.asked().len();
    cast(&mut t, P0, "Void", "3BR", &[pl(P1)]);
    assert!(first_asked(&t, from, |d| matches!(d, Decision::ChooseNumber { .. })).is_none());
    assert!(first_asked(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })).is_some());
    let at = t.asked().len();
    answer_number(&mut t, P0, 2);
    t.resolve_all();
    // The number is asked as Void resolves; the destruction and the discard follow in the
    // same resolution, with no priority in between.
    assert!(first_asked(&t, at, |d| matches!(d, Decision::ChooseNumber { .. })).is_some());
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn extinction_event_chooses_odd_or_even_as_it_resolves() {
    cr!("608.2d", "202.3");
    ruling!(
        "Extinction Event",
        "You choose odd or even while Extinction Event is resolving. Players can't take any actions between the time you make that choice and the time some creatures go extinct."
    );
    supported("Extinction Event");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let thopter = t.battlefield(P0, "Ornithopter");
    let from = t.asked().len();
    cast(&mut t, P0, "Extinction Event", "3B", &[]);
    assert!(first_asked(&t, from, |d| matches!(d, Decision::ChooseOption { .. })).is_none());
    // Even (zero is even).
    crate::r_s03_common::respond(&mut t, P0, even);
    let at = t.asked().len();
    t.resolve_all();
    assert!(first_asked(&t, at, |d| matches!(d, Decision::ChooseOption { .. })).is_some());
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(thopter), Zone::Exile);
    assert!(t.on_battlefield(goblin));
}

/// Answers an odd/even choice with "even".
fn even(_: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseOption { options, .. } => options
            .iter()
            .position(|o| o.to_lowercase().contains("even"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn wave_of_terror_destroys_exactly_the_number_of_age_counters() {
    cr!("702.24a", "503.1a", "504.1", "701.19c");
    ruling!(
        "Wave of Terror",
        "This destroys creatures with mana value exactly equal to the appropriate number, not equal to or less than that number."
    );
    ruling!(
        "Wave of Terror",
        "Will not generally kill zero cost creatures because the age counters go on before the ability triggers. If would be possible if something like Power Conduit were removing counters."
    );
    // Cumulative upkeep puts the first age counter on in the upkeep: at the draw step,
    // Raging Goblin (1) is destroyed; Ornithopter (0) and Grizzly Bears (2) aren't.
    let mut t = TestGame::new(2);
    let wave = t.battlefield(P0, "Wave of Terror");
    let thopter = t.battlefield(P1, "Ornithopter");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(wave, "age"), 1);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
    assert!(!t.on_battlefield(goblin));
    assert!(t.on_battlefield(bears));
    // With two counters (the next turn): Bears, not Goblin-sized or smaller.
    let mut t = TestGame::new(2);
    let wave = t.battlefield(P0, "Wave of Terror");
    put(&mut t, wave, "age", 1);
    let thopter = t.battlefield(P1, "Ornithopter");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(wave, "age"), 2);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert!(t.on_battlefield(thopter) && t.on_battlefield(goblin));
    assert!(!t.on_battlefield(bears));
    // A counter removed after the upkeep (as Power Conduit could): 0, and the Ornithopter
    // is destroyed.
    let mut t = TestGame::new(2);
    let wave = t.battlefield(P0, "Wave of Terror");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    crate::r_p151_common::remove_counters(&mut t, wave, "age", 1);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
}

#[test]
fn favor_of_the_mighty_follows_the_greatest_mana_value_continuously() {
    cr!("611.3a", "702.16a", "613.1f");
    ruling!(
        "Favor of the Mighty",
        "This effect is checked continuously. If a new creature enters with the highest mana value, it gains protection from all colors and the previous highest loses it."
    );
    supported("Favor of the Mighty");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Favor of the Mighty");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(protected_from(&t, giant, bolt));
    assert!(!protected_from(&t, bears, bolt));
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.g.recompute();
    assert!(protected_from(&t, wurm, bolt));
    assert!(!protected_from(&t, giant, bolt));
}

#[test]
fn blot_out_finds_the_greatest_mana_value_as_it_resolves() {
    cr!("608.2d", "608.2c");
    ruling!(
        "Blot Out",
        "Which creature or planeswalker will be exiled is determined as Blot Out resolves."
    );
    supported("Blot Out");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Blot Out", "2B", &[pl(P1)]);
    // In response, the Hill Giant leaves and a Craw Wurm arrives.
    bounce(&mut t, giant);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.zone(wurm), Zone::Exile);
    assert!(t.on_battlefield(bears));
}

#[test]
fn culling_scales_lets_you_choose_among_tied_lowest() {
    cr!("115.1", "603.3d");
    ruling!(
        "Culling Scales",
        "You choose the target. If there's more than one nonland permanent tied for lowest mana value, you choose which one to target."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Culling Scales");
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Memnite");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    let mut names = target_candidate_names(&t, P0, from);
    names.sort();
    assert_eq!(names, ["Memnite", "Ornithopter"]);
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn talion_may_choose_1_or_10() {
    cr!("607.2a", "614.12b", "601.2");
    ruling!("Talion, the Kindly Lord", "The numbers you may choose include 1 and 10.");
    supported("Talion, the Kindly Lord");
    for n in [1, 10] {
        let mut t = TestGame::new(2);
        answer_number(&mut t, P0, n);
        let from = t.asked().len();
        t.enter(P0, "Talion, the Kindly Lord");
        t.settle();
        let range = t.asked()[from..].iter().find_map(|(_, d)| match d {
            Decision::ChooseNumber { min, max, .. } => Some((*min, *max)),
            _ => None,
        });
        assert_eq!(range, Some((1, 10)));
        // P1 casts a spell with that mana value: Opt (1) or Kozilek, the Great Distortion
        // (10).
        t.set_step(P1, Step::PrecombatMain);
        let hand = t.hand_size(P0);
        if n == 1 {
            cast_new(&mut t, P1, "Opt", &[]);
        } else {
            mana(&mut t, P1, "8CC");
            let k = t.hand(P1, "Kozilek, the Great Distortion");
            t.g.turn.priority = Some(P1);
            t.cast(P1, k).go();
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 18, "{n}");
        assert_eq!(t.hand_size(P0), hand + 1, "{n}");
    }
}

#[test]
fn nahiris_sacrifice_chooses_x_targets_and_division_as_it_is_cast() {
    cr!("601.2b", "601.2c", "601.2d", "115.1d");
    ruling!(
        "Nahiri's Sacrifice",
        "You choose the value of X, the targets, and how damage will be divided as you cast Nahiri's Sacrifice."
    );
    ruling!(
        "Nahiri's Sacrifice",
        "You can't choose more than X targets, and each chosen target must receive at least 1 damage."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Raging Goblin");
    let b = t.battlefield(P1, "Raging Goblin");
    t.battlefield(P1, "Raging Goblin");
    let from = t.asked().len();
    answer_x(&mut t, P0, 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![obj(a), obj(b)]));
    divide(&mut t, P0, &[1, 1]);
    let spell = cast(&mut t, P0, "Nahiri's Sacrifice", "1R", &[]);
    assert_eq!(t.zone(spell), Zone::Stack);
    let mine = asked_of(&t, P0, from);
    let max = mine.iter().find_map(|d| match d {
        Decision::ChooseTargets { max, .. } => Some(*max),
        _ => None,
    });
    assert_eq!(max, Some(2), "{mine:?}");
    let min_each = mine.iter().find_map(|d| match d {
        Decision::Divide { min_each, .. } => Some(*min_each),
        _ => None,
    });
    assert_eq!(min_each, Some(1));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(count_named(&t, "Raging Goblin"), 1);
}

#[test]
fn black_suns_twilight_with_no_target_still_returns_a_creature() {
    cr!("608.2b", "601.2c", "115.1");
    ruling!(
        "Black Sun's Twilight",
        "You can cast Black Sun's Twilight without a target (presumably with X equal to 5 or more) just to return a creature card. However, if you do choose a target, and that target is illegal at the time the spell tries to resolve, the spell won't resolve and none of its effects will happen. You won't get to return a creature card to the battlefield."
    );
    supported("Black Sun's Twilight");
    // No target, X = 5: Air Elemental (5) returns.
    let mut t = TestGame::new(2);
    let elemental = t.graveyard(P0, "Air Elemental");
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![]));
    t.answer_choose_moved(P0, &[obj(elemental)]);
    cast_x_resolve(&mut t, P0, "Black Sun's Twilight", 5, &[]);
    assert!(t.on_battlefield(elemental), "{}", t.dump_log());
    // Targeting Grizzly Bears, which leaves first: nothing returns.
    let mut t = TestGame::new(2);
    let elemental = t.graveyard(P0, "Air Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose_moved(P0, &[obj(elemental)]);
    let spell = cast_x(&mut t, P0, "Black Sun's Twilight", 5, &[obj(bears)]);
    bounce(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.zone(elemental), Zone::Graveyard(P0));
}

#[test]
fn pair_o_dice_lost_returns_mana_value_0_cards_whatever_the_roll() {
    cr!("706.2", "202.3a", "608.2c");
    ruling!(
        "Pair o' Dice Lost",
        "You can return any number of cards with mana value 0 (such as land cards) from your graveyard to your hand regardless of the results."
    );
    supported("Pair o' Dice Lost");
    // Rolls 1 and 1 (X = 2): both Forests and Grizzly Bears (2), not Hill Giant (4).
    let mut t = TestGame::new(2);
    let f1 = t.graveyard(P0, "Forest");
    let f2 = t.graveyard(P0, "Forest");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.g.dice.loaded.extend([1, 1]);
    t.answer_choose(P0, &[obj(f1), obj(f2), obj(bears)]);
    cast_resolve(&mut t, P0, "Pair o' Dice Lost", "3GG", &[]);
    assert_eq!(t.zone(t.g.current(f1)), Zone::Hand(P0), "{}", t.dump_log());
    assert_eq!(t.zone(t.g.current(f2)), Zone::Hand(P0));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert!(t.in_exile("Pair o' Dice Lost"));
}

#[test]
fn hollow_marauders_opponents_choose_in_turn_order_from_the_active_one() {
    cr!("101.4", "608.2e", "701.9b");
    ruling!(
        "Hollow Marauder",
        "When Hollow Marauder’s triggered ability resolves, the next target opponent in turn order (or, if it’s a target opponent’s turn, that opponent) chooses a card in hand without revealing it, then each other target opponent in turn order (if any) does the same. All chosen cards are then discarded at the same time."
    );
    supported("Hollow Marauder");
    // In P0's turn: P1, P2, P3. In P2's turn (Leyline of Anticipation lets P0 cast the
    // Marauder then): P2, P3, P1.
    for active in [P0, P2] {
        let mut t = TestGame::new(4);
        if active != P0 {
            t.battlefield(P0, "Leyline of Anticipation");
        }
        for p in [P1, P2, P3] {
            t.hand(p, "Hill Giant");
            t.hand(p, "Hill Giant");
        }
        t.set_step(active, Step::PrecombatMain);
        let from = t.asked().len();
        t.answer(
            P0,
            DecisionKind::Targets,
            Answer::Entities(vec![pl(P1), pl(P2), pl(P3)]),
        );
        cast_resolve(&mut t, P0, "Hollow Marauder", "6B", &[]);
        let mut order: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p != P0 && matches!(d, Decision::ChooseEntities { .. }))
            .map(|(p, _)| *p)
            .collect();
        order.dedup();
        let want = if active == P0 { vec![P1, P2, P3] } else { vec![P2, P3, P1] };
        assert_eq!(order, want, "{}", t.dump_log());
        for p in [P1, P2, P3] {
            assert!(t.in_graveyard(p, "Hill Giant"));
        }
    }
}

#[test]
fn polygraph_orbs_opponent_may_always_lose_3_life() {
    cr!("608.2d", "118.12");
    ruling!(
        "Polygraph Orb",
        "While resolving Polygraph Orb's last ability, your opponent chooses a card to be discarded without revealing it, chooses a creature to be sacrificed, or chooses to do neither. Then that player discards that card, sacrifices that creature, or loses 3 life. Your opponent can always choose to lose 3 life, even if they have cards to discard or creatures to sacrifice."
    );
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Polygraph Orb");
    let evidence = t.graveyard(P0, "Hill Giant");
    t.hand(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, "2");
    t.answer_choose(P0, &[obj(evidence)]);
    t.answer_yes(P1, false);
    t.answer_yes(P1, false);
    act(&mut t, P0, orb, "Each opponent loses 3 life", &[]).expect("Orb");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn valefors_opponents_choose_in_turn_order_and_the_creatures_return_together() {
    cr!("101.4", "608.2e", "714.2b");
    ruling!(
        "Summon: Valefor",
        "While resolving Valefor's first chapter ability, the next opponent in turn order chooses a creature with the greatest mana value among creatures they control, then each other opponent in turn order does the same. Then the chosen creatures are returned to their owners' hands simultaneously."
    );
    supported("Summon: Valefor");
    // Three players: P1 has two Hill Giants (tied), P2 a Hill Giant and Grizzly Bears.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Hill Giant");
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[obj(b)]);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Summon: Valefor", "4U", &[]);
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p != P0 && matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order.first(), Some(&P1), "{:?}", t.asked()[from..].to_vec());
    assert!(t.on_battlefield(a));
    assert_eq!(t.zone(b), Zone::Hand(P1));
    assert_eq!(t.zone(c), Zone::Hand(P2));
    assert!(t.on_battlefield(bears));
}
