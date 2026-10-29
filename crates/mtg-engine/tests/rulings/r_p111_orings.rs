//! Rulings batch P111 — "exile target nonland permanent an opponent controls until this
//! enchantment leaves the battlefield" (CR 610.3): if the enchantment leaves before its
//! ability resolves nothing is exiled (CR 610.3c); the card returns immediately after the
//! enchantment leaves, with nothing in between (CR 610.3a), even when its owner leaves
//! the game (CR 800.4a).

use crate::r_p079_common::concede;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::*;

const ORINGS: [&str; 4] = [
    "Cast Out",
    "Journey to Oblivion",
    "Lassoed by the Law",
    "Leyline Binding",
];

/// `name` enters under P0's control with its exile trigger targeting `target` (the trigger
/// is left on the stack). Returns the enchantment.
fn oring(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    supported(name);
    t.answer_targets(P0, &[Entity::Object(target)]);
    let e = t.enter(P0, name);
    t.settle();
    assert!(t.stack_len() >= 1, "{name}: no trigger");
    e
}

/// Destroys `id` now (no state-based actions are checked afterwards).
fn destroy_now(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
}

#[test]
fn if_the_enchantment_leaves_before_its_ability_resolves_nothing_is_exiled() {
    cr!("610.3c", "603.2");
    ruling!(
        "Cast Out",
        "If Cast Out leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Journey to Oblivion",
        "If Journey to Oblivion leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Lassoed by the Law",
        "If Lassoed by the Law leaves the battlefield before its first triggered ability resolves, the target permanent won’t be exiled at all."
    );
    ruling!(
        "Leyline Binding",
        "If Leyline Binding leaves the battlefield before its enters-the-battlefield ability resolves, the target permanent won't be exiled."
    );
    for name in ORINGS {
        // Control: it's exiled until the enchantment leaves.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let e = oring(&mut t, name, bears);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}: not exiled");
        destroy_now(&mut t, e);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{name}");
        // Gone first: nothing happens.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let e = oring(&mut t, name, bears);
        destroy_now(&mut t, e);
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{name}: exiled anyway");
        assert!(!t.in_exile("Grizzly Bears"));
    }
}

#[test]
fn the_card_returns_immediately_after_the_enchantment_leaves() {
    cr!("610.3a", "610.3", "704.3");
    ruling!(
        "Cast Out",
        "The exiled card returns to the battlefield immediately after Cast Out leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Leyline Binding",
        "Leyline Binding's last ability is a single ability that creates two one-shot effects: one that exiles the permanent when the ability resolves, and another that returns the exiled card to the battlefield immediately after Leyline Binding leaves the battlefield."
    );
    for name in ["Cast Out", "Leyline Binding"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let e = oring(&mut t, name, bears);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        // Right after the enchantment is destroyed (as its events are processed), before
        // state-based actions are checked or triggers are put on the stack, the card is
        // back (under its owner's control).
        destroy_now(&mut t, e);
        t.g.flush_events();
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1, "{name}: not back immediately");
        assert_eq!(t.obj(back[0]).controller, P1);
        assert_eq!(t.stack_len(), 0, "the return isn't an ability on the stack");
    }
}

#[test]
fn the_card_returns_when_the_enchantments_owner_leaves_the_game() {
    cr!("800.4a", "610.3a");
    ruling!(
        "Cast Out",
        "In a multiplayer game, if Cast Out's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Leyline Binding",
        "In a multiplayer game, if Leyline Binding's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    for name in ["Cast Out", "Leyline Binding"] {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        oring(&mut t, name, bears);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        concede(&mut t, P0);
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1, "{name}: the card didn't return");
        assert_eq!(t.obj(back[0]).controller, P1);
    }
}
