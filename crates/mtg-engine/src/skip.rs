//! Skipping steps, phases, and turns (CR 614.1b, 614.10): "skip" effects are replacement
//! effects that replace a step, phase, or turn with nothing.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// Applies static "skip" replacement effects ("Skip your draw step", "Players skip their
/// upkeep steps", "If you would begin your draw step, you may skip that step instead")
/// to a step that is about to begin. Returns true if the step is skipped. An effect that
/// also has the player take another action queues it as the first thing that happens in
/// the next step that actually occurs (CR 614.10b).
pub fn static_skip(g: &mut Game, kind: StepKind, active: PlayerId) -> bool {
    let cands: Vec<(ObjectId, PlayerId, ReplacementDef)> = g
        .statics
        .replacements
        .iter()
        .filter(|(s, c, _, _, d)| match &d.event {
            ReplacementEvent::SkipStep { step, whose } => {
                *step == kind && g.player_rel_matches(*whose, active, &Ctx::new(Some(*s), *c))
            }
            _ => false,
        })
        .map(|(s, c, _, _, d)| (*s, *c, d.clone()))
        .collect();
    for (src, ctl, d) in cands {
        if d.optional
            && !g.ask_yes_no(
                active,
                Some(src),
                &format!("Skip your {kind:?} step?"),
                false,
            )
        {
            continue;
        }
        match &d.action {
            ReplacementAction::Instead(e) | ReplacementAction::Also(e) => {
                g.step_start_actions
                    .push((Ctx::new(Some(src), ctl), (**e).clone()));
            }
            _ => {}
        }
        return true;
    }
    false
}

/// Performs actions that a skip effect scheduled as the first thing that happens during
/// the next step, phase, or turn to actually occur (CR 614.10b).
pub fn run_step_start_actions(g: &mut Game) {
    let actions = std::mem::take(&mut g.step_start_actions);
    for (mut ctx, e) in actions {
        g.exec(&e, &mut ctx);
    }
}

/// Whether a player's turn that's about to begin is skipped ("skip your next turn"). A
/// skipped turn uses up one skip effect (CR 614.10a).
pub fn consume_turn_skip(g: &mut Game, p: PlayerId) -> bool {
    consume_skip(g, p, StepKind::Turn)
}

/// Uses up one "skip your next [step/turn]" of `p` — with shared team turns, of any player
/// on `p`'s team: if an effect causes a player to skip a step, phase or turn, that
/// player's team does so (CR 805.8). Returns true if one was used up.
pub fn consume_skip(g: &mut Game, p: PlayerId, kind: StepKind) -> bool {
    let holders = if g.uses_shared_team_turns() {
        let mut v = vec![p];
        v.extend(g.team_members(p).into_iter().filter(|q| *q != p));
        v
    } else {
        vec![p]
    };
    for q in holders {
        if let Some(i) = g.player(q).skips.iter().position(|k| *k == kind) {
            g.players[q.idx()].skips.remove(i);
            return true;
        }
    }
    false
}

/// The players an effect that makes players skip something or take an extra turn applies
/// to: with shared team turns, a single effect causing more than one player on the same
/// team to add or skip the same step, phase or turn makes that team add or skip it only
/// once (CR 805.8).
pub fn once_per_team(g: &Game, players: Vec<PlayerId>) -> Vec<PlayerId> {
    if !g.uses_shared_team_turns() {
        return players;
    }
    let mut teams: Vec<u8> = Vec::new();
    players
        .into_iter()
        .filter(|p| {
            let t = g.player(*p).team;
            let first = !teams.contains(&t);
            teams.push(t);
            first
        })
        .collect()
}
