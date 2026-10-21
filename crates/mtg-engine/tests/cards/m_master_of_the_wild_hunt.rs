//! Master of the Wild Hunt (hand-written, `src/cards/master_of_the_wild_hunt.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn wolves_gang_up_and_take_damage_back() {
    cr!("120.3");
    ruling!("Master of the Wild Hunt", "Only a Wolf creature tapped as part of the ability's effect can be dealt damage");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of the Wild Hunt");
    let w1 = t.battlefield(P0, "Grizzled Wolverine");
    let wolves: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Dire Wolves")).collect();
    let _ = w1;
    let wurm = t.battlefield(P1, "Craw Wurm");
    // The Wurm (6/4) assigns 6 damage: 3 to each Wolf.
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![3, 3]));
    t.activate(P0, master, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    for w in wolves {
        assert!(!t.on_battlefield(w));
    }
}
