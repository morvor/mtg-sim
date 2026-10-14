//! Rulings batch S04 — cumulative upkeep (CR 702.24): "At the beginning of your upkeep, if
//! this permanent is on the battlefield, put an age counter on this permanent. Then you
//! may pay [cost] for each age counter on it. If you don't, sacrifice it."
//!
//! Also the rulings of Naked Singularity and Reality Twist (cumulative upkeep cards) about
//! the mana their lands produce.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::ability::{Duration, Effect, Sel};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::text_change::TextWords;
use mtg_engine::*;

/// Number of "Pay ...?" questions asked of `p` since decision `from`.
fn pay_questions_since(t: &TestGame, p: PlayerId, from: usize) -> usize {
    asked_of_since(
        t,
        p,
        from,
        |d| matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")),
    )
}

#[test]
fn the_whole_cumulative_upkeep_is_paid_or_the_permanent_is_sacrificed() {
    cr!("702.24a", "118.3", "118.12");
    ruling!(
        "Soldevi Simulacrum",
        "Paying cumulative upkeep is always optional. If it's not paid, the permanent with cumulative upkeep is sacrificed. Partial payments of the total cumulative upkeep cost can't be made. For example, if a permanent with \"cumulative upkeep {1}\" has three age counters on it when its cumulative upkeep ability triggers, it gets another age counter and then its controller chooses to either pay {4} or sacrifice the permanent."
    );
    supported("Soldevi Simulacrum");
    // Soldevi Simulacrum: "Cumulative upkeep {1}".
    // Three age counters: it gets a fourth, and its controller pays {4}.
    let mut t = TestGame::new(2);
    let sim = t.battlefield(P0, "Soldevi Simulacrum");
    t.g.add_counters(Entity::Object(sim), "age", 3, None);
    t.lands(P0, "Wastes", 5);
    next_upkeep(&mut t, P0);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(pay_questions_since(&t, P0, from), 1);
    assert!(t.on_battlefield(sim));
    assert_eq!(t.counters(sim, "age"), 4);
    assert_eq!(untapped_lands(&t, P0), 1);

    // Paying is optional: declining sacrifices it, though the cost could be paid.
    let mut t = TestGame::new(2);
    let sim = t.battlefield(P0, "Soldevi Simulacrum");
    t.g.add_counters(Entity::Object(sim), "age", 3, None);
    t.lands(P0, "Wastes", 5);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(!t.on_battlefield(sim));
    assert!(t.in_graveyard(P0, "Soldevi Simulacrum"));
    assert_eq!(untapped_lands(&t, P0), 5);

    // No partial payment: with only three lands for {4}, nothing is paid and it's
    // sacrificed.
    let mut t = TestGame::new(2);
    let sim = t.battlefield(P0, "Soldevi Simulacrum");
    t.g.add_counters(Entity::Object(sim), "age", 3, None);
    t.lands(P0, "Wastes", 3);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(sim));
    assert!(t.in_graveyard(P0, "Soldevi Simulacrum"));
    assert_eq!(untapped_lands(&t, P0), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn a_non_mana_cumulative_upkeep_is_paid_once_for_each_age_counter() {
    cr!("702.24a");
    ruling!(
        "Aboroth",
        "Paying cumulative upkeep is always optional. If it's not paid, the permanent with cumulative upkeep is sacrificed."
    );
    supported("Aboroth");
    // Aboroth: a 9/9 with "Cumulative upkeep—Put a -1/-1 counter on this creature."
    let mut t = TestGame::new(2);
    let aboroth = t.battlefield(P0, "Aboroth");
    t.g.add_counters(Entity::Object(aboroth), "age", 2, None);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    // Three age counters: three -1/-1 counters.
    assert!(t.on_battlefield(aboroth));
    assert_eq!(t.counters(aboroth, "age"), 3);
    assert_eq!(t.counters(aboroth, "-1/-1"), 3);
    assert_eq!(t.pt(aboroth), (6, 6));
    // Next upkeep it isn't paid: it's sacrificed, with no more -1/-1 counters.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Aboroth"));
    assert_eq!(t.g.obj(aboroth).counter("-1/-1"), 3);
}

/// The mana in `p`'s pool, by type.
fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

/// P0 taps `land` for mana with its first mana ability, answering the choice of which
/// mana replacement applies first with `first` (if the choice is asked). Returns the
/// mana produced, and empties the pool.
fn tap_for_mana(t: &mut TestGame, land: ObjectId, first: Option<usize>) -> Vec<ManaType> {
    if let Some(i) = first {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    t.activate(P0, land, 0, &[]).unwrap();
    t.clear_answers();
    let produced = pool(t, P0);
    t.g.players[0].mana_pool.empty();
    t.g.untap(land);
    produced
}

#[test]
fn a_land_with_two_basic_land_types_produces_either_color_but_all_the_same() {
    cr!("106.12b", "616.1", "616.1e", "305.6");
    ruling!(
        "Naked Singularity",
        "If a land has more than one basic land type, the player tapping the land for mana can choose to produce mana of either color. But if it produces more than one mana, all mana is of the same color."
    );
    supported("Naked Singularity");
    supported("Mana Reflection");
    // Naked Singularity: "If tapped for mana, Plains produce {R}, Islands produce {G},
    // Swamps produce {W}, Mountains produce {U}, and Forests produce {B} instead of any
    // other type." Tundra is a Plains Island.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naked Singularity");
    let tundra = t.battlefield(P0, "Tundra");
    let island = t.battlefield(P0, "Island");
    assert_eq!(tap_for_mana(&mut t, island, None), vec![ManaType::G]);
    // The Plains replacement applies first, then the Islands one: {G}. The other way
    // around: {R}. Never {W} or {U}.
    assert_eq!(tap_for_mana(&mut t, tundra, Some(0)), vec![ManaType::G]);
    assert_eq!(tap_for_mana(&mut t, tundra, Some(1)), vec![ManaType::R]);
    // Paying costs, it makes whichever color is needed.
    for (spell, target) in [("Lightning Bolt", Entity::Player(P1)), ("Giant Growth", Entity::Object(island))] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Naked Singularity");
        t.battlefield(P0, "Tundra");
        let bear = t.battlefield(P0, "Grizzly Bears");
        let card = t.hand(P0, spell);
        let target = match target {
            Entity::Object(_) => Entity::Object(bear),
            p => p,
        };
        assert!(t.cast(P0, card).target(target).try_go().is_ok(), "{spell}");
    }
    // It can't make {W}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naked Singularity");
    t.battlefield(P0, "Tundra");
    let blessing = t.hand(P0, "Chaplain's Blessing");
    assert!(t.cast(P0, blessing).try_go().is_err());

    // Producing two mana (Mana Reflection: "If you tap a permanent for mana, it produces
    // twice as much of that mana instead"), both are the same color.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naked Singularity");
    t.battlefield(P0, "Mana Reflection");
    let tundra = t.battlefield(P0, "Tundra");
    for first in 0..3 {
        let produced = tap_for_mana(&mut t, tundra, Some(first));
        assert_eq!(produced.len(), 2);
        assert!(
            produced == vec![ManaType::R, ManaType::R] || produced == vec![ManaType::G, ManaType::G],
            "{produced:?}"
        );
    }
}

#[test]
fn a_land_type_listed_twice_after_a_text_change_produces_either_color() {
    cr!("106.12b", "612.1", "616.1");
    ruling!(
        "Naked Singularity",
        "If a player uses a text-changing effect to make a land type be listed as producing two different colors, the player tapping the land for mana can choose to produce mana of either color. But if it produces more than one mana, all mana is of the same color."
    );
    // "Island" is changed to "Plains" in Naked Singularity's text: "Plains produce {R},
    // Plains produce {G}, ...".
    let mut t = TestGame::new(2);
    let singularity = t.battlefield(P0, "Naked Singularity");
    // The words offered are the basic land types in order (Plains, Island, Swamp,
    // Mountain, Forest): replace Island (index 1) with Plains (index 0 among the others).
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    run_with(
        &mut t,
        P0,
        Effect::ChangeText {
            what: Sel::Target(0),
            words: TextWords::BasicLandType,
            exclude: vec![],
            duration: Duration::Permanent,
        },
        &[Entity::Object(singularity)],
    );
    let plains = t.battlefield(P0, "Plains");
    let island = t.battlefield(P0, "Island");
    // An Island isn't affected any more.
    assert_eq!(tap_for_mana(&mut t, island, None), vec![ManaType::U]);
    // A Plains makes {R} or {G}, as its controller chooses.
    let a = tap_for_mana(&mut t, plains, Some(0));
    let b = tap_for_mana(&mut t, plains, Some(1));
    let mut both = vec![a[0], b[0]];
    both.sort();
    assert_eq!(both, vec![ManaType::R, ManaType::G]);
    // Two mana at a time are of the same color.
    t.battlefield(P0, "Mana Reflection");
    for first in 0..3 {
        let produced = tap_for_mana(&mut t, plains, Some(first));
        assert_eq!(produced.len(), 2);
        assert_eq!(produced[0], produced[1]);
        assert!(matches!(produced[0], ManaType::R | ManaType::G));
    }
}
