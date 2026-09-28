//! Rulings batch S22 — Bismuth Mindrender ("Whenever this creature deals combat damage to
//! a player, that player exiles cards from the top of their library until they exile a
//! nonland card. You may cast that card by paying life equal to the spell's mana value
//! rather than paying its mana cost."): an alternative cost of life (CR 118.9), paid
//! only for that spell as the ability resolves (CR 608.2g); no other alternative cost,
//! additional costs paid (CR 118.9a), X is 0 (CR 107.3b).

use crate::r_s01_common::*;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s22_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `name` on top of P1's library under a land (exiled first), with Bismuth
/// Mindrender on P0's battlefield.
fn bismuth(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Bismuth Mindrender");
    stack_library(t, P1, &["Forest", name])[1]
}

fn run_bismuth(t: &mut TestGame, _card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let b = t.named_on_battlefield("Bismuth Mindrender")[0];
    t.answer_yes(P0, true);
    answers(t);
    attack_p1_unblocked(t, b);
    t.clear_answers();
}

#[test]
fn bismuth_mindrender_casts_for_life_but_additional_costs_are_paid() {
    cr!("118.9", "118.9a", "118.8a", "601.2b", "608.2g", "119.4");
    ruling!(
        "Bismuth Mindrender",
        "If you cast a spell for another cost \"rather than paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    supported("Bismuth Mindrender");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: bismuth,
        run: run_bismuth,
    });
    // Burst Lightning (mana value 1) costs 1 life; Tormenting Voice (2), 2 life.
    let mut t = TestGame::new(2);
    let voice = bismuth(&mut t, "Tormenting Voice");
    t.hand(P0, "Forest");
    run_bismuth(&mut t, voice, &|_| {});
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.life(P0), 18);
    // Bismuth Mindrender (4/4) dealt its combat damage.
    assert_eq!(t.life(P1), 20 - 4);
}

#[test]
fn bismuth_mindrender_x_is_zero() {
    cr!("107.3b", "118.9");
    ruling!(
        "Bismuth Mindrender",
        "If a spell you cast this way has {X} in its mana cost, you must choose 0 as the value of X when casting it."
    );
    supported("Bismuth Mindrender");
    // Blaze ({X}{R}) is cast for 1 life with X = 0 at P1's Hill Giant.
    let mut t = TestGame::new(2);
    let blaze = bismuth(&mut t, "Blaze");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 6);
    run_bismuth(&mut t, blaze, &|t| {
        t.answer(P0, DecisionKind::X, Answer::Number(5));
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert!(t.in_graveyard(P1, "Blaze"));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
    assert_eq!(tapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn bismuth_mindrender_a_card_not_cast_stays_exiled() {
    cr!("608.2g", "406.3");
    ruling!(
        "Bismuth Mindrender",
        "If you choose not to cast the card, it remains in exile."
    );
    ruling!(
        "Bismuth Mindrender",
        "You choose whether or not to cast the exiled card as Bismuth Mindrender's triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn."
    );
    supported("Bismuth Mindrender");
    let mut t = TestGame::new(2);
    let bolt = bismuth(&mut t, "Lightning Bolt");
    let b = t.named_on_battlefield("Bismuth Mindrender")[0];
    t.answer_yes(P0, false);
    attack_p1_unblocked(&mut t, b);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.life(P0), 20);
    // It can't be cast later in the turn.
    t.lands(P0, "Mountain", 1);
    assert!(legal_cast_methods(&mut t, P0, bolt).is_empty());
}
