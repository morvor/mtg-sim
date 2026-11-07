//! Rulings batch P202 — buyback (CR 702.27): "You may pay an additional [cost] as you
//! cast this spell. If the buyback cost was paid, put this spell into its owner's hand
//! instead of into that player's graveyard as it resolves."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The optional costs offered to P0 since decision `from`.
fn offered(t: &TestGame, from: usize) -> Vec<String> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::OptionalCost { name, .. } if p == P0 => Some(name),
            _ => None,
        })
        .collect()
}

#[test]
fn blast_from_the_past_with_flashback_and_buyback_is_exiled() {
    cr!("702.27a", "702.34a", "614.1a");
    ruling!(
        "Blast from the Past",
        "Flashback and buyback don't mix. If you cast Blast from the Past using flashback, it will be exiled rather than go back to your hand."
    );
    supported("Blast from the Past");
    let mut t = TestGame::new(2);
    // Flashback {3}{R}, buyback {4}{R}, kicker {2}{R}: twelve mana.
    t.lands(P0, "Mountain", 12);
    let c = t.graveyard(P0, "Blast from the Past");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let from = t.asked().len();
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(Entity::Player(P1))
        .go();
    assert!(offered(&t, from).contains(&"buyback".to_string()));
    assert_eq!(tapped_lands(&t, P0), 12);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Blast from the Past"));
    assert!(!t.in_hand(P0, "Blast from the Past"));
}

#[test]
fn elvish_fury_with_an_illegal_target_goes_to_the_graveyard_despite_buyback() {
    cr!("702.27a", "608.2b");
    ruling!(
        "Elvish Fury",
        "If the target creature is an illegal target by the time Elvish Fury tries to resolve, the spell doesn’t resolve. If its buyback cost was paid, you won’t return it to your hand."
    );
    supported("Elvish Fury");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Elvish Fury");
    t.cast(P0, c).kicked(true).target(bears).go();
    assert_eq!(tapped_lands(&t, P0), 5);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Elvish Fury"));
    assert!(!t.in_hand(P0, "Elvish Fury"));

    // With a legal target, it returns to hand.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let c = t.hand(P0, "Elvish Fury");
    t.cast(P0, c).kicked(true).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.in_hand(P0, "Elvish Fury"));
}

#[test]
fn innocuous_insect_with_buyback_goes_to_hand_instead_of_the_battlefield() {
    cr!("702.27a", "608.3");
    ruling!(
        "Innocuous Insect",
        "If you pay the buyback cost for a permanent spell, it doesn’t enter the battlefield as it resolves. It moves from the stack to its owner’s hand."
    );
    supported("Innocuous Insect");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let c = t.hand(P0, "Innocuous Insect");
    let hand = t.hand_size(P0);
    t.cast(P0, c).kicked(true).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Innocuous Insect").is_empty());
    assert!(t.in_hand(P0, "Innocuous Insect"));
    // The insect is back, and its cast trigger drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);

    // Without buyback, it enters the battlefield.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let c = t.hand(P0, "Innocuous Insect");
    t.cast(P0, c).kicked(false).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Innocuous Insect").len(), 1);
}

#[test]
fn flowstone_flood_buyback_needs_a_card_to_discard() {
    cr!("702.27a", "601.2f", "601.2h");
    ruling!(
        "Flowstone Flood",
        "You can’t pay the Buyback cost unless you have at least one card in your hand to discard."
    );
    supported("Flowstone Flood");
    // Its hand is empty once Flowstone Flood is on the stack: the buyback can't be paid
    // (it isn't offered).
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Flowstone Flood");
    assert_eq!(t.hand_size(P0), 1);
    let from = t.asked().len();
    t.cast(P0, c).kicked(true).target(land).go();
    assert!(offered(&t, from).is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Flowstone Flood"));
    assert!(!t.on_battlefield(land));
    assert_eq!(t.life(P0), 20);

    // With another card in hand, it can.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Mountain", 4);
    t.hand(P0, "Grizzly Bears");
    let c = t.hand(P0, "Flowstone Flood");
    let from = t.asked().len();
    t.cast(P0, c).kicked(true).target(land).go();
    assert_eq!(offered(&t, from), vec!["buyback".to_string()]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    assert!(t.in_hand(P0, "Flowstone Flood"));
}

#[test]
fn slaughters_buyback_needs_four_life_even_if_you_cant_lose() {
    cr!("702.27a", "119.4", "601.2h");
    ruling!(
        "Slaughter",
        "You can’t pay the buyback if you have less than 4 life, even if an effect would keep you from losing for having 0 or less life."
    );
    supported("Slaughter");
    supported("Platinum Angel");
    for (life, ok) in [(3, false), (4, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Platinum Angel");
        t.g.players[P0.idx()].life = life;
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let c = t.hand(P0, "Slaughter");
        let from = t.asked().len();
        t.cast(P0, c).kicked(true).target(bears).go();
        // At 3 life the buyback isn't offered (it can't be paid).
        assert_eq!(!offered(&t, from).is_empty(), ok, "life {life}");
        assert_eq!(t.life(P0), if ok { 0 } else { 3 });
        t.resolve_all();
        assert!(!t.on_battlefield(bears));
        assert_eq!(t.in_hand(P0, "Slaughter"), ok);
        assert_eq!(t.in_graveyard(P0, "Slaughter"), !ok);
        assert!(!t.has_lost(P0));
    }
}
