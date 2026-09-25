//! Damage sources and prevention effects (CR 609.7, 615): choosing "a source of your
//! choice", locking the objects a resolved replacement or prevention effect refers to,
//! damage that can't be prevented, prevention shields shared by simultaneous damage, and
//! "damage is prevented" events.

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::*;
use crate::object::*;
use crate::replacement::ReplEvent;
use crate::types::*;

/// Replaces references to chosen objects in a filter (targets, variables, "a source of
/// your choice") with those objects, so an effect created by a resolving spell or
/// ability keeps referring to them (CR 609.7b, 611.2c).
pub fn lock_filter(g: &Game, f: &Filter, ctx: &Ctx) -> Filter {
    match f {
        Filter::In(sel) => match &**sel {
            Sel::This => Filter::In(sel.clone()),
            _ => Filter::Objects(g.eval_sel_objects(sel, ctx)),
        },
        Filter::And(v) => Filter::And(v.iter().map(|x| lock_filter(g, x, ctx)).collect()),
        Filter::Or(v) => Filter::Or(v.iter().map(|x| lock_filter(g, x, ctx)).collect()),
        Filter::Not(x) => Filter::Not(Box::new(lock_filter(g, x, ctx))),
        other => other.clone(),
    }
}

/// Locks the filters of a replacement definition (see [`lock_filter`]).
pub fn lock_def(g: &Game, d: &ReplacementDef, ctx: &Ctx) -> ReplacementDef {
    let lf = |f: &Filter| lock_filter(g, f, ctx);
    let event = match &d.event {
        ReplacementEvent::EntersBattlefield(f) => ReplacementEvent::EntersBattlefield(lf(f)),
        ReplacementEvent::ZoneChange { filter, from, to } => ReplacementEvent::ZoneChange {
            filter: lf(filter),
            from: *from,
            to: *to,
        },
        ReplacementEvent::Dies(f) => ReplacementEvent::Dies(lf(f)),
        ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        } => ReplacementEvent::Damage {
            source: lf(source),
            to_players: to_players.clone(),
            to_objects: to_objects.as_ref().map(lf),
            combat_only: *combat_only,
        },
        ReplacementEvent::PutCounters {
            on_objects,
            on_players,
            kind,
        } => ReplacementEvent::PutCounters {
            on_objects: on_objects.as_ref().map(lf),
            on_players: on_players.clone(),
            kind: kind.clone(),
        },
        ReplacementEvent::Destroy(f) => ReplacementEvent::Destroy(lf(f)),
        other => other.clone(),
    };
    ReplacementDef {
        event,
        action: d.action.clone(),
        self_replacement: d.self_replacement,
        optional: d.optional,
    }
}

/// The objects a player may choose as a source of damage (CR 609.7a): permanents,
/// spells on the stack, objects referred to by objects on the stack, by replacement or
/// prevention effects waiting to apply, or by delayed triggered abilities waiting to
/// trigger (even if they've left the zone they were in), and face-up objects in the
/// command zone.
pub fn source_candidates(g: &Game) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = g.permanent_ids();
    let push = |o: ObjectId, out: &mut Vec<ObjectId>| {
        if !out.contains(&o) {
            out.push(o);
        }
    };
    for s in g.stack.clone() {
        let o = g.obj(s);
        if o.is_spell() {
            push(s, &mut out);
        }
        if let Some(si) = &o.stack {
            if let StackKind::Activated { source, .. } | StackKind::Triggered { source, .. } =
                &si.kind
            {
                push(*source, &mut out);
            }
            for cm in &si.chosen {
                for e in cm.targets.iter().flatten() {
                    if let Entity::Object(x) = e {
                        push(*x, &mut out);
                    }
                }
            }
        }
    }
    for r in &g.replacements {
        if let Some(s) = r.source {
            push(s, &mut out);
        }
        for o in r.objects.iter().flatten() {
            push(*o, &mut out);
        }
        if let ReplacementEvent::Damage { source, .. } = &r.def.event {
            for o in filter_objects(source) {
                push(o, &mut out);
            }
        }
    }
    for d in &g.delayed_triggers {
        if let Some(s) = d.source {
            push(s, &mut out);
        }
    }
    for c in g.command.clone() {
        if !g.obj(c).face_down {
            push(c, &mut out);
        }
    }
    out
}

fn filter_objects(f: &Filter) -> Vec<ObjectId> {
    match f {
        Filter::Objects(v) => v.clone(),
        Filter::And(v) | Filter::Or(v) => v.iter().flat_map(filter_objects).collect(),
        _ => vec![],
    }
}

/// "A source of your choice" (CR 609.7a): the source is chosen as the effect is created.
/// A source doesn't need to be able to deal damage. The choice is stored in `var`.
pub fn exec_choose_source(g: &mut Game, who: &PlayerRef, filter: &Filter, var: Var, ctx: &mut Ctx) {
    let p = g.eval_player(who, ctx).unwrap_or(ctx.controller);
    let cands: Vec<ObjectId> = source_candidates(g)
        .into_iter()
        .filter(|o| g.matches(*o, filter, ctx))
        .collect();
    let chosen = g.ask_objects(p, ctx.source, "Choose a source of damage", cands, 1, 1);
    ctx.vars
        .insert(var, chosen.into_iter().map(Entity::Object).collect());
}

/// Whether damage can't be prevented right now (CR 615.12).
pub fn damage_cant_be_prevented(g: &Game) -> bool {
    g.statics
        .restrictions
        .iter()
        .any(|(_, _, r)| matches!(r, Restriction::DamageCantBePrevented))
        || g.rule_effects
            .iter()
            .any(|e| matches!(e.restriction, Restriction::DamageCantBePrevented))
}

/// Whether a replacement action is a prevention effect (CR 615.1a).
pub fn is_prevention(a: &ReplacementAction) -> bool {
    matches!(
        a,
        ReplacementAction::Prevent
            | ReplacementAction::PreventAmount(_)
            | ReplacementAction::PreventAndThen(..)
    )
}

/// Records that a prevention effect prevented damage (CR 615.13). `key` identifies the
/// prevention effect so that events for simultaneous damage can be merged.
pub fn damage_prevented(
    g: &mut Game,
    key: u64,
    by: Option<ObjectId>,
    source: ObjectId,
    target: Entity,
    amount: u32,
) {
    if amount == 0 {
        return;
    }
    g.emit(Event::DamagePrevented {
        source,
        target,
        amount,
        by,
        key,
    });
}

/// CR 615.13: an ability that triggers when damage is prevented triggers once each time a
/// prevention effect is applied to one or more simultaneous damage events. Merges the
/// events emitted since `start` for the same prevention effect and recipient.
pub fn merge_prevention_events(g: &mut Game, start: usize) {
    if g.events.len() <= start + 1 {
        return;
    }
    let tail: Vec<Event> = g.events.split_off(start);
    let mut out: Vec<Event> = Vec::new();
    for e in tail {
        if let Event::DamagePrevented {
            target,
            amount,
            key,
            ..
        } = &e
        {
            if let Some(Event::DamagePrevented { amount: a, .. }) = out.iter_mut().find(|x| {
                matches!(x, Event::DamagePrevented { target: t2, key: k2, .. }
                    if t2 == target && k2 == key)
            }) {
                *a += *amount;
                continue;
            }
        }
        out.push(e);
    }
    g.events.extend(out);
}

/// CR 615.7: when a prevention shield would apply to damage from two or more sources
/// dealt to the same permanent or player at the same time, that player or the
/// permanent's controller chooses which damage the shield prevents (by ordering the
/// damage events the shield is applied to).
pub fn order_for_shields(g: &mut Game, events: &mut [(ObjectId, Entity, u32)], combat: bool) {
    let recipients: Vec<Entity> = {
        let mut v: Vec<Entity> = Vec::new();
        for (_, t, _) in events.iter() {
            if !v.contains(t) {
                v.push(*t);
            }
        }
        v
    };
    for r in recipients {
        let idx: Vec<usize> = (0..events.len()).filter(|i| events[*i].1 == r).collect();
        if idx.len() < 2 {
            continue;
        }
        let shield_applies = g.replacements.iter().any(|inst| {
            inst.remaining.is_some() && {
                let n = idx
                    .iter()
                    .filter(|i| {
                        let (s, t, a) = events[**i];
                        g.instance_matches(
                            inst.id,
                            &ReplEvent::Damage {
                                source: s,
                                target: t,
                                amount: a,
                                combat,
                            },
                        )
                    })
                    .count();
                n >= 2
            }
        });
        if !shield_applies {
            continue;
        }
        let chooser = match r {
            Entity::Player(p) => p,
            Entity::Object(o) => g.obj(o).controller,
        };
        let items: Vec<String> = idx
            .iter()
            .map(|i| format!("{} damage from {}", events[*i].2, g.describe(events[*i].0)))
            .collect();
        let order = g.ask_order(
            chooser,
            "Order the damage the prevention shield applies to (first is prevented first)",
            items,
        );
        let originals: Vec<(ObjectId, Entity, u32)> = idx.iter().map(|i| events[*i]).collect();
        for (k, i) in idx.iter().enumerate() {
            events[*i] = originals[order[k]];
        }
    }
}
