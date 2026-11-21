//! Rulings batch S03 — convoke (CR 702.51): "Your creatures can help cast this spell. Each
//! creature you tap while casting this spell pays for {1} or one mana of that creature's
//! color."

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

#[test]
fn convoke_can_help_pay_an_alternative_cost() {
    cr!("702.51a", "702.51b", "702.34a", "118.9", "601.2f");
    ruling!(
        "Clever Concealment",
        "Because convoke isn't an alternative cost, it can be used in conjunction with alternative costs."
    );
    supported("Clever Concealment");
    supported("Snapcaster Mage");
    supported("Savannah Lions");
    // Clever Concealment ({2}{W}{W}): "Convoke. Any number of target nonland permanents
    // you control phase out." Snapcaster Mage gives it flashback {2}{W}{W}.
    let mut t = TestGame::new(2);
    let lions: Vec<ObjectId> = (0..2)
        .map(|_| t.battlefield(P0, "Savannah Lions"))
        .collect();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Clever Concealment");
    t.answer_targets(P0, &[Entity::Object(card)]);
    let snapcaster = t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    // No lands: the four creatures pay the whole flashback cost (the two white Lions pay
    // {W}{W}, the Bears and the Snapcaster {2}).
    assert!(can_cast(&mut t, P0, card, FLASHBACK));
    let convokers = [lions[0], lions[1], bears, snapcaster];
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(convokers.iter().map(|c| Entity::Object(*c)).collect()),
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    let info = t.obj(spell).stack.as_ref().unwrap().cast.clone();
    assert_eq!(info.method, FLASHBACK);
    assert_eq!(info.convoked.len(), 4);
    assert!(convokers.iter().all(|c| t.obj(*c).tapped));
    t.resolve_all();
    assert!(t.obj(bears).phased_out);
    // Cast with flashback, it's exiled.
    assert_eq!(t.zone(card), Zone::Exile);
}
