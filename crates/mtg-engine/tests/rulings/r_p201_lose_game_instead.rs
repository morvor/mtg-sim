//! Rulings batch P201 — The Golden Throne, compiled by the same "If you would lose the
//! game, instead ..." pattern as Lich's Mirror (CR 104.3, 119.5, 614.1a, 810.8a, 810.9).

use crate::r_p116_common::{set_life, two_headed_giant};
use crate::r_s01_common::supported;
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn the_replacement_sets_the_life_total_by_gaining_or_losing_life() {
    cr!("104.3b", "614.1a", "119.5", "704.5a");
    ruling!(
        "The Golden Throne",
        "For your life total to become 1, you will gain or lose the appropriate amount of life. For example, if your life total is -4"
    );
    // (card, life total after)
    for (name, after) in [("The Golden Throne", 1)] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        set_life(&mut t, P0, -4);
        t.settle();
        assert!(!t.has_lost(P0), "{name}");
        assert!(t.g.result.is_none(), "{name}");
        assert_eq!(t.life(P0), after, "{name}");
        // It exiles itself, so it can't apply again.
        assert!(!t.g.is_live(src), "{name}");
        assert!(t.in_exile(name), "{name}");
    }
}

#[test]
fn the_replacement_applies_to_drawing_from_an_empty_library_and_poison() {
    cr!("104.3c", "704.5b", "704.5c", "614.1a");
    ruling!(
        "The Golden Throne",
        "If you would have lost the game because you tried to draw from an empty library, your life total becomes 1"
    );
    // Empty library: The Golden Throne makes P0 lose 19 life.
    for (name, after) in [("The Golden Throne", 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let lib = t.player(P0).library.clone();
        for c in lib {
            t.g.move_object(
                c,
                mtg_engine::object::Zone::Exile,
                mtg_engine::events::MoveCause::Effect,
                None,
            );
        }
        t.g.draw_cards(P0, 1);
        t.settle();
        assert!(!t.has_lost(P0), "{name}");
        assert_eq!(t.life(P0), after, "{name}");
        assert!(t.in_exile(name), "{name}");
    }
    // Ten poison counters: life becomes 1, then P0 loses at the next check.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Golden Throne");
    t.g.players[0].counters.insert(counters::POISON.into(), 10);
    t.g.check_sbas();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 1);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn the_replacement_does_nothing_on_concession() {
    cr!("104.3a");
    ruling!(
        "The Golden Throne",
        "The Golden Throne's effect does nothing if you concede the game."
    );
    for name in ["The Golden Throne"] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        t.g.perform_action(P0, Action::Concede).unwrap();
        t.settle();
        assert!(t.has_lost(P0), "{name}");
        assert_eq!(t.obj(src).zone, mtg_engine::object::Zone::Battlefield, "{name}");
    }
}

#[test]
fn two_headed_giant_team_life_total() {
    cr!("810.8a", "810.9c", "614.1a");
    ruling!(
        "The Golden Throne",
        "In a Two-Headed Giant game, The Golden Throne's ability set's the team's life total to 1"
    );
    for (name, after) in [("The Golden Throne", 1)] {
        let mut t = two_headed_giant();
        t.battlefield(P1, name);
        set_life(&mut t, P0, -2);
        set_life(&mut t, P1, -2);
        t.settle();
        assert!(!t.has_lost(P0) && !t.has_lost(P1), "{name}");
        assert_eq!(t.life(P0), after, "{name}");
        assert_eq!(t.life(P1), after, "{name}");
        assert!(t.in_exile(name), "{name}");
    }
}
