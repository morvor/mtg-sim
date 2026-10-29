//! Per-player outcomes of an instruction several players were given (CR 101.4, 118.12,
//! 608.2c, 608.2e): "each player who doesn't ...", "for each opponent who didn't discard
//! two nonland cards this way ...", "each player who sacrificed a creature this way ...".

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn braids_each_opponent_who_doesnt_sacrifice_loses_life_and_you_draw() {
    cr!("101.4", "118.12", "608.2e");
    ruling!(
        "Braids, Arisen Nightmare",
        "each opponent may choose to sacrifice any permanent they control that shares any card type with it"
    );
    supported("Braids, Arisen Nightmare");
    // Three players: P1 sacrifices a creature, P2 declines, P3 accepts but controls no
    // creature (so doesn't sacrifice one).
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Braids, Arisen Nightmare");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let p1 = t.battlefield(P1, "Grizzly Bears");
    let p2 = t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P3, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(p1)]);
    t.answer_yes(P2, false);
    t.answer_yes(P3, true);
    t.advance_to(P0, Step::End);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(p1));
    assert!(t.on_battlefield(p2));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P3), 18);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn creeping_dread_opponents_sharing_a_card_type_with_your_discard_lose_life() {
    cr!("101.4", "608.2c");
    ruling!(
        "Creeping Dread",
        "If an opponent discards a card that shares two card types with the card you discarded"
    );
    supported("Creeping Dread");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Creeping Dread");
    let mine = t.hand(P0, "Grizzly Bears");
    let p1 = t.hand(P1, "Hill Giant");
    let p2 = t.hand(P2, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P1, &[Entity::Object(p1)]);
    t.answer_choose(P2, &[Entity::Object(p2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn infernal_offering_each_player_who_sacrificed_draws_two() {
    cr!("101.4", "608.2c");
    ruling!(
        "Infernal Offering",
        "If you control no creatures, you won’t sacrifice one and you won’t draw two cards"
    );
    supported("Infernal Offering");
    // You control no creature; the opponent does.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Infernal Offering");
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.cast(P0, spell).go();
    t.resolve();
    // Their creature was sacrificed (then returned by the second part, a new object).
    assert!(!t.g.is_live(theirs));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // The spell left P0's hand; P0 sacrificed nothing and drew nothing.
    assert_eq!(t.hand_size(P0), h0 - 1);
    assert_eq!(t.hand_size(P1), h1 + 2);
}

#[test]
fn kwain_each_player_who_drew_gains_one_life() {
    cr!("101.4", "608.2e");
    supported("Kwain, Itinerant Meddler");
    let mut t = TestGame::new(3);
    let kwain = t.battlefield(P0, "Kwain, Itinerant Meddler");
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.activate(P0, kwain, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (hands.0 + 1, hands.1, hands.2 + 1)
    );
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 20, 21));
}

#[test]
fn orzhov_advokist_each_player_who_puts_counters() {
    cr!("101.4", "118.12");
    ruling!(
        "Orzhov Advokist",
        "you choose a creature you control or choose not to put any counters on a creature"
    );
    supported("Orzhov Advokist");
    let mut t = TestGame::new(2);
    let advokist = t.battlefield(P0, "Orzhov Advokist");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Entities, Answer::Entities(vec![Entity::Object(theirs)]));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 2);
    assert_eq!(t.counters(advokist, "+1/+1"), 0);
}
