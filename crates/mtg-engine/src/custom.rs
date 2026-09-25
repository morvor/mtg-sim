//! Named custom predicates, values, conditions, triggers, and effects — escape hatches
//! for behavior that doesn't fit the ability language. Referenced from the AST by name
//! (e.g. `Filter::Custom("...")`, `TriggerCond::Custom("chapter:1")`).

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> bool {
    let _ = (g, id, ctx);
    match name {
        _ => false,
    }
}

pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> i64 {
    let _ = (g, ctx);
    match name {
        _ => 0,
    }
}

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> bool {
    let _ = (g, ctx);
    match name {
        _ => false,
    }
}

/// Custom triggers. `chapter:N[,M]` implements Saga chapter abilities (CR 714.2c):
/// triggers when lore counters are put on the source, bringing the count from below N
/// to N or more.
pub fn custom_trigger(
    g: &Game,
    name: &str,
    src: ObjectId,
    ctl: PlayerId,
    ev: &Event,
) -> Vec<EventInfo> {
    let _ = ctl;
    if let Some(ns) = name.strip_prefix("chapter:") {
        if let Event::CountersAdded {
            target: Entity::Object(o),
            kind,
            n,
        } = ev
        {
            if *o == src && kind.as_str() == counters::LORE {
                let after = g.obj(src).counter(counters::LORE);
                let before = after.saturating_sub(*n);
                let mut out = Vec::new();
                for part in ns.split(',') {
                    if let Ok(k) = part.trim().parse::<u32>() {
                        if before < k && after >= k {
                            out.push(EventInfo {
                                object: Some(src),
                                amount: k as i32,
                                ..Default::default()
                            });
                        }
                    }
                }
                return out;
            }
        }
        return vec![];
    }
    vec![]
}

pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) {
    // "named-token:N:Name": create N tokens by name (CR 111.11).
    if let Some(spec) = name.strip_prefix("named-token:") {
        crate::tokens::create_named_tokens(g, spec, ctx);
        return;
    }
    let _ = (g, ctx);
    match name {
        _ => {}
    }
}

/// "can't have more than N [kind] counters on it" (CR 704.5r).
pub fn counter_limits(g: &Game, id: ObjectId) -> Vec<(CounterKind, u32)> {
    let _ = (g, id);
    vec![]
}
