//! CR 702.104 Tribute.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::tokens;
use crate::common_k702_052_066::{destroy, yes_no_asked};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Casts `name` for P0 with enough lands and resolves it; `pays` is P1's tribute choice.
fn cast_with_tribute(t: &mut TestGame, name: &str, pays: bool) -> ObjectId {
    t.lands(P0, "Wastes", 5);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, name);
    t.cast(P0, c).go();
    t.settle();
    t.answer_yes(P1, pays);
    t.resolve();
    c
}

#[test]
fn an_opponent_may_pay_tribute_as_the_creature_enters() {
    cr!("702.104", "702.104a", "702.104b");
    ruling!(
        "Fanatic of Xenagos",
        "If the opponent pays tribute, the creature will enter the battlefield with the specified number of +1/+1 counters on it."
    );
    assert_supported("Fanatic of Xenagos");
    let mut t = TestGame::new(2);
    // Fanatic of Xenagos: 3/3 trample, tribute 1, "When this creature enters, if tribute
    // wasn't paid, it gets +1/+1 and gains haste until end of turn."
    let fanatic = cast_with_tribute(&mut t, "Fanatic of Xenagos", true);
    assert!(t.on_battlefield(fanatic));
    assert_eq!(t.counters(fanatic, counters::PLUS1), 1);
    // Tribute was paid: nothing triggers.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(fanatic), (4, 4));
    assert!(!t.obj_now(fanatic).chars.has_keyword(KeywordKind::Haste));
}

#[test]
fn if_tribute_isnt_paid_the_creatures_ability_triggers() {
    cr!("702.104a", "702.104b");
    ruling!(
        "Fanatic of Xenagos",
        "if the opponent doesn’t pay tribute, the triggered ability will trigger before any player has a chance to remove the creature."
    );
    let mut t = TestGame::new(2);
    let fanatic = cast_with_tribute(&mut t, "Fanatic of Xenagos", false);
    assert_eq!(t.counters(fanatic, counters::PLUS1), 0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(fanatic), (4, 4));
    assert!(t.obj_now(fanatic).chars.has_keyword(KeywordKind::Haste));
}

#[test]
fn the_tribute_choices_are_made_as_the_creature_enters() {
    cr!("702.104a");
    ruling!(
        "Ornitharch",
        "The choice of whether to pay tribute is made as the creature with tribute is entering the battlefield."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let c = t.hand(P0, "Ornitharch");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(yes_no_asked(&t, P1, "tribute"), 0);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P1, "tribute"), 1);
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert_eq!(tokens(&t, P0), 0);
}

#[test]
fn the_controller_chooses_which_opponent_decides() {
    cr!("702.104a");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 5);
    let c = t.hand(P0, "Ornitharch");
    t.cast(P0, c).go();
    // P0 chooses P2; P2 pays (P1 would have declined).
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    t.resolve_all();
    let chooses: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(chooses, vec![vec![Entity::Player(P1), Entity::Player(P2)]]);
    assert_eq!(yes_no_asked(&t, P1, "tribute"), 0);
    assert_eq!(yes_no_asked(&t, P2, "tribute"), 1);
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert_eq!(tokens(&t, P0), 0);
}

#[test]
fn the_ability_resolves_even_if_the_creature_left_the_battlefield() {
    cr!("702.104b");
    ruling!(
        "Ornitharch",
        "The triggered ability will resolve even if the creature with tribute isn’t on the battlefield at that time."
    );
    assert_supported("Ornitharch");
    let mut t = TestGame::new(2);
    // Ornitharch: "When this creature enters, if tribute wasn't paid, create two 1/1
    // white Bird creature tokens with flying."
    let orni = cast_with_tribute(&mut t, "Ornitharch", false);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, orni);
    assert!(!t.on_battlefield(orni));
    t.resolve_all();
    assert_eq!(tokens(&t, P0), 2);
}

#[test]
fn the_abilitys_targets_are_chosen_once_it_triggers() {
    cr!("702.104b");
    ruling!(
        "Nessian Demolok",
        "If the triggered ability has a target, that target will not be known while the creature spell with tribute is on the stack."
    );
    assert_supported("Nessian Demolok");
    let mut t = TestGame::new(2);
    let saw = t.battlefield(P1, "Bone Saw");
    // Nessian Demolok: tribute 3, "When this creature enters, if tribute wasn't paid,
    // destroy target noncreature permanent."
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Nessian Demolok");
    let from = t.asked().len();
    t.cast(P0, c).go();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_yes(P1, false);
    t.answer_targets(P0, &[Entity::Object(saw)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Bone Saw"));
    assert_eq!(t.pt(c), (3, 3));
}
