//! Engine support for the counter instructions of `oracle/patterns/counters_grammar_2.rs`
//! (CR 122): counters removed or moved from among a group of permanents with the player
//! choosing how many of each kind come from each permanent, "a counter of each kind not
//! on ~", a kind of counter chosen at random among those an object doesn't have, and
//! counters copied from the counters a triggering event put ("the same number and kind of
//! counters").

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Sel, Value, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// `Effect::Custom` prefix for a [`GroupOp`] (followed by its JSON).
pub const GROUP_OP: &str = "counter groups:op:";

/// `Effect::Custom` prefix: a kind of counter chosen at random among `kinds` that the
/// first object of `what` doesn't have on it; its position in the list (from 1, 0 if
/// there's none) is stored in the numeric variable. JSON of `(Var, Vec<CounterKind>, Sel)`.
pub const RANDOM_MISSING_KIND: &str = "counter groups:random missing kind:";

/// `Effect::Custom` prefix: counters of the kind the triggering event put are put on the
/// selected objects: as many as it put, or one (`true`). JSON of `(Sel, bool)`.
pub const SAME_AS_EVENT: &str = "counter groups:same as event:";

/// Counters removed from among the objects (or players) of `from`, or moved from them
/// onto `to` (CR 122.5): "remove any number of counters from among permanents on the
/// battlefield", "you may remove three quest counters from among permanents you control",
/// "move any number of counters from among creatures you control onto that creature",
/// "move one or more counters from ~ onto another target permanent you control", "move a
/// counter of each kind not on ~ from another target permanent you control onto ~".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupOp {
    pub from: Sel,
    /// Where they're moved (`None`: they're removed).
    pub to: Option<Sel>,
    /// Only counters of this kind (`None`: of any kinds).
    pub kind: Option<CounterKind>,
    /// How many in all: at least `min` (nothing happens if there aren't that many), at
    /// most `max` (`None`: no limit).
    pub min: Value,
    pub max: Option<Value>,
    /// "a counter of each kind not on [to]": exactly one counter of each kind the
    /// destination doesn't have (`min` and `max` are ignored).
    #[serde(default)]
    pub each_missing_kind: bool,
}

impl GroupOp {
    pub fn effect(&self) -> crate::ability::Effect {
        crate::ability::Effect::Custom(
            format!(
                "{GROUP_OP}{}",
                serde_json::to_string(self).unwrap_or_default()
            )
            .into(),
        )
    }
}

fn counters_on(g: &Game, e: Entity) -> Vec<(CounterKind, u32)> {
    let map = match e {
        Entity::Object(o) if !g.is_live(o) => return vec![],
        Entity::Object(o) => &g.obj(o).counters,
        Entity::Player(p) => &g.player(p).counters,
    };
    map.iter()
        .filter(|(_, n)| **n > 0)
        .map(|(k, n)| (k.clone(), *n))
        .collect()
}

fn entity_name(g: &Game, e: Entity) -> String {
    match e {
        Entity::Object(o) => g.obj(o).chars.name.to_string(),
        Entity::Player(p) => format!("player {}", p.0),
    }
}

/// Performs a [`GroupOp`]; returns how many counters were removed or moved.
fn perform(g: &mut Game, op: &GroupOp, ctx: &mut Ctx) -> u32 {
    let sources = g.resolve_sel(&op.from, ctx);
    let dest = match &op.to {
        Some(t) => match g.resolve_sel(t, ctx).into_iter().next() {
            Some(d) => Some(d),
            // Moved onto something that isn't there: nothing moves (CR 122.5).
            None => return 0,
        },
        None => None,
    };
    let me = ctx.controller;
    let src = ctx.source;
    let mut total = 0;
    let act = |g: &mut Game, from: Entity, k: &CounterKind, n: u32| -> u32 {
        if n == 0 {
            return 0;
        }
        match (dest, from) {
            (Some(d), Entity::Object(o)) => {
                crate::counter_rules::move_counters_by(g, o, d, k, n, src)
            }
            (Some(_), Entity::Player(_)) => 0,
            (None, f) => g.remove_counters_by(f, k, n, Some(me)),
        }
    };
    if op.each_missing_kind {
        let Some(d) = dest else {
            return 0;
        };
        let have: Vec<CounterKind> = counters_on(g, d).into_iter().map(|(k, _)| k).collect();
        for from in sources {
            for (k, _) in counters_on(g, from) {
                if !have.contains(&k) && op.kind.as_ref().is_none_or(|x| *x == k) {
                    total += act(g, from, &k, 1);
                }
            }
        }
        return total;
    }
    // What can be taken: (holder, kind, how many it has), the destination excluded.
    let mut pool: Vec<(Entity, CounterKind, u32)> = Vec::new();
    for from in sources {
        if Some(from) == dest {
            continue;
        }
        for (k, n) in counters_on(g, from) {
            if op.kind.as_ref().is_none_or(|x| *x == k) {
                pool.push((from, k, n));
            }
        }
    }
    let available: u32 = pool.iter().map(|(_, _, n)| n).sum();
    let min = g.eval_value(&op.min, ctx).max(0) as u32;
    let max = match &op.max {
        Some(v) => (g.eval_value(v, ctx).max(0) as u32).min(available),
        None => available,
    };
    if available < min || max == 0 || max < min {
        return 0;
    }
    let verb = if dest.is_some() { "move" } else { "remove" };
    let mut chosen: Vec<(Entity, CounterKind, u32)> = Vec::new();
    let mut sofar = 0u32;
    for (i, (from, k, n)) in pool.iter().enumerate() {
        let after: u32 = pool[i + 1..].iter().map(|(_, _, n)| n).sum();
        let lo = min.saturating_sub(sofar + after).min(*n);
        let hi = (*n).min(max - sofar);
        if hi == 0 {
            continue;
        }
        let pick = if lo == hi {
            lo
        } else {
            let prompt = format!(
                "How many {k} counters to {verb} from {}",
                entity_name(g, *from)
            );
            g.ask_number(me, ctx.source, &prompt, lo as i64, hi as i64) as u32
        };
        if pick > 0 {
            chosen.push((*from, k.clone(), pick));
            sofar += pick;
        }
        if sofar >= max {
            break;
        }
    }
    for (from, k, n) in chosen {
        total += act(g, from, &k, n);
    }
    total
}

struct CounterGroups;

impl KeywordRules for CounterGroups {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(json) = name.strip_prefix(GROUP_OP) {
            let Ok(op) = serde_json::from_str::<GroupOp>(json) else {
                return true;
            };
            let n = perform(g, &op, ctx);
            ctx.prev_value = n as i64;
            ctx.prev_happened = n > 0;
            return true;
        }
        if let Some(json) = name.strip_prefix(RANDOM_MISSING_KIND) {
            use rand::seq::SliceRandom;
            let Ok((var, kinds, what)) = serde_json::from_str::<(Var, Vec<CounterKind>, Sel)>(json)
            else {
                return true;
            };
            let holder = g.resolve_sel(&what, ctx).into_iter().next();
            let have: Vec<CounterKind> = holder
                .map(|h| counters_on(g, h).into_iter().map(|(k, _)| k).collect())
                .unwrap_or_default();
            let missing: Vec<usize> = (0..kinds.len())
                .filter(|i| !have.contains(&kinds[*i]))
                .collect();
            let pick = missing.choose(&mut g.rng).map_or(0, |i| *i as i64 + 1);
            ctx.nums.insert(var, pick);
            ctx.prev_happened = pick > 0;
            return true;
        }
        if let Some(json) = name.strip_prefix(SAME_AS_EVENT) {
            let Ok((to, one)) = serde_json::from_str::<(Sel, bool)>(json) else {
                return true;
            };
            let Some(ev) = ctx.event.clone() else {
                return true;
            };
            let Some(kind) = ev.counter_kind else {
                return true;
            };
            let n = if one { 1 } else { ev.amount.max(0) as u32 };
            let mut placed = 0;
            for t in g.resolve_sel(&to, ctx) {
                placed += g.put_counters(t, &kind, n, crate::event_causes::CounterPut::of(ctx));
            }
            ctx.prev_value = placed as i64;
            ctx.prev_happened = placed > 0;
            return true;
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&CounterGroups) }
