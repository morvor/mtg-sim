//! Where a spell goes as it resolves or is countered when a replacement effect of how it was
//! cast and others would apply (CR 616.1). Nexus of Fate with rebound: its controller
//! chooses between rebound and Nexus of Fate's own effect, and each is offered once (the
//! card's own ability is one of the static abilities functioning on the stack, and also one
//! the moving object is looked at for, CR 400.6: one effect either way). A countered
//! flashback spell: flashback's "exile this card instead of putting it anywhere else"
//! exiles it, with no choice between it and Rest in Peace's effect.

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

/// The engine's reading (a judgment call; CR 616.1 read literally would have P0 choose
/// between the two effects): a countered spell whose keyword exiles it "instead of putting
/// it anywhere else" is moved to exile directly (`Game::counter_or_fizzle`), and the other
/// effects that would replace its being put into its owner's graveyard (CR 701.6a) aren't
/// offered against that keyword's; only a self-replacement effect of the spell countering it
/// is applied first (CR 616.1a: Memory Lapse, Delay). A resolving flashback spell is handled
/// the same way (`choose_resolved_spell_destination`). Both effects would exile Think Twice
/// here, so P0 isn't asked to choose.
#[test]
fn a_countered_flashback_spell_is_exiled_under_rest_in_peace_with_no_choice() {
    cr!("701.6a", "702.34a");
    ruling!(
        "Think Twice",
        "A spell cast using flashback will always be exiled afterward, whether it resolves, is countered, or leaves the stack in some other way."
    );
    supported("Think Twice");
    supported("Counterspell");
    supported("Rest in Peace");
    // P1's Rest in Peace: "If a card or token would be put into a graveyard from anywhere,
    // exile it instead." P0 casts Think Twice with flashback; P1 counters it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    let card = t.graveyard(P0, "Think Twice");
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    let counterspell = t.hand(P1, "Counterspell");
    t.cast(P1, counterspell).target(spell).go();
    t.resolve_all();
    let now = t.g.current(spell);
    assert_eq!(t.zone(now), Zone::Exile);
    assert!(!t.in_graveyard(P0, "Think Twice"));
    // Counterspell itself, resolved, is exiled by Rest in Peace (the only effect).
    assert!(t.in_exile("Counterspell"));
    let choices = replacement_choices(&t);
    assert!(choices.is_empty(), "{choices:?}");
}
