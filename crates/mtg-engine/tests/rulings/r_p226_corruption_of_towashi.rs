//! Rulings batch P226 — Corruption of Towashi: "Whenever a permanent you control
//! transforms or a permanent you control enters transformed, you may draw a card. Do
//! this only once each turn." (CR 701.27, 712.14a), and incubate (CR 701.53).

use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::{enter_transformed, name_of, transform};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const CAPTIVE: &str = "Wolfbitten Captive // Krallenhorde Killer";

#[test]
fn corruption_of_towashi_draws_when_a_permanent_transforms_either_way_once_a_turn() {
    cr!("701.27a", "712.14a", "701.53a", "603.2");
    ruling!(
        "Corruption of Towashi",
        "The last ability of Corruption of Towashi will trigger if a permanent you control transforms in either direction, going from front face up to back face up or vice versa."
    );
    supported("Corruption of Towashi");
    // "When this enchantment enters, incubate 4."
    let mut t = TestGame::new(2);
    t.enter(P0, "Corruption of Towashi");
    t.resolve_all();
    let incubator = with_subtype(&t, P0, "Incubator")[0];
    assert_eq!(t.counters(incubator, counters::PLUS1), 4);
    // Front to back: the Incubator's "{2}: Transform this token."
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, incubator, "Transform").unwrap();
    t.resolve_all();
    assert!(t.obj_now(incubator).chars.has_subtype("Phyrexian"));
    assert_eq!(t.hand_size(P0), 1);
    // Only once each turn.
    let captive = t.battlefield(P0, CAPTIVE);
    transform(&mut t, captive);
    assert_eq!(t.stack_len(), 0);
    // Next turn: Krallenhorde Killer back to Wolfbitten Captive (back to front). An
    // opponent's permanent doesn't count.
    t.advance_to(P1, Step::Upkeep);
    let theirs = t.battlefield(P1, CAPTIVE);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let hand = t.hand_size(P0);
    transform(&mut t, theirs);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, captive), "Krallenhorde Killer");
    t.answer_yes(P0, true);
    transform(&mut t, captive);
    assert_eq!(name_of(&t, captive), "Wolfbitten Captive");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn corruption_of_towashi_draws_when_a_permanent_enters_transformed() {
    cr!("712.14a", "701.27g", "603.6a");
    // A double-faced card entering back face up triggers it; declining to draw doesn't
    // use up the turn's draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corruption of Towashi");
    t.answer_yes(P0, false);
    enter_transformed(&mut t, P0, CAPTIVE);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    t.answer_yes(P0, true);
    enter_transformed(&mut t, P0, CAPTIVE);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Entering front face up: no trigger.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    t.enter(P0, CAPTIVE);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

const EDDIE: &str = "Eddie Brock // Venom, Lethal Protector";

/// A game where P0 controls Corruption of Towashi and will draw when it triggers.
fn towashi() -> TestGame {
    supported("Corruption of Towashi");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corruption of Towashi");
    t.answer_yes(P0, true);
    t
}

/// Resolves everything and returns the number of cards P0 drew from Towashi.
fn towashi_draws(t: &mut TestGame) -> usize {
    t.resolve_all();
    t.hand_size(P0)
}

#[test]
fn corruption_of_towashi_draws_when_a_modal_double_faced_card_transforms_or_is_put_onto_the_battlefield_transformed(
) {
    cr!("701.27a", "712.3", "712.14a", "701.27g");
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "A modal double-faced card can be transformed or be put onto the battlefield transformed. This is a change from previous rules."
    );
    // Put onto the battlefield transformed: Venom, Lethal Protector.
    let mut t = towashi();
    let venom = enter_transformed(&mut t, P0, EDDIE);
    assert_eq!(name_of(&t, venom), "Venom, Lethal Protector");
    assert_eq!(towashi_draws(&mut t), 1);
    // Eddie Brock's own "{3}{B}{R}{G}: Transform Eddie Brock."
    let mut t = towashi();
    let eddie = t.battlefield(P0, EDDIE);
    for land in ["Swamp", "Mountain", "Forest", "Wastes", "Wastes", "Wastes"] {
        t.lands(P0, land, 1);
    }
    activate_containing(&mut t, P0, eddie, "Transform").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, eddie), "Venom, Lethal Protector");
    assert_eq!(towashi_draws(&mut t), 1);
}

#[test]
fn corruption_of_towashi_draws_when_a_modal_double_faced_card_is_cast_or_played_back_face_up(
) {
    // The 2023-04-14 ruling "modal double-faced permanents can't enter transformed, even
    // if they enter with their back faces up" predates the Marvel's Spider-Man CR update
    // effective 2025-09-19 (docs/rulings-exemptions/rulings-P225.tsv); a modal
    // double-faced permanent that enters back face up is now a transformed permanent as
    // it enters (CR 701.27g), and so enters transformed (provisional).
    cr!("701.27g", "712.11b", "712.13", "712.12");
    interp!("mdfc-back-face-up-enters-transformed");
    // Eddie Brock cast as Venom, Lethal Protector.
    let mut t = towashi();
    for land in ["Swamp", "Mountain", "Forest", "Wastes", "Wastes", "Wastes"] {
        t.lands(P0, land, 1);
    }
    let eddie = t.hand(P0, EDDIE);
    t.cast(P0, eddie)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    let venom = t.g.current(eddie);
    assert_eq!(name_of(&t, venom), "Venom, Lethal Protector");
    assert_eq!(towashi_draws(&mut t), 1);
    // Brightclimb Pathway // Grimclimb Pathway played as Grimclimb Pathway; as Brightclimb
    // Pathway (front face up) it doesn't.
    for (choice, draws) in [(0, 0), (1, 1)] {
        let mut t = towashi();
        let land = t.hand(P0, "Brightclimb Pathway // Grimclimb Pathway");
        t.answer(
            P0,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(choice),
        );
        t.play_land(P0, land).unwrap();
        t.settle();
        assert_eq!(t.stack_len(), draws);
        assert_eq!(towashi_draws(&mut t), draws);
    }
}

#[test]
fn corruption_of_towashi_doesnt_draw_for_a_melded_permanent() {
    // The still-current half of the 2023-04-14 ruling ("melded permanents can't enter
    // transformed"); the ruling text stays exempt for its outdated modal double-faced half.
    cr!("701.27g", "712.14c");
    let mut t = towashi();
    t.battlefield(P0, "Graf Rats");
    t.battlefield(P0, "Midnight Scavengers");
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Chittering Host").len(), 1);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), 0);
}
