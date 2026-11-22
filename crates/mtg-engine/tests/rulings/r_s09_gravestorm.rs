//! Rulings batch S09 — gravestorm (CR 702.69): "When you cast this spell, copy it for
//! each permanent that was put into a graveyard from the battlefield this turn."

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s04_common::is_spell_copy;
use crate::r_s06_common::give_control;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Copies of spells named `name` on the stack.
fn copies_on_stack(t: &TestGame, name: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| is_spell_copy(t, **id) && t.g.obj(**id).chars.name == name)
        .count()
}

#[test]
fn gravestorm_counts_every_permanent_put_into_a_graveyard_whoever_controlled_or_owned_it() {
    cr!("702.69a", "111.7");
    ruling!(
        "Follow the Bodies",
        "Gravestorm counts all permanents put into graveyards from the battlefield this turn. It doesn't matter who controlled those permanents, who owns them, or whether or not they were tokens."
    );
    supported("Follow the Bodies");
    let mut t = TestGame::new(2);
    // An opponent's creature, one of P0's tokens, a creature P0 controls but P1 owns,
    // and an opponent's land.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    let token = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, token);
    let borrowed = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, borrowed, P0);
    assert_eq!(t.obj_now(borrowed).controller, P0);
    destroy(&mut t, borrowed);
    assert_eq!(t.zone(borrowed), Zone::Graveyard(P1));
    let land = t.battlefield(P1, "Forest");
    destroy(&mut t, land);
    t.resolve_all();
    // Four copies plus the original: five Clues.
    t.lands(P0, "Island", 3);
    let ftb = t.hand(P0, "Follow the Bodies");
    t.cast(P0, ftb).go();
    t.resolve();
    assert_eq!(copies_on_stack(&t, "Follow the Bodies"), 4);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 5);
}

#[test]
fn countering_the_gravestorm_spell_doesnt_counter_its_copies() {
    cr!("702.69a", "707.10", "701.6a");
    ruling!(
        "Ominous Harvest",
        "A copy of a spell can be countered like any other spell, but it must be countered individually. Countering a spell with gravestorm won't affect the copies."
    );
    supported("Ominous Harvest");
    supported("Counterspell");
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let b = t.battlefield(P0, "Grizzly Bears");
        destroy(&mut t, b);
    }
    t.lands(P0, "Swamp", 3);
    let harvest = t.hand(P0, "Ominous Harvest");
    t.cast(P0, harvest).target(P1).go();
    // The gravestorm trigger resolves: two copies.
    t.resolve();
    assert_eq!(copies_on_stack(&t, "Ominous Harvest"), 2);
    // P1 counters the original spell.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(harvest).go();
    let hand = t.hand_size(P1);
    t.resolve();
    assert_eq!(t.zone(harvest), Zone::Graveyard(P0));
    assert_eq!(copies_on_stack(&t, "Ominous Harvest"), 2);
    // The copies still resolve: P1 draws two cards and loses 2 life.
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 2);
    assert_eq!(t.life(P1), 18);
}
