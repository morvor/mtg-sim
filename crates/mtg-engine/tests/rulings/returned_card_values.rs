//! A value of a card that an effect put onto the battlefield from a graveyard, read by a
//! later instruction of the same effect (work item ruling-conflicts-5; provisional register
//! entry `returned-card-values-by-wording`): "its mana value" and "that card's mana value"
//! are the card's as it last existed in the graveyard (Sister Hospitaller's ruling, and
//! Reanimate's in `r_p069_costs.rs`); "that creature's toughness" is the permanent's (Cleric
//! Class's ruling, in `r_p179_toughness.rs`). Each card here returns a card that enters as a
//! copy of Grizzly Bears (mana value 2), so the graveyard's mana value and the
//! battlefield's differ.
//!
//! Also the card-or-permanent half of the same split for a card exiled from the
//! battlefield: "the exiled card" is the card in exile (Deicide's rulings).

use crate::r_p067_common::{exiled, mana, o};
use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The permanent `id` became is a Grizzly Bears (mana value 2): the copy the returned card
/// entered as.
fn entered_as_bears(t: &TestGame, id: ObjectId) {
    assert!(t.on_battlefield(id), "{}", t.dump_log());
    assert_eq!(t.obj_now(id).chars.name, "Grizzly Bears");
    assert_eq!(t.g.mana_value_of(t.g.current(id)), 2);
}

#[test]
fn sister_hospitaller_gains_life_equal_to_the_returned_cards_mana_value_in_the_graveyard() {
    cr!("608.2h");
    interp!("returned-card-values-by-wording");
    ruling!(
        "Sister Hospitaller",
        "Use the mana value of the card as it last existed in the graveyard to determine the amount of life you gain."
    );
    supported("Sister Hospitaller");
    // "When this creature enters, return target creature card from your graveyard to the
    // battlefield. You gain life equal to its mana value.": Clone ({3}{U}) returns as a
    // copy of Grizzly Bears. 4 life, not 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let clone = t.graveyard(P0, "Clone");
    t.answer_targets(P0, &[o(clone)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.enter(P0, "Sister Hospitaller");
    t.resolve_all();
    entered_as_bears(&t, clone);
    assert_eq!(t.life(P0), 24, "{}", t.dump_log());
}

#[test]
fn death_loses_life_equal_to_the_returned_cards_mana_value_in_the_graveyard() {
    cr!("608.2h");
    interp!("returned-card-values-by-wording");
    supported("Life // Death");
    // Death: "Return target creature card from your graveyard to the battlefield. You lose
    // life equal to its mana value." The same wording as Sister Hospitaller's, in a spell:
    // 4 life, not 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let clone = t.graveyard(P0, "Clone");
    let spell = t.hand(P0, "Life // Death");
    mana(&mut t, P0, ManaType::B, 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.cast(P0, spell)
        .method(CastMethod::Half(1))
        .target(clone)
        .go();
    t.resolve_all();
    entered_as_bears(&t, clone);
    assert_eq!(t.life(P0), 16, "{}", t.dump_log());
}

#[test]
fn queens_bay_paladin_loses_life_equal_to_the_returned_cards_mana_value_in_the_graveyard() {
    cr!("608.2h");
    interp!("returned-card-values-by-wording");
    supported("Queen's Bay Paladin");
    // "... return up to one target Vampire card from your graveyard to the battlefield with
    // a finality counter on it. You lose life equal to its mana value.": Moritte of the
    // Frost ({2}{G}{U}{U}, a changeling, so a Vampire card) returns as a copy of Grizzly
    // Bears. 5 life, not 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let moritte = t.graveyard(P0, "Moritte of the Frost");
    t.answer_targets(P0, &[o(moritte)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.enter(P0, "Queen's Bay Paladin");
    t.resolve_all();
    entered_as_bears(&t, moritte);
    assert_eq!(t.counters(moritte, "finality"), 1);
    assert_eq!(t.life(P0), 15, "{}", t.dump_log());
}

#[test]
fn deicide_checks_the_card_in_exile_for_a_god() {
    cr!("400.7j", "701.23b");
    interp!("returned-card-values-by-wording");
    ruling!(
        "Deicide",
        "Deicide looks at the card in exile, not the permanent that was exiled, to determine if it is a God. For each of the Gods in the Theros block, it won't matter what your devotion to its color(s) was. The card is a God card when not on the battlefield."
    );
    ruling!(
        "Deicide",
        "When you search the target's controller's graveyard, hand, and library, you may leave any cards with that name in the zones they're in. You don't have to exile them."
    );
    supported("Deicide");
    // "Exile target enchantment. If the exiled card is a God card, search its controller's
    // graveyard, hand, and library for any number of cards with the same name as that card
    // and exile them. Then that player shuffles." P1's Heliod, God of the Sun, with P1's
    // devotion to white below five, isn't a creature, and so not a God, on the battlefield
    // (Heliod's ruling: "If a God stops being a creature, it loses the type creature and the
    // creature type God."); in exile it's a God card. P0 exiles the Heliod in P1's
    // graveyard and leaves the one in P1's hand.
    let mut t = TestGame::new(2);
    let heliod = t.battlefield(P1, "Heliod, God of the Sun");
    t.g.recompute();
    assert!(!t.obj_now(heliod).is(CardType::Creature));
    assert!(!t.obj_now(heliod).chars.has_subtype("God"));
    let in_graveyard = t.graveyard(P1, "Heliod, God of the Sun");
    let in_hand = t.hand(P1, "Heliod, God of the Sun");
    let deicide = t.hand(P0, "Deicide");
    mana(&mut t, P0, ManaType::W, 2);
    t.answer_choose(P0, &[o(in_graveyard)]);
    t.cast(P0, deicide).target(heliod).go();
    t.resolve_all();
    assert!(exiled(&t, heliod), "{}", t.dump_log());
    assert!(exiled(&t, in_graveyard), "{}", t.dump_log());
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
}

#[test]
fn deicide_checks_the_card_in_exile_not_the_god_the_permanent_was() {
    cr!("400.7j");
    interp!("returned-card-values-by-wording");
    ruling!(
        "Deicide",
        "Deicide looks at the card in exile, not the permanent that was exiled, to determine if it is a God. For each of the Gods in the Theros block, it won't matter what your devotion to its color(s) was. The card is a God card when not on the battlefield."
    );
    supported("Deicide");
    supported("Copy Enchantment");
    // The other way round: P1's Copy Enchantment entered as a copy of P0's Heliod, God of
    // the Sun, and with P1's devotion to white at five (its own {W} and four Savannah
    // Lions) it's a God creature on the battlefield. In exile it's Copy Enchantment, not a
    // God card, so Deicide searches for nothing: the Heliod in P1's graveyard stays there,
    // although P0 would choose to exile it if Deicide searched.
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.battlefield(P1, "Savannah Lions");
    }
    let heliod = t.battlefield(P0, "Heliod, God of the Sun");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[o(heliod)]);
    let copy = t.enter(P1, "Copy Enchantment");
    t.g.recompute();
    assert_eq!(t.obj_now(copy).chars.name, "Heliod, God of the Sun");
    assert!(t.obj_now(copy).is(CardType::Creature));
    assert!(t.obj_now(copy).chars.has_subtype("God"));
    let in_graveyard = t.graveyard(P1, "Heliod, God of the Sun");
    let deicide = t.hand(P0, "Deicide");
    mana(&mut t, P0, ManaType::W, 2);
    t.answer_choose(P0, &[o(in_graveyard)]);
    t.cast(P0, deicide).target(copy).go();
    t.resolve_all();
    assert!(exiled(&t, copy), "{}", t.dump_log());
    assert_eq!(t.obj_now(copy).chars.name, "Copy Enchantment");
    assert_eq!(t.zone(in_graveyard), Zone::Graveyard(P1), "{}", t.dump_log());
}
