//! Possibility Storm (hand-written, `src/cards/possibility_storm.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_spell_is_replaced_and_the_exiled_cards_go_to_the_bottom() {
    cr!("603.2");
    ruling!("Possibility Storm", "The original spell is part of the group of exiled cards put on the bottom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Possibility Storm");
    t.lands(P0, "Mountain", 1);
    // Library from the top: Island, Shock (an instant), ...
    t.library_top(P0, "Shock");
    t.library_top(P0, "Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).go();
    // Cast Shock for free, at P1.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.g.exile.iter().all(|e| !t.g.is_live(*e)));
    let lib = &t.g.player(P0).library;
    // The Bolt and the Island are at the bottom (in some order).
    let bottom: Vec<String> = lib[..2]
        .iter()
        .map(|c| t.obj(*c).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Lightning Bolt".to_string()));
    assert!(bottom.contains(&"Island".to_string()));
}
