//! Rulings batch S32 — the Souls (Soul of Zendikar: "{3}{G}{G}: Create a 3/3 green Beast
//! creature token." and "{3}{G}{G}, Exile this card from your graveyard: Create a 3/3
//! green Beast creature token."): an activated ability whose cost moves the card out of
//! the graveyard functions only there (CR 113.6m), and that cost is paid as it's activated
//! (CR 602.2b, 601.2h).

use crate::r_s01_common::*;
use crate::r_s25_common::creature_tokens;
use crate::r_s27_common::{ability_containing, activatable};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const SOUL: &str = "Soul of Zendikar";

#[test]
fn the_last_ability_can_be_activated_only_from_the_graveyard() {
    cr!("113.6m", "602.1");
    ruling!(
        "Soul of Zendikar",
        "You can activate the last ability only if the Soul is in your graveyard."
    );
    supported(SOUL);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    // On the battlefield: only the first ability.
    let soul = t.battlefield(P0, SOUL);
    let acts = activatable(&mut t, P0, soul);
    assert_eq!(acts.len(), 1, "{acts:?}");
    assert!(!acts[0].to_lowercase().contains("exile"), "{acts:?}");
    // In hand: neither.
    let in_hand = t.hand(P0, SOUL);
    assert!(activatable(&mut t, P0, in_hand).is_empty());
    // In the graveyard: only the last one.
    let card = t.graveyard(P0, SOUL);
    let acts = activatable(&mut t, P0, card);
    assert_eq!(acts.len(), 1, "{acts:?}");
    assert!(acts[0].to_lowercase().contains("exile"), "{acts:?}");
    // Not from an opponent's graveyard either.
    let theirs = t.graveyard(P1, SOUL);
    assert!(activatable(&mut t, P0, theirs).is_empty());
}

#[test]
fn exiling_the_soul_is_a_cost_so_it_cant_be_removed_in_response() {
    cr!("602.2b", "601.2h", "113.7a");
    ruling!(
        "Soul of Zendikar",
        "Exiling the Soul from your graveyard is part of the last ability's activation cost. A player can't remove the Soul in response to prevent you from activating the ability."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let card = t.graveyard(P0, SOUL);
    let uid = ability_containing(&mut t, card, "Exile");
    t.g.turn.priority = Some(P0);
    let ability = t
        .g
        .activate_ability(P0, card, uid)
        .expect("activate from the graveyard")
        .expect("an ability on the stack");
    t.g.flush_events();
    // As it's activated, the card is already exiled: there's nothing left in the graveyard
    // for an opponent to remove.
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.in_exile(SOUL));
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.g.stack, vec![ability]);
    // The ability resolves without its source.
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0), 1);
}
