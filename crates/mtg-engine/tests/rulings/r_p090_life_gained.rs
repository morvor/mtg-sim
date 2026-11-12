//! Rulings batch P090 — "if you gained life this turn" (infusion, Regal Bloodlord, Lunar
//! Convocation, Markov Purifier, Starseer Mentor, Gwaihir): an intervening "if" clause
//! checked as the end step begins (CR 603.4, 513.1a) against life gained, not the life
//! total (CR 119.3, 119.9); and Tenured Concocter's "becomes the target" trigger (CR
//! 603.2e, 115.3).

use crate::r_s01_common::*;
use crate::r_p108_common::two_headed_giant;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

fn lose(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}

fn bats(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s05_common::tokens_with_subtype(t, p, "Bat").len()
}

/// Advances to P0's end step and puts its triggered abilities on the stack; returns how
/// many there are.
fn to_end_step(t: &mut TestGame, p: PlayerId) -> usize {
    t.advance_to(p, Step::End);
    t.settle();
    t.stack_len()
}

#[test]
fn bloodlord_and_convocation_dont_trigger_without_life_gained_before_the_end_step() {
    cr!("603.4", "513.1a");
    ruling!(
        "Regal Bloodlord",
        "If you didn’t gain life during the turn before the end step begins, Regal Bloodlord’s last ability won’t trigger at all. Gaining life during the end step won’t cause the ability to trigger."
    );
    ruling!(
        "Lunar Convocation",
        "If you haven’t gained life during the turn when your end step begins, Lunar Convocation’s first ability won’t trigger at all. Gaining life during your end step won’t cause the ability to trigger."
    );
    ruling!(
        "Gwaihir, Greatest of the Eagles",
        "If you haven't gained 3 or more life by the time an end step begins, Gwaihir, Greatest of the Eagles's ability won't trigger at all."
    );
    for name in [
        "Regal Bloodlord",
        "Lunar Convocation",
        "Gwaihir, Greatest of the Eagles",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        if name.starts_with("Gwaihir") {
            // 2 life isn't enough.
            gain(&mut t, P0, 2);
        }
        assert_eq!(to_end_step(&mut t, P0), 0, "{name}");
        gain(&mut t, P0, 3);
        assert_eq!(t.stack_len(), 0, "{name}");
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "{name}");
        assert_eq!(tokens(&t, P0).len(), 0, "{name}");
    }
}

#[test]
fn life_gained_counts_even_if_more_was_lost() {
    cr!("119.3", "603.4");
    ruling!(
        "Lunar Convocation",
        "Lunar Convocation’s first ability cares whether you gained life this turn, not how your life total changed. For example, if you gained 2 life and lost 1 life in the same turn, you still lost life. You also still gained life, which is relevant for Lunar Convocation’s second ability."
    );
    ruling!(
        "Markov Purifier",
        "Markov Purifier's ability triggers if you gained any amount of life that turn, even if you also lost that much or more life."
    );
    ruling!(
        "Regal Bloodlord",
        "Regal Bloodlord’s last ability checks if you gained life at any point during the turn. It doesn’t matter if you also lost life or whether your life total is greater than it was at the beginning of the turn. It also doesn’t matter whether Regal Bloodlord was on the battlefield when the life gain happened."
    );
    ruling!(
        "Starseer Mentor",
        "Starseer Mentor’s ability cares whether you gained or lost life this turn, not how your life total changed. For example, if you gained 2 life and lost 2 life in the same turn, you still gained and lost life that turn, even though your life total is the same as it was at the start of the turn."
    );
    ruling!(
        "Gwaihir, Greatest of the Eagles",
        "Gwaihir, Greatest of the Eagles's ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Regal Bloodlord",
        "You create only one Bat token, no matter how much life you gained."
    );
    ruling!(
        "Gwaihir, Greatest of the Eagles",
        "You create just one Bird token, no matter how much life you've gained past 3 life."
    );
    // Lunar Convocation: gained 2, lost 1: both abilities trigger.
    supported("Lunar Convocation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lunar Convocation");
    gain(&mut t, P0, 2);
    lose(&mut t, P0, 1);
    assert_eq!(to_end_step(&mut t, P0), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(bats(&t, P0), 1);
    // Gained 2 and lost 3: still triggers (Markov Purifier, paying {2} to draw).
    supported("Markov Purifier");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Markov Purifier");
    t.lands(P0, "Wastes", 2);
    gain(&mut t, P0, 2);
    lose(&mut t, P0, 3);
    let hand = t.hand_size(P0);
    assert_eq!(to_end_step(&mut t, P0), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Regal Bloodlord and Gwaihir arrive after the life was gained (and more was lost):
    // one token each, however much was gained.
    for (name, sub) in [
        ("Regal Bloodlord", "Bat"),
        ("Gwaihir, Greatest of the Eagles", "Bird"),
    ] {
        let mut t = TestGame::new(2);
        gain(&mut t, P0, 3);
        gain(&mut t, P0, 7);
        lose(&mut t, P0, 15);
        t.battlefield(P0, name);
        assert_eq!(to_end_step(&mut t, P0), 1, "{name}");
        t.resolve_all();
        assert_eq!(
            crate::r_s05_common::tokens_with_subtype(&t, P0, sub).len(),
            1,
            "{name}"
        );
    }
    // Starseer Mentor: gained 2 and lost 2.
    supported("Starseer Mentor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starseer Mentor");
    gain(&mut t, P0, 2);
    lose(&mut t, P0, 2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(to_end_step(&mut t, P0), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn regal_bloodlord_ignores_a_teammates_life_gain() {
    cr!("810.9", "603.4");
    ruling!(
        "Regal Bloodlord",
        "In a Two-Headed Giant game, life gained by your teammate won’t satisfy Regal Bloodlord’s last ability, even though it caused your team’s life total to increase."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Regal Bloodlord");
    let life = t.life(P0);
    gain(&mut t, P1, 2);
    assert_eq!(t.life(P0), life + 2, "the team's life total rose");
    assert_eq!(to_end_step(&mut t, P0), 0);
    // Positive control: P0's own gain.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Regal Bloodlord");
    gain(&mut t, P0, 2);
    assert_eq!(to_end_step(&mut t, P0), 1);
}

#[test]
fn starseer_mentor_the_opponent_may_always_lose_three_life() {
    cr!("118.12", "608.2d");
    ruling!(
        "Starseer Mentor",
        "While resolving Starseer Mentor’s last ability, the target opponent chooses a card to be discarded without revealing it, chooses a nonland permanent to be sacrificed, or chooses to do neither. Then that player discards that card, sacrifices that permanent, or loses 3 life. That player can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    // (P1 declines; P1 sacrifices; P1 discards.)
    for choice in 0..3 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Starseer Mentor");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let forest = t.hand(P1, "Forest");
        gain(&mut t, P0, 1);
        assert_eq!(to_end_step(&mut t, P0), 1);
        let from = t.asked().len();
        // P1 is asked whether to sacrifice, then whether to discard.
        match choice {
            0 => {
                t.answer_yes(P1, false);
                t.answer_yes(P1, false);
            }
            1 => {
                t.answer_yes(P1, true);
                t.answer_choose(P1, &[Entity::Object(bears)]);
            }
            _ => {
                t.answer_yes(P1, false);
                t.answer_yes(P1, true);
                t.answer_choose(P1, &[Entity::Object(forest)]);
            }
        }
        t.resolve_all();
        let asked: Vec<_> = asked_since(&t, from)
            .into_iter()
            .filter(|(p, _)| *p == P1)
            .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
            .collect();
        match choice {
            0 => {
                assert_eq!(t.life(P1), 17, "{asked:?}\n{}", t.dump_log());
                assert!(t.on_battlefield(bears));
                assert_eq!(t.zone(forest), Zone::Hand(P1));
            }
            1 => {
                assert_eq!(t.life(P1), 20, "{asked:?}\n{}", t.dump_log());
                assert!(!t.on_battlefield(bears));
            }
            _ => {
                assert_eq!(t.life(P1), 20, "{asked:?}\n{}", t.dump_log());
                assert!(t.in_graveyard(P1, "Forest"));
            }
        }
    }
}

#[test]
fn foolish_fate_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Foolish Fate",
        "If the target creature is an illegal target as Foolish Fate tries to resolve, it won't resolve and none of its effects will happen. No player will lose life."
    );
    supported("Foolish Fate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    gain(&mut t, P0, 1);
    give_mana_for(&mut t, P0, "Foolish Fate");
    let ff = t.hand(P0, "Foolish Fate");
    t.cast(P0, ff).target(bears).go();
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Positive control: with a legal target, its controller loses 3 life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    gain(&mut t, P0, 1);
    give_mana_for(&mut t, P0, "Foolish Fate");
    let ff = t.hand(P0, "Foolish Fate");
    t.cast(P0, ff).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn tenured_concocter_triggers_once_per_spell_and_resolves_first() {
    cr!("603.2e", "115.3", "405.5");
    ruling!(
        "Tenured Concocter",
        "If a spell or ability targets Tenured Concocter more than once, its second ability still triggers only once."
    );
    ruling!(
        "Tenured Concocter",
        "Tenured Concocter's second ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Tenured Concocter");
    supported("Seeds of Strength");
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn. Target creature
    // gets +1/+1 until end of turn. Target creature gets +1/+1 until end of turn." — all
    // three at Tenured Concocter.
    let mut t = TestGame::new(2);
    let tc = t.battlefield(P0, "Tenured Concocter");
    give_mana_for(&mut t, P1, "Seeds of Strength");
    let seeds = t.hand(P1, "Seeds of Strength");
    let o = Entity::Object(tc);
    let s = t.cast(P1, seeds).target(tc).target(tc).target(tc).go();
    assert_eq!(mtg_engine::copy::chosen_targets(&t.g, s), vec![o, o, o]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Shock at it, then the Shock is countered: the trigger (on top) still resolves.
    let mut t = TestGame::new(2);
    let tc = t.battlefield(P0, "Tenured Concocter");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(tc).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_ne!(top, shock);
    assert_eq!(t.g.obj(top).controller, P0);
    // Counter the Shock (Cancel from P0).
    t.lands(P0, "Island", 3);
    let cancel = t.hand(P0, "Cancel");
    let shock_now = t.g.current(shock);
    t.cast(P0, cancel).target(shock_now).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Shock"));
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.on_battlefield(tc));
}
