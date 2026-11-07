//! CR 101.4, 608.2e: "each player chooses ... and [does something with it]": every player
//! makes their own choice, and their part of the instruction uses that choice, not the
//! choice of the player after them.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn creature_type(name: &str) -> usize {
    subtype_lists()
        .creature
        .iter()
        .position(|t| t == name)
        .expect("a creature type")
}

#[test]
fn grave_sifter_each_player_returns_cards_of_the_type_they_chose() {
    cr!("101.4", "608.2e");
    ruling!(
        "Grave Sifter",
        "The creature type each player chooses applies only to that player."
    );
    // "When this creature enters, each player chooses a creature type and returns any
    // number of cards of that type from their graveyard to their hand."
    let mut t = TestGame::new(2);
    let p0_bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Llanowar Elves");
    let p1_goblin = t.graveyard(P1, "Raging Goblin");
    t.graveyard(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Bear")));
    t.answer_choose(P0, &[Entity::Object(p0_bears)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(creature_type("Goblin")));
    t.answer_choose(P1, &[Entity::Object(p1_goblin)]);
    t.enter(P0, "Grave Sifter");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert!(t.in_hand(P1, "Raging Goblin"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
