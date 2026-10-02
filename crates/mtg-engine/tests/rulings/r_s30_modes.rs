//! Rulings batch S30 — the modes of a modal spell are followed in the order they're
//! printed, whatever order they were chosen in (CR 700.2, 608.2c).

use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn dromokas_command_puts_the_counter_on_before_the_fight() {
    cr!("700.2", "608.2c", "701.14a");
    ruling!(
        "Dromoka's Command",
        "As the spell resolves, follow the instructions of the modes you chose in the order they are printed on the card."
    );
    supported("Dromoka's Command");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // "Put a +1/+1 counter on target creature." and "Target creature you control fights
    // target creature you don't control." — chosen in the other order.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3, 2]));
    lands_for_cost(&mut t, P0, "Dromoka's Command");
    let cmd = t.hand(P0, "Dromoka's Command");
    t.cast_with(
        P0,
        cmd,
        &[
            Entity::Object(bears),
            Entity::Object(bears),
            Entity::Object(giant),
        ],
    )
    .unwrap();
    t.resolve_all();
    // The Bears fought as a 3/3: 3 damage to Hill Giant (3/3), which died.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn ojutais_command_returns_the_creature_before_drawing() {
    cr!("700.2", "608.2c");
    ruling!(
        "Ojutai's Command",
        "As the spell resolves, follow the instructions of the modes you chose in the order they are printed on the card. For example, if you chose the second and fourth modes of Ojutai’s Command"
    );
    supported("Ojutai's Command");
    let mut t = TestGame::new(2);
    // P0 has drawn one card this turn.
    t.g.draw_cards(P0, 1);
    t.settle();
    // Knowledge Seeker (mana value 2): "Whenever you draw your second card each turn,
    // put a +1/+1 counter on this creature."
    let seeker = t.graveyard(P0, "Knowledge Seeker");
    // "Draw a card." and "Return target creature card with mana value 2 or less from
    // your graveyard to the battlefield." — chosen in the other order.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3, 0]));
    lands_for_cost(&mut t, P0, "Ojutai's Command");
    let cmd = t.hand(P0, "Ojutai's Command");
    t.cast_with(P0, cmd, &[Entity::Object(seeker)]).unwrap();
    t.resolve_all();
    // Knowledge Seeker was on the battlefield when the second card was drawn.
    let seeker = t.g.current(seeker);
    assert!(t.on_battlefield(seeker));
    assert_eq!(t.counters(seeker, "+1/+1"), 1);
}
