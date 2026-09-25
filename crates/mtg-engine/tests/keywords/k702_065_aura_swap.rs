//! CR 702.65 Aura swap.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const AURA_SWAP: &str = "Aura Swap";

/// Puts Arcanum Wings onto the battlefield for `controller` (owned by `owner`) attached
/// to `host`.
fn wings_on(t: &mut TestGame, owner: PlayerId, controller: PlayerId, host: ObjectId) -> ObjectId {
    let wings = t.battlefield(owner, "Arcanum Wings");
    let o = &mut t.g.objects[wings.0 as usize];
    o.attached_to = Some(Entity::Object(host));
    o.base_controller = controller;
    o.controller = controller;
    t.g.recompute();
    wings
}

#[test]
fn aura_swap_exchanges_the_aura_with_an_aura_card_in_hand() {
    cr!("702.65", "702.65a");
    ruling!("Arcanum Wings", "The exchange is simultaneous, and happens on resolution.");
    assert_supported("Arcanum Wings");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = wings_on(&mut t, P0, P0, bears);
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Flying));
    t.lands(P0, "Island", 3);
    activate_named(&mut t, P0, wings, AURA_SWAP, 0).unwrap();
    // The Aura card only needs to be in the hand as the ability resolves.
    let strength = t.hand(P0, "Holy Strength");
    t.resolve();
    // Holy Strength is attached to the Bears; Arcanum Wings is back in its owner's hand.
    assert!(t.on_battlefield(strength));
    assert_eq!(t.obj_now(strength).attached_to, Some(Entity::Object(bears)));
    assert!(t.in_hand(P0, "Arcanum Wings"));
    assert_eq!(t.zone(wings), Zone::Hand(P0));
    assert_eq!(t.pt(bears), (3, 4));
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Flying));
}

#[test]
fn aura_swap_does_nothing_if_the_aura_card_cant_enchant_the_permanent() {
    cr!("702.65b");
    ruling!(
        "Arcanum Wings",
        "If on resolution, half the exchange can’t be completed (such as if the only Aura in your hand can’t enchant the permanent), nothing happens."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = wings_on(&mut t, P0, P0, bears);
    // Wild Growth enchants lands only.
    t.hand(P0, "Wild Growth");
    t.lands(P0, "Island", 3);
    activate_named(&mut t, P0, wings, AURA_SWAP, 0).unwrap();
    t.resolve();
    assert!(t.on_battlefield(wings));
    assert!(t.in_hand(P0, "Wild Growth"));
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Flying));
    // With no Aura card in hand at all, nothing happens either.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = wings_on(&mut t, P0, P0, bears);
    t.lands(P0, "Island", 3);
    activate_named(&mut t, P0, wings, AURA_SWAP, 0).unwrap();
    t.resolve();
    assert!(t.on_battlefield(wings));
}

#[test]
fn aura_swap_does_nothing_if_you_dont_own_the_aura() {
    cr!("702.65b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P0 controls an Arcanum Wings that P1 owns.
    let wings = wings_on(&mut t, P1, P0, bears);
    assert_eq!(t.obj_now(wings).controller, P0);
    t.hand(P0, "Holy Strength");
    t.lands(P0, "Island", 3);
    activate_named(&mut t, P0, wings, AURA_SWAP, 0).unwrap();
    t.resolve();
    assert!(t.on_battlefield(wings));
    assert!(t.in_hand(P0, "Holy Strength"));
}

#[test]
fn the_exchange_is_optional() {
    cr!("702.65a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = wings_on(&mut t, P0, P0, bears);
    t.hand(P0, "Holy Strength");
    t.lands(P0, "Island", 3);
    activate_named(&mut t, P0, wings, AURA_SWAP, 0).unwrap();
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.on_battlefield(wings));
    assert!(t.in_hand(P0, "Holy Strength"));
}
