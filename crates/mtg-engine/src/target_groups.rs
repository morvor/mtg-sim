//! Requirements on the targets of one instance of the word "target" taken together
//! (`TargetSpec::together`): "up to two target cards from a single graveyard", "two
//! target cards from an opponent's graveyard" (one opponent's graveyard), "two target
//! creature cards that share a creature type".
//!
//! The requirement is part of what makes the chosen targets legal: it's checked as
//! targets are chosen (CR 601.2c) — a spell or ability can't be put on the stack unless
//! a group of the required size exists — and again as the spell or ability resolves
//! (CR 608.2b). On resolution, a target that has left its zone is illegal but is still
//! compared with the others using its last known information: of two cards that share a
//! creature type, the one still in the graveyard is returned as long as it has a creature
//! type the other had as it left.

use crate::ability::TargetGroup;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Characteristics;
use crate::types::*;

fn creature_types(c: &Characteristics) -> impl Iterator<Item = &Subtype> {
    c.subtypes.iter().filter(|s| is_creature_type(s))
}

/// Whether these targets, taken together, have the relationship. Objects that left their
/// zone are compared using their last known information. `ctx` gives X for
/// [`TargetGroup::TotalAtMostX`].
pub fn group_ok(g: &Game, group: &TargetGroup, targets: &[Entity], ctx: &Ctx) -> bool {
    let objs: Vec<&crate::object::GameObject> = targets
        .iter()
        .filter_map(|e| e.object())
        .map(|o| g.obj(o))
        .collect();
    // Totals (CR 601.2c: "with total mana value 6 or less") limit even a single object.
    match group {
        TargetGroup::TotalAtMost(stat, n) => {
            return crate::relational::total(g, *stat, targets) <= g.eval_value(n, ctx)
        }
        _ => {}
    }
    if objs.len() < 2 {
        return true;
    }
    let every = crate::kw::changeling::every_creature_type;
    match group {
        TargetGroup::SameOwner => objs.iter().all(|o| o.owner == objs[0].owner),
        TargetGroup::SameController => objs.iter().all(|o| o.controller == objs[0].controller),
        TargetGroup::ShareCreatureType => {
            let fixed: Vec<&Characteristics> = objs
                .iter()
                .map(|o| &o.chars)
                .filter(|c| !every(c))
                .collect();
            match fixed.first() {
                // Each of them is every creature type.
                None => true,
                // The others are every creature type, so they have any type these have.
                Some(first) => {
                    creature_types(first).any(|t| fixed.iter().all(|c| c.has_subtype(t)))
                }
            }
        }
        TargetGroup::ShareNoCreatureType => objs.iter().enumerate().all(|(i, a)| {
            objs.iter().skip(i + 1).all(|b| {
                let (a, b) = (&a.chars, &b.chars);
                let a_any = every(a) || creature_types(a).next().is_some();
                let b_any = every(b) || creature_types(b).next().is_some();
                !((every(a) && b_any)
                    || (every(b) && a_any)
                    || creature_types(a).any(|t| b.has_subtype(t)))
            })
        }),
        TargetGroup::ShareCardType => objs[0]
            .chars
            .card_types
            .iter()
            .any(|t| objs.iter().all(|o| o.chars.is(t))),
        TargetGroup::SharePermanentType => objs[0]
            .chars
            .card_types
            .iter()
            .filter(|t| t.is_permanent_type())
            .any(|t| objs.iter().all(|o| o.chars.is(t))),
        // CR 201.2: objects with no name have no name in common with anything.
        TargetGroup::DifferentNames => objs.iter().enumerate().all(|(i, a)| {
            objs.iter()
                .skip(i + 1)
                .all(|b| !a.chars.shares_name_with(&b.chars))
        }),
        TargetGroup::TotalAtMost(..) => true,
    }
}

/// A group of `n` of the candidates that has the relationship, if there is one.
pub fn find_group(
    g: &Game,
    group: &TargetGroup,
    cands: &[Entity],
    n: usize,
    ctx: &Ctx,
) -> Option<Vec<Entity>> {
    if n == 0 {
        return Some(vec![]);
    }
    // Smallest totals first, so a total limit is met whenever any group meets it.
    let mut cands = cands.to_vec();
    if let TargetGroup::TotalAtMost(stat, _) = group {
        cands.sort_by_key(|c| crate::relational::total(g, *stat, std::slice::from_ref(c)));
    }
    let mut chosen = Vec::new();
    let mut budget = 10_000u32;
    search(g, group, &cands, n, &mut chosen, ctx, &mut budget).then_some(chosen)
}

fn search(
    g: &Game,
    group: &TargetGroup,
    cands: &[Entity],
    n: usize,
    chosen: &mut Vec<Entity>,
    ctx: &Ctx,
    budget: &mut u32,
) -> bool {
    if chosen.len() == n {
        return true;
    }
    for (i, c) in cands.iter().enumerate() {
        // Bounded: large candidate sets give up rather than search exponentially.
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        chosen.push(*c);
        if group_ok(g, group, chosen, ctx)
            && search(g, group, &cands[i + 1..], n, chosen, ctx, budget)
        {
            return true;
        }
        chosen.pop();
    }
    false
}

/// The chosen targets if they have the relationship; otherwise the largest group, in
/// choice order, of them that does, completed to `min` targets from the candidates.
pub fn fit(
    g: &Game,
    group: &TargetGroup,
    picked: Vec<Entity>,
    cands: &[Entity],
    min: usize,
    ctx: &Ctx,
) -> Option<Vec<Entity>> {
    if group_ok(g, group, &picked, ctx) {
        return Some(picked);
    }
    let mut kept: Vec<Entity> = Vec::new();
    for e in picked {
        kept.push(e);
        if !group_ok(g, group, &kept, ctx) {
            kept.pop();
        }
    }
    if kept.len() >= min {
        return Some(kept);
    }
    find_group(g, group, cands, min, ctx)
}
