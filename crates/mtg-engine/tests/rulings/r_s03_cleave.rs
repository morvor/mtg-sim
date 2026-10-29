//! Rulings batch S03 — cleave (CR 702.148): "You may cast this spell for its cleave cost.
//! If you do, remove the words in square brackets."

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, target_candidates};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_spell_cant_be_cast_for_its_cleave_cost_and_another_alternative_cost() {
    cr!("702.148a", "118.9a", "702.34a", "601.2b");
    ruling!(
        "Alchemist's Retrieval",
        "You can't cast a spell for both its cleave cost and another alternative cost. For example, if an effect gives an Alchemist's Retrieval in your graveyard a flashback cost of {U}, you can't cast it from your graveyard for its cleave cost."
    );
    supported("Alchemist's Retrieval");
    supported("Snapcaster Mage");
    // Alchemist's Retrieval: "Cleave {1}{U}. Return target nonland permanent [you
    // control] to its owner's hand."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    let card = t.graveyard(P0, "Alchemist's Retrieval");
    // Snapcaster Mage gives it flashback, with a flashback cost of {U} (its mana cost).
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    assert!(t.obj(card).chars.has_keyword(KeywordKind::Flashback));
    // In hand, the cleave cost could be paid; from the graveyard only flashback is possible.
    assert!(can_cast(
        &mut t,
        P0,
        card,
        CastMethod::Keyword(KeywordKind::Flashback)
    ));
    assert!(!can_cast(
        &mut t,
        P0,
        card,
        CastMethod::Keyword(KeywordKind::Cleave)
    ));
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    // Cast for its flashback cost, it still says "you control": only P0's permanents can
    // be targeted.
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&Entity::Object(mine)));
    assert!(!cands[0].contains(&Entity::Object(theirs)));
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.zone(card), Zone::Exile);

    // The card in hand can be cast for its cleave cost.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Alchemist's Retrieval");
    t.lands(P0, "Island", 2);
    assert!(can_cast(
        &mut t,
        P0,
        card,
        CastMethod::Keyword(KeywordKind::Cleave)
    ));
}
