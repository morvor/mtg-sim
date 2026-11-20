//! The options of a choice among replacement effects (CR 616.1) are named so the affected
//! player can tell them apart: an effect whose text no other option shares is named by its
//! source's name and text alone, and effects with the same text (three Hardened Scales) also
//! name their source's controller and object (`Game::replacement_options`).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls three Hardened Scales ("If one or more +1/+1 counters would be put on a
/// creature you control, that many plus one +1/+1 counters are put on it instead."),
/// Corpsejack Menace ("If one or more +1/+1 counters would be put on a creature you
/// control, twice that many +1/+1 counters are put on it instead.") and Grizzly Bears, and
/// casts Battlegrowth ("Put a +1/+1 counter on target creature.") on the Bears, answering
/// the first choice of the effect to apply with `first` (the default if `None`). Returns
/// the game, the three Scales and the Bears.
fn battlegrowth_under_three_scales_and_a_menace(
    first: Option<usize>,
) -> (TestGame, Vec<ObjectId>, ObjectId) {
    let mut t = TestGame::new(2);
    let scales: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Hardened Scales"))
        .collect();
    t.battlefield(P0, "Corpsejack Menace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    if let Some(i) = first {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    cast_resolve(&mut t, P0, "Battlegrowth", &[obj(bears)]);
    (t, scales, bears)
}

#[test]
fn effects_with_the_same_text_are_told_apart_and_a_unique_one_is_named_by_its_text() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Hardened Scales",
        "If two or more effects attempt to modify how many counters would be put on a creature you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Hardened Scales");
    supported("Corpsejack Menace");
    let (probe, scales, _) = battlegrowth_under_three_scales_and_a_menace(None);
    let choices = replacement_choices(&probe);
    assert_eq!(choices[0].0, P0);
    let first = &choices[0].1;
    assert_eq!(first.len(), 4, "{first:?}");
    // Each Hardened Scales is named with its controller and object, so the three options
    // differ; Corpsejack Menace's, the only one with its text, is just that text.
    for s in &scales {
        let tag = format!(" (controlled by P0, object {})", s.0);
        let named = first
            .iter()
            .filter(|o| o.starts_with("Hardened Scales") && o.ends_with(&tag))
            .count();
        assert_eq!(named, 1, "{tag:?} in {first:?}");
    }
    let menace: Vec<&String> = first
        .iter()
        .filter(|o| o.starts_with("Corpsejack Menace"))
        .collect();
    assert_eq!(menace.len(), 1, "{first:?}");
    assert!(!menace[0].contains("(controlled by"), "{first:?}");
    // P0 applies the Menace's effect first: the counter is doubled, then each Scales adds
    // one. The next choice is among the three Scales, still told apart.
    let m = option_index(first, "Corpsejack Menace");
    let (t, scales, bears) = battlegrowth_under_three_scales_and_a_menace(Some(m));
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 3, "{choices:?}");
    let second = &choices[1].1;
    assert_eq!(second.len(), 3, "{second:?}");
    for s in &scales {
        let tag = format!(" (controlled by P0, object {})", s.0);
        assert_eq!(
            second.iter().filter(|o| o.ends_with(&tag)).count(),
            1,
            "{tag:?} in {second:?}"
        );
    }
    assert_eq!(plus_counters(&t, bears), 2 + 3);
}
