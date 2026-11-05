//! Rulings batch P055 — ending the turn and losing the game: Ultima and Glorious End
//! (CR 724.1), Time Stop exiling what can't be countered, Stunning Reversal's replacement
//! of the next loss (CR 104.3, 614, 810.8a), and Worst Fears controlling a player
//! (CR 723).

use crate::r_p054_common::{o, pool};
use crate::r_p055_common::*;
use crate::r_s01_common::{supported, tokens, triggers_on_stack};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::player_control::decider;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

#[test]
fn ultima_destroys_then_ends_the_turn_and_its_death_triggers_never_go_on_the_stack() {
    cr!("724.1a", "724.1b", "724.1c", "724.1d", "514.1", "514.2");
    ruling!(
        "Ultima",
        "Ending the turn this way means the following things happen in order: (1) All spells and abilities on the stack are exiled. This includes any abilities that triggered as a result of destroying all artifacts and creatures as well as Ultima, though it will continue to resolve."
    );
    supported("Ultima");
    let mut t = TestGame::new(2);
    // "When Doomed Traveler dies, create a 1/1 white Spirit creature token with flying."
    t.battlefield(P0, "Doomed Traveler");
    let thopter = t.battlefield(P1, "Ornithopter");
    // Indestructible: it survives, with damage marked on it.
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(myr).go();
    t.resolve_all();
    assert_eq!(t.obj_now(myr).damage, 2);
    // Nine other cards in hand: P0 discards down to seven in the cleanup step.
    for _ in 0..9 {
        t.hand(P0, "Grizzly Bears");
    }
    t.lands(P0, "Plains", 5);
    let ultima = t.hand(P0, "Ultima");
    t.cast(P0, ultima).go();
    all_pass(&mut t, P0);
    assert!(t.in_graveyard(P0, "Doomed Traveler"));
    assert!(t.in_graveyard(P1, "Ornithopter"), "{:?}", t.zone(thopter));
    assert!(t.on_battlefield(myr));
    // Ultima was exiled, not put into the graveyard; the Traveler's ability (which
    // triggered before the turn began ending) ceased to exist.
    assert!(t.in_exile("Ultima"));
    assert!(!t.in_graveyard(P0, "Ultima"));
    assert!(t.g.stack.is_empty());
    assert!(t.g.pending_triggers.is_empty());
    let (steps, priority) = run_to_turn_of(&mut t, P1);
    // Straight to the cleanup step; no player received priority.
    assert_eq!(steps, vec![Step::Cleanup]);
    assert!(!priority);
    assert!(tokens(&t, P0).is_empty(), "no Spirit token");
    // The cleanup step happened in its entirety.
    assert_eq!(t.obj_now(myr).damage, 0);
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn ultima_puts_abilities_that_trigger_while_the_turn_ends_on_the_stack_in_the_cleanup_step() {
    cr!("724.1c", "724.1f", "514.3a", "704.5m");
    ruling!(
        "Ultima",
        "If any triggered abilities do trigger during the process of ending the turn, they're put onto the stack during the cleanup step. If this happens, players will have a chance to cast spells and activate abilities, then there will be another cleanup step before the turn finally ends."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    // "When Rancor is put into a graveyard from the battlefield, return Rancor to its
    // owner's hand." Ultima destroys the creature; Rancor is put into the graveyard by the
    // state-based action check during the process (CR 724.1c).
    t.lands(P0, "Forest", 1);
    let rancor = t.hand(P0, "Rancor");
    t.cast(P0, rancor).target(bear).go();
    t.resolve_all();
    let rancor = t.named_on_battlefield("Rancor")[0];
    assert_eq!(t.obj_now(rancor).attached_to, Some(o(bear)));
    t.lands(P0, "Plains", 5);
    let ultima = t.hand(P0, "Ultima");
    t.cast(P0, ultima).go();
    all_pass(&mut t, P0);
    assert!(t.in_graveyard(P0, "Rancor"));
    assert!(t.g.stack.is_empty());
    // In the cleanup step the ability is put onto the stack and P0 receives priority.
    let ok = t.g.run_until(50, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok, "no priority in the cleanup step");
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(triggers_on_stack(&t, "owner's hand"), 1);
    let cleanups = |t: &TestGame| {
        t.g.turn
            .step_log
            .iter()
            .filter(|s| **s == Step::Cleanup)
            .count()
    };
    assert_eq!(cleanups(&t), 1);
    let (steps, _) = run_to_turn_of(&mut t, P1);
    // Rancor returned, then another cleanup step followed before the turn ended.
    assert!(t.in_hand(P0, "Rancor"));
    assert_eq!(steps, vec![Step::Cleanup]);
}

#[test]
fn glorious_end_puts_abilities_that_trigger_while_the_turn_ends_on_the_stack_in_the_cleanup_step() {
    cr!("724.1b", "724.1f");
    ruling!(
        "Glorious End",
        "If any triggered abilities do trigger during this process, they’re put onto the stack during the cleanup step. If this happens, players will have a chance to cast spells and activate abilities, then there will be another cleanup step before the turn is over."
    );
    supported("Stonebinder's Familiar");
    let mut t = TestGame::new(2);
    // "Whenever one or more cards are put into exile during your turn, put a +1/+1 counter
    // on this creature." Glorious End is exiled as the turn ends (CR 724.1b).
    let familiar = t.battlefield(P0, "Stonebinder's Familiar");
    t.lands(P0, "Mountain", 3);
    let end = t.hand(P0, "Glorious End");
    t.cast(P0, end).go();
    all_pass(&mut t, P0);
    assert!(t.in_exile("Glorious End"));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.counters(familiar, "+1/+1"), 0);
    let ok = t.g.run_until(50, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok, "no priority in the cleanup step");
    assert_eq!(t.g.turn.priority, Some(P0));
    let (steps, _) = run_to_turn_of(&mut t, P1);
    assert_eq!(t.counters(familiar, "+1/+1"), 1);
    assert_eq!(steps, vec![Step::Cleanup]);
}

#[test]
fn next_end_step_triggers_wait_for_the_next_turns_end_step_after_glorious_end() {
    cr!("724.1e", "603.7b");
    ruling!(
        "Glorious End",
        "Any “at the beginning of the next end step” triggered abilities won’t get the chance to trigger that turn because the end step is skipped. Those abilities will trigger at the beginning of the end step of the next turn."
    );
    supported("Sneak Attack");
    let mut t = TestGame::new(2);
    // Sneak Attack: "Sacrifice the creature at the beginning of the next end step."
    let sneak = t.battlefield(P0, "Sneak Attack");
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(giant)]);
    t.activate(P0, sneak, 0, &[]).expect("Sneak Attack");
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    let end = t.hand(P0, "Glorious End");
    t.cast(P0, end).go();
    all_pass(&mut t, P0);
    run_to_turn_of(&mut t, P1);
    // The end step was skipped: the Giant survived the turn.
    assert!(t.on_battlefield(giant));
    // It's sacrificed at the beginning of the next turn's end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Glorious End's own ability waits for P0's end step.
    assert!(!t.has_lost(P0));
}

/// Glorious End's delayed "you lose the game" abilities on the stack.
fn lose_triggers(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                    if matches!(&ability.kind, mtg_engine::ability::AbilityKind::Triggered(ta)
                        if matches!(ta.body.effect, mtg_engine::ability::Effect::LoseGame { .. })))
            })
        })
        .count()
}

/// P0 casts Glorious End in their main phase and the turn ends; the game advances to the
/// beginning of P0's next end step, with Glorious End's delayed ability on the stack.
fn glorious_end_then_next_end_step(t: &mut TestGame) {
    t.lands(P0, "Mountain", 3);
    let end = t.hand(P0, "Glorious End");
    t.cast(P0, end).go();
    all_pass(t, P0);
    run_to_turn_of(t, P1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(lose_triggers(t), 1);
}

#[test]
fn glorious_ends_delayed_ability_triggers_only_once() {
    cr!("603.7c", "724.1b", "701.6b");
    ruling!(
        "Glorious End",
        "If Glorious End’s delayed triggered ability is countered, it won’t trigger again. The same is true if it’s removed from the stack in any other way (such as by a second even more Glorious End) or if it resolves and you don’t lose the game (perhaps because of the emblem of Gideon of the Trials)."
    );
    // Positive control: it resolves and P0 loses.
    let mut t = TestGame::new(2);
    glorious_end_then_next_end_step(&mut t);
    t.resolve_all();
    assert!(t.has_lost(P0));

    // Countered (Stifle): P0 doesn't lose, then or at any later end step.
    let mut t = TestGame::new(2);
    glorious_end_then_next_end_step(&mut t);
    let trigger = *t.g.stack.last().unwrap();
    t.lands(P0, "Island", 1);
    let stifle = t.hand(P0, "Stifle");
    t.cast(P0, stifle).target(trigger).go();
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert!(!t.has_lost(P0));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(lose_triggers(&t), 0);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));

    // Removed by a second Glorious End: only the second one's ability triggers at the
    // following end step.
    let mut t = TestGame::new(2);
    glorious_end_then_next_end_step(&mut t);
    t.lands(P0, "Mountain", 3);
    let second = t.hand(P0, "Glorious End");
    t.cast(P0, second).go();
    all_pass(&mut t, P0);
    assert!(t.g.stack.is_empty());
    run_to_turn_of(&mut t, P1);
    assert!(!t.has_lost(P0));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(lose_triggers(&t), 1);

    // It resolves while P0 can't lose (Gideon of the Trials's emblem with a Gideon): it
    // doesn't trigger again later, even with the emblem's condition gone.
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon of the Trials");
    t.activate(P0, gideon, 2, &[]).expect("emblem");
    t.resolve_all();
    glorious_end_then_next_end_step(&mut t);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    let gideon = t.g.current(gideon);
    t.g.sacrifice(gideon, P0);
    t.settle();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(lose_triggers(&t), 0);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));
}

#[test]
fn time_stop_exiles_spells_that_cant_be_countered_without_countering_them() {
    cr!("724.1b", "701.6a");
    ruling!(
        "Time Stop",
        "Exiling a spell or ability prevents that spell or ability from resolving, but it doesn't technically \"counter\" anything. This means that Time Stop can exile spells and abilities that \"can't be countered.\""
    );
    // "Whenever a spell you've cast is countered, draw a card."
    supported("Multani's Presence");
    // Positive control: a spell P0 cast is countered; P0 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Multani's Presence");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P1), 20);

    // Abrupt Decay can't be countered; Time Stop exiles it anyway, and nothing was
    // countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Multani's Presence");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let decay = t.hand(P0, "Abrupt Decay");
    t.cast(P0, decay).target(thopter).go();
    t.lands(P1, "Island", 6);
    let stop = t.hand(P1, "Time Stop");
    t.cast(P1, stop).go();
    let hand = t.hand_size(P0);
    all_pass(&mut t, P0);
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Abrupt Decay"));
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.hand_size(P0), hand, "nothing was countered");
}

// ---------------------------------------------------------------------------
// Stunning Reversal
// ---------------------------------------------------------------------------

/// P0 casts Stunning Reversal (it resolves) with a 20-card library.
fn reversal(t: &mut TestGame) {
    supported("Stunning Reversal");
    t.lands(P0, "Swamp", 4);
    for _ in 0..20 {
        t.library_top(P0, "Island");
    }
    let sr = t.hand(P0, "Stunning Reversal");
    t.cast(P0, sr).go();
    t.resolve_all();
}

#[test]
fn stunning_reversal_doesnt_apply_while_you_cant_lose() {
    cr!("104.3b", "101.2", "614.1a");
    ruling!(
        "Stunning Reversal",
        "If an effect says you can’t lose the game, Stunning Reversal’s effect can’t apply."
    );
    ruling!(
        "Stunning Reversal",
        "While the replacement effect it creates lasts until end of turn (or until the event it replaces), Stunning Reversal is exiled as it resolves."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Platinum Angel");
    reversal(&mut t);
    // Exiled as it resolved, its effect still waiting.
    assert!(t.in_exile("Stunning Reversal"));
    assert!(!t.in_graveyard(P0, "Stunning Reversal"));
    let hand = t.hand_size(P0);
    // Lightning Bolt: P0 at 0 life doesn't lose (Platinum Angel); the replacement isn't
    // used.
    t.g.players[P0.idx()].life = 3;
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.hand_size(P0), hand);
    // Once the Angel is gone, P0 would lose: now it applies.
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(angel).go();
    t.resolve_all();
    assert!(!t.on_battlefield(angel));
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.hand_size(P0), hand + 7);
}

#[test]
fn stunning_reversal_doesnt_stop_an_opponent_winning() {
    cr!("104.2b", "104.3", "614.1a");
    ruling!(
        "Stunning Reversal",
        "If an effect says that an opponent wins the game, Stunning Reversal’s effect doesn’t apply."
    );
    supported("Felidar Sovereign");
    let mut t = TestGame::new(2);
    // "At the beginning of your upkeep, if you have 40 or more life, you win the game."
    t.battlefield(P1, "Felidar Sovereign");
    t.g.players[P1.idx()].life = 40;
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "win the game"), 1);
    // P0 casts Stunning Reversal in response.
    t.lands(P0, "Swamp", 4);
    let sr = t.hand(P0, "Stunning Reversal");
    t.cast(P0, sr).go();
    t.resolve();
    assert!(t.in_exile("Stunning Reversal"));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t.g.player(P1).has_won);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn stunning_reversal_wins_when_every_player_would_lose_at_once() {
    cr!("104.4a", "704.5a", "614.1a");
    ruling!(
        "Stunning Reversal",
        "If each player would lose the game at the same time, but Stunning Reversal’s effect applies to you losing the game, you win the game as soon as everyone else has lost the game. This is true even if you’d lose the game immediately afterwards, perhaps because you don’t have seven cards in your library to draw or because you couldn’t gain life to raise your life total to 1."
    );
    // Flame Rift deals 4 damage to each player; both are at 4 life.
    for case in ["library", "no life gain", "control"] {
        let mut t = TestGame::new(2);
        if case != "control" {
            t.lands(P0, "Swamp", 4);
            let sr = t.hand(P0, "Stunning Reversal");
            t.cast(P0, sr).go();
            t.resolve_all();
        }
        match case {
            "library" => {
                t.g.players[P0.idx()].library.truncate(3);
            }
            "no life gain" => {
                // "Players can't gain life." (Erebos, God of the Dead)
                t.battlefield(P1, "Erebos, God of the Dead");
            }
            _ => {}
        }
        t.g.players[P0.idx()].life = 4;
        t.g.players[P1.idx()].life = 4;
        t.lands(P0, "Mountain", 2);
        let rift = t.hand(P0, "Flame Rift");
        t.cast(P0, rift).go();
        t.resolve_all();
        assert!(t.has_lost(P1), "{case}");
        if case == "control" {
            // Without the replacement both lose: a draw.
            assert!(t.has_lost(P0));
            assert!(matches!(
                t.g.result,
                Some(mtg_engine::game::GameResult::Draw)
            ));
            continue;
        }
        assert!(!t.has_lost(P0), "{case}");
        assert!(t.g.player(P0).has_won, "{case}");
        assert!(t.g.is_over());
        if case == "no life gain" {
            // P0 couldn't gain life: still at 0.
            assert_eq!(t.life(P0), 0);
        }
    }
}

#[test]
fn stunning_reversal_does_nothing_if_you_concede() {
    cr!("104.3a");
    ruling!(
        "Stunning Reversal",
        "Stunning Reversal’s effect does nothing if you concede the game. A player who concedes leaves the game."
    );
    let mut t = TestGame::new(3);
    reversal(&mut t);
    let from = t.g.turn_events.len();
    t.g.take_action(P0, Action::Concede);
    t.settle();
    // P0 lost and left the game; the replacement didn't apply: no cards drawn, no life
    // total change.
    assert!(t.has_lost(P0));
    assert!(t.g.player(P0).left_game);
    assert_eq!(t.life(P0), 20);
    assert!(!t.g.turn_events[from..].iter().any(|e| matches!(e,
        mtg_engine::events::Event::Drew { player, .. }
        | mtg_engine::events::Event::LifeGained { player, .. }
        | mtg_engine::events::Event::LifeLost { player, .. } if *player == P0)));
    assert!(!t.g.is_over());
    // Positive control: a loss by a state-based action is replaced.
    let mut t = TestGame::new(3);
    reversal(&mut t);
    let hand = t.hand_size(P0);
    t.g.players[P0.idx()].life = 2;
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.hand_size(P0), hand + 7);
}

#[test]
fn stunning_reversal_in_two_headed_giant_replaces_the_teams_loss_for_you_alone() {
    cr!("810.8a", "810.9", "614.1a");
    ruling!(
        "Stunning Reversal",
        "In a Two-Headed Giant game, if a rule or effect says that your team or your teammate loses the game, your team won’t lose the game, but only you draw seven cards and only you gain or lose life to make your team’s life total become 1."
    );
    // Teams: P0 and P1 against P2 and P3. P0 casts Stunning Reversal; then P2's Lightning
    // Bolt to P1, P0's teammate, brings the team's life total to 0.
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    supported("Stunning Reversal");
    for _ in 0..10 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Swamp", 4);
    let sr = t.hand(P0, "Stunning Reversal");
    t.cast(P0, sr).go();
    t.resolve_all();
    for p in [P0, P1] {
        t.g.players[p.idx()].life = 3;
    }
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.lands(P2, "Mountain", 1);
    let bolt = t.hand(P2, "Lightning Bolt");
    t.cast(P2, bolt).target(P1).go();
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert_eq!((t.life(P0), t.life(P1)), (1, 1));
    assert_eq!(t.hand_size(P0), h0 + 7);
    assert_eq!(t.hand_size(P1), h1);
    // Only P0 gained the life that made the team's life total 1 (P1 lost the 3).
    let life: Vec<(PlayerId, i32)> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::LifeGained { player, amount } => {
                Some((*player, *amount as i32))
            }
            mtg_engine::events::Event::LifeLost { player, amount } => {
                Some((*player, -(*amount as i32)))
            }
            _ => None,
        })
        .collect();
    assert_eq!(life, vec![(P1, -3), (P0, 1)]);
}

// ---------------------------------------------------------------------------
// Worst Fears
// ---------------------------------------------------------------------------

/// P0 casts Worst Fears targeting `target`, and it resolves.
fn worst_fears(t: &mut TestGame, target: PlayerId) {
    supported("Worst Fears");
    pool(t, P0);
    let wf = t.hand(P0, "Worst Fears");
    t.cast(P0, wf).target(target).go();
    t.resolve_all();
    assert!(t.in_exile("Worst Fears"));
}

#[test]
fn worst_fears_controls_the_next_turn_the_player_actually_takes() {
    cr!("723.1b");
    ruling!(
        "Worst Fears",
        "If the target player skips their next turn, you'll control the next turn the affected player actually takes."
    );
    let mut t = TestGame::new(2);
    pool(&mut t, P0);
    let wf = t.hand(P0, "Worst Fears");
    t.cast(P0, wf).target(P1).go();
    // In response, P1 casts Meditate: "Draw four cards. You skip your next turn."
    t.lands(P1, "Island", 3);
    let med = t.hand(P1, "Meditate");
    t.cast(P1, med).go();
    t.resolve_all();
    assert_eq!(next_turns(&mut t, 1), vec![P0]);
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
}

#[test]
fn worst_fears_controls_only_the_player() {
    cr!("723.5", "723.6", "723.5a");
    ruling!(
        "Worst Fears",
        "You only control the player. You don't control any of their permanents, spells, or abilities."
    );
    ruling!(
        "Worst Fears",
        "You can't make the affected player concede. That player may choose to concede at any time, even while you're controlling them."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    worst_fears(&mut t, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.obj_now(bear).controller, P1);
    // The priority actions P0 is offered for P1 don't include conceding; trying anyway
    // does nothing.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    t.g.advance();
    assert!(!t.has_lost(P1));
    let offered: Vec<Vec<Action>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Priority { actions } => Some(actions.clone()),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty());
    assert!(offered.iter().all(|a| !a.contains(&Action::Concede)));
    // P1 may concede.
    t.g.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
}

#[test]
fn worst_fears_controller_makes_only_legal_choices_and_master_warcraft_takes_precedence() {
    cr!("723.5", "723.7");
    ruling!(
        "Worst Fears",
        "You can't make any illegal decisions or illegal choices—you can't do anything that player couldn't do. You can't make choices or decisions for that player that aren't called for by the game rules or by any cards, permanents, spells, abilities, and so on. If an effect causes another player to make decisions that the affected player would normally make (such as Master Warcraft does), that effect takes precedence. In other words, if the affected player wouldn't make a decision, you wouldn't make that decision on their behalf."
    );
    supported("Master Warcraft");
    // An illegal choice: P0 has P1 attack with a creature with defender. It's refused.
    let mut t = TestGame::new(3);
    let fresh = t.battlefield(P1, "Wall of Wood");
    let bear = t.battlefield(P1, "Grizzly Bears");
    worst_fears(&mut t, P1);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(decider(&t.g, P1), P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (fresh, Entity::Player(P2)),
            (bear, Entity::Player(P2)),
        ]),
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P2), 20, "the illegal declaration was refused");
    assert!(!t.obj_now(fresh).tapped);
    // Master Warcraft cast by P2: P2 chooses P1's attackers, not P0.
    let mut t = TestGame::new(3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    worst_fears(&mut t, P1);
    t.advance_to(P1, Step::Upkeep);
    pool(&mut t, P2);
    let mw = t.hand(P2, "Master Warcraft");
    t.cast(P2, mw).go();
    t.resolve_all();
    t.clear_answers();
    let from = t.asked().len();
    // P0 (for P1) would have the Bears attack P2; P2 decides that no creature attacks.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P2))]),
    );
    t.answer(P2, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P1, Step::EndOfCombat);
    let attack_deciders: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::DeclareAttackers { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(attack_deciders, vec![P2]);
    assert!(!t.obj_now(bear).tapped);
    assert_eq!((t.life(P0), t.life(P2)), (20, 20));
    // Positive control: without Master Warcraft, P0's choice for P1 is the one made.
    let mut t = TestGame::new(3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    worst_fears(&mut t, P1);
    t.advance_to(P1, Step::Upkeep);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P2))]),
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P2), 18);
}
