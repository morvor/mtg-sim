//! Titania's Song (hand-written, `src/cards/titanias_song.rs`): noncreature artifacts lose
//! all abilities and are artifact creatures with P/T equal to their mana value; when it
//! leaves the battlefield the effect continues until end of turn.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is_creature()
}

#[test]
fn noncreature_artifacts_become_creatures_without_abilities() {
    cr!("613.1d", "613.1f", "613.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Titania's Song");
    let stone = t.battlefield(P1, "Mind Stone");
    t.g.recompute();
    assert!(is_creature(&t, stone));
    assert_eq!(t.obj_now(stone).chars.power, Some(2));
    assert_eq!(t.obj_now(stone).chars.toughness, Some(2));
    assert!(t.obj_now(stone).chars.abilities.is_empty());
}

#[test]
fn the_effect_continues_until_end_of_turn_after_it_leaves() {
    cr!("611.2c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let song = t.battlefield(P0, "Titania's Song");
    let stone = t.battlefield(P1, "Mind Stone");
    t.g.recompute();
    t.g.destroy(song, None);
    t.settle();
    t.g.recompute();
    assert!(!t.on_battlefield(song));
    assert!(is_creature(&t, stone));
    assert_eq!(t.obj_now(stone).chars.power, Some(2));
    assert!(t.obj_now(stone).chars.abilities.is_empty());
    // An artifact that arrives later isn't affected.
    let later = t.battlefield(P1, "Mind Stone");
    t.g.recompute();
    assert!(!is_creature(&t, later));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_creature(&t, stone));
    assert!(!t.obj_now(stone).chars.abilities.is_empty());
}
