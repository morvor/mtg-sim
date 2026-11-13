//! Herald of Leshrac (hand-written, `src/cards/herald_of_leshrac.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn gains_control_of_a_land_for_each_age_counter() {
    cr!("702.24a");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Leshrac");
    let lands = t.lands(P1, "Forest", 3);
    t.answer_yes(P0, true);
    upkeep(&mut t);
    assert_eq!(t.counters(herald, "age"), 1);
    let mine = lands.iter().filter(|l| t.obj_now(**l).controller == P0).count();
    assert_eq!(mine, 1);
    // +1/+1 for each land you control but don't own.
    assert_eq!(t.pt(herald), (3, 5));
}

#[test]
fn sacrificed_without_enough_lands() {
    cr!("702.24a");
    ruling!("Herald of Leshrac", "you can't pay its cumulative upkeep cost and must sacrifice it");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Leshrac");
    t.g.add_counters(Entity::Object(herald), "age", 1, None);
    t.lands(P1, "Forest", 1);
    t.answer_yes(P0, true);
    upkeep(&mut t);
    assert!(t.in_graveyard(P0, "Herald of Leshrac"));
}
