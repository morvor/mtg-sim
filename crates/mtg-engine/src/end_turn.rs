//! Ending turns and phases (CR 724): "End the turn" (Time Stop, Sundial of the Infinite)
//! and "End the combat phase" (Mandate of Peace). Both follow a special procedure rather
//! than the normal process for resolving spells and abilities (CR 724.1, 724.2).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::*;
use crate::turn::{Stage, Step};
use smol_str::SmolStr;

/// `Effect::Custom` name of "End the turn" (CR 724.1).
pub const END_THE_TURN: &str = "end the turn";
/// `Effect::Custom` name of "End the combat phase" (CR 724.2).
pub const END_THE_COMBAT_PHASE: &str = "end the combat phase";

/// The effect "end the turn".
pub fn end_the_turn_effect() -> Effect {
    Effect::Custom(SmolStr::new(END_THE_TURN))
}

/// The effect "end the combat phase".
pub fn end_the_combat_phase_effect() -> Effect {
    Effect::Custom(SmolStr::new(END_THE_COMBAT_PHASE))
}

/// The first steps shared by ending the turn and ending the combat phase (CR 724.1a–c,
/// 724.2a–c).
fn begin_ending(g: &mut Game) {
    // (a) Abilities that triggered before this process began but haven't been put onto
    // the stack yet cease to exist.
    g.flush_events();
    g.pending_triggers.clear();
    // (b) Exile every object on the stack, including the one that's resolving. Abilities
    // and copies of spells exiled this way cease to exist.
    for id in g.stack.clone().into_iter().rev() {
        if !g.stack.contains(&id) {
            continue;
        }
        if g.obj(id).kind == ObjKind::StackAbility {
            g.remove_from_stack(id);
        } else if g.exile_object(id, None).is_none() && g.stack.contains(&id) {
            g.remove_from_stack(id);
        }
    }
    // (c) Check state-based actions (repeatedly, CR 704.3). No player gets priority and
    // no triggered abilities are put onto the stack: those that trigger now wait.
    for _ in 0..1000 {
        g.flush_events();
        if g.result.is_some() || !g.check_sbas() {
            break;
        }
    }
    g.flush_events();
}

/// Ends the turn (CR 724.1): exiles the stack, checks state-based actions without giving
/// anyone priority, ends the current phase and step, removes creatures and planeswalkers
/// from combat, and skips straight to the cleanup step (a new cleanup step if it's
/// already the cleanup step). "At the beginning of the end step" abilities don't trigger
/// because the end step is skipped (CR 724.1e). Abilities that trigger during the
/// process are put onto the stack during the cleanup step, after which another cleanup
/// step follows (CR 724.1f, 514.3a).
pub fn end_the_turn(g: &mut Game) {
    if g.turn.stage == Stage::PreGame {
        return;
    }
    g.log(|_| "the turn ends".to_string());
    begin_ending(g);
    // (d) The current phase and/or step ends.
    if g.combat.is_some() {
        crate::combat::end_combat(g);
        // The combat phase ended: "until end of combat" effects expire (CR 500.5a).
        g.expire_effects(|d| matches!(d, Duration::EndOfCombat));
    }
    g.turn.schedule = vec![Step::Cleanup];
    // A cleanup step that granted priority is followed by the new cleanup step anyway.
    g.turn.cleanup_priority = false;
    g.turn.stage = Stage::End;
    g.turn.passes = 0;
    g.dirty = true;
}

/// Ends the combat phase (CR 724.2): exiles the stack, checks state-based actions without
/// giving anyone priority, removes all creatures and planeswalkers from combat, ends
/// "until end of combat" effects, and skips straight to the next phase. "At end of
/// combat" abilities don't trigger because the end of combat step is skipped
/// (CR 724.2e); abilities that trigger during the process are put onto the stack during
/// the following phase (CR 724.2f). Outside a combat phase, nothing happens (CR 724.2g).
pub fn end_the_combat_phase(g: &mut Game) {
    if g.turn.stage == Stage::PreGame || !g.turn.step.is_combat() {
        return;
    }
    g.log(|_| "the combat phase ends".to_string());
    begin_ending(g);
    // (d) The combat phase ends: its remaining steps are skipped.
    crate::combat::end_combat(g);
    g.expire_effects(|d| matches!(d, Duration::EndOfCombat));
    if g.turn.step != Step::EndOfCombat {
        while let Some(s) = g.turn.schedule.first().copied() {
            if !s.is_combat() || s == Step::BeginningOfCombat {
                break;
            }
            g.turn.schedule.remove(0);
            if s == Step::EndOfCombat {
                break;
            }
        }
    }
    g.turn.stage = Stage::End;
    g.turn.passes = 0;
    g.dirty = true;
}

/// Performs the effects of this module.
struct EndTurnRules;

impl KeywordRules for EndTurnRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        match name {
            END_THE_TURN => end_the_turn(g),
            END_THE_COMBAT_PHASE => end_the_combat_phase(g),
            _ => return false,
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&EndTurnRules) }
