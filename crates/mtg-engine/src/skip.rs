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

/// `StaticEffect::Custom` name of "If a player would begin an extra turn, that player
/// skips that turn instead." (Ugin's Nexus, Gerrard's Hourglass Pendant).
pub const SKIP_EXTRA_TURNS: &str = "skip extra turns";
/// `StaticEffect::Custom` name of "If an opponent would begin an extra turn, that player
/// skips that turn instead." (Stranglehold).
pub const OPPONENTS_SKIP_EXTRA_TURNS: &str = "opponents skip extra turns";

/// Whether an extra turn `p` would begin is skipped instead (CR 614.10): a replacement
/// effect of a permanent on the battlefield as the turn would begin, not when the turn was
/// created.
pub fn extra_turn_skipped(g: &Game, p: PlayerId) -> bool {
    g.statics
        .customs
        .iter()
        .any(|(_, ctl, name)| match name.as_str() {
            SKIP_EXTRA_TURNS => true,
            OPPONENTS_SKIP_EXTRA_TURNS => g.are_opponents(*ctl, p),
            _ => false,
        })
}

/// Queues an extra turn for `p` (CR 500.7) and what happens as that turn begins (see
/// `Effect::ExtraTurnWith`): queued turns are a stack, so the entry keeps its index until
/// it's taken or skipped.
pub fn queue_extra_turn(g: &mut Game, p: PlayerId, ctx: &Ctx, at_start: &Effect) {
    g.extra_turns.push(p);
    let i = g.extra_turns.len() - 1;
    g.extra_turn_actions
        .insert(i, vec![(ctx.clone(), at_start.clone())]);
}

/// What happens as the extra turn just taken off the queue begins (it was at index
/// `g.extra_turns.len()`); forgotten if the turn doesn't begin.
pub fn take_extra_turn_actions(g: &mut Game) -> Vec<(Ctx, Effect)> {
    let i = g.extra_turns.len();
    g.extra_turn_actions.remove(&i).unwrap_or_default()
}
