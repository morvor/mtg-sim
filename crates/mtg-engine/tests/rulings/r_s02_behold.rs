//! Rulings batch S02 — behold (CR 701.4): "To behold a [quality], choose a [quality] you
//! control or reveal a [quality] card from your hand."

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The candidates of P0's "Behold" choices so far.
fn behold_candidates(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if *p == P0 && prompt == "Behold" => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_card_revealed_by_telepathy_can_be_revealed_to_behold() {
    cr!("701.4a", "701.20c");
    ruling!(
        "Silvergill Mentor",
        "If a card in your hand is already revealed (perhaps because it was revealed to pay a cost of a spell that's still on the stack or due to the effect of a card like Telepathy), you may reveal it again to pay the cost of another spell or ability that requires you to reveal a card from your hand."
    );
    supported("Silvergill Mentor");
    supported("Telepathy");
    let mut t = TestGame::new(2);
    // P1's Telepathy: "Your opponents play with their hands revealed."
    t.battlefield(P1, "Telepathy");
    let merfolk = t.hand(P0, "Silvergill Mentor");
    assert!(is_revealed(&t.g, merfolk));
    // "As an additional cost to cast this spell, behold a Merfolk or pay {2}. ... When this
    // creature enters, create a 1/1 white and blue Merfolk creature token."
    let plains = t.lands(P0, "Island", 2);
    let mentor = t.hand(P0, "Silvergill Mentor");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(merfolk)]);
    t.cast(P0, mentor).go();
    // Only {1}{U} was paid: the already revealed Merfolk card was revealed again.
    assert!(plains.iter().all(|l| t.obj(*l).tapped));
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.obj(merfolk).zone, Zone::Hand(P0));
    assert_eq!(behold_candidates(&t), vec![vec![Entity::Object(merfolk)]]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Silvergill Mentor").len(), 1);
}

#[test]
fn a_card_that_only_mentions_a_subtype_isnt_a_card_of_that_subtype() {
    cr!("701.4a", "205.3a");
    ruling!(
        "Kinsbaile Aspirant",
        "If an effect refers to a \"[subtype] card,\" it refers only to a card that has that subtype."
    );
    supported("Kinsbaile Aspirant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    // Cloudgoat Ranger creates Kithkin tokens, but it's a Giant Warrior Ranger card.
    t.hand(P0, "Cloudgoat Ranger");
    let aspirant = t.hand(P0, "Kinsbaile Aspirant");
    // "As an additional cost to cast this spell, behold a Kithkin or pay {2}."
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert!(t.cast(P0, aspirant).try_go().is_err());
    assert!(behold_candidates(&t).iter().all(|c| c.is_empty()));
    assert_eq!(t.obj(aspirant).zone, Zone::Hand(P0));
    // A Kithkin card can be beheld.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let kithkin = t.hand(P0, "Goldmeadow Harrier");
    let aspirant = t.hand(P0, "Kinsbaile Aspirant");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(kithkin)]);
    t.cast(P0, aspirant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kinsbaile Aspirant").len(), 1);
}

#[test]
fn teeming_dragonstorm_isnt_a_dragon_card() {
    cr!("701.4a", "205.3a");
    ruling!(
        "Caustic Exhale",
        "Teeming Dragonstorm is a card that cares about Dragons and features Dragons in its art, but it isn’t a Dragon card."
    );
    supported("Caustic Exhale");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.hand(P0, "Teeming Dragonstorm");
    let exhale = t.hand(P0, "Caustic Exhale");
    // "As an additional cost to cast this spell, behold a Dragon or pay {1}." There's no
    // Dragon to behold.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert!(t.cast(P0, exhale).target(bears).try_go().is_err());
    assert!(behold_candidates(&t).iter().all(|c| c.is_empty()));
    // With a Dragon card in hand instead, beholding it pays the cost.
    let dragon = t.hand(P0, "Shivan Dragon");
    t.clear_answers();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.cast(P0, exhale).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
