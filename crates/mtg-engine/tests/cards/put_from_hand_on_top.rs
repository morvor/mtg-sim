//! "Draw three cards, then put two cards from your hand on top of your library in any
//! order." (Brainstorm, Cavalier of Gales, Hidetsugu and Kairi; CR 401.4, 608.2c).

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
fn brainstorm_draws_then_puts_two_cards_back_while_resolving() {
    cr!("401.4", "608.2c", "121.1");
    ruling!(
        "Brainstorm",
        "You draw three cards and put two cards back all while Brainstorm is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    compiles("Brainstorm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let forest = t.hand(P0, "Forest");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let brainstorm = t.hand(P0, "Brainstorm");
    let library = t.library_size(P0);
    // P0 puts back the Forest (already in hand) and the Lightning Bolt it just drew.
    t.answer_choose(P0, &[Entity::Object(forest), Entity::Object(bolt)]);
    t.cast(P0, brainstorm).go();
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.library_size(P0), library - 3 + 2);
    assert_eq!(t.hand_size(P0), 2);
    let top: Vec<String> = t.g.player(P0).library[library - 3..]
        .iter()
        .map(|c| t.obj(*c).chars.name.to_string())
        .collect();
    let mut sorted = top.clone();
    sorted.sort();
    assert_eq!(sorted, vec!["Forest", "Lightning Bolt"]);
    // Nobody got priority during the resolution.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    // P0 was asked to arrange them (CR 401.4); arranged the other way, the other card
    // ends up on top.
    let orders = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
        .count();
    assert_eq!(orders, 1);
    let top_of = |t: &TestGame| {
        let c = *t.g.player(P0).library.last().unwrap();
        t.obj(c).chars.name.to_string()
    };
    let mut u = TestGame::new(2);
    u.lands(P0, "Island", 1);
    let forest = u.hand(P0, "Forest");
    let bolt = u.library_top(P0, "Lightning Bolt");
    let brainstorm = u.hand(P0, "Brainstorm");
    u.answer_choose(P0, &[Entity::Object(forest), Entity::Object(bolt)]);
    u.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    u.cast(P0, brainstorm).go();
    u.resolve();
    assert_ne!(top_of(&t), top_of(&u));
}

#[test]
fn hidetsugu_and_kairi_can_put_back_cards_already_in_hand() {
    cr!("401.4", "603.6a");
    ruling!(
        "Hidetsugu and Kairi",
        "The two cards you put on top of your library can be from the three you just drew or ones that were already in your hand."
    );
    compiles("Hidetsugu and Kairi");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Island");
    let library = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Hidetsugu and Kairi");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert!(!t.in_hand(P0, "Forest") && !t.in_hand(P0, "Island"));
    assert_eq!(t.library_size(P0), library - 3 + 2);
}
