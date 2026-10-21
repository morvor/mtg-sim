//! Fight triggers (CR 701.14a): "Whenever a creature you control fights" (Foe-Razer
//! Regent), "Whenever one or more creatures you control fight or become blocked" (Neyith of
//! the Dire Hunt).

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn foe_razer_regent_counters_at_the_next_end_step_after_a_fight() {
    cr!("701.14a", "603.7a");
    ruling!(
        "Foe-Razer Regent",
        "the delayed triggered ability that puts +1/+1 counters on that creature won’t trigger until the beginning of the following turn’s end step"
    );
    supported("Foe-Razer Regent");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    let regent = t.enter(P0, "Foe-Razer Regent");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.counters(regent, "+1/+1"), 0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(regent, "+1/+1"), 2);
    // Once only.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(regent, "+1/+1"), 2);
    // A fight during an end step: the following turn's end step.
    let elf = t.battlefield(P0, "Craw Wurm");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let prey = t.hand(P0, "Prey Upon");
    mana(&mut t, P0);
    // (Prey Upon is a sorcery: put the fight in the end step directly.)
    let mut ctx = mtg_engine::eval::Ctx::new(Some(prey), P0);
    t.g.exec(
        &mtg_engine::ability::Effect::Fight {
            a: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::Objects(vec![elf])),
            b: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::Objects(vec![other])),
        },
        &mut ctx,
    );
    t.resolve_all();
    assert!(!t.on_battlefield(other));
    assert_eq!(t.counters(elf, "+1/+1"), 0, "not in this end step");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 2);
}

#[test]
fn foe_razer_regent_counts_any_creature_you_control_fighting() {
    cr!("701.14a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Foe-Razer Regent");
    let rhino = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let prey = t.hand(P0, "Prey Upon");
    mana(&mut t, P0);
    t.cast_with(P0, prey, &[Entity::Object(rhino), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(rhino, "+1/+1"), 2);
}

#[test]
fn neyith_draws_once_for_creatures_fighting_or_becoming_blocked() {
    cr!("701.14a", "509.3c", "603.2c");
    ruling!(
        "Neyith of the Dire Hunt",
        "Dealing combat damage isn’t fighting."
    );
    supported("Neyith of the Dire Hunt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neyith of the Dire Hunt");
    let a = t.battlefield(P0, "Craw Wurm");
    let b = t.battlefield(P0, "Craw Wurm");
    let w1 = t.battlefield(P1, "Wall of Stone");
    let w2 = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    // A fight: one card.
    let prey = t.hand(P0, "Prey Upon");
    mana(&mut t, P0);
    t.cast_with(P0, prey, &[Entity::Object(a), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    // (Prey Upon left the hand; one card drawn.)
    assert_eq!(t.hand_size(P0), hand + 1);
    // Two creatures becoming blocked at once: one card.
    t.answer_yes(P0, false);
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    let before = t.hand_size(P0);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(w1, a), (w2, b)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
}
