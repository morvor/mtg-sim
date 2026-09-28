//! Rulings batch S22 — Djeru and Hazoret ("Whenever Djeru and Hazoret attacks, look at the
//! top six cards of your library. You may exile a legendary creature card from among
//! them. Put the rest on the bottom of your library in a random order. Until end of turn,
//! you may cast the exiled card without paying its mana cost."): the card is cast without
//! paying its mana cost (CR 118.9): no other alternative cost, additional costs paid
//! (CR 118.9a).

use crate::r_s01_common::*;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Djeru and Hazoret attacks P1 unblocked with `card` on top of P0's library, which P0
/// exiles; then P0's second main phase.
fn djeru_attacks(t: &mut TestGame, card: ObjectId) -> ObjectId {
    let dh = t.battlefield(P0, "Djeru and Hazoret");
    t.answer_choose(P0, &[Entity::Object(card)]);
    attack_with(t, &[(dh, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
    t.clear_answers();
    t.advance_to(P0, Step::PostcombatMain);
    let now = t.g.current(card);
    assert_eq!(t.zone(now), Zone::Exile);
    now
}

#[test]
fn djeru_and_hazoret_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9", "118.9a", "118.8a", "601.2b", "702.33a");
    ruling!(
        "Djeru and Hazoret",
        "If you cast a spell without paying its mana cost, you can’t choose to cast it for any alternative costs. You can, however, pay any additional costs. If the spell has any mandatory additional costs, you must pay those."
    );
    supported("Djeru and Hazoret");
    supported("Verix Bladewing");
    supported("Zurgo Bellstriker");
    // Verix Bladewing ({2}{R}{R}, kicker {3}: "When Verix Bladewing enters, if it was
    // kicked, create Karox Bladewing, a legendary 4/4 red Dragon creature token with
    // flying."): the kicker may be paid.
    let mut t = TestGame::new(2);
    let verix = t.library_top(P0, "Verix Bladewing");
    let verix = djeru_attacks(&mut t, verix);
    t.lands(P0, "Mountain", 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, verix).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.named_on_battlefield("Verix Bladewing").len(), 1);
    assert_eq!(t.named_on_battlefield("Karox Bladewing").len(), 1);
    // Zurgo Bellstriker (dash {1}{R}) can't be cast for its dash cost this way.
    let mut t = TestGame::new(2);
    let zurgo = t.library_top(P0, "Zurgo Bellstriker");
    let zurgo = djeru_attacks(&mut t, zurgo);
    t.lands(P0, "Mountain", 2);
    assert_eq!(legal_cast_methods(&mut t, P0, zurgo), vec![CastMethod::Free]);
    // A legendary creature card with a mandatory additional cost ("discard a card"): it
    // must be paid.
    let legend = custom_card(
        "Discarding Legend",
        "Legendary Creature — Human",
        "{1}{B}",
        Some((3, 3)),
        "As an additional cost to cast this spell, discard a card.",
    );
    let mut t = TestGame::new(2);
    let card = t.custom(P0, legend.clone(), Zone::Library(P0));
    let card = djeru_attacks(&mut t, card);
    // With no card in hand to discard, it can't be cast.
    assert_eq!(t.hand_size(P0), 0);
    assert!(t
        .cast(P0, card)
        .method(CastMethod::Free)
        .try_go()
        .is_err());
    t.hand(P0, "Forest");
    t.cast(P0, card).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.named_on_battlefield("Discarding Legend").len(), 1);
}
