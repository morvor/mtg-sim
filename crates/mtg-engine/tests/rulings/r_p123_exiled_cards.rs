//! Rulings batch P123 — Pull from Eternity and Riftsweeper move a face-up exiled card out
//! of exile: it becomes a new object (CR 400.7), so a card haunting a creature stops
//! haunting it (CR 702.55c) and a suspended card loses its time counters and isn't
//! suspended any more (CR 702.62b); face-down exiled cards can't be targeted (CR 406.3).

use crate::r_p123_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's Blind Hunter dies and haunts P1's Grizzly Bears; returns (the exiled Hunter, the
/// Bears).
fn haunting(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Blind Hunter");
    let hunter = t.battlefield(P0, "Blind Hunter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(t, hunter);
    t.resolve_all();
    let exiled = t.g.current(hunter);
    assert_eq!(t.zone(exiled), Zone::Exile);
    (exiled, bears)
}

/// The haunted Bears die: P0's haunted-creature-dies trigger (if any) targets P1.
fn bears_die(t: &mut TestGame, bears: ObjectId) {
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(t, bears);
    t.resolve_all();
}

/// Riftsweeper enters under P0's control targeting the exiled card `card`.
fn riftsweeper(t: &mut TestGame, card: ObjectId) {
    supported("Riftsweeper");
    t.answer_targets(P0, &[obj(card)]);
    t.enter(P0, "Riftsweeper");
    t.resolve_all();
}

#[test]
fn a_haunting_card_leaving_exile_stops_haunting() {
    cr!("702.55c", "400.7");
    ruling!(
        "Pull from Eternity",
        "If Pull from Eternity affects a card that’s haunting a creature, the haunt effect ends."
    );
    ruling!(
        "Riftsweeper",
        "If Riftsweeper affects a card that’s haunting a creature, the haunt effect ends."
    );
    supported("Pull from Eternity");
    // Without either: the haunted creature dying triggers the haunt card.
    let mut t = TestGame::new(2);
    let (_, bears) = haunting(&mut t);
    bears_die(&mut t, bears);
    assert_eq!(t.life(P1), 18);
    // Pull from Eternity.
    let mut t = TestGame::new(2);
    let (hunter, bears) = haunting(&mut t);
    cast_resolve_at(&mut t, P0, "Pull from Eternity", &[obj(hunter)]);
    assert!(t.in_graveyard(P0, "Blind Hunter"));
    bears_die(&mut t, bears);
    assert_eq!(t.life(P1), 20);
    // Riftsweeper.
    let mut t = TestGame::new(2);
    let (hunter, bears) = haunting(&mut t);
    riftsweeper(&mut t, hunter);
    assert_eq!(t.zone(hunter), Zone::Library(P0));
    bears_die(&mut t, bears);
    assert_eq!(t.life(P1), 20);
}

/// P0 suspends Rift Bolt; returns the suspended card in exile (one time counter).
fn suspended_bolt(t: &mut TestGame) -> ObjectId {
    use mtg_engine::decision::{Action, SpecialAction};
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Suspend { card: bolt }))
        .expect("suspended");
    let exiled = t.g.current(bolt);
    assert_eq!(t.counters(exiled, "time"), 1);
    exiled
}

#[test]
fn a_suspended_card_leaving_exile_is_no_longer_suspended() {
    cr!("702.62b", "400.7");
    ruling!(
        "Pull from Eternity",
        "If Pull from Eternity affects a suspended card, the card loses its time counters and is no longer suspended. (This doesn’t trigger the last ability of suspend.)"
    );
    ruling!(
        "Riftsweeper",
        "If Riftsweeper affects a suspended card, the card loses its time counters and is no longer suspended."
    );
    let mut t = TestGame::new(2);
    let bolt = suspended_bolt(&mut t);
    cast_resolve_at(&mut t, P0, "Pull from Eternity", &[obj(bolt)]);
    let gy = t.g.current(bolt);
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
    assert_eq!(t.counters(gy, "time"), 0);
    assert_eq!(t.stack_len(), 0, "it isn't cast");
    // No upkeep trigger from it any more.
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    let mut t = TestGame::new(2);
    let bolt = suspended_bolt(&mut t);
    riftsweeper(&mut t, bolt);
    let lib = t.g.current(bolt);
    assert_eq!(t.zone(lib), Zone::Library(P0));
    assert_eq!(t.counters(lib, "time"), 0);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn riftsweeper_any_exiled_card_whoever_owns_it() {
    cr!("115.1", "406.3");
    ruling!(
        "Riftsweeper",
        "It doesn’t matter why the card was exiled or who owns it."
    );
    let mut t = TestGame::new(2);
    let theirs = t.exile(P1, "Lightning Bolt");
    let lib = t.library_size(P1);
    riftsweeper(&mut t, theirs);
    assert_eq!(t.zone(theirs), Zone::Library(P1), "its owner's library");
    assert_eq!(t.library_size(P1), lib + 1);
}

#[test]
fn pull_from_eternity_targets_only_face_up_exiled_cards() {
    cr!("406.3", "115.1");
    ruling!(
        "Pull from Eternity",
        "Pull from Eternity targets a card in the exile zone. A card is only face down in the exile zone if the effect that put it there specifically says so."
    );
    let mut t = TestGame::new(2);
    let up = t.exile(P1, "Lightning Bolt");
    let down = t.exile(P1, "Shock");
    t.g.objects[down.0 as usize].face_down = true;
    // Exiled by an effect that doesn't say face down (Swords to Plowshares): face up.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve_at(&mut t, P0, "Swords to Plowshares", &[obj(bears)]);
    let exiled_bears = t.g.current(bears);
    assert!(!t.g.obj(exiled_bears).face_down);
    let from = t.asked().len();
    cast_resolve_at(&mut t, P0, "Pull from Eternity", &[obj(up)]);
    let candidates = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(candidates[0].contains(&obj(exiled_bears)));
    assert!(candidates[0].contains(&obj(up)));
    assert!(!candidates[0].contains(&obj(down)));
    assert_eq!(t.zone(up), Zone::Graveyard(P1));
}
