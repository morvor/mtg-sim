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
    if let Some(i) = g.player(p).skips.iter().position(|k| *k == StepKind::Turn) {
        g.players[p.idx()].skips.remove(i);
        return true;
    }
    false
}
