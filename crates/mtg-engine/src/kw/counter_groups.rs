//! Engine support for the counter instructions of `oracle/patterns/counters_grammar_2.rs`
//! (CR 122): counters removed or moved from among a group with the player choosing how
//! many of each kind come from each ([`crate::ability::Effect::CountersFromAmong`]),
//! counters distributed among objects chosen on resolution
//! ([`crate::ability::Effect::DistributeCounters`]), counters copied from the counters a
//! triggering event put ([`crate::ability::Effect::PutEventCounters`]), and a kind of
//! counter chosen at random among those an object doesn't have.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Sel, Value, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Effect::Custom` prefix: a kind of counter chosen at random among `kinds` that the
/// source doesn't have on it; its position in the list (from 1, 0 if there's none) is
/// stored in the numeric variable. JSON of `(Var, Vec<CounterKind>)`.
pub const RANDOM_MISSING_KIND: &str = "counter groups:random missing kind:";

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

/// [`crate::ability::Effect::CountersFromAmong`]; returns how many counters were removed
/// or moved.
#[allow(clippy::too_many_arguments)]
pub fn counters_from_among(
    g: &mut Game,
    from: &Sel,
    to: Option<&Sel>,
    kind: Option<&CounterKind>,
    min: &Value,
    max: Option<&Value>,
    each_missing_kind: bool,
    ctx: &mut Ctx,
) -> u32 {
    let sources = g.resolve_sel(from, ctx);
    let dest = match to {
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
    if each_missing_kind {
        let Some(d) = dest else {
            return 0;
        };
        let have: Vec<CounterKind> = counters_on(g, d).into_iter().map(|(k, _)| k).collect();
        for from in sources {
            for (k, _) in counters_on(g, from) {
                if !have.contains(&k) && kind.is_none_or(|x| *x == k) {
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
            if kind.is_none_or(|x| *x == k) {
                pool.push((from, k, n));
            }
        }
    }
    let available: u32 = pool.iter().map(|(_, _, n)| n).sum();
    let min = g.eval_value(min, ctx).max(0) as u32;
    let max = match max {
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
        // Enough must be left for the rest to reach the minimum.
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
            g.ask_number(me, src, &prompt, lo as i64, hi as i64) as u32
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

/// [`crate::ability::Effect::DistributeCounters`]; returns how many counters were put.
#[allow(clippy::too_many_arguments)]
pub fn distribute(
    g: &mut Game,
    n: &Value,
    kind: &CounterKind,
    among: &Sel,
    min_objects: u32,
    max_objects: Option<u32>,
    up_to: bool,
    ctx: &mut Ctx,
) -> u32 {
    use crate::decision::{Answer, Decision};
    let me = ctx.controller;
    let mut total = g.eval_value(n, ctx).max(0) as u32;
    let cands: Vec<Entity> = g.resolve_sel(among, ctx);
    if total == 0 || cands.is_empty() {
        return 0;
    }
    if up_to {
        total = g.ask_number(
            me,
            ctx.source,
            "How many counters to distribute",
            0,
            total as i64,
        ) as u32;
        if total == 0 {
            return 0;
        }
    }
    // Each chosen object gets at least one counter.
    let max = max_objects
        .unwrap_or(u32::MAX)
        .min(total)
        .min(cands.len() as u32);
    let min = min_objects.max(1).min(max);
    let chosen = g.ask_entities(
        me,
        ctx.source,
        &format!("Choose where to distribute {total} {kind} counters"),
        cands,
        min,
        max,
    );
    if chosen.is_empty() {
        return 0;
    }
    let k = chosen.len() as u32;
    let default: Vec<u32> = (0..k)
        .map(|i| total / k + u32::from(i < total % k))
        .collect();
    let source = ctx.stack_obj.or(ctx.source).unwrap_or(ObjectId(0));
    let div = match g.ask(
        me,
        Decision::Divide {
            source,
            total,
            recipients: chosen.clone(),
            min_each: 1,
        },
    ) {
        Answer::Numbers(v)
            if v.len() == chosen.len()
                && v.iter().all(|x| *x >= 1)
                && v.iter().sum::<i64>() == total as i64 =>
        {
            v.into_iter().map(|x| x as u32).collect()
        }
        _ => default,
    };
    let mut placed = 0;
    for (t, n) in chosen.into_iter().zip(div) {
        placed += g.put_counters(t, kind, n, crate::event_causes::CounterPut::of(ctx));
    }
    placed
}

/// [`crate::ability::Effect::PutEventCounters`]; returns how many counters were put.
pub fn put_event_counters(g: &mut Game, to: &Sel, one_each: bool, ctx: &mut Ctx) -> u32 {
    let Some(ev) = ctx.event.clone() else {
        return 0;
    };
    let Some(kind) = ev.counter_kind else {
        return 0;
    };
    let n = if one_each { 1 } else { ev.amount.max(0) as u32 };
    let mut placed = 0;
    for t in g.resolve_sel(to, ctx) {
        placed += g.put_counters(t, &kind, n, crate::event_causes::CounterPut::of(ctx));
    }
    placed
}

/// `Value::Custom` name: the mana value of the card discarded to pay the cost of the
/// spell (or of the spell the permanent was), "the discarded card's mana value" (Dralnu's
/// Pet's kicker); 0 if none.
pub const DISCARDED_FOR_COST_MANA_VALUE: &str =
    "counter groups:mana value of the card discarded for its cost";

struct CounterGroups;

impl KeywordRules for CounterGroups {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != DISCARDED_FOR_COST_MANA_VALUE {
            return None;
        }
        let cast = g.cast_info(ctx)?;
        Some(
            cast.cost_objects
                .iter()
                .find(|o| {
                    let now = g.current(**o);
                    matches!(g.obj(now).zone, crate::object::Zone::Graveyard(_))
                        && g.obj(**o).kind == crate::object::ObjKind::Card
                })
                .map_or(0, |o| g.mana_value_of(*o) as i64),
        )
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(json) = name.strip_prefix(RANDOM_MISSING_KIND) {
            use rand::seq::SliceRandom;
            let Ok((var, kinds)) = serde_json::from_str::<(Var, Vec<CounterKind>)>(json) else {
                return true;
            };
            let holder = g.resolve_sel(&Sel::This, ctx).into_iter().next();
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
        false
    }
}

inventory::submit! { KeywordRegistration(&CounterGroups) }
