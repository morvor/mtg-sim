//! Rulings batch S05 — dredge (CR 702.52): "If you would draw a card, you may mill N cards
//! instead. If you do, return this card from your graveyard to your hand."

use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn nothing_happens_between_choosing_to_dredge_and_finishing_it() {
    cr!("702.52a", "614.6");
    ruling!(
        "Golgari Thug",
        "Once you've announced that you're applying a card's dredge ability to replace a draw, players can't take any actions until you've put that card into your hand and milled cards."
    );
    supported("Golgari Thug");
    // Golgari Thug (dredge 4) in P0's graveyard; P0 dredges instead of drawing in the draw
    // step. Every time either player is asked to act, the dredge is either not begun or
    // complete: the Thug in hand and four cards milled.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let thug = t.graveyard(P0, "Golgari Thug");
    let library = t.library_size(P0);
    let snapshot = |g: &Game| {
        let thug_in_hand = g
            .player(P0)
            .hand
            .iter()
            .any(|c| g.obj(*c).chars.name.as_str() == "Golgari Thug");
        (thug_in_hand, g.player(P0).library.len(), g.player(P0).graveyard.len())
    };
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let seen0 = watch(&mut t, P0, is_priority, snapshot);
    let seen1 = watch(&mut t, P1, is_priority, snapshot);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(thug), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(t.library_size(P0), library - 4);
    assert_eq!(t.graveyard_size(P0), 4);
    let all: Vec<(bool, usize, usize)> = seen0
        .lock()
        .unwrap()
        .iter()
        .chain(seen1.lock().unwrap().iter())
        .copied()
        .collect();
    assert!(all.contains(&(true, library - 4, 4)));
    for s in all {
        assert!(
            s == (false, library, 1) || s == (true, library - 4, 4),
            "a player acted in the middle of dredging: {s:?}"
        );
    }
}
