//! Where a spell goes as it resolves or is countered when a replacement effect of how it was
//! cast and others would apply (CR 616.1). Nexus of Fate with rebound: its controller
//! chooses between rebound and Nexus of Fate's own effect, and each is offered once (the
//! card's own ability is one of the static abilities functioning on the stack, and also one
//! the moving object is looked at for, CR 400.6: one effect either way). A flashback spell:
//! flashback's "exile this card instead of putting it anywhere else any time it would leave
//! the stack" is a replacement effect of a static ability of the card (CR 702.34a), not a
//! self-replacement effect (CR 614.15), so its controller chooses between it and Rest in
//! Peace's or Dauthi Voidwalker's effect, countered (CR 701.6a) or resolving (CR 608.2n)
//! (register entry `leaving-the-stack-exile-among-other-replacements`).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

#[test]
fn nexus_of_fate_with_rebound_offers_rebound_and_its_own_effect_once_each() {
    cr!("616.1", "616.1e", "702.88a", "400.6");
    ruling!(
        "Cast Through Time",
        "you choose whether to apply the rebound effect or the other effect as the spell resolves"
    );
    ruling!(
        "Nexus of Fate",
        "applies if it would be put into a graveyard in any way, including while it"
    );
    supported("Nexus of Fate");
    supported("Cast Through Time");
    // P0's Cast Through Time ("Instant and sorcery spells you control have rebound.") gives
    // Nexus of Fate, cast from P0's hand, rebound; Nexus of Fate's own "If Nexus of Fate
    // would be put into a graveyard from anywhere, reveal Nexus of Fate and shuffle it into
    // its owner's library instead." also applies as it resolves. P0 picks rebound (0) or
    // Nexus of Fate's effect (1).
    for choice in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Cast Through Time");
        t.lands(P0, "Island", 7);
        let card = t.hand(P0, "Nexus of Fate");
        let library = t.library_size(P0);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(choice));
        let spell = t.cast(P0, card).go();
        t.resolve();
        let choices = replacement_choices(&t);
        assert_eq!(choices.len(), 1, "{choices:?}");
        let (who, options) = &choices[0];
        assert_eq!(*who, P0);
        // Two options, not three: Nexus of Fate's own ability is offered once.
        assert_eq!(options.len(), 2, "{options:?}");
        assert!(options[0].contains("Rebound"), "{options:?}");
        assert!(options[1].starts_with("Nexus of Fate"), "{options:?}");
        let now = t.g.current(spell);
        if choice == 0 {
            assert_eq!(t.zone(now), Zone::Exile);
            assert_eq!(t.library_size(P0), library);
        } else {
            assert_eq!(t.zone(now), Zone::Library(P0));
            assert_eq!(t.library_size(P0), library + 1);
        }
        assert!(!t.in_graveyard(P0, "Nexus of Fate"));
    }
}

/// P1 controls `permanent`; P0 casts Think Twice ("Draw a card.") from P0's graveyard with
/// flashback, and P1 counters it with Counterspell if `countered` (else it resolves). P0
/// answers the choice among replacement effects with `pick` (the default if `None`).
/// Returns the game and the Think Twice spell.
fn think_twice_leaving_the_stack(
    permanent: &str,
    countered: bool,
    pick: Option<usize>,
) -> (TestGame, ObjectId) {
    supported("Think Twice");
    supported("Counterspell");
    supported(permanent);
    let mut t = TestGame::new(2);
    t.battlefield(P1, permanent);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    let card = t.graveyard(P0, "Think Twice");
    if let Some(i) = pick {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    if countered {
        let counterspell = t.hand(P1, "Counterspell");
        t.cast(P1, counterspell).target(spell).go();
    }
    t.resolve_all();
    (t, spell)
}

/// The choice P0 was asked as Think Twice left the stack (see
/// [`think_twice_leaving_the_stack`]): the options, and the index of flashback's effect and
/// of the other one.
fn think_twice_choice(permanent: &str, countered: bool) -> (Vec<String>, usize, usize) {
    let (probe, _) = think_twice_leaving_the_stack(permanent, countered, None);
    let choices = replacement_choices(&probe);
    assert_eq!(choices.len(), 1, "{choices:?}");
    let (who, options) = choices[0].clone();
    assert_eq!(who, P0, "the spell's controller chooses");
    assert_eq!(options.len(), 2, "{options:?}");
    let flashback = option_index(&options, "Flashback");
    let other = option_index(&options, permanent);
    (options, flashback, other)
}

#[test]
fn a_countered_flashback_spell_under_rest_in_peace_is_exiled_by_the_effect_its_controller_chooses(
) {
    cr!("701.6a", "702.34a", "616.1", "616.1e", "616.1f");
    interp!("leaving-the-stack-exile-among-other-replacements");
    ruling!(
        "Think Twice",
        "A spell cast using flashback will always be exiled afterward, whether it resolves, is countered, or leaves the stack in some other way."
    );
    // P1's Rest in Peace: "If a card or token would be put into a graveyard from anywhere,
    // exile it instead." Countered, Think Twice would be put into P0's graveyard: P0
    // chooses between flashback's effect and Rest in Peace's, and either exiles it.
    let (_, flashback, other) = think_twice_choice("Rest in Peace", true);
    for pick in [flashback, other] {
        let (t, spell) = think_twice_leaving_the_stack("Rest in Peace", true, Some(pick));
        let now = t.g.current(spell);
        assert_eq!(t.zone(now), Zone::Exile);
        assert!(!t.in_graveyard(P0, "Think Twice"));
        // Counterspell itself, resolved, is exiled by Rest in Peace (the only effect).
        assert!(t.in_exile("Counterspell"));
    }
}

/// Dauthi Voidwalker ("If a card would be put into an opponent's graveyard from anywhere,
/// instead exile it with a void counter on it."): the choice changes what happens. Applying
/// flashback's effect, Think Twice is exiled with no counter; applying Dauthi Voidwalker's
/// first, it's exiled with a void counter (and flashback's no longer applies to a card
/// already going to exile), whether it's countered or resolves.
#[test]
fn a_flashback_spell_leaving_the_stack_under_dauthi_voidwalker_gets_a_void_counter_only_if_chosen(
) {
    cr!("701.6a", "608.2n", "702.34a", "616.1", "616.1e", "616.1f");
    interp!("leaving-the-stack-exile-among-other-replacements");
    ruling!(
        "Think Twice",
        "A spell cast using flashback will always be exiled afterward, whether it resolves, is countered, or leaves the stack in some other way."
    );
    for countered in [true, false] {
        let (_, flashback, other) = think_twice_choice("Dauthi Voidwalker", countered);
        for (pick, void) in [(flashback, 0), (other, 1)] {
            let (t, spell) =
                think_twice_leaving_the_stack("Dauthi Voidwalker", countered, Some(pick));
            let now = t.g.current(spell);
            let what = format!("countered {countered}, void counters {void}");
            assert_eq!(t.zone(now), Zone::Exile, "{what}");
            assert_eq!(t.counters(now, "void"), void, "{what}");
        }
    }
}
