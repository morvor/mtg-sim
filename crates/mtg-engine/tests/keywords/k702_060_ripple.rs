//! CR 702.60 Ripple.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const RIPPLE: &str = "Ripple";

/// Names of the cards of `p`'s library, top first.
fn library_top_first(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

#[test]
fn ripple_casts_revealed_cards_with_the_same_name_for_free() {
    cr!("702.60", "702.60a");
    assert_supported("Surging Flame");
    let mut t = TestGame::new(2);
    // Top of the library: Surging Flame, Grizzly Bears, Surging Flame, Filler.
    let b = on_top(&mut t, P0, "Surging Flame");
    on_top(&mut t, P0, "Grizzly Bears");
    let a = on_top(&mut t, P0, "Surging Flame");
    let library = t.library_size(P0);
    t.lands(P0, "Mountain", 2);
    let flame = t.hand(P0, "Surging Flame");
    t.cast(P0, flame).target(P1).go();
    t.settle();
    assert_eq!(stack_triggers(&t, RIPPLE).len(), 1);
    // Reveal; cast both revealed Surging Flames (at P1); the new ones' ripple abilities
    // reveal nothing.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve();
    // Both were cast without paying their mana costs (no lands were left).
    assert_eq!(t.zone(a), object::Zone::Stack);
    assert_eq!(t.zone(b), object::Zone::Stack);
    assert_eq!(t.g.history.spells_cast.len(), 3);
    // The revealed cards not cast went to the bottom.
    assert_eq!(t.library_size(P0), library - 2);
    assert_eq!(t.g.player(P0).library.first().map(|c| t.g.obj(*c).chars.name.to_string()), Some("Filler".to_string()));
    let names = library_top_first(&t, P0);
    assert_eq!(names[names.len() - 2], "Grizzly Bears");
    // The cast copies have ripple too: their triggers go on the stack.
    t.settle();
    assert_eq!(stack_triggers(&t, RIPPLE).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn ripple_is_optional_and_casting_the_revealed_cards_too() {
    cr!("702.60a");
    let mut t = TestGame::new(2);
    on_top(&mut t, P0, "Surging Flame");
    let before = library_top_first(&t, P0);
    t.lands(P0, "Mountain", 2);
    let flame = t.hand(P0, "Surging Flame");
    t.cast(P0, flame).target(P1).go();
    // Don't reveal: nothing happens to the library.
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(library_top_first(&t, P0), before);
    assert_eq!(t.life(P1), 18);
    // Reveal but don't cast: the Surging Flame goes to the bottom with the others.
    t.lands(P0, "Mountain", 2);
    let flame = t.hand(P0, "Surging Flame");
    t.cast(P0, flame).target(P1).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let after = library_top_first(&t, P0);
    assert_eq!(after.len(), before.len());
    assert!(after[after.len() - 4..].contains(&"Surging Flame".to_string()));
    assert_ne!(after[0], "Surging Flame");
}

#[test]
fn ripple_reveals_the_whole_library_if_it_has_fewer_cards() {
    cr!("702.60a");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    on_top(&mut t, P0, "Grizzly Bears");
    on_top(&mut t, P0, "Surging Flame");
    t.lands(P0, "Mountain", 2);
    let flame = t.hand(P0, "Surging Flame");
    t.cast(P0, flame).target(P1).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(library_top_first(&t, P0), vec!["Grizzly Bears".to_string()]);
}

#[test]
fn each_instance_of_ripple_triggers_separately() {
    cr!("702.60b");
    ruling!(
        "Thrumming Stone",
        "both ripple abilities will trigger separately"
    );
    assert_supported("Thrumming Stone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thrumming Stone");
    t.lands(P0, "Mountain", 2);
    let flame = t.hand(P0, "Surging Flame");
    t.cast(P0, flame).target(P1).go();
    t.settle();
    assert_eq!(stack_triggers(&t, RIPPLE).len(), 2);
    // A spell without ripple of its own gets one instance from Thrumming Stone.
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(stack_triggers(&t, RIPPLE).len(), 1);
}
