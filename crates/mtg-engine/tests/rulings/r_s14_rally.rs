//! Rulings batch S14 — rally: "Whenever this creature or another Ally you control enters,
//! ..." (an ability word, CR 207.2c). Permanents entering at the same time see each other
//! enter (CR 603.6a), and the rally abilities that trigger together are put on the stack
//! in the order their controller chooses (CR 603.3b).

use crate::r_s01_common::*;
use crate::r_s03_common::{respond, run_effect};
use crate::r_s04_common::stack_items;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_rally_creature_entering_with_other_allies_triggers_for_each_of_them() {
    cr!("603.6a", "603.2", "603.2c");
    ruling!(
        "Kalastria Healer",
        "If a creature with a rally ability enters the battlefield under your control at the same time as other Allies, that ability will trigger once for each of those creatures and once for the creature with the ability itself."
    );
    supported("Kalastria Healer");
    supported("Kor Castigator");
    // Two Kalastria Healers ("Rally — ... each opponent loses 1 life and you gain 1
    // life") and Kor Castigator (an Ally) are returned from P0's graveyard to the
    // battlefield at the same time: each Healer's rally ability triggers three times.
    let mut t = TestGame::new(2);
    let cards = vec![
        Entity::Object(t.graveyard(P0, "Kalastria Healer")),
        Entity::Object(t.graveyard(P0, "Kalastria Healer")),
        Entity::Object(t.graveyard(P0, "Kor Castigator")),
    ];
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &cards,
    );
    assert_eq!(t.named_on_battlefield("Kalastria Healer").len(), 2);
    assert_eq!(stack_items(&t).len(), 6);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn rally_abilities_triggering_together_go_on_the_stack_in_any_order() {
    cr!("603.3b", "405.2");
    ruling!(
        "Kor Bladewhirl",
        "When an Ally enters the battlefield under your control, each rally ability of the permanents you control will trigger. You can put them on the stack in any order. The last ability you put on the stack will be the first one to resolve."
    );
    supported("Kor Bladewhirl");
    supported("Chasm Guide");
    // Kor Bladewhirl (creatures you control gain first strike) and Chasm Guide (haste)
    // are on the battlefield; Kor Castigator enters. Both rally abilities trigger, and
    // P0 is asked to order them.
    for bladewhirl_last in [false, true] {
        let mut t = TestGame::new(2);
        let bladewhirl = t.battlefield(P0, "Kor Bladewhirl");
        t.battlefield(P0, "Chasm Guide");
        // The first ability in the chosen order goes on the stack first.
        respond(
            &mut t,
            P0,
            if bladewhirl_last {
                bladewhirl_last_on_stack
            } else {
                bladewhirl_first_on_stack
            },
        );
        let from = t.asked().len();
        let castigator = crate::r_s05_common::enter(&mut t, P0, "Kor Castigator");
        assert_eq!(
            count_order_decisions(&t, from),
            1,
            "P0 chooses the order of the two rally abilities"
        );
        assert_eq!(stack_items(&t).len(), 2);
        // Resolve only the top ability: it's the one put on the stack last.
        t.resolve();
        let first_strike = t.obj(castigator).has_keyword(KeywordKind::FirstStrike);
        let haste = t.obj(castigator).has_keyword(KeywordKind::Haste);
        assert_eq!(first_strike, bladewhirl_last);
        assert_eq!(haste, !bladewhirl_last);
        t.resolve_all();
        assert!(t.obj(castigator).has_keyword(KeywordKind::FirstStrike));
        assert!(t.obj(castigator).has_keyword(KeywordKind::Haste));
        assert!(t.obj(bladewhirl).has_keyword(KeywordKind::Haste));
    }
}

/// Orders triggered abilities with Kor Bladewhirl's put on the stack last (on top).
fn bladewhirl_last_on_stack(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    order_bladewhirl(d, true)
}

/// Orders triggered abilities with Kor Bladewhirl's put on the stack first.
fn bladewhirl_first_on_stack(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    order_bladewhirl(d, false)
}

fn order_bladewhirl(d: &Decision, last: bool) -> Option<Answer> {
    let Decision::Order { items, .. } = d else {
        return None;
    };
    let (mut v, others): (Vec<usize>, Vec<usize>) =
        (0..items.len()).partition(|i| items[*i].contains("Kor Bladewhirl"));
    if last {
        let mut o = others;
        o.append(&mut v);
        v = o;
    } else {
        v.extend(others);
    }
    Some(Answer::Indices(v))
}

fn count_order_decisions(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .count()
}
