//! Targets (CR 115): what a spell or ability targets (CR 115.9) and changing targets or
//! choosing new targets (CR 115.7, 115.8).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

/// The targets of a stack object, one entry per time an object or player was chosen as a
/// target (CR 115.9a).
fn target_instances(g: &Game, id: ObjectId) -> Vec<Entity> {
    g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|cm| cm.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// A target that's still where it's expected to be: an object that hasn't changed zones,
/// or a player still in the game (CR 115.9b).
fn still_there(g: &Game, e: Entity) -> bool {
    match e {
        Entity::Object(o) => g.is_live(o),
        Entity::Player(p) => g.player(p).in_game(),
    }
}

fn target_matches(
    g: &Game,
    e: Entity,
    objects: &Option<Filter>,
    players: &Option<PlayerFilter>,
    ctx: &Ctx,
) -> bool {
    still_there(g, e)
        && match e {
            Entity::Object(o) => objects.as_ref().is_some_and(|f| g.matches(o, f, ctx)),
            Entity::Player(p) => players
                .as_ref()
                .is_some_and(|f| g.player_filter_matches(f, p, ctx)),
        }
}

/// Whether the stack object `id` matches a [`TargetsFilter`] (CR 115.9).
pub fn stack_targets_match(g: &Game, id: ObjectId, tf: &TargetsFilter, ctx: &Ctx) -> bool {
    if g.obj(id).zone != Zone::Stack {
        return false;
    }
    let all = target_instances(g, id);
    match tf {
        // CR 115.9a: instances chosen, even ones that are no longer legal, each counted.
        TargetsFilter::Count(n) => all.len() as u32 == *n,
        // CR 115.9b: the current state of its targets; targets that left are ignored.
        TargetsFilter::Targets { objects, players } => all
            .iter()
            .any(|e| target_matches(g, *e, objects, players, ctx)),
        // CR 115.9c: the number of different objects or players chosen as its targets.
        TargetsFilter::Only { objects, players } => {
            let mut distinct = all.clone();
            distinct.sort();
            distinct.dedup();
            distinct.len() == 1 && target_matches(g, distinct[0], objects, players, ctx)
        }
    }
}

/// One place a target was chosen: (chosen mode index, target slot, position in slot).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Instance {
    cm: usize,
    slot: usize,
    pos: usize,
}

fn specs_for<'a>(body: &'a Body, cm: &ChosenMode) -> &'a [TargetSpec] {
    match (cm.mode, &body.modal) {
        (Some(m), Some(modal)) => modal.modes.get(m).map(|x| &x.targets[..]).unwrap_or(&[]),
        _ => &body.targets,
    }
}

fn spec_of<'a>(body: &'a Body, chosen: &[ChosenMode], i: Instance) -> Option<&'a TargetSpec> {
    let specs = specs_for(body, &chosen[i.cm]);
    specs.get(i.slot.min(specs.len().saturating_sub(1)))
}

fn entity_at(chosen: &[ChosenMode], i: Instance) -> Entity {
    chosen[i.cm].targets[i.slot][i.pos]
}

/// Whether the target at `i` is legal given the whole set of targets `chosen`: legal for
/// its slot (CR 115.2, 115.4, 115.5), different from the other targets chosen for the
/// same instance of the word "target" (CR 115.3) and from the slots it must differ from
/// ("another target").
fn legal_at(g: &Game, id: ObjectId, body: &Body, chosen: &[ChosenMode], i: Instance) -> bool {
    let Some(spec) = spec_of(body, chosen, i) else {
        return false;
    };
    let e = entity_at(chosen, i);
    if e == Entity::Object(id) {
        return false;
    }
    let cm = &chosen[i.cm];
    if cm.targets[i.slot]
        .iter()
        .enumerate()
        .any(|(k, x)| k != i.pos && *x == e)
    {
        return false;
    }
    if spec
        .distinct_from
        .iter()
        .any(|d| cm.targets.get(*d as usize).is_some_and(|v| v.contains(&e)))
    {
        return false;
    }
    let mut ctx = g.stack_ctx(id);
    ctx.targets = cm.targets.clone();
    g.is_legal_target(spec, e, &ctx, id)
}

/// Changes the targets of the spell or ability `id` as `chooser` chooses (CR 115.7).
/// With `forced`, a changed target must become that object or player. Modes and the
/// division of damage or counters are kept (CR 115.7f, 115.8). Returns true if any target
/// changed.
pub fn change_targets(
    g: &mut Game,
    chooser: PlayerId,
    id: ObjectId,
    how: TargetChange,
    forced: Option<Entity>,
) -> bool {
    if !g.is_live(id) || g.obj(id).zone != Zone::Stack {
        return false;
    }
    if g.dirty {
        g.recompute();
    }
    let body = g.stack_body(id);
    let orig: Vec<ChosenMode> = g.obj(id).stack.as_deref().unwrap().chosen.clone();
    let instances: Vec<Instance> =
        orig.iter()
            .enumerate()
            .flat_map(|(cm, m)| {
                m.targets.iter().enumerate().flat_map(move |(slot, v)| {
                    (0..v.len()).map(move |pos| Instance { cm, slot, pos })
                })
            })
            .collect();
    if instances.is_empty() {
        return false;
    }
    let legal_before: Vec<bool> = instances
        .iter()
        .map(|i| legal_at(g, id, &body, &orig, *i))
        .collect();
    let mut cur = orig.clone();
    // The new targets instance `k` may be changed to: another object or player that is
    // legal for its slot (CR 115.7a), or the forced one. Duplicates and "another target"
    // requirements are checked on the final set only (CR 115.7e).
    let alternatives = |g: &Game, cur: &[ChosenMode], k: usize| -> Vec<Entity> {
        let i = instances[k];
        let Some(spec) = spec_of(&body, cur, i) else {
            return vec![];
        };
        let mut ctx = g.stack_ctx(id);
        ctx.targets = cur[i.cm].targets.clone();
        let original = entity_at(&orig, i);
        g.legal_target_candidates(spec, &ctx, id)
            .into_iter()
            .filter(|c| *c != original && *c != Entity::Object(id))
            .filter(|c| forced.is_none_or(|f| f == *c))
            .collect()
    };
    let ask_new = |g: &mut Game,
                   cur: &[ChosenMode],
                   k: usize,
                   cands: Vec<Entity>,
                   optional: bool|
     -> Option<Entity> {
        if cands.is_empty() {
            return None;
        }
        // Default: a candidate that isn't already a target for the same word "target".
        let i = instances[k];
        let default = cands
            .iter()
            .copied()
            .find(|c| !cur[i.cm].targets[i.slot].contains(c))
            .unwrap_or(cands[0]);
        let ans = g.ask(
            chooser,
            Decision::ChooseTargets {
                source: id,
                text: format!("New target instead of {:?}", entity_at(&orig, instances[k])),
                candidates: cands.clone(),
                min: if optional { 0 } else { 1 },
                max: 1,
            },
        );
        match ans {
            Answer::Entities(v) if v.len() == 1 && cands.contains(&v[0]) => Some(v[0]),
            Answer::Entities(v) if v.is_empty() && optional => None,
            _ if optional && forced.is_none() => None,
            _ => Some(default),
        }
    };
    match how {
        TargetChange::All => {
            // CR 115.7a: each target changes to another legal target, or none change.
            for k in 0..instances.len() {
                let cands = alternatives(g, &cur, k);
                let Some(e) = ask_new(g, &cur, k, cands, false) else {
                    return false;
                };
                let i = instances[k];
                cur[i.cm].targets[i.slot][i.pos] = e;
            }
        }
        TargetChange::One => {
            // CR 115.7b: only one of the targets may be changed.
            let options: Vec<usize> = (0..instances.len())
                .filter(|k| !alternatives(g, &cur, *k).is_empty())
                .collect();
            let k = match options.len() {
                0 => return false,
                1 => options[0],
                _ => {
                    let labels = options
                        .iter()
                        .map(|k| format!("Change {:?}", entity_at(&orig, instances[*k])))
                        .collect();
                    let pick = g.ask_option(chooser, Some(id), "Change which target?", labels);
                    options[pick.min(options.len() - 1)]
                }
            };
            let cands = alternatives(g, &cur, k);
            let Some(e) = ask_new(g, &cur, k, cands, false) else {
                return false;
            };
            let i = instances[k];
            cur[i.cm].targets[i.slot][i.pos] = e;
        }
        TargetChange::Any | TargetChange::ChooseNew => {
            // CR 115.7c, 115.7d: any number of targets may be changed; the rest stay,
            // even if they're illegal.
            for k in 0..instances.len() {
                let cands = alternatives(g, &cur, k);
                if let Some(e) = ask_new(g, &cur, k, cands, true) {
                    let i = instances[k];
                    cur[i.cm].targets[i.slot][i.pos] = e;
                }
            }
        }
    }
    // CR 115.7e: only the final set of targets is evaluated.
    let final_ok = instances.iter().enumerate().all(|(j, i)| {
        let changed = entity_at(&cur, *i) != entity_at(&orig, *i);
        if changed {
            legal_at(g, id, &body, &cur, *i)
        } else {
            !legal_before[j] || legal_at(g, id, &body, &cur, *i)
        }
    });
    let new_targets: Vec<Entity> = instances
        .iter()
        .filter(|i| entity_at(&cur, **i) != entity_at(&orig, **i))
        .map(|i| entity_at(&cur, *i))
        .collect();
    if !final_ok || new_targets.is_empty() {
        return false;
    }
    let controller = g.obj(id).controller;
    if let Some(si) = g.objects[id.0 as usize].stack.as_mut() {
        si.chosen = cur;
    }
    g.log(|_| format!("the targets of {id} are changed"));
    let mut seen = Vec::new();
    for t in new_targets {
        if !seen.contains(&t) {
            seen.push(t);
            g.emit(Event::BecameTarget {
                target: t,
                by: id,
                controller,
            });
        }
    }
    true
}
