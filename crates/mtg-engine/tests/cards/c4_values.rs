//! cards4-referents-statics, family `values`.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Rooms with both doors unlocked (`full`) and Rooms with only the left door unlocked.
fn elevator_game(full: &[&str], half: &[&str]) -> TestGame {
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Island");
        t.library_top(P1, "Island");
    }
    for (names, both) in [(full, true), (half, false)] {
        for n in names {
            let r = t.battlefield(P0, n);
            t.g.special.rooms.unlocked.insert(r, [true, both]);
        }
    }
    t.g.dirty = true;
    // Through P0's next upkeep (the game ends there if P0 wins).
    let start = t.g.turn.number;
    t.g.run_until(10_000, |g| {
        g.player(P1).has_lost
            || (g.turn.number > start && g.turn.active == P0 && g.turn.step == Step::Draw)
    });
    t
}

#[test]
fn central_elevator_wins_with_eight_different_door_names() {
    cr!("709.5", "104.2b");
    supported("Central Elevator // Promising Stairs");
    let rooms = [
        "Central Elevator // Promising Stairs",
        "Cramped Vents // Access Maze",
        "Painter's Studio // Defaced Gallery",
        "Underwater Tunnel // Slimy Aquarium",
    ];
    let t = elevator_game(&rooms, &[]);
    assert!(t.has_lost(P1), "{}", t.dump_log());
}

#[test]
fn central_elevator_counts_names_not_doors() {
    cr!("709.5", "201.2");
    // Seven different names (Cramped Vents twice; one door of Underwater Tunnel locked).
    let t = elevator_game(
        &[
            "Central Elevator // Promising Stairs",
            "Cramped Vents // Access Maze",
            "Painter's Studio // Defaced Gallery",
        ],
        &["Underwater Tunnel // Slimy Aquarium", "Cramped Vents // Access Maze"],
    );
    assert!(!t.has_lost(P1), "{}", t.dump_log());
}
