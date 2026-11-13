//! The worked examples of CR 610 (one-shot effects), played out as tests.

use crate::r600_examples_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn two_banisher_priests_destroyed_together_return_their_cards_at_the_same_time() {
    cr!("610.3d", "610.3c");
    cr_example!("610.3d", "Two Banisher Priests");
    supported("Banisher Priest");
    supported("Day of Judgment");
    let mut t = TestGame::new(2);
    // Each exiled creature sees the other one enter: they return simultaneously.
    let watcher = |name: &str| {
        custom_card(
            name,
            "Creature — Spirit",
            Some((1, 1)),
            "Whenever another creature enters, you gain 1 life.",
        )
    };
    let a = t.custom(P1, watcher("Watcher A"), Zone::Battlefield);
    let b = t.custom(P1, watcher("Watcher B"), Zone::Battlefield);
    t.lands(P0, "Plains", 10);
    for target in [a, b] {
        let priest = t.hand(P0, "Banisher Priest");
        t.cast(P0, priest).go();
        t.resolve();
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.resolve_all();
    }
    assert!(t.in_exile("Watcher A") && t.in_exile("Watcher B"));
    assert_eq!(t.named_on_battlefield("Banisher Priest").len(), 2);
    let life = t.life(P1);
    let doj = t.hand(P0, "Day of Judgment");
    t.cast(P0, doj).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Banisher Priest").is_empty());
    assert_eq!(t.named_on_battlefield("Watcher A").len(), 1);
    assert_eq!(t.named_on_battlefield("Watcher B").len(), 1);
    assert_eq!(t.life(P1), life + 2);
}
