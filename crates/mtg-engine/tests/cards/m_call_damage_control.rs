//! Call Damage Control (hand-written, `src/cards/call_damage_control.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn returns_the_targets_of_the_chosen_modes() {
    cr!("700.2");
    let mut t = TestGame::new(2);
    let sol = t.graveyard(P0, "Sol Ring");
    let bear = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Call Damage Control");
    t.cast(P0, s)
        .modes(&[0, 1])
        .targets(&[Entity::Object(sol)])
        .targets(&[Entity::Object(bear)])
        .go();
    t.resolve();
    assert!(t.in_hand(P0, "Sol Ring"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Forest"));
}
