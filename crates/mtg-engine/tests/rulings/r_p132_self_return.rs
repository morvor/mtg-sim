//! Rulings batch P132 — cards that return themselves from the graveyard to their owner's
//! hand: abilities that function only in the graveyard (CR 113.6k?), trigger only if the
//! card is already there at the moment of the event (CR 603.2, 603.10), and do nothing if
//! the card left the graveyard before they resolve, even if it came back (CR 400.7).

use crate::r_p132_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, destroy};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Each case: a card in P0's graveyard whose ability triggers on an event; the card leaves
/// the graveyard (and comes back) with the trigger on the stack, so it isn't returned.
#[test]
fn self_returns_do_nothing_once_the_card_left_the_graveyard() {
    cr!("400.7", "608.2b", "603.2");
    ruling!(
        "Furious Forebear",
        "If Furious Forebear leaves your graveyard before its ability resolves, it won’t be returned to your hand even if you pay {1}{W}."
    );
    ruling!(
        "Punishing Fire",
        "If Punishing Fire has left your graveyard by the time its triggered ability resolves, you may still pay {R}, but you won't return it to your hand."
    );
    ruling!(
        "Reach of Branches",
        "If Reach of Branches leaves your graveyard before its triggered ability resolves, the ability will have no effect, even if the card returns to your graveyard before then."
    );
    ruling!(
        "Zodiac Dragon",
        "If Zodiac Dragon is removed from the graveyard after the ability triggers but before it resolves, you can’t return it to your hand."
    );
    ruling!(
        "Immortal Phoenix",
        "If Immortal Phoenix leaves the graveyard after it dies but before its triggered ability resolves, it won’t be returned to its owner’s hand."
    );
    ruling!(
        "Mortus Strider",
        "Mortus Strider will return to its owner's hand only if it is still in the graveyard when its ability resolves."
    );
    type Setup = fn(&mut TestGame) -> ObjectId;
    let cases: &[(&str, Setup)] = &[
        ("Furious Forebear", |t| {
            let f = t.graveyard(P0, "Furious Forebear");
            t.lands(P0, "Plains", 2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            destroy(t, bears);
            f
        }),
        ("Punishing Fire", |t| {
            let f = t.graveyard(P0, "Punishing Fire");
            t.lands(P0, "Mountain", 1);
            gain(t, P1, 1);
            f
        }),
        ("Reach of Branches", |t| {
            let f = t.graveyard(P0, "Reach of Branches");
            t.enter(P0, "Forest");
            flush(t);
            f
        }),
        ("Zodiac Dragon", |t| {
            let f = t.battlefield(P0, "Zodiac Dragon");
            destroy(t, f);
            f
        }),
        ("Immortal Phoenix", |t| {
            let f = t.battlefield(P0, "Immortal Phoenix");
            destroy(t, f);
            f
        }),
        ("Mortus Strider", |t| {
            let f = t.battlefield(P0, "Mortus Strider");
            destroy(t, f);
            f
        }),
    ];
    for (name, setup) in cases {
        supported(name);
        for leave in [false, true] {
            let mut t = TestGame::new(2);
            let card = setup(&mut t);
            assert_eq!(t.stack_len(), 1, "{name}: the trigger");
            if leave {
                blink_in_graveyard(&mut t, card);
            }
            t.answer_yes(P0, true);
            t.resolve_all();
            if *name == "Punishing Fire" {
                // Paid either way.
                assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
            }
            let want = if leave { "graveyard" } else { "hand" };
            assert_eq!(where_is(&t, card), want, "{name} (left: {leave})");
        }
    }
}

#[test]
fn self_returns_trigger_only_if_the_card_is_already_in_the_graveyard() {
    cr!("603.2", "603.10");
    ruling!(
        "Punishing Fire",
        "Punishing Fire's triggered ability triggers only if it's already in your graveyard at the time an opponent gains life."
    );
    ruling!(
        "Punishing Fire",
        "You choose whether to pay {R} as the triggered ability resolves."
    );
    ruling!(
        "Veilborn Ghoul",
        "Veilborn Ghoul's ability triggers only if it's in your graveyard at the moment the Swamp enters."
    );
    ruling!(
        "Ovalchase Daredevil",
        "Ovalchase Daredevil's ability triggers only if it's already in your graveyard as an artifact enters the battlefield under your control."
    );
    ruling!(
        "Masked Admirers",
        "The second triggered ability triggers only if Masked Admirers is in your graveyard when you cast a creature spell."
    );
    // Punishing Fire: cast in response to the life gain, it's in the graveyard when the
    // opponent gains life; cast afterwards, too late.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let fire = t.hand(P0, "Punishing Fire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    gain(&mut t, P1, 2);
    assert_eq!(t.stack_len(), 0, "in hand: no trigger");
    t.cast(P0, fire).target(bears).go();
    t.resolve_all();
    assert_eq!(where_is(&t, fire), "graveyard");
    gain(&mut t, P1, 2);
    assert_eq!(t.stack_len(), 1);
    // The payment is chosen as it resolves: declining leaves it in the graveyard.
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(where_is(&t, fire), "graveyard");
    gain(&mut t, P1, 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(where_is(&t, fire), "hand");

    // Veilborn Ghoul: a Swamp entering while the Ghoul is in hand or on the battlefield
    // doesn't trigger it; in the graveyard it does.
    let mut t = TestGame::new(2);
    let ghoul = t.hand(P0, "Veilborn Ghoul");
    t.enter(P0, "Swamp");
    flush(&mut t);
    assert_eq!(t.stack_len(), 0);
    let ghoul = move_to(&mut t, ghoul, Zone::Battlefield);
    t.enter(P0, "Swamp");
    flush(&mut t);
    assert_eq!(t.stack_len(), 0);
    destroy(&mut t, ghoul);
    t.enter(P0, "Swamp");
    flush(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(where_is(&t, ghoul), "hand");

    // Ovalchase Daredevil: in hand no; in the graveyard yes (an artifact you control).
    let mut t = TestGame::new(2);
    let dd = t.hand(P0, "Ovalchase Daredevil");
    t.enter(P0, "Ornithopter");
    flush(&mut t);
    assert_eq!(t.stack_len(), 0);
    move_to(&mut t, dd, Zone::Graveyard(P0));
    t.enter(P1, "Ornithopter");
    flush(&mut t);
    assert_eq!(t.stack_len(), 0, "an opponent's artifact");
    t.enter(P0, "Ornithopter");
    flush(&mut t);
    t.resolve_all();
    assert_eq!(where_is(&t, dd), "hand");

    // Masked Admirers: cast from the graveyard's point of view.
    let mut t = TestGame::new(2);
    let adm = t.hand(P0, "Masked Admirers");
    t.lands(P0, "Forest", 6);
    let b1 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, b1).go();
    assert_eq!(t.stack_len(), 1, "Admirers in hand: no trigger");
    t.resolve_all();
    move_to(&mut t, adm, Zone::Graveyard(P0));
    let b2 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, b2).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(where_is(&t, adm), "hand");
}

#[test]
fn masked_admirers_returns_before_the_creature_spell_resolves() {
    cr!("603.3", "405.5");
    ruling!(
        "Masked Admirers",
        "Masked Admirers will return to your hand before the creature spell you cast resolves."
    );
    let mut t = TestGame::new(2);
    let adm = t.graveyard(P0, "Masked Admirers");
    t.lands(P0, "Forest", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(where_is(&t, adm), "hand");
    assert_eq!(where_is(&t, bears), "stack");
}
