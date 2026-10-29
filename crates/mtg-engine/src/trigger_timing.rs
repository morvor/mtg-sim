//! When triggered abilities trigger (CR 603.2, 603.10, 608.2c).
//!
//! An ability triggers as soon as its trigger event occurs (CR 603.2), and whether an
//! event matches, and what the objects involved look like, is determined from the game
//! immediately after the event (CR 603.10; leaves-the-battlefield and the other
//! exceptions look back to immediately before it). A resolving spell or ability performs
//! its instructions one after another (CR 608.2c), so the events of one instruction are
//! checked for triggers before the next instruction happens: "Create a 1/1 token. Put
//! three +1/+1 counters on it." makes the token enter as a 1/1, and "whenever a creature
//! with power 4 or greater enters" doesn't trigger.
//!
//! Events are queued in [`Game::events`] and checked by [`Game::flush_events`]. The
//! effect interpreter calls [`Game::action_boundary`] before each instruction (and the
//! stack calls it once a spell's or ability's instructions are done), which checks the
//! events queued so far — unless an action is still in progress: an atomic effect
//! (destroying, creating tokens, moving objects, ...), a zone move, or the application of
//! replacement effects. Their nested instructions ("as this enters, choose ...", the
//! "instead" part of a replacement effect) are part of that action: checking in the
//! middle would show triggers a half-done action, and a check splits the "one or more"
//! batch (CR 603.2c) at that point.
//!
//! Every instruction, nested ones included, still ends the current batch of simultaneous
//! events ([`Game::end_event_batch`]): an "instead" instruction run for one object in the
//! middle of an event that affects several (a replacement effect applied while all
//! creatures are destroyed) starts a new batch. Keeping such an event in one batch is a
//! matter of simultaneity, not of when triggers are checked.
//!
//! An atomic action that itself consists of steps the rules perform one after another
//! (amass: create an Army, then put counters on it, CR 701.47a) calls
//! [`Game::sequential_step`] between them. Conversely, code that performs several
//! instructions as one simultaneous event (each player doing something at the same time)
//! runs them inside [`Game::atomically`], so that no trigger check happens between them.

use crate::ability::Effect;
use crate::game::Game;

/// Bookkeeping for when trigger events are checked.
#[derive(Debug, Default)]
pub struct TriggerTiming {
    /// Actions in progress whose events must be checked together: atomic effects, zone
    /// moves, replacement effects being applied.
    pub(crate) atomic: u32,
    /// Set while [`Game::flush_events`] runs: events emitted while triggers are being
    /// detected (by a triggered mana ability resolving right away, CR 605.4a) are checked
    /// once this check is done instead of in the middle of it.
    pub(crate) flushing: bool,
}

impl Clone for TriggerTiming {
    /// A copy of the game made while triggers are being detected (a sandbox copy made when
    /// a player is asked a decision then) checks its own events: it isn't in the middle of
    /// that check. The actions in progress are kept: a copy restored as a snapshot
    /// continues them.
    fn clone(&self) -> Self {
        TriggerTiming {
            atomic: self.atomic,
            flushing: false,
        }
    }
}

/// Whether an effect only sequences, chooses between, or repeats other instructions
/// (CR 608.2c) rather than performing an action itself: the instructions inside it are
/// separate actions, each checked for triggers when it's done.
pub fn is_sequencing(e: &Effect) -> bool {
    if let Effect::Seq(v) = e {
        return !happens_together(v);
    }
    matches!(
        e,
        Effect::Noop
            | Effect::If { .. }
            | Effect::May { .. }
            | Effect::PayOptional { .. }
            | Effect::ForEach { .. }
            | Effect::ForEachPlayer { .. }
            | Effect::InTurnOrder { .. }
            | Effect::AsPlayer { .. }
            | Effect::Repeat { .. }
            | Effect::RepeatProcess { .. }
            | Effect::RepeatThisProcess
            | Effect::ChooseOne { .. }
            | Effect::Store { .. }
            | Effect::StoreValue { .. }
            | Effect::Note { .. }
            | Effect::SetX { .. }
            | Effect::SelfReplace { .. }
    )
}

/// Whether a sequence is "create a [token] and a [token]": one instruction that the
/// compiler splits into one creation per kind. The tokens enter at the same time, as one
/// event (CR 603.2c, 608.2c): one batch, checked for triggers once they've all entered.
pub fn creates_tokens_together(v: &[Effect]) -> bool {
    v.len() > 1
        && v.iter().all(|e| {
            matches!(
                e,
                Effect::CreateToken { .. }
                    | Effect::CreateTokenWithPT { .. }
                    | Effect::CreateTokenCopy { .. }
                    | Effect::CreateTokenAttached { .. }
            )
        })
}

/// Whether a sequence is "~ deals 3 damage to target player and 3 damage to target
/// creature": one instruction (one "deals") that the compiler splits into one damage
/// effect per recipient. The source deals all that damage at the same time, as one event
/// (CR 603.2c, 608.2c): a "whenever [a source] deals damage" ability triggers once.
pub fn deals_damage_together(v: &[Effect]) -> bool {
    let Some(Effect::DealDamage { source, .. }) = v.first() else {
        return false;
    };
    // `Sel` has no equality; its debug form identifies it.
    let source = format!("{source:?}");
    v.len() > 1
        && v.iter().all(
            |e| matches!(e, Effect::DealDamage { source: s, .. } if format!("{s:?}") == source),
        )
}

/// The instructions of several sentences (or spell abilities), in order. Separate
/// sentences are separate instructions (CR 608.2c): "~ deals 2 damage to any target. ~
/// deals 2 damage to you." deals damage twice, while one sentence's "~ deals 3 damage to A
/// and 3 damage to B" deals it once, as one event ([`deals_damage_together`]; Blaze
/// Commando's ruling counts "the number of times the word 'deals' appears"). Such a
/// sentence stays a sequence of its own, and sentences that each deal damage from the same
/// source are kept apart, so none is merged with its neighbours into one damage event.
pub fn join_instructions(effects: Vec<Effect>) -> Effect {
    if effects.len() < 2 {
        return Effect::seq(effects);
    }
    let mut out = Vec::new();
    for e in effects {
        match e {
            Effect::Seq(v) if deals_damage_together(&v) => out.push(Effect::Seq(v)),
            e => match Effect::seq(vec![e]) {
                Effect::Seq(v) => out.extend(v),
                Effect::Noop => {}
                e => out.push(e),
            },
        }
    }
    if deals_damage_together(&out) {
        out = out.into_iter().map(|e| Effect::Seq(vec![e])).collect();
    }
    match out.len() {
        0 => Effect::Noop,
        1 => out.pop().expect("one element"),
        _ => Effect::Seq(out),
    }
}

/// Whether the elements of a sequence are one simultaneous event (see
/// [`creates_tokens_together`] and [`deals_damage_together`]).
pub fn happens_together(v: &[Effect]) -> bool {
    creates_tokens_together(v) || deals_damage_together(v)
}

impl Game {
    /// An action is complete (CR 603.2, 608.2c): its events form their own batch for
    /// "one or more" triggers, and, unless a larger action is still in progress, they're
    /// checked for triggers now, with the game as it is right after them (CR 603.10).
    pub fn action_boundary(&mut self) {
        self.end_event_batch();
        if self.timing.atomic == 0 {
            self.flush_events();
        }
    }

    /// Between the steps of an atomic action that the rules perform one after another
    /// (CR 701.47a: amass creates the Army token, then puts counters on it): the events of
    /// the steps so far are checked for triggers, if this action is the only one in
    /// progress.
    pub fn sequential_step(&mut self) {
        self.end_event_batch();
        if self.timing.atomic <= 1 {
            self.flush_events();
        }
    }

    /// Runs `f` as one atomic action: no trigger check happens inside it (see the module
    /// documentation).
    pub fn atomically<R>(&mut self, f: impl FnOnce(&mut Game) -> R) -> R {
        // Restored rather than decremented: `f` may restore a snapshot of the game.
        let depth = self.timing.atomic;
        self.timing.atomic = depth + 1;
        let r = f(self);
        self.timing.atomic = depth;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::TriggerTiming;

    /// A snapshot of the game taken in the middle of an atomic action (a spell cast while
    /// an effect resolves, rolled back when the cast turns out to be illegal) still counts
    /// the actions in progress once it's restored, so no trigger check happens in the
    /// middle of them; only the "being detected right now" flag is the original's own.
    #[test]
    fn a_copy_keeps_the_actions_in_progress_but_not_the_check_in_progress() {
        let t = TriggerTiming {
            atomic: 2,
            flushing: true,
        };
        let c = t.clone();
        assert_eq!(c.atomic, 2);
        assert!(!c.flushing);
    }
}
