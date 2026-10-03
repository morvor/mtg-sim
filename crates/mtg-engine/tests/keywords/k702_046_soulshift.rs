//! CR 702.46 Soulshift.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_038_051::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The candidates of the most recent target choice.
fn last_target_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn soulshift_returns_a_spirit_card_with_mana_value_n_or_less() {
    cr!("702.46", "702.46a");
    assert_supported("Kami of Empty Graves");
    let mut t = TestGame::new(2);
    // Kami of Empty Graves: 4/1, soulshift 3.
    let kami = t.battlefield(P0, "Kami of Empty Graves");
    let thief = t.graveyard(P0, "Thief of Hope"); // Spirit, mana value 3
    let lunacy = t.graveyard(P0, "Kami of Lunacy"); // Spirit, mana value 6
    let bears = t.graveyard(P0, "Grizzly Bears"); // not a Spirit
    let theirs = t.graveyard(P1, "Deathknell Kami"); // an opponent's Spirit
    t.answer_targets(P0, &[Entity::Object(thief)]);
    t.g.destroy(kami, None);
    t.settle();
    assert_eq!(triggers_named(&t, "Soulshift 3").len(), 1);
    let cands = last_target_candidates(&t);
    assert!(cands.contains(&Entity::Object(thief)));
    assert!(!cands.contains(&Entity::Object(lunacy)));
    assert!(!cands.contains(&Entity::Object(bears)));
    assert!(!cands.contains(&Entity::Object(theirs)));
    // The Kami itself has mana value 4: it can't return itself.
    let kami_card = t.g.current(kami);
    assert!(!cands.contains(&Entity::Object(kami_card)));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Thief of Hope"));
    assert!(t.in_graveyard(P0, "Kami of Empty Graves"));
}

#[test]
fn returning_the_card_is_optional() {
    cr!("702.46a");
    let mut t = TestGame::new(2);
    let kami = t.battlefield(P0, "Kami of Empty Graves");
    let thief = t.graveyard(P0, "Thief of Hope");
    t.answer_targets(P0, &[Entity::Object(thief)]);
    t.g.destroy(kami, None);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Thief of Hope"));
}

#[test]
fn a_non_spirit_with_soulshift_returns_spirits_only() {
    cr!("702.46a");
    ruling!(
        "Promised Kannushi",
        "Promised Kannushi can return any Spirit with mana value of 7 or less. However, Promised Kannushi is a Human Druid, not a Spirit."
    );
    let mut t = TestGame::new(2);
    let kannushi = t.battlefield(P0, "Promised Kannushi");
    let pus = t.graveyard(P0, "Pus Kami"); // Spirit, mana value 7
    t.answer_targets(P0, &[Entity::Object(pus)]);
    t.g.destroy(kannushi, None);
    t.settle();
    let cands = last_target_candidates(&t);
    assert_eq!(cands, vec![Entity::Object(pus)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Pus Kami"));
    assert!(t.in_graveyard(P0, "Promised Kannushi"));
}

#[test]
fn soulshift_needs_the_permanent_to_go_to_the_graveyard_from_the_battlefield() {
    cr!("702.46a");
    let mut t = TestGame::new(2);
    let kami = t.battlefield(P0, "Kami of Empty Graves");
    t.graveyard(P0, "Thief of Hope");
    // Exiled, not put into a graveyard: no trigger.
    t.g.move_object(kami, object::Zone::Exile, events::MoveCause::Effect, None);
    t.settle();
    assert!(triggers_named(&t, "Soulshift 3").is_empty());
}

#[test]
fn each_instance_of_soulshift_triggers_separately() {
    cr!("702.46b");
    ruling!(
        "Forked-Branch Garami",
        "When it is put into a graveyard from the battlefield, both soulshift abilities trigger."
    );
    ruling!(
        "Forked-Branch Garami",
        "Each soulshift ability can return one Spirit"
    );
    assert_supported("Forked-Branch Garami");
    let mut t = TestGame::new(2);
    let garami = t.battlefield(P0, "Forked-Branch Garami");
    let thief = t.graveyard(P0, "Thief of Hope");
    let deathknell = t.graveyard(P0, "Deathknell Kami");
    t.answer_targets(P0, &[Entity::Object(thief)]);
    t.answer_targets(P0, &[Entity::Object(deathknell)]);
    t.g.destroy(garami, None);
    t.settle();
    assert_eq!(triggers_named(&t, "Soulshift 4").len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Thief of Hope"));
    assert!(t.in_hand(P0, "Deathknell Kami"));
}

#[test]
fn two_soulshift_triggers_targeting_the_same_card_return_it_once() {
    cr!("702.46b");
    ruling!(
        "Forked-Branch Garami",
        "or both abilities can attempt to return just one Spirit (causing the second ability to be countered)"
    );
    let mut t = TestGame::new(2);
    let garami = t.battlefield(P0, "Forked-Branch Garami");
    let thief = t.graveyard(P0, "Thief of Hope");
    t.answer_targets(P0, &[Entity::Object(thief)]);
    t.answer_targets(P0, &[Entity::Object(thief)]);
    t.g.destroy(garami, None);
    t.settle();
    let trig = triggers_named(&t, "Soulshift 4");
    assert_eq!(trig.len(), 2);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_hand(P0, "Thief of Hope"));
    // The second trigger's target is gone: it doesn't resolve (no choice is offered).
    let asked_before = t.asked().len();
    t.resolve();
    assert!(t.stack_len() == 0);
    assert!(!t.asked()[asked_before..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
}
