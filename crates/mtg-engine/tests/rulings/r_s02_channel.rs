//! Rulings batch S02 — channel (an ability word, CR 207.2c): "Channel — [cost], Discard
//! this card: [effect]", an activated ability that functions while the card is in its
//! owner's hand.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn discarding_the_card_is_part_of_the_channel_cost() {
    cr!("207.2c", "602.2", "118.3", "701.9a");
    ruling!(
        "Mnemonic Sphere",
        "Discarding the card is part of the cost to activate a channel ability."
    );
    supported("Mnemonic Sphere");
    supported("Stifle");
    let mut t = TestGame::new(2);
    // "Channel — {U}, Discard this card: Draw a card."
    let sphere = t.hand(P0, "Mnemonic Sphere");
    // Without {U}, the cost can't be paid: nothing happens, the card stays in hand.
    assert!(!can_activate(&mut t, P0, sphere));
    assert!(t.activate(P0, sphere, 1, &[]).is_err());
    assert_eq!(t.obj(sphere).zone, Zone::Hand(P0));
    t.lands(P0, "Island", 1);
    assert!(can_activate(&mut t, P0, sphere));
    let hand = t.hand_size(P0);
    t.activate(P0, sphere, 1, &[]).unwrap();
    // The card was discarded as the ability was activated.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_graveyard(P0, "Mnemonic Sphere"));
    assert_eq!(t.hand_size(P0), hand - 1);
    // The ability is countered: the card stays discarded, and no card is drawn.
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    let ability = *t.g.stack.last().unwrap();
    t.cast(P1, stifle).target(Entity::Object(ability)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mnemonic Sphere"));
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn a_targeted_channel_ability_cant_be_activated_without_a_target() {
    cr!("207.2c", "602.2b", "601.2c", "115.1");
    ruling!(
        "Bamboo Grove Archer",
        "If a channel ability requires a target, you may not activate it without a target just to discard the card."
    );
    supported("Bamboo Grove Archer");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    // "Channel — {4}{G}, Discard this card: Destroy target creature with flying."
    let archer = t.hand(P0, "Bamboo Grove Archer");
    t.battlefield(P1, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, archer));
    assert!(t.activate(P0, archer, 0, &[]).is_err());
    assert_eq!(t.obj(archer).zone, Zone::Hand(P0));
    assert_eq!(tapped_lands(&t, P0), 0);
    assert_eq!(t.stack_len(), 0);
    // With a creature with flying to target, it can be.
    let bird = t.battlefield(P1, "Birds of Paradise");
    assert!(can_activate(&mut t, P0, archer));
    t.activate(P0, archer, 0, &[Entity::Object(bird)]).unwrap();
    assert!(t.in_graveyard(P0, "Bamboo Grove Archer"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Birds of Paradise"));
}
