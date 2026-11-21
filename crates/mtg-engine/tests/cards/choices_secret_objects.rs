//! Objects chosen secretly by two players ("When Malik, Grim Manipulator enters, you and
//! target opponent each secretly choose a creature that player controls. Then those
//! choices are revealed, and that player sacrifices those creatures."): each player's
//! choice is kept (both creatures are sacrificed), secret until revealed (what other
//! players see is tested through mtg-api observations in `mtg-api/tests/hidden_info.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn malik(p0_picks_bears: bool, p1_picks_bears: bool) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let pick = |b: bool| Entity::Object(if b { bears } else { giant });
    t.answer_choose(P0, &[pick(p0_picks_bears)]);
    t.answer_choose(P1, &[pick(p1_picks_bears)]);
    let malik = t.enter(P0, "Malik, Grim Manipulator");
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
    (t, malik, bears, giant)
}

fn treasures(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn malik_the_opponent_sacrifices_both_chosen_creatures() {
    cr!("608.2c", "701.21a");
    let (t, malik, bears, giant) = malik(true, false);
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    assert!(t.in_graveyard(P1, "Grizzly Bears") && t.in_graveyard(P1, "Hill Giant"));
    // Two sacrifices by an opponent: two Treasures.
    assert_eq!(treasures(&t), 2);
    // The choices were revealed: each player's choice is public now.
    let ch = &t.g.obj(malik).choices;
    assert!(ch.secret.is_empty());
    let chose = |p: PlayerId| {
        ch.by_player
            .iter()
            .find(|(q, _)| *q == p)
            .map(|(_, c)| c.objects.clone())
    };
    assert_eq!(chose(P0), Some(vec![bears]));
    assert_eq!(chose(P1), Some(vec![giant]));
}

#[test]
fn malik_the_same_creature_chosen_twice_is_sacrificed_once() {
    cr!("608.2c", "701.21a");
    let (t, _, bears, giant) = malik(false, false);
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears));
    assert_eq!(treasures(&t), 1);
}
