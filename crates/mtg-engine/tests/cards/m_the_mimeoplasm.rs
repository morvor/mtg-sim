//! The Mimeoplasm (hand-written, `src/cards/the_mimeoplasm.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn enters_as_a_copy_of_one_with_counters_for_the_others_power() {
    cr!("707.9", "614.1c");
    ruling!("The Mimeoplasm", "You choose which one The Mimeoplasm is copying");
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(wurm), Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    let m = t.enter(P0, "The Mimeoplasm");
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm") && t.in_exile("Hill Giant"));
    let o = t.obj_now(m);
    let (name, counters) = (o.chars.name.to_string(), t.counters(m, "+1/+1"));
    // A copy of the one chosen (the Wurm) with counters equal to the other's power (3).
    assert_eq!((name.as_str(), counters), ("Craw Wurm", 3));
    assert_eq!(t.pt(m), (9, 7));
}

#[test]
fn without_two_creature_cards_it_enters_as_itself() {
    cr!("614.1c");
    ruling!("The Mimeoplasm", "You can't choose to exile just one creature card");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Craw Wurm");
    let m = t.enter(P0, "The Mimeoplasm");
    t.resolve_all();
    assert_eq!(t.obj_now(m).chars.name, "The Mimeoplasm");
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}
