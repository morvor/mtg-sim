//! "Any player may [cost]. If a player does, [effect]." (CR 118.12, 101.4): each player
//! in turn order may pay a cost of their own (life, a sacrifice of their choice, discards,
//! cards exiled from their graveyard); the effect happens if one does.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn shivan_wumpus_any_player_may_sacrifice_a_land_of_their_choice() {
    cr!("118.12", "101.4");
    ruling!(
        "Shivan Wumpus",
        "If a player elects to sacrifice a land, Shivan Wumpus is put on top of its owner's library, but then all remaining players still get the option."
    );
    compiles("Shivan Wumpus");
    // "When this creature enters, any player may sacrifice a land of their choice. If a
    // player does, put this creature on top of its owner's library."
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P1, &[Entity::Object(forest)]);
    let from = t.asked().len();
    let wumpus = t.enter(P0, "Shivan Wumpus");
    t.g.flush_events();
    t.resolve_all();
    // P0 controls no land and isn't asked.
    let asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P1]);
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(!t.on_battlefield(wumpus));
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Shivan Wumpus");
    // Nobody sacrifices: it stays.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    let wumpus = t.enter(P0, "Shivan Wumpus");
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(wumpus));
}

#[test]
fn carrion_rats_any_player_may_exile_a_card_from_their_graveyard() {
    cr!("118.12", "510.1a");
    compiles("Carrion Rats");
    // "Whenever this creature attacks or blocks, any player may exile a card from their
    // graveyard. If a player does, this creature assigns no combat damage this turn."
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Carrion Rats");
    let card = t.graveyard(P1, "Forest");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P1, &[Entity::Object(card)]);
    t.attack(&[(rats, Entity::Player(P1))], &[]);
    assert!(t.in_exile("Forest"));
    assert_eq!(t.life(P1), 20);
    // With nothing exiled, it deals its damage.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Carrion Rats");
    t.attack(&[(rats, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}
