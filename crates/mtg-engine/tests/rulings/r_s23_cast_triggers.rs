//! Rulings batch S23 — "When you cast this spell" abilities (CR 603.2, 603.3): the ability
//! triggers as the spell becomes cast and is put on the stack on top of it, so it resolves
//! first. "Any player may [cost]. If a player does, counter this spell." (CR 118.12): in
//! turn order, each player may pay; if one does, the spell is countered.

use crate::r_s01_common::*;
use crate::r_s04_common::stack_items;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn dash_hopes_cast_trigger_goes_on_the_stack_on_top_of_it() {
    cr!("603.2", "603.3", "118.12", "101.4");
    ruling!(
        "Dash Hopes",
        "When this spell is cast, its “when you cast” ability triggers and goes on the stack on top of it."
    );
    supported("Dash Hopes");
    // Dash Hopes: "When you cast this spell, any player may pay 5 life. If a player does,
    // counter Dash Hopes. / Counter target spell." P1 pays 5 life: Dash Hopes is
    // countered before it can counter the Bolt.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Swamp", 2);
    let hopes = t.hand(P0, "Dash Hopes");
    t.cast(P0, hopes).target(Entity::Object(bolt)).go();
    t.settle();
    let items = stack_items(&t);
    assert_eq!(items.len(), 3);
    assert_eq!(&items[..2], &["Lightning Bolt", "Dash Hopes"]);
    assert!(items[2].starts_with("ability: When you cast"));
    // P0 (the active player) is asked first and declines; P1 pays.
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.resolve();
    let asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P0, P1]);
    assert_eq!(t.life(P1), 15);
    assert!(t.in_graveyard(P0, "Dash Hopes"));
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // Nobody pays: Dash Hopes resolves and counters the Bolt.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Swamp", 2);
    let hopes = t.hand(P0, "Dash Hopes");
    t.cast(P0, hopes).target(Entity::Object(bolt)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn brain_gorgers_cast_trigger_goes_on_the_stack_on_top_of_it() {
    cr!("603.2", "603.3", "118.12", "118.3");
    ruling!(
        "Brain Gorgers",
        "When this spell is cast, its \"when you cast\" ability triggers and goes on the stack on top of it."
    );
    supported("Brain Gorgers");
    // Brain Gorgers: "When you cast this spell, any player may sacrifice a creature of
    // their choice. If a player does, counter Brain Gorgers."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let gorgers = t.hand(P0, "Brain Gorgers");
    t.cast(P0, gorgers).go();
    t.settle();
    let items = stack_items(&t);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], "Brain Gorgers");
    assert!(items[1].starts_with("ability: When you cast"));
    // P0 controls no creature and can't take the option; P1 sacrifices the Bears.
    let from = t.asked().len();
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.resolve_all();
    let yes_no_of = |p: PlayerId| {
        t.asked()[from..]
            .iter()
            .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
            .count()
    };
    assert_eq!(yes_no_of(P0), 0);
    assert_eq!(yes_no_of(P1), 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Brain Gorgers"));
    assert!(t.named_on_battlefield("Brain Gorgers").is_empty());
}

#[test]
fn phantasmagorian_each_player_in_turn_order_gets_the_option() {
    cr!("118.12", "101.4", "118.3");
    ruling!(
        "Phantasmagorian",
        "As the triggered ability resolves, the active player gets the option to perform the action. If that player declines, the next player in turn order gets the option. As soon as any player performs the action, the spell is countered, but the remaining players still get the option."
    );
    ruling!(
        "Phantasmagorian",
        "A player can't choose to discard unless they actually have three cards in their hand."
    );
    supported("Phantasmagorian");
    // Phantasmagorian: "When you cast this spell, any player may discard three cards. If
    // a player does, counter Phantasmagorian." Four players: P0 (active) declines, P1 has
    // only two cards and can't take the option, P2 discards three cards (countering it),
    // and P3 still gets the option.
    let mut t = TestGame::new(4);
    for p in [P0, P1, P2, P3] {
        for _ in 0..if p == P1 { 2 } else { 3 } {
            t.hand(p, "Forest");
        }
    }
    t.lands(P0, "Swamp", 7);
    let card = t.hand(P0, "Phantasmagorian");
    t.cast(P0, card).go();
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.answer(P2, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P3, DecisionKind::YesNo, Answer::Bool(false));
    t.resolve();
    let asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P0, P2, P3]);
    assert_eq!(t.hand_size(P2), 0);
    assert_eq!(t.hand_size(P3), 3);
    assert!(t.in_graveyard(P0, "Phantasmagorian"));
    // If all players decline, the spell remains on the stack and resolves.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    t.lands(P0, "Swamp", 7);
    let card = t.hand(P0, "Phantasmagorian");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Phantasmagorian").len(), 1);
    assert_eq!(t.hand_size(P1), 3);
}
