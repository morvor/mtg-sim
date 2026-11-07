//! Durations ending at a step of a later turn (CR 611.2a, 500.4, 500.5):
//! "Target land becomes a Swamp until its controller's next untap step." (Orcish Farmer)
//! and "~'s base power and toughness become equal to ... until the end of your next
//! upkeep" (Halfdane).
//!
//! - [`Duration::UntilItsControllersNextStep`]: as that step of a player's turn begins,
//!   the effect stops applying to the objects that player controls (an effect on no
//!   particular objects ends as that step of its controller's turn begins).
//! - [`Duration::UntilEndOfYourNextStep`]: as that step of its controller's turn begins,
//!   an effect that existed before the step began lasts until the step ends
//!   ([`Duration::UntilEndOfThisStep`]); one created during the step waits for the next
//!   such step.

use crate::ability::{Duration, TriggerStep};
use crate::game::{Affected, Game};
use crate::types::PlayerId;

/// As a step begins (before anything happens in it).
pub fn step_began(g: &mut Game, ts: TriggerStep) {
    let actives: Vec<PlayerId> = g.active_players();
    let controls = |g: &Game, o: crate::types::ObjectId| {
        g.is_live(o) && actives.contains(&g.obj(o).controller)
    };
    let its = |d: &Duration| matches!(d, Duration::UntilItsControllersNextStep(s) if *s == ts);
    // Objects the effects apply to that the active player controls are no longer affected.
    let mut drop_effects = Vec::new();
    for i in 0..g.effects.len() {
        if !its(&g.effects[i].duration) {
            continue;
        }
        let keep = match &g.effects[i].affected {
            Affected::Objects(v) => Some(
                v.iter()
                    .copied()
                    .filter(|o| !controls(g, *o))
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        };
        match keep {
            Some(v) if v.is_empty() => drop_effects.push(g.effects[i].id),
            Some(v) => g.effects[i].affected = Affected::Objects(v),
            None if actives.contains(&g.effects[i].controller) => {
                drop_effects.push(g.effects[i].id)
            }
            None => {}
        }
    }
    g.effects.retain(|e| !drop_effects.contains(&e.id));
    let mut drop_rules = Vec::new();
    for i in 0..g.rule_effects.len() {
        if !its(&g.rule_effects[i].duration) {
            continue;
        }
        let keep = g.rule_effects[i].objects.as_ref().map(|v| {
            v.iter()
                .copied()
                .filter(|o| !controls(g, *o))
                .collect::<Vec<_>>()
        });
        match keep {
            Some(v) if v.is_empty() => drop_rules.push(g.rule_effects[i].id),
            Some(v) => g.rule_effects[i].objects = Some(v),
            None if actives.contains(&g.rule_effects[i].controller) => {
                drop_rules.push(g.rule_effects[i].id)
            }
            None => {}
        }
    }
    g.rule_effects.retain(|e| !drop_rules.contains(&e.id));
    let ends = |d: &Duration, c: PlayerId| {
        matches!(d, Duration::UntilItsControllersNextStep(s) if *s == ts) && actives.contains(&c)
    };
    g.player_effects
        .retain(|e| !ends(&e.duration, e.controller));
    g.replacements.retain(|e| !ends(&e.duration, e.controller));
    g.play_grants
        .retain(|p| !ends(&p.duration, p.duration_player()));
    // "Until the end of your next upkeep": that step has begun.
    let next = |d: &mut Duration, c: PlayerId| {
        if matches!(d, Duration::UntilEndOfYourNextStep(s) if *s == ts) && actives.contains(&c) {
            *d = Duration::UntilEndOfThisStep;
        }
    };
    for e in g.effects.iter_mut() {
        next(&mut e.duration, e.controller);
    }
    for e in g.rule_effects.iter_mut() {
        next(&mut e.duration, e.controller);
    }
    for e in g.player_effects.iter_mut() {
        next(&mut e.duration, e.controller);
    }
    for e in g.replacements.iter_mut() {
        next(&mut e.duration, e.controller);
    }
    for p in g.play_grants.iter_mut() {
        let c = p.duration_player();
        next(&mut p.duration, c);
    }
    g.dirty = true;
}

/// As a step ends: effects lasting until the end of this step end (CR 500.5).
pub fn step_ended(g: &mut Game) {
    let this = |d: &Duration| matches!(d, Duration::UntilEndOfThisStep);
    let count = |g: &Game| {
        g.effects.len()
            + g.rule_effects.len()
            + g.player_effects.len()
            + g.replacements.len()
            + g.play_grants.len()
    };
    let before = count(g);
    g.effects.retain(|e| !this(&e.duration));
    g.rule_effects.retain(|e| !this(&e.duration));
    g.player_effects.retain(|e| !this(&e.duration));
    g.replacements.retain(|e| !this(&e.duration));
    g.play_grants.retain(|p| !this(&p.duration));
    if count(g) != before {
        g.dirty = true;
    }
}
