//! Rulings batch P096 — "At the beginning of [your/each/the] end step, if you gained N or
//! more life this turn, ...": an intervening "if" clause (CR 603.4) checked as the end step
//! begins, that counts the life you gained this turn whether or not the permanent was on
//! the battlefield then, and ignores life you lost (CR 119.9, 603.2).

use crate::r_p096_common::*;
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::{set_life, two_headed_giant};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The end step abilities of this batch: the card, the life it needs, and a check of its
/// effect (given the game, the permanent, P0's hand size and P1's life before the end
/// step) returning how many times it happened.
struct Threshold {
    name: &'static str,
    needs: i32,
    effect: fn(&TestGame, ObjectId, usize, i32) -> usize,
}

fn tokens_named(t: &TestGame, subtype: &str) -> usize {
    with_subtype(t, P0, subtype)
        .into_iter()
        .filter(|o| t.obj_now(*o).is_token())
        .count()
}

const THRESHOLDS: [Threshold; 7] = [
    // "At the beginning of your end step, if you gained 3 or more life this turn, create a
    // 2/2 white Griffin creature token with flying."
    Threshold {
        name: "Griffin Aerie",
        needs: 3,
        effect: |t, _, _, _| tokens_named(t, "Griffin"),
    },
    // "At the beginning of each end step, if you gained 3 or more life this turn, draw a
    // card."
    Threshold {
        name: "The Gaffer",
        needs: 3,
        effect: |t, _, hand, _| t.hand_size(P0) - hand,
    },
    // "At the beginning of each end step, if you gained 4 or more life this turn, create a
    // 4/4 white Angel creature token with flying and vigilance."
    Threshold {
        name: "Valkyrie Harbinger",
        needs: 4,
        effect: |t, _, _, _| tokens_named(t, "Angel"),
    },
    // "At the beginning of each end step, if you gained 4 or more life this turn, create a
    // 4/4 white Angel creature token with flying."
    Threshold {
        name: "Angelic Accord",
        needs: 4,
        effect: |t, _, _, _| tokens_named(t, "Angel"),
    },
    // "At the beginning of your end step, if you gained 3 or more life this turn, each
    // opponent loses 3 life."
    Threshold {
        name: "Indulging Patrician",
        needs: 3,
        effect: |t, _, _, life| ((life - t.life(P1)) / 3) as usize,
    },
    // "At the beginning of the end step, if you gained 3 or more life this turn, transform
    // this creature."
    Threshold {
        name: "Lone Rider // It That Rides as One",
        needs: 3,
        effect: |t, c, _, _| (t.obj_now(c).face == mtg_engine::object::FaceState::Back) as usize,
    },
    // "At the beginning of your end step, if you gained 3 or more life this turn,
    // transform this creature."
    Threshold {
        name: "Panicked Bystander // Cackling Culprit",
        needs: 3,
        effect: |t, c, _, _| (t.obj_now(c).face == mtg_engine::object::FaceState::Back) as usize,
    },
];

/// P0 gains exactly `n` life (Blessed Wind: "Target player's life total becomes 20." from
/// 20 - n).
fn gain(t: &mut TestGame, n: i32) {
    set_life(t, P0, 20 - n);
    cast_and_resolve(t, P0, "Blessed Wind", &[Entity::Player(P0)]);
    assert_eq!(t.life(P0), 20);
}

/// Runs `th`'s card through P0's end step: `before` happens in P0's main phase (with the
/// permanent already on the battlefield, or put there afterward when `later`). Returns
/// the number of its triggers put on the stack as the end step began, and how many times
/// its effect happened.
fn run(th: &Threshold, later: bool, before: impl FnOnce(&mut TestGame)) -> (usize, usize) {
    supported(th.name);
    let mut t = TestGame::new(2);
    let mut c = (!later).then(|| t.battlefield(P0, th.name));
    before(&mut t);
    if later {
        c = Some(t.battlefield(P0, th.name));
    }
    let c = c.unwrap();
    let (hand, life) = (t.hand_size(P0), t.life(P1));
    to_end_step(&mut t, P0);
    let triggers = triggers_from(&t, c);
    t.resolve_all();
    (triggers, (th.effect)(&t, c, hand, life))
}

#[test]
fn gained_life_thresholds_trigger_once_or_not_at_all() {
    cr!("603.4", "119.9");
    ruling!(
        "Griffin Aerie",
        "If you haven't gained 3 life by the time your end step begins, Griffin Aerie's ability won't trigger at all."
    );
    ruling!(
        "The Gaffer",
        "If you haven't gained 3 or more life by the time an end step begins, The Gaffer's ability won't trigger at all."
    );
    ruling!(
        "Valkyrie Harbinger",
        "If you haven't gained 4 life by the time an end step begins, Valkyrie Harbinger's ability won't trigger at all."
    );
    ruling!(
        "Indulging Patrician",
        "If you haven't gained 3 life by the time your end step begins, Indulging Patrician's last ability won't trigger at all."
    );
    ruling!(
        "Valkyrie Harbinger",
        "You create just one Angel token, no matter how much life you've gained past 4 life."
    );
    ruling!(
        "Griffin Aerie",
        "You create just one Griffin token, no matter how much life you've gained past 3 life."
    );
    ruling!(
        "The Gaffer",
        "You draw just one card, no matter how much life you've gained past 3 life."
    );
    ruling!(
        "Indulging Patrician",
        "Each opponent loses just 3 life, no matter how much life you've gained past 3 life."
    );
    for th in &THRESHOLDS {
        // One life short: no trigger at all.
        assert_eq!(run(th, false, |t| gain(t, th.needs - 1)), (0, 0), "{}", th.name);
        // Exactly enough, and far more: one trigger, one effect.
        assert_eq!(run(th, false, |t| gain(t, th.needs)), (1, 1), "{}", th.name);
        assert_eq!(run(th, false, |t| gain(t, 15)), (1, 1), "{}", th.name);
    }
}

#[test]
fn gained_life_counts_life_gained_before_the_permanent_was_there_and_ignores_losses() {
    cr!("603.4", "119.9", "119.4");
    ruling!(
        "The Gaffer",
        "The Gaffer's ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Valkyrie Harbinger",
        "Valkyrie Harbinger's ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Indulging Patrician",
        "Indulging Patrician's last ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Lone Rider // It That Rides as One",
        "Lone Rider's last ability triggers even if it wasn't on the battlefield when you gained the life this turn."
    );
    ruling!(
        "Lone Rider // It That Rides as One",
        "Lone Rider's ability checks how much life you've gained during the turn, not what your life total is compared to what it was when the turn began. For example, if you gained 4 life and then lost 7 life this turn, it still triggers."
    );
    ruling!(
        "Panicked Bystander // Cackling Culprit",
        "Panicked Bystander's last ability cares about life you gained this turn and ignores any life loss. If you gained 3 life this turn but also lost 3 life, the ability still triggers."
    );
    // P0 gains 4 life, then pays 7 life for Toxic Deluge ("As an additional cost to cast
    // this spell, pay X life. All creatures get -X/-X until end of turn."), losing more
    // than they gained; the permanent is put onto the battlefield afterward.
    for th in &THRESHOLDS {
        let got = run(th, true, |t| {
            gain(t, 4);
            t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(7));
            cast_new(t, P0, "Toxic Deluge", &[]);
            t.resolve_all();
            assert_eq!(t.life(P0), 13);
        });
        assert_eq!(got, (1, 1), "{}", th.name);
    }
    // Panicked Bystander: gained 3, lost 3.
    let th = THRESHOLDS
        .iter()
        .find(|th| th.name.starts_with("Panicked"))
        .unwrap();
    let got = run(th, true, |t| {
        gain(t, 3);
        t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(3));
        cast_new(t, P0, "Toxic Deluge", &[]);
        t.resolve_all();
        assert_eq!(t.life(P0), 17);
    });
    assert_eq!(got, (1, 1));
}

#[test]
fn life_gained_during_the_end_step_is_too_late() {
    cr!("603.4", "513.1");
    ruling!(
        "Angelic Accord",
        "If you haven’t gained 4 or more life during the turn when the end step begins, the ability won’t trigger at all. Gaining life during the end step won’t cause the ability to trigger."
    );
    ruling!(
        "Lone Rider // It That Rides as One",
        "You must have gained life before your end step begins. Lone Rider's last ability won't trigger if you haven't, even if an ability triggering at the same time will cause you to gain life."
    );
    supported("Cosmos Elixir");
    // Cosmos Elixir: "At the beginning of your end step, draw a card if your life total is
    // greater than your starting life total. Otherwise, you gain 2 life." At 20 life, it
    // triggers at the same time and gains P0 2 life during the end step.
    for (name, before) in [
        ("Angelic Accord", 2),
        ("Lone Rider // It That Rides as One", 1),
    ] {
        let th = THRESHOLDS.iter().find(|th| th.name == name).unwrap();
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.battlefield(P0, "Cosmos Elixir");
        gain(&mut t, before);
        let (hand, life) = (t.hand_size(P0), t.life(P1));
        to_end_step(&mut t, P0);
        assert_eq!(triggers_from(&t, c), 0, "{name}");
        t.resolve_all();
        // P0 gained enough life this turn by the end of the end step, too late.
        assert_eq!(t.life(P0), 22);
        assert!(2 + before >= th.needs);
        assert_eq!((th.effect)(&t, c, hand, life), 0, "{name}");
        assert_eq!(triggers_from(&t, c), 0, "{name}");
    }
    // Angelic Accord without Cosmos Elixir: the 4 life gained earlier is enough.
    let th = &THRESHOLDS[3];
    assert_eq!(run(th, false, |t| gain(t, 4)), (1, 1));
}

#[test]
fn indulging_patrician_two_headed_giant_opposing_team_loses_6() {
    cr!("810.9", "810.9a");
    ruling!(
        "Indulging Patrician",
        "In a Two-Headed Giant game, Indulging Patrician's last ability causes the opposing team to lose 6 life."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Indulging Patrician");
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    let from = t.g.turn_events.len();
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(losses_since(&t, from, P2), vec![3]);
    assert_eq!(losses_since(&t, from, P3), vec![3]);
    assert_eq!((t.life(P2), t.life(P3)), (24, 24));
}
