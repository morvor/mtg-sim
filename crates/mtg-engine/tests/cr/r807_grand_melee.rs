//! CR 807: the Grand Melee variant — turn markers and multiple stacks.

use super::r800_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{AttackSide, GameConfig};
use mtg_engine::multiplayer::grand_melee::{self, TurnMarker};
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
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

fn pid(i: usize) -> PlayerId {
    PlayerId(i as u8)
}

/// (number, holder, taking a turn) for each marker.
fn markers(t: &TestGame) -> Vec<(u32, PlayerId, bool)> {
    grand_melee::markers(&t.g)
        .iter()
        .map(|m: &TurnMarker| (m.number, m.holder, m.taking_turn))
        .collect()
}

fn holders(t: &TestGame) -> Vec<PlayerId> {
    markers(t).into_iter().map(|(_, h, _)| h).collect()
}

/// Advances the game until `pred` holds.
fn run(t: &mut TestGame, what: &str, pred: impl FnMut(&mtg_engine::Game) -> bool) {
    assert!(t.g.run_until(40_000, pred), "never: {what}");
}

/// Whether `p` holds a marker and is taking a turn with it.
fn taking_turn(g: &mtg_engine::Game, p: PlayerId) -> bool {
    grand_melee::markers(g)
        .iter()
        .any(|m| m.holder == p && m.taking_turn)
}

#[test]
fn grand_melee_is_free_for_all_for_many_players() {
    cr!("807.1");
    let t = gm(10);
    // Individuals: everyone else is an opponent.
    assert_eq!(t.g.opponents(P0).len(), 9);
    assert!(GameConfig {
        teams: Some(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4]),
        ..GameConfig::grand_melee()
    }
    .validate(10)
    .unwrap_err()
    .contains(&SetupError::TeamsInIndividualVariant));
    assert_eq!(GameConfig::grand_melee().validate(10), Ok(()));
}

#[test]
fn grand_melee_options_are_decided_before_play() {
    cr!("807.2", "807.2a", "807.2b", "807.2c");
    let c = GameConfig::grand_melee();
    assert_eq!(c.range_of_influence, Some(1));
    assert_eq!(c.attack_side, Some(AttackSide::Left));
    assert!(!c.attack_multiple_players && !c.deploy_creatures);
    for bad in [
        GameConfig {
            attack_multiple_players: true,
            ..GameConfig::grand_melee()
        },
        GameConfig {
            deploy_creatures: true,
            ..GameConfig::grand_melee()
        },
    ] {
        assert!(bad.validate(10).is_err());
    }
    // Range 1 and attack left: P0 can attack only P1.
    let mut t = gm(10);
    assert_eq!(t.g.players_in_range(P0), vec![P0, P1, pid(9)]);
    let a = bear(&mut t, P0);
    assert_eq!(targets_of(&attack_choices(&mut t, P0), a), vec![Entity::Player(P1)]);
}

#[test]
fn grand_melee_players_are_seated_at_random() {
    cr!("807.3");
    let seatings: std::collections::BTreeSet<Vec<usize>> = (0..8u64)
        .map(|seed| {
            mtg_engine::multiplayer::setup::new_seated(
                GameConfig {
                    seed,
                    ..GameConfig::grand_melee()
                },
                (0..10).map(|_| super::r100_common::fillers(5)).collect(),
                vec![],
            )
            .multiplayer
            .seats
        })
        .collect();
    assert!(seatings.len() > 3);
}

#[test]
fn several_players_take_turns_at_the_same_time() {
    cr!("807.4");
    let mut t = gm(10);
    // P0's and P4's turns are both under way; the game plays them side by side.
    assert_eq!(markers(&t), vec![(1, P0, true), (2, P4, true)]);
    let mut actives = std::collections::BTreeSet::new();
    for _ in 0..40 {
        t.g.advance();
        actives.insert(t.g.turn.active);
    }
    assert_eq!(actives.into_iter().collect::<Vec<_>>(), vec![P0, P4]);
}

#[test]
fn there_is_one_turn_marker_for_each_full_four_players() {
    cr!("807.4a");
    for (n, k) in [(7, 1), (8, 2), (10, 2), (12, 3), (15, 3)] {
        assert_eq!(grand_melee::markers(&gm(n).g).len(), k, "{n} players");
    }
}

#[test]
fn the_starting_player_gets_the_first_marker_and_every_fourth_player_the_next() {
    cr!("807.4b");
    let t = gm(12);
    assert_eq!(
        markers(&t),
        vec![(1, P0, true), (2, P4, true), (3, pid(8), true)]
    );
    // All of them start their turns at the same time.
    assert!(t.g.turn.number >= 1);
    // From a real start: the markers go out as the first turn begins.
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P3),
            skip_mulligans: true,
            ..GameConfig::grand_melee()
        },
        (0..10).map(|_| super::r100_common::fillers(30)).collect(),
    );
    t.g.start();
    assert_eq!(holders(&t), vec![P3, pid(7)]);
}

#[test]
fn a_player_passes_the_marker_to_their_left_after_their_turn() {
    cr!("807.4c");
    let mut t = gm(10);
    run(&mut t, "P5 takes a turn", |g| taking_turn(g, P5));
    assert!(holders(&t).contains(&P5));
    run(&mut t, "P1 takes a turn", |g| taking_turn(g, P1));
    // A holder who leaves the game before their turn begins: the player to their left
    // takes the marker immediately.
    let mut t = gm(10);
    // Slow down P4's turn so that P1 has to wait for it (CR 807.4d)...
    grand_melee::switch_to(&mut t.g, 1);
    t.g.add_extra_combat(true);
    t.g.add_extra_combat(true);
    run(&mut t, "P1 waits with the first marker", |g| {
        grand_melee::markers(g)
            .iter()
            .any(|m| m.holder == P1 && !m.taking_turn)
    });
    concede(&mut t, P1);
    assert!(holders(&t).contains(&P2), "{:?}", markers(&t));
    // A holder who leaves during their turn: the turn continues without them (CR 800.4j),
    // the players next to their seat getting priority for its stack, and the player to
    // their left takes the marker once that turn ends.
    let mut t = gm(10);
    concede(&mut t, P0);
    assert!(holders(&t).contains(&P0));
    grand_melee::switch_to(&mut t.g, 0);
    assert_eq!(grand_melee::priority_players(&t.g), vec![P1, pid(9)]);
    run(&mut t, "P1 takes the first marker's turn", |g| {
        grand_melee::markers(g)
            .iter()
            .any(|m| m.number == 1 && m.holder == P1 && m.taking_turn)
    });
    // The second marker is still on its way: it hasn't gone around the table to P1.
    assert!(holders(&t).iter().all(|h| h.idx() >= 4 || *h == P1), "{:?}", markers(&t));
}

#[test]
fn a_player_cant_begin_a_turn_with_a_marker_close_on_their_left() {
    cr!("807.4d");
    let mut t = gm(10);
    // P4's turn gets two extra combat phases, so P0's turn ends first. P1 gets the first
    // marker, but P4 (three seats to P1's left) still has the second: P1 waits.
    grand_melee::switch_to(&mut t.g, 1);
    t.g.add_extra_combat(true);
    t.g.add_extra_combat(true);
    run(&mut t, "P1 has the first marker", |g| {
        grand_melee::markers(g).iter().any(|m| m.holder == P1)
    });
    assert!(!taking_turn(&t.g, P1));
    assert!(taking_turn(&t.g, P4));
    // Once P5 (four seats to P1's left) takes the second marker, P1 begins.
    run(&mut t, "P1 takes a turn", |g| taking_turn(g, P1));
    assert!(holders(&t).contains(&P5));
}

#[test]
fn a_departure_that_reduces_the_marker_count_designates_one_for_removal() {
    cr!("807.4e");
    // Eight players, two markers (P0, P4). P6 leaves: seven players need only one. The
    // marker immediately to P6's right (P4's) is designated for removal.
    let mut t = gm(8);
    concede(&mut t, pid(6));
    let ms = grand_melee::markers(&t.g);
    assert_eq!(ms.len(), 2, "P4 is taking a turn: the marker stays until it ends");
    let p4 = ms.iter().find(|m| m.holder == P4).unwrap();
    assert_eq!(p4.removals, 1);
    assert_eq!(ms.iter().find(|m| m.holder == P0).unwrap().removals, 0);
}

#[test]
fn markers_already_designated_are_disregarded_when_counting() {
    cr!("807.4f");
    // Twelve players, three markers (P0, P4, P8).
    let mut t = gm(12);
    concede(&mut t, pid(2));
    let designated = |t: &TestGame| -> u32 {
        grand_melee::markers(&t.g).iter().map(|m| m.removals).sum()
    };
    // Eleven players need two markers: one is designated.
    assert_eq!(designated(&t), 1);
    // Ten, nine and eight players still need two: disregarding the designated marker,
    // there are two.
    for p in [3, 5, 6] {
        concede(&mut t, pid(p));
    }
    assert_eq!(designated(&t), 1);
    // Seven players need one: another is designated.
    concede(&mut t, pid(7));
    assert_eq!(designated(&t), 2);
}

#[test]
fn a_designated_marker_is_removed_rather_than_passed() {
    cr!("807.4g");
    // Twelve players, markers at P0, P4 and P8. P2 leaves (eleven players need two): the
    // marker to P2's right, P0's, is designated for removal. After P9, P10 and P1 leave,
    // P3 leaves (seven players need one): P0's marker is designated a second time.
    let mut t = gm(12);
    for p in [2, 9, 10, 1, 3] {
        concede(&mut t, pid(p));
    }
    let rm = |t: &TestGame, h: PlayerId| {
        grand_melee::markers(&t.g)
            .iter()
            .find(|m| m.holder == h)
            .map(|m| m.removals)
    };
    assert_eq!(rm(&t, P0), Some(2));
    // P0 is taking a turn: the marker is removed when it ends, not passed; the second
    // designation goes to the marker on its right, P8's.
    run(&mut t, "P0's turn ends", |g| {
        !grand_melee::markers(g).iter().any(|m| m.number == 1)
    });
    let second = grand_melee::markers(&t.g)
        .iter()
        .find(|m| m.number == 2)
        .map(|m| m.removals);
    assert_eq!(second, Some(0));
    let third = grand_melee::markers(&t.g)
        .iter()
        .find(|m| m.number == 3)
        .map(|m| m.removals);
    assert_eq!(third, Some(1));
    // It's removed as its holder's turn ends: one marker is left.
    run(&mut t, "the third marker's turn ends", |g| {
        grand_melee::markers(g).len() == 1
    });
    assert_eq!(grand_melee::markers(&t.g)[0].number, 2);
    // A designated marker whose holder isn't taking a turn is removed at once.
    let mut t = gm(12);
    // Slow P4 down so that P1 waits with the first marker.
    grand_melee::switch_to(&mut t.g, 1);
    t.g.add_extra_combat(true);
    t.g.add_extra_combat(true);
    run(&mut t, "P1 waits with the first marker", |g| {
        grand_melee::markers(g)
            .iter()
            .any(|m| m.holder == P1 && !m.taking_turn)
    });
    // P2 leaves (eleven players need two of three markers): the marker to P2's right, the
    // waiting one P1 has, is designated and removed immediately.
    concede(&mut t, P2);
    assert_eq!(grand_melee::markers(&t.g).len(), 2);
    assert!(!holders(&t).contains(&P1));
}

#[test]
fn neighbors_of_departed_players_enter_range_only_as_a_turn_begins() {
    cr!("807.4h");
    let mut t = gm(10);
    // P2 and P3 leave: P1 and P4 now sit next to each other, but aren't within each
    // other's range of influence until the next turn begins.
    run(&mut t, "a turn is under way", |g| g.turn.stage == Stage::Priority);
    concede(&mut t, P2);
    concede(&mut t, P3);
    assert!(!t.g.players_in_range(P1).contains(&P4));
    let turns = t.g.turn.number;
    run(&mut t, "another turn begins", |g| g.turn.number > turns + 1);
    assert!(t.g.players_in_range(P1).contains(&P4));
}

#[test]
fn a_marker_holder_takes_an_extra_turn_with_their_marker() {
    cr!("807.4i");
    // Twelve players (markers at P0, P4, P8). P0 casts Time Warp on themself: no marker
    // within three seats on either side, so P0 keeps the marker for the extra turn.
    let mut t = gm(12);
    t.lands(P0, "Island", 5);
    let warp = t.hand(P0, "Time Warp");
    t.cast(P0, warp).target(P0).go();
    t.resolve();
    let first = t.g.turn.number;
    grand_melee::switch_to(&mut t.g, 0);
    run(&mut t, "P0's extra turn", |g| {
        g.turn.active == P0 && g.turn.extra && g.turn.number > first
    });
    assert!(holders(&t).contains(&P0));
    // With a marker within three seats on the right, the marker passes on and the extra
    // turn comes immediately before the player's next turn.
    let mut t = gm(8);
    // P4 (markers at P0 and P4; P0 is four seats to P4's right) — move the first marker
    // closer: slow P4 down until P1 holds the first marker.
    grand_melee::switch_to(&mut t.g, 1);
    t.g.add_extra_combat(true);
    t.g.add_extra_combat(true);
    t.lands(P4, "Island", 5);
    t.g.turn.step = Step::PrecombatMain;
    let warp = t.hand(P4, "Time Warp");
    t.cast(P4, warp).target(P4).go();
    t.resolve();
    run(&mut t, "P1 holds the first marker", |g| {
        grand_melee::markers(g).iter().any(|m| m.holder == P1)
    });
    run(&mut t, "P4's turn ends", |g| {
        !grand_melee::markers(g).iter().any(|m| m.holder == P4)
    });
    // P1 (three seats to P4's right) has a marker: P4 passed theirs on.
    assert!(holders(&t).contains(&P5));
    assert!(grand_melee::markers(&t.g).len() == 2);
}

#[test]
fn an_extra_turn_without_a_marker_comes_before_the_players_next_turn() {
    cr!("807.4j");
    // P0 gives P1 (who has no marker) an extra turn. P1 takes it immediately before their
    // next turn, when the marker reaches them.
    let mut t = gm(10);
    t.lands(P0, "Island", 5);
    let warp = t.hand(P0, "Time Warp");
    t.cast(P0, warp).target(P1).go();
    t.resolve();
    assert_eq!(t.g.extra_turns, vec![P1]);
    // P1's next turns: the extra turn, then their normal turn, both with the marker.
    let mut p1_turns: Vec<bool> = Vec::new();
    let mut last = 0;
    let _ = t.g.run_until(60_000, |g| {
        if g.turn.active == P1 && g.turn.number != last {
            last = g.turn.number;
            p1_turns.push(g.turn.extra);
        }
        p1_turns.len() >= 2
    });
    assert_eq!(p1_turns, vec![true, false]);
    assert!(holders(&t).contains(&P1));
}

#[test]
fn each_turn_marker_has_its_own_stack() {
    cr!("807.5");
    let mut t = gm(10);
    // P0 casts a spell during their turn: it's on the first marker's stack only.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert_eq!(t.stack_len(), 1);
    grand_melee::switch_to(&mut t.g, 1);
    assert_eq!(t.g.turn.active, P4);
    assert_eq!(t.stack_len(), 0, "the second marker's stack is empty");
    grand_melee::switch_to(&mut t.g, 0);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(bolt), Zone::Stack);
}

#[test]
fn priority_for_a_stack_goes_to_players_near_its_marker_or_objects() {
    cr!("807.5a");
    let mut t = gm(10);
    // For the first marker's stack (P0's turn, range 1): P9, P0 and P1.
    let players = grand_melee::priority_players(&t.g);
    assert_eq!(players, vec![P0, P1, pid(9)]);
    // With a spell of P1's on it, the players within P1's range get priority too.
    let s = t.custom(P1, super::r114_common::free_instant("Quick Thought"), Zone::Hand(P1));
    t.cast(P1, s).go();
    let players = grand_melee::priority_players(&t.g);
    assert_eq!(players, vec![P0, P1, P2, pid(9)]);
    // P5 never gets priority for it.
    assert!(!grand_melee::gets_priority(&t.g, P5));
    // Passing priority goes around only those players.
    t.script.lock().unwrap().asked.clear();
    let before = t.stack_len();
    for _ in 0..8 {
        if t.stack_len() < before {
            break;
        }
        let p = t.g.turn.priority.unwrap();
        t.g.take_action(p, mtg_engine::decision::Action::Pass);
    }
    assert_eq!(t.stack_len(), before - 1);
}

#[test]
fn spells_go_on_one_stack_and_target_only_objects_on_it() {
    cr!("807.5b");
    let mut t = gm(10);
    // P0 casts a spell on the first marker's stack; P4 casts one on the second's.
    let a = t.custom(P0, super::r114_common::free_instant("First Thought"), Zone::Hand(P0));
    t.cast(P0, a).go();
    grand_melee::switch_to(&mut t.g, 1);
    t.g.turn.step = Step::PrecombatMain;
    let b = t.custom(P4, super::r114_common::free_instant("Second Thought"), Zone::Hand(P4));
    t.cast(P4, b).go();
    // A counterspell cast on the second marker's stack can target only the spell on that
    // stack.
    t.lands(P3, "Island", 2);
    let cs = t.hand(P3, "Counterspell");
    t.cast(P3, cs).try_go().unwrap();
    let offered = last_target_candidates(&t, P3);
    assert!(offered.contains(&Entity::Object(t.g.current(b))));
    assert!(!offered.contains(&Entity::Object(t.g.current(a))));
}

/// The source of the triggered ability `id` on the stack, if it is one.
fn trigger_source(g: &mtg_engine::Game, id: ObjectId) -> Option<ObjectId> {
    match g.obj(id).stack.as_ref().map(|s| &s.kind) {
        Some(mtg_engine::object::StackKind::Triggered { source, .. }) => Some(*source),
        _ => None,
    }
}

#[test]
fn a_triggered_ability_goes_on_the_stack_of_its_cause_or_a_chosen_one() {
    cr!("807.5b");
    let mut t = gm(10);
    // P2 gets priority for both stacks: P1 casts a spell on the first marker's stack and
    // P3 one on the second's.
    let feather = t.battlefield(P2, "Angel's Feather");
    let warden = t.battlefield(P2, "Soul Warden");
    let a = t.custom(P1, super::r114_common::free_instant("Quick Thought"), Zone::Hand(P1));
    t.cast(P1, a).go();
    grand_melee::switch_to(&mut t.g, 1);
    t.g.turn.step = Step::PrecombatMain;
    t.lands(P3, "Plains", 2);
    let alarm = t.hand(P3, "Raise the Alarm");
    t.script.lock().unwrap().asked.clear();
    t.cast(P3, alarm).go();
    t.settle();
    assert!(grand_melee::gets_priority(&t.g, P2));
    // The white spell on the second marker's stack caused Angel's Feather to trigger: its
    // ability goes on that stack, without a choice.
    let stack = t.g.stack.clone();
    assert_eq!(stack.len(), 2);
    assert_eq!(trigger_source(&t.g, stack[1]), Some(feather));
    let chose_stack = |t: &TestGame| {
        t.asked().iter().any(|(p, d)| {
            *p == P2
                && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("stack"))
        })
    };
    assert!(!chose_stack(&t));
    // Back on the first marker's turn, P1's creature enters: nothing on a stack caused Soul
    // Warden to trigger, so P2 chooses the stack — here the second marker's.
    grand_melee::switch_to(&mut t.g, 0);
    assert!(grand_melee::gets_priority(&t.g, P2));
    t.answer(P2, DecisionKind::Option, Answer::Index(1));
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert!(chose_stack(&t));
    assert_eq!(t.g.turn.active, P0, "back on the first marker's turn");
    assert_eq!(t.stack_len(), 1, "only P1's spell is on the first stack");
    grand_melee::switch_to(&mut t.g, 1);
    assert_eq!(t.stack_len(), 3);
    assert_eq!(trigger_source(&t.g, t.g.stack[2]), Some(warden));
}
