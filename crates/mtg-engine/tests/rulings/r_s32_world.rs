//! Rulings batch S32 — the world rule (CR 704.5k): of two or more permanents with the
//! world supertype, only the newest stays; a tie for the newest puts them all into their
//! owners' graveyards.
//!
//! The two cards with this ruling each have rules text the compiler doesn't support yet
//! (Mana Abundance's mana replacement, Problematic Volcano's left/right assignment), but
//! the ruling is about the world supertype on their type lines, which is all this test
//! relies on: neither card's text does anything in these scenarios.

use mtg_engine::ability::LibraryPosition;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn world_rule_keeps_the_newest_world_permanent_and_a_tie_loses_all() {
    cr!("704.5k", "205.4f");
    ruling!(
        "Mana Abundance",
        "all but the most recent one are put into their owners’ graveyards. In case of a tie, all are put into their owners’ graveyards."
    );
    ruling!(
        "Problematic Volcano",
        "all but the most recent one are put into their owners’ graveyards. In case of a tie, all are put into their owners’ graveyards."
    );

    // Mana Abundance was there first; Problematic Volcano, a world enchantment controlled
    // by another player, enters later: only the older world permanent goes.
    let mut t = TestGame::new(2);
    let abundance = t.battlefield(P0, "Mana Abundance");
    t.settle();
    let volcano = t.enter(P1, "Problematic Volcano");
    t.settle();
    assert!(t.on_battlefield(volcano));
    assert!(!t.on_battlefield(abundance));
    assert!(t.in_graveyard(P0, "Mana Abundance"));

    // Two world enchantments enter at the same time: a tie for the most recent one, so
    // both are put into their owners' graveyards.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Mana Abundance");
    let b = t.graveyard(P1, "Problematic Volcano");
    let moves = [(a, P0), (b, P1)]
        .iter()
        .map(|(id, p)| MoveEv {
            obj: *id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(*p),
            etb: EtbInfo {
                controller: Some(*p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
    t.g.recompute();
    assert_eq!(t.named_on_battlefield("Mana Abundance").len(), 1);
    assert_eq!(t.named_on_battlefield("Problematic Volcano").len(), 1);
    t.settle();
    assert!(t.named_on_battlefield("Mana Abundance").is_empty());
    assert!(t.named_on_battlefield("Problematic Volcano").is_empty());
    assert!(t.in_graveyard(P0, "Mana Abundance"));
    assert!(t.in_graveyard(P1, "Problematic Volcano"));
}
