//! Rulings batch S31 — exiling cards to pay costs: an additional cost of exiling a
//! creature card from your graveyard is paid while the spell is cast, before anyone can
//! respond (CR 601.2h, 117.3c), and exactly one card is exiled; a card in hand can't be
//! exiled to pay its own alternative cost, as it's on the stack by then (CR 601.2a, 601.2h).

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use crate::r_s07_common::cast_methods;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn makeshift_mauler_exiles_exactly_one_creature_card() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Makeshift Mauler",
        "You must exile exactly one creature card from your graveyard to cast this spell; you cannot cast it without exiling a creature card, and you cannot exile additional creature cards."
    );
    supported("Makeshift Mauler");
    // "As an additional cost to cast this spell, exile a creature card from your
    // graveyard."
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let mauler = t.hand(P0, "Makeshift Mauler");
    add_mana(&mut t, P0, ManaType::U, 4);
    // No creature card in P0's graveyard: it can't be cast.
    assert!(t.cast(P0, mauler).try_go().is_err());
    assert_eq!(t.zone(mauler), Zone::Hand(P0));
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // With two creature cards there, P0 can't exile both.
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.cast(P0, mauler).go();
    let exiled = [bears, giant]
        .iter()
        .filter(|c| t.zone(**c) == Zone::Exile)
        .count();
    assert_eq!(exiled, 1);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(mauler));
}

#[test]
fn makeshift_maulers_exiled_card_is_gone_before_anyone_can_respond() {
    cr!("601.2h", "117.3c", "601.2i");
    ruling!(
        "Makeshift Mauler",
        "Players can only respond once this spell has been cast and all its costs have been paid. No one can try to otherwise remove the creature card you exiled in order to prevent you from casting this spell."
    );
    // P1 has Tormod's Crypt ("{T}, Sacrifice this artifact: Exile all cards from target
    // player's graveyard."), ready to empty P0's graveyard.
    let mut t = TestGame::new(2);
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let mauler = t.hand(P0, "Makeshift Mauler");
    add_mana(&mut t, P0, ManaType::U, 4);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, mauler).go();
    // As soon as the spell is cast (before any player receives priority), the creature
    // card has been exiled to pay its cost.
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(mauler), Zone::Stack);
    // P1 responds with the Crypt: too late to stop the spell.
    t.activate(P1, crypt, 0, &[Entity::Player(P0)])
        .expect("activate Tormod's Crypt");
    t.resolve_all();
    assert!(t.on_battlefield(mauler));
}

#[test]
fn fury_of_the_horde_cant_exile_itself_to_pay_its_alternative_cost() {
    cr!("118.9", "601.2a", "601.2h");
    ruling!(
        "Fury of the Horde",
        "You can’t exile a card from your hand to pay for itself. At the time you would pay costs, that card is on the stack, not in your hand."
    );
    supported("Fury of the Horde");
    // "You may exile two red cards from your hand rather than pay this spell's mana
    // cost." With only one other red card, Fury of the Horde isn't the second.
    let mut t = TestGame::new(2);
    let fury = t.hand(P0, "Fury of the Horde");
    let bolt = t.hand(P0, "Lightning Bolt");
    let alt = |t: &mut TestGame| {
        cast_methods(t, P0, fury)
            .into_iter()
            .find(|m| matches!(m, CastMethod::Alternative(_)))
    };
    assert!(alt(&mut t).is_none());
    // A second red card: it's cast exiling those two.
    let shock = t.hand(P0, "Shock");
    let m = alt(&mut t).expect("the alternative cost is available");
    t.cast(P0, fury).method(m).go();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(shock), Zone::Exile);
    assert_eq!(t.zone(fury), Zone::Stack);
}
