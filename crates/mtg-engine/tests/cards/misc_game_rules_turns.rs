//! Extra turns and "that turn" (compiled by `oracle/patterns/misc_game_rules_turns.rs`):
//! "Take an extra turn after this one. At the beginning of that turn's end step, you lose
//! the game.", "Skip the untap step of that turn.", and "If a player/an opponent would
//! begin an extra turn, that player skips that turn instead."

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// P0 casts `spell` (paying with Islands/Mountains) in their first main phase.
fn cast_extra_turn_spell(t: &mut TestGame, spell: &str, land: &str, lands: usize) {
    t.lands(P0, land, lands);
    let s = t.hand(P0, spell);
    t.cast(P0, s).go();
    t.resolve_all();
}

#[test]
fn take_an_extra_turn_then_lose_at_its_end_step() {
    cr!("500.7", "603.7", "104.3e");
    compiles("Last Chance");
    let mut t = TestGame::new(2);
    let turn = t.g.turn.number;
    cast_extra_turn_spell(&mut t, "Last Chance", "Mountain", 2);
    // The next turn is P0's extra turn.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.turn.number, turn + 1);
    assert!(t.g.turn.extra);
    assert!(!t.has_lost(P0));
    // At the beginning of that turn's end step, P0 loses.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn a_skipped_extra_turn_never_has_its_end_step_trigger() {
    cr!("614.10", "500.7");
    compiles("Final Fortune");
    compiles("Stranglehold");
    ruling!(
        "Final Fortune",
        "If you end up skipping the extra turn that is gained, you do not lose the game."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stranglehold");
    cast_extra_turn_spell(&mut t, "Final Fortune", "Mountain", 2);
    // P0's extra turn is skipped: P1's turn is next.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.turn.extra);
    // P0's next (regular) turn comes and goes without P0 losing.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));
}

#[test]
fn only_opponents_skip_their_extra_turns() {
    cr!("614.10", "500.7");
    ruling!(
        "Stranglehold",
        "An \u{201c}extra turn\u{201d} means a turn created by a spell or ability."
    );
    let mut t = TestGame::new(2);
    // P0's own Stranglehold doesn't affect P0.
    t.battlefield(P0, "Stranglehold");
    cast_extra_turn_spell(&mut t, "Savor the Moment", "Island", 3);
    let turn = t.g.turn.number;
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.turn.extra);
    assert_eq!(t.g.turn.number, turn + 1);
    // Its opponent's regular turn isn't an extra turn, so it isn't skipped.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.turn.extra);
    assert_eq!(t.g.turn.number, turn + 2);
}

#[test]
fn extra_turns_are_skipped_only_if_the_permanent_is_there_as_they_would_begin() {
    cr!("614.10", "500.7");
    // (Its other ability, returning permanents, isn't needed here.)
    let pendant = card("Gerrard's Hourglass Pendant");
    assert!(pendant
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("extra turn")));
    ruling!(
        "Gerrard's Hourglass Pendant",
        "if Gerrard\u{2019}s Hourglass Pendant leaves the battlefield before that happens, the extra turns will be unaffected"
    );
    // With the Pendant on the battlefield, anyone's extra turn is skipped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gerrard's Hourglass Pendant");
    cast_extra_turn_spell(&mut t, "Last Chance", "Mountain", 2);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.turn.extra);
    assert!(!t.has_lost(P0));

    // If it leaves before the turn would begin, the extra turn happens.
    let mut t = TestGame::new(2);
    let pendant = t.battlefield(P0, "Gerrard's Hourglass Pendant");
    cast_extra_turn_spell(&mut t, "Last Chance", "Mountain", 2);
    t.g.destroy(pendant, None);
    t.settle();
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.turn.extra);
}

#[test]
fn the_most_recently_created_extra_turn_is_taken_first() {
    cr!("500.7");
    ruling!(
        "Final Fortune",
        "the most recently created extra turn is taken first"
    );
    // Last Chance, then Savor the Moment: the Savor turn (no untap step) comes first, and
    // the Last Chance turn (which ends with P0 losing) second.
    compiles("Savor the Moment");
    let mut t = TestGame::new(2);
    cast_extra_turn_spell(&mut t, "Last Chance", "Mountain", 2);
    t.lands(P0, "Island", 3);
    let savor = t.hand(P0, "Savor the Moment");
    t.cast(P0, savor).go();
    t.resolve_all();
    // Everything P0 had is tapped now.
    let tapped_before: Vec<_> = t
        .g
        .permanent_ids()
        .into_iter()
        .filter(|id| t.g.obj(*id).controller == P0 && t.g.obj(*id).tapped)
        .collect();
    assert_eq!(tapped_before.len(), 5);
    // First extra turn: Savor the Moment's, whose untap step is skipped.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.turn.extra);
    assert!(tapped_before.iter().all(|id| t.g.obj(*id).tapped));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    // Second extra turn: Last Chance's. Its untap step happens; P0 loses at its end step.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.turn.extra);
    assert!(tapped_before.iter().all(|id| !t.g.obj(*id).tapped));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.has_lost(P0));
}
