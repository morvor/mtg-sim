//! CR 118.3, 602.2b, 701.20a: a cost to reveal cards with a quality from your hand is paid
//! only with cards that have it: an answer naming another card doesn't pay it.

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn martyr_of_spores_reveals_only_green_cards() {
    cr!("118.3", "602.2b", "701.20a");
    ruling!(
        "Martyr of Spores",
        "Each card that’s revealed to pay the cost remains revealed until the ability leaves the stack."
    );
    // "{1}, Reveal X green cards from your hand, Sacrifice this creature: Target creature
    // gets +X/+X until end of turn." X = 1; P0 names Shock, which isn't green.
    let mut t = TestGame::new(2);
    let martyr = t.battlefield(P0, "Martyr of Spores");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    let shock = t.hand(P0, "Shock");
    t.g.players[0].mana_pool.add_type(ManaType::C, 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.activate(P0, martyr, 0, &[Entity::Object(bears)])
        .expect("activate Martyr of Spores");
    let revealed: Vec<ObjectId> = t.g.reveals.revealed.iter().map(|r| r.card).collect();
    assert_eq!(revealed, vec![elves]);
    assert!(mtg_engine::reveal::is_revealed(&t.g, elves));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    // It stops being revealed once the ability has left the stack.
    assert!(!mtg_engine::reveal::is_revealed(&t.g, elves));
}
