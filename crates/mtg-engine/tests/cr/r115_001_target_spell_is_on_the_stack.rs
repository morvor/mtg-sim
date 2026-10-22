//! CR 115.1, 112.1, 110.1: "target spell" can be only a spell on the stack, not a permanent on
//! the battlefield (a card on the battlefield is a permanent, not a spell).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn counterspell_cant_target_a_permanent() {
    cr!("115.1", "112.1", "110.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let counter = t.hand(P0, "Counterspell");
    assert!(t
        .cast(P0, counter)
        .target(Entity::Object(bears))
        .try_go()
        .is_err());
    assert!(t.on_battlefield(bears));
    assert!(t.g.stack.is_empty());
}

#[test]
fn unsubstantiate_cant_target_a_creature_card_in_a_graveyard() {
    cr!("115.1", "112.1", "110.1");
    // "Return target spell or creature to its owner's hand.": a creature card in a
    // graveyard is neither a spell nor a creature permanent.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let unsub = t.hand(P0, "Unsubstantiate");
    assert!(t
        .cast(P0, unsub)
        .target(Entity::Object(bears))
        .try_go()
        .is_err());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // It can target the creature once it's on the battlefield.
    let giant = t.battlefield(P1, "Hill Giant");
    t.cast(P0, unsub).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
}
