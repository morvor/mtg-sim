//! Skophos Maze-Warden (hand-written, `src/cards/skophos_maze_warden.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn fights_a_creature_targeted_by_your_labyrinth() {
    cr!("701.14a", "603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skophos Maze-Warden");
    let lab = t.battlefield(P0, "Labyrinth of Skophos");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    t.set_step(P1, Step::DeclareBlockers);
    t.g.combat.as_mut().unwrap().attackers.push(mtg_engine::combat::AttackerInfo {
        id: bear,
        target: Some(Entity::Player(P0)),
        original_target: Some(Entity::Player(P0)),
        declared: true,
        blocked: false,
        blockers: vec![],
        band: None,
        defending_player: Some(P0),
    });
    t.answer_yes(P0, true);
    // The Labyrinth's second activated ability.
    t.activate(P0, lab, 1, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
