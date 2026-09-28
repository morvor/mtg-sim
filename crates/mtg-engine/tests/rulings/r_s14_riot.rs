//! Rulings batch S14 — riot (CR 702.136): "You may have this permanent enter with an
//! additional +1/+1 counter on it. If you don't, it gains haste." A replacement effect
//! (CR 614.1c) applied as the permanent enters.

use crate::r_s01_common::*;
use crate::r_s06_common::give_control;
use crate::r_s14_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn haste(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Haste)
}

fn arynx(t: &TestGame) -> ObjectId {
    t.named_on_battlefield("Frenzied Arynx")[0]
}

#[test]
fn riot_haste_lasts_through_turns_and_control_changes() {
    cr!("702.136a", "611.2a", "613.1f");
    ruling!(
        "Frenzied Arynx",
        "If you choose for the creature to gain haste, it gains haste indefinitely. It won't lose it as the turn ends or as another player gains control of it."
    );
    supported("Frenzied Arynx");
    // Frenzied Arynx (3/3 riot, trample) enters with haste (no counter).
    let mut t = TestGame::new(2);
    t.answer_yes(P0, false);
    cast_from_hand(&mut t, P0, "Frenzied Arynx", &[]);
    t.resolve_all();
    let cat = arynx(&t);
    assert_eq!(t.counters(cat, counters::PLUS1), 0);
    assert!(haste(&t, cat));
    // Still hasty on the next turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(haste(&t, cat));
    // P1 gains control of it: still hasty.
    give_control(&mut t, cat, P1);
    assert_eq!(t.obj_now(cat).controller, P1);
    assert!(haste(&t, cat));
    t.advance_to(P0, Step::Upkeep);
    assert!(haste(&t, cat));
}

#[test]
fn a_riot_creature_that_cant_have_counters_gains_haste() {
    cr!("702.136a", "614.12");
    ruling!(
        "Rhythm of the Wild",
        "If a creature entering the battlefield has riot but can't have a +1/+1 counter put onto it, it gains haste."
    );
    supported("Rhythm of the Wild");
    supported("Melira's Keepers");
    // Rhythm of the Wild: "Nontoken creatures you control have riot." Melira's Keepers
    // (4/4): "This creature can't have counters put on it." P0 would like the counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhythm of the Wild");
    t.answer_yes(P0, true);
    cast_from_hand(&mut t, P0, "Melira's Keepers", &[]);
    t.resolve_all();
    let keepers = t.named_on_battlefield("Melira's Keepers")[0];
    assert_eq!(t.counters(keepers, counters::PLUS1), 0);
    assert!(haste(&t, keepers));
    assert_eq!(t.pt(keepers), (4, 4));
}

#[test]
fn nobody_can_respond_to_the_riot_choice() {
    cr!("702.136a", "614.1c", "614.12");
    ruling!(
        "Frenzied Arynx",
        "Riot is a replacement effect. Players can't respond to your choice of +1/+1 counter or haste, and they can't take actions while the creature is on the battlefield without one or the other."
    );
    // The choice is made while Frenzied Arynx isn't on the battlefield yet, and whenever
    // a player has priority, it's on the battlefield with its counter (or not at all).
    let mut t = TestGame::new(2);
    let is_yes_no = |d: &Decision| matches!(d, Decision::YesNo { .. });
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let on_bf = |g: &mtg_engine::game::Game| {
        g.permanents()
            .filter(|o| o.chars.name == "Frenzied Arynx")
            .map(|o| o.counter(counters::PLUS1) > 0 || o.has_keyword(KeywordKind::Haste))
            .collect::<Vec<bool>>()
    };
    let at_choice = watch(&mut t, P0, is_yes_no, on_bf);
    let at_priority = watch(&mut t, P1, is_priority, on_bf);
    t.answer_yes(P0, true);
    cast_from_hand(&mut t, P0, "Frenzied Arynx", &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    let cat = arynx(&t);
    assert_eq!(t.counters(cat, counters::PLUS1), 1);
    assert!(!haste(&t, cat));
    let choices = at_choice.lock().unwrap().clone();
    assert_eq!(
        choices,
        vec![Vec::<bool>::new()],
        "chosen before it entered"
    );
    let prios = at_priority.lock().unwrap().clone();
    assert!(prios.iter().any(|v| v == &vec![true]));
    assert!(prios.iter().all(|v| v.iter().all(|x| *x)));
}
