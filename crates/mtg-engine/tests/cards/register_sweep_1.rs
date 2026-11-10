//! Checks from the register sweep of Nov 10, 2026 (docs/agents/items/register-sweep-1.md):
//! behaviors the round-trip items' "Remaining" sections described as "not observable" and
//! that the sweep verified in play.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// "You may discard one or more land cards. When you do, ..." is compiled with the count
/// "any number": a player who accepts the offer and chooses no card has not discarded one
/// or more cards, so the reflexive ability doesn't trigger, exactly as if the offer had been
/// declined (CR 603.12: a reflexive triggered ability triggers when the player takes the
/// action).
#[test]
fn choosing_no_card_for_an_optional_one_or_more_discard_is_declining() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    // Accept the offer, then choose no land card.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.enter(P0, "Loamcrafter Faun");
    t.settle();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    t.resolve();
    assert_eq!(t.hand_size(P0), 2, "no land card was discarded");
    assert_eq!(
        t.stack_len(),
        0,
        "no reflexive trigger for a discard that didn't happen: {}",
        t.dump_log()
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Choosing one card does trigger it, and the Bears come back.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.hand(P0, "Forest");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Loamcrafter Faun");
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the reflexive trigger: {}", t.dump_log());
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Forest"));
}
