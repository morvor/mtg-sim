//! Rulings batch S22 — Mindclaw Shaman ("When this creature enters, target opponent
//! reveals their hand. You may cast an instant or sorcery spell from among those cards
//! without paying its mana cost."): the card is cast from the opponent's hand while the
//! ability resolves (CR 608.2g), without paying its mana cost (CR 118.9).

use crate::r_s01_common::*;
use crate::r_s22_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn mindclaw(t: &mut TestGame, name: &str) -> ObjectId {
    t.hand(P1, name)
}

fn run_mindclaw(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    t.enter(P0, "Mindclaw Shaman");
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn mindclaw_shaman_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Mindclaw Shaman",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. You can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Mindclaw Shaman",
        "You cast the card from the opponent’s hand immediately. Ignore timing restrictions based on the card’s type."
    );
    supported("Mindclaw Shaman");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: mindclaw,
        run: run_mindclaw,
    });
}

#[test]
fn mindclaw_shaman_x_is_zero_and_the_card_goes_to_its_owners_graveyard() {
    cr!("107.3b", "118.9", "404.1");
    ruling!(
        "Mindclaw Shaman",
        "If the card has X in its mana cost, you must choose 0 as its value."
    );
    ruling!(
        "Mindclaw Shaman",
        "The card goes to its owner’s graveyard after it resolves (or otherwise leaves the stack), not yours."
    );
    supported("Mindclaw Shaman");
    // Blaze ("Blaze deals X damage to any target.") from P1's hand, cast with X = 0 at
    // P1's Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P1, "Blaze");
    t.lands(P0, "Mountain", 6);
    run_mindclaw(&mut t, blaze, &|t| {
        t.answer(P0, DecisionKind::X, Answer::Number(5));
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert!(t.on_battlefield(giant));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
    assert_eq!(tapped_lands(&t, P0), 0);
    assert!(t.in_graveyard(P1, "Blaze"));
    assert!(!t.in_graveyard(P0, "Blaze"));
}

#[test]
fn mindclaw_shaman_if_no_card_can_be_cast_the_ability_just_finishes() {
    cr!("608.2g", "601.2c");
    ruling!(
        "Mindclaw Shaman",
        "If you can’t cast any instant or sorcery card (perhaps because there are no legal targets available) or if you choose not to cast one, then Mindclaw Shaman’s ability finishes resolving and the game moves on."
    );
    supported("Mindclaw Shaman");
    // Naturalize ("Destroy target artifact or enchantment.") has no legal target.
    let mut t = TestGame::new(2);
    let nat = t.hand(P1, "Naturalize");
    run_mindclaw(&mut t, nat, &|_| {});
    assert_eq!(t.zone(nat), Zone::Hand(P1));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.named_on_battlefield("Mindclaw Shaman").len(), 1);
    // P0 may choose not to cast one: Lightning Bolt stays in P1's hand.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[]);
    t.enter(P0, "Mindclaw Shaman");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
    assert_eq!(t.life(P1), 20);
}
