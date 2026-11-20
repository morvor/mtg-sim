//! A card played "this way" is one played with that permission (CR 400.7h, 400.7i, 601.3;
//! `permissions::played_this_way`), never one played with another permission: neither the
//! delayed triggered ability that comes with an effect's permission (Fires of Mount Doom)
//! nor the triggered ability of a static permission's source (Galea, Kindler of Hope)
//! triggers for it. Written for survivors of the November 2026 mutation run of
//! `permissions.rs` (docs/MUTATION.md).

use mtg_engine::testing::*;
use mtg_engine::*;

/// Kills `permissions.rs:74:28` (`Some(w) => *w != 0 && u.way as i64 == *w` with `||`).
/// The way an effect's permissions are given with is never 0 (`permissions::note_way`), so
/// the mutant takes every card played with any permission as played "this way": the land
/// played with Crucible of Worlds's permission would make Fires deal 2 damage to each
/// player (18/18 instead of 20/20).
#[test]
fn fires_of_mount_doom_ignores_a_land_played_with_another_permission() {
    cr!("400.7h", "400.7i", "603.7a");
    ruling!(
        "Fires of Mount Doom",
        "but don't play the exiled card this turn using that casting permission, Fires of Mount Doom won't deal damage to each player"
    );
    let mut t = TestGame::new(2);
    let fires = t.battlefield(P0, "Fires of Mount Doom");
    t.battlefield(P0, "Crucible of Worlds");
    t.lands(P0, "Mountain", 3);
    let thopter = t.library_top(P0, "Ornithopter");
    let mountain = t.graveyard(P0, "Mountain");
    t.activate(P0, fires, 0, &[]).unwrap();
    t.resolve_all();
    // The Mountain is played from the graveyard with Crucible of Worlds's permission.
    t.play_land(P0, mountain).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // The exiled Ornithopter is cast with Fires's: the delayed ability was still waiting.
    let thopter = t.g.current(thopter);
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));
}

/// Kills `permissions.rs:75:28` and `75:50` (`None => u.way == 0 && u.source.is_some() &&
/// u.source == ctx.source` with the first or the second `&&` as `||`). A static
/// permission's use is recorded as way 0 with its source, so both mutants take a spell cast
/// with any static permission as cast "this way": Galea's ability would trigger for an
/// Equipment cast from the graveyard with Muldrotha's permission (two objects on the stack,
/// and the Equipment would enter attached to a creature).
#[test]
fn galea_ignores_an_equipment_cast_with_another_permission() {
    cr!("601.3", "603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galea, Kindler of Hope");
    t.battlefield(P0, "Muldrotha, the Gravetide");
    t.lands(P0, "Plains", 1);
    let bs = t.graveyard(P0, "Bonesplitter");
    t.cast(P0, bs).go();
    t.settle();
    // Only the spell: Galea's ability didn't trigger.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(bs));
    assert_eq!(t.obj_now(bs).attached_to, None);
}
