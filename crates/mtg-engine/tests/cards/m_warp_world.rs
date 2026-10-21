//! Warp World (hand-written, `src/cards/warp_world.rs`).

use mtg_engine::events::MoveCause;
use mtg_engine::mana::{Mana, ManaType};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn empty_library(t: &mut TestGame, p: PlayerId) {
    for c in t.g.player(p).library.clone() {
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, None);
    }
}

#[test]
fn permanents_are_shuffled_away_and_replaced() {
    cr!("701.24a");
    ruling!("Warp World", "All of these cards enter at the same time");
    ruling!("Warp World", "Each player puts all enchantment cards revealed this way onto the battlefield");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    empty_library(&mut t, P1);
    // P0 owns three permanents and has an empty library, so the same three cards are
    // revealed.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Glorious Anthem");
    // P1 owns one permanent and has an empty library.
    t.battlefield(P1, "Hill Giant");
    t.g.add_mana(P0, vec![Mana::new(ManaType::R); 7], None);
    let warp = t.hand(P0, "Warp World");
    t.cast(P0, warp).go();
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    // A new object: it went through the library.
    assert_ne!(back[0], bears);
    // Glorious Anthem came back after the creatures and applies to them.
    assert_eq!(t.pt(back[0]), (3, 3));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.g.player(P0).library.is_empty());
}
