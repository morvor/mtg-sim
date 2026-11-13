//! Instructions after a replacement effect's modified event (CR 614.6): "If a creature an
//! opponent controls would die, exile it instead. When you do, create a 2/2 green Wolf
//! creature token." (a reflexive triggered ability each time it applies, CR 603.12), and
//! "If a permanent you control would be put into a graveyard from the battlefield this
//! turn, exile it instead. Return it to the battlefield under its owner's control at the
//! beginning of the next end step." (a delayed triggered ability for each, CR 603.7).

use crate::basic_effects_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn tokens(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.kind == mtg_engine::object::ObjKind::Token)
        .count()
}

fn with_subtype(t: &TestGame, s: &str) -> usize {
    t.g.permanents().filter(|o| o.chars.has_subtype(s)).count()
}

#[test]
fn head_of_the_hunt_exiles_dying_opposing_creatures_and_makes_wolves() {
    cr!("614.6", "603.12", "700.4");
    ruling!("Head of the Hunt", "abilities that would trigger when those creatures die won’t trigger");
    assert_supported("Head of the Hunt");
    let mut t = TestGame::new(2);
    let hoh = t.battlefield(P0, "Head of the Hunt");
    let traveler = t.battlefield(P1, "Doomed Traveler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(traveler, None);
    t.resolve_all();
    assert!(t.in_exile("Doomed Traveler"));
    // No Spirit for the Traveler (it didn't die); a Wolf for P0 (Head of the Hunt itself is
    // a Wolf too).
    assert_eq!(with_subtype(&t, "Spirit"), 0);
    assert_eq!(with_subtype(&t, "Wolf"), 2);
    assert!(t.on_battlefield(hoh));
    // P0's own creatures die normally.
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(with_subtype(&t, "Wolf"), 2);
}

#[test]
fn nemata_doesnt_trigger_if_another_exile_effect_is_applied() {
    cr!("616.1", "603.12");
    ruling!("Nemata, Primeval Warden", "the triggered ability that creates a Saproling token won’t trigger");
    assert_supported("Nemata, Primeval Warden");
    // P1 chooses which effect applies to their creature: Nemata's (a Saproling) or Rest in
    // Peace's (none). Try both choices.
    let mut saprolings = Vec::new();
    for choice in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Nemata, Primeval Warden");
        t.battlefield(P1, "Rest in Peace");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(P1, DecisionKind::Replacement, Answer::Index(choice));
        t.g.destroy(bears, None);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        let asked = t
            .asked()
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseReplacement { .. }));
        assert!(asked);
        saprolings.push(with_subtype(&t, "Saproling"));
    }
    saprolings.sort();
    assert_eq!(saprolings, vec![0, 1]);
}

#[test]
fn valentin_exiles_nontoken_creatures_and_may_pay_for_a_pest() {
    cr!("614.6", "603.12", "118.12");
    assert_supported("Valentin, Dean of the Vein // Lisette, Dean of the Root");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valentin, Dean of the Vein // Lisette, Dean of the Root");
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(with_subtype(&t, "Pest"), 1);
}

#[test]
fn cosmic_intervention_returns_each_exiled_permanent_at_the_next_end_step() {
    cr!("614.6", "603.7a", "111.7");
    ruling!("Cosmic Intervention", "creates its own triggered ability to return itself");
    ruling!("Cosmic Intervention", "it’s exiled and then ceases to exist");
    ruling!("Cosmic Intervention", "won’t trigger for the rest of the turn");
    assert_supported("Cosmic Intervention");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Doomed Traveler");
    t.lands(P0, "Plains", 4);
    let ci = t.hand(P0, "Cosmic Intervention");
    t.cast(P0, ci).go();
    t.resolve();
    // Tokens too.
    let alarm = t.hand(P0, "Raise the Alarm");
    t.lands(P0, "Plains", 2);
    t.cast(P0, alarm).go();
    t.resolve();
    assert_eq!(tokens(&t), 2);
    let wrath = t.hand(P0, "Wrath of God");
    t.lands(P0, "Plains", 4);
    t.battlefield(P1, "Hill Giant");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Doomed Traveler"));
    // P1's Hill Giant isn't P0's: it dies.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // The Traveler didn't die: no Spirit token from it. The Soldier tokens are gone.
    assert_eq!(tokens(&t), 0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Doomed Traveler").len(), 1);
    assert_eq!(tokens(&t), 0);
}
