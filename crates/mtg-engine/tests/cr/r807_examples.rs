//! The worked examples of CR 807 (the Grand Melee variant), played out as tests.

use super::r600_examples_common::supported;
use super::r800_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::multiplayer::grand_melee;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A Grand Melee game of `n` players in P0's first main phase, with the turn markers
/// handed out.
fn gm(n: usize) -> TestGame {
    let mut t = TestGame::with_config(
        n,
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::grand_melee()
        },
    );
    grand_melee::ensure(&mut t.g);
    t
}

#[test]
fn sixteen_players_have_four_turn_markers_and_fifteen_have_three() {
    cr!("807.4a", "807.4b");
    cr_example!("807.4a", "sixteen players has four turn markers");
    let t = gm(16);
    let holders: Vec<PlayerId> = grand_melee::markers(&t.g).iter().map(|m| m.holder).collect();
    assert_eq!(holders, vec![P0, P4, P8, PlayerId(12)]);
    let t = gm(15);
    assert_eq!(grand_melee::markers(&t.g).len(), 3);
}

#[test]
fn time_walk_waits_until_alex_next_receives_a_turn_marker() {
    cr!("807.4j", "807.4e", "807.4g", "500.7");
    cr_example!("807.4j", "Time Walk");
    supported("Time Walk");
    // Eight players: turn markers at Alex (P0) and P4.
    let alex = P0;
    let mut t = gm(8);
    t.g.turn.step = Step::PrecombatMain;
    t.lands(alex, "Island", 2);
    let walk = t.hand(alex, "Time Walk");
    t.cast(alex, walk).go();
    t.resolve();
    assert_eq!(t.g.extra_turns, vec![alex]);
    // The player to Alex's left leaves: seven players need only one marker, and Alex's
    // (immediately to P1's right) is designated for removal.
    concede(&mut t, P1);
    let mine = grand_melee::markers(&t.g)
        .iter()
        .find(|m| m.holder == alex)
        .map(|m| m.removals);
    assert_eq!(mine, Some(1));
    // Record the turns that begin from now on until Alex has taken two.
    let mut turns: Vec<(PlayerId, bool)> = Vec::new();
    // (Turns of different markers are played side by side: a turn is recorded as its
    // untap step begins.)
    let mut seen: std::collections::BTreeSet<u32> = grand_melee::markers(&t.g)
        .iter()
        .filter_map(|m| m.ctx.as_ref().map(|c| c.turn.number))
        .chain([t.g.turn.number])
        .collect();
    let mut alex_marker_gone = false;
    let ok = t.g.run_until(80_000, |g| {
        if !alex_marker_gone && !grand_melee::markers(g).iter().any(|m| m.holder == alex) {
            alex_marker_gone = true;
        }
        if g.turn.step == Step::Untap && seen.insert(g.turn.number) {
            turns.push((g.turn.active, g.turn.extra));
        }
        turns.iter().filter(|(p, _)| *p == alex).count() >= 2
    });
    assert!(ok, "{turns:?}");
    assert!(alex_marker_gone, "Alex's marker was removed after his turn");
    assert_eq!(grand_melee::markers(&t.g).len(), 1);
    // The remaining marker went around from P4: P5, P6 and P7 took their turns before
    // Alex took the extra turn, just before his normal turn.
    let alex_turns: Vec<bool> = turns
        .iter()
        .filter(|(p, _)| *p == alex)
        .map(|(_, extra)| *extra)
        .collect();
    assert_eq!(alex_turns, vec![true, false]);
    let before: Vec<PlayerId> = turns
        .iter()
        .take_while(|(p, _)| *p != alex)
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(before, vec![P5, P6, PlayerId(7)], "{turns:?}");
}
