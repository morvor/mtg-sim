//! Relational object filters: an object's values compared with other values that may
//! mention other objects or the object itself ("with toughness greater than its power",
//! "with lesser mana value", "with power less than or equal to the number of Islands you
//! control"), extremes ("with the greatest power among creatures you control", ties
//! included), and requirements on the objects chosen together for one selection ("up to
//! four cards with different names", "any number of target creature cards with total mana
//! value 6 or less", CR 601.2c).
//!
//! [`Filter::ValueCmp`] and [`Value::Extreme`] evaluate their values with the object being
//! tested or measured in [`vars::TESTED`]. [`Filter::Together`] marks a group requirement
//! inside a selection's filter: each object matches it on its own, and whatever chooses
//! the objects checks the group with [`selection_ok`] / [`fit_selection`] (target slots
//! carry it as [`TargetSpec::together`] instead, see `target_groups.rs`).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// `ctx` with `id` as the object being tested.
fn testing(ctx: &Ctx, id: ObjectId) -> Ctx {
    let mut c = ctx.clone();
    c.vars.insert(vars::TESTED, vec![Entity::Object(id)]);
    c
}

/// [`Filter::ValueCmp`]: compares the two values with `id` as the tested object.
pub fn value_cmp(g: &Game, id: ObjectId, lhs: &Value, cmp: Cmp, rhs: &Value, ctx: &Ctx) -> bool {
    let c = testing(ctx, id);
    cmp.eval(g.eval_value(lhs, &c), g.eval_value(rhs, &c))
}

/// [`Value::Extreme`]: the greatest or least of `of` among the selected objects (0 if
/// there are none).
pub fn extreme(g: &Game, of: &Value, sel: &Sel, greatest: bool, ctx: &Ctx) -> i64 {
    let vals = g
        .eval_sel_objects(sel, ctx)
        .into_iter()
        .map(|o| g.eval_value(of, &testing(ctx, o)));
    if greatest {
        vals.max().unwrap_or(0)
    } else {
        vals.min().unwrap_or(0)
    }
}

/// The total of a value over objects (CR 601.2c: "with total mana value 6 or less").
/// Objects that left their zone count with their last known information.
pub fn total(g: &Game, stat: TotalStat, ents: &[Entity]) -> i64 {
    ents.iter()
        .filter_map(|e| e.object())
        .map(|o| {
            let obj = g.obj(o);
            match stat {
                TotalStat::ManaValue => g.mana_value_of(o) as i64,
                TotalStat::Power => obj.power() as i64,
                TotalStat::Toughness => obj.toughness() as i64,
                TotalStat::PowerAndToughness => obj.power() as i64 + obj.toughness() as i64,
            }
        })
        .sum()
}

/// The group requirements ([`Filter::Together`]) of a selection's filter.
pub fn groups_of(f: &Filter) -> Vec<TargetGroup> {
    match f {
        Filter::Together(g) => vec![g.clone()],
        Filter::And(v) => v.iter().flat_map(groups_of).collect(),
        _ => vec![],
    }
}

/// Whether the filter has a group requirement nested where no selection can check it
/// (inside "or" or "not"): such a filter isn't understood.
pub fn has_nested_group(f: &Filter) -> bool {
    fn inside(f: &Filter) -> bool {
        match f {
            Filter::Together(_) => true,
            Filter::And(v) | Filter::Or(v) => v.iter().any(inside),
            Filter::Not(x) => inside(x),
            _ => false,
        }
    }
    match f {
        Filter::And(v) => v.iter().any(has_nested_group),
        Filter::Or(v) => v.iter().any(inside),
        Filter::Not(x) => inside(x),
        _ => false,
    }
}

/// The filter without its group requirements, and the requirements.
pub fn split_groups(f: Filter) -> (Filter, Vec<TargetGroup>) {
    match f {
        Filter::Together(g) => (Filter::Any, vec![g]),
        Filter::And(v) => {
            let mut groups = Vec::new();
            let mut rest = Vec::new();
            for x in v {
                let (x, g) = split_groups(x);
                groups.extend(g);
                rest.push(x);
            }
            (Filter::and(rest), groups)
        }
        other => (other, vec![]),
    }
}

/// Whether the objects chosen together for a selection with filter `f` meet its group
/// requirements.
pub fn selection_ok(g: &Game, f: &Filter, chosen: &[Entity], ctx: &Ctx) -> bool {
    groups_of(f)
        .into_iter()
        .all(|grp| crate::target_groups::group_ok(g, &grp, chosen, ctx))
}

/// The objects chosen for a selection with filter `f` if they meet its group
/// requirements; otherwise the largest group, in choice order, of them that does,
/// completed to `min` objects from the candidates if possible.
pub fn fit_selection(
    g: &Game,
    f: &Filter,
    picked: Vec<Entity>,
    cands: &[Entity],
    min: usize,
    ctx: &Ctx,
) -> Vec<Entity> {
    if selection_ok(g, f, &picked, ctx) {
        return picked;
    }
    let mut kept: Vec<Entity> = Vec::new();
    for e in picked {
        kept.push(e);
        if !selection_ok(g, f, &kept, ctx) {
            kept.pop();
        }
    }
    for c in cands {
        if kept.len() >= min {
            break;
        }
        if kept.contains(c) {
            continue;
        }
        kept.push(*c);
        if !selection_ok(g, f, &kept, ctx) {
            kept.pop();
        }
    }
    kept
}
