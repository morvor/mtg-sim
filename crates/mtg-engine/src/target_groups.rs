//! Requirements on the targets of one instance of the word "target" taken together
//! (`TargetSpec::together`): "up to two target cards from a single graveyard", "two target
//! cards from an opponent's graveyard" (one opponent's graveyard), "two target creature
//! cards that share a creature type", "any number of target tapped creatures with
//! different controllers", "any number of target creatures with equal toughness", "up to
//! two target creatures with total mana value 6 or less"; and requirements between the
//! targets of two instances ("another target creature with the same controller",
//! `TargetSpec::related_to`).
//!
//! The requirement is part of what makes the chosen targets legal: it's checked as
//! targets are chosen (CR 601.2c) — a spell or ability can't be put on the stack unless
//! a group of the required size exists — and again as the spell or ability resolves
//! (CR 608.2b), where targets that no longer have the relationship are all illegal. Which
//! targets are compared on resolution depends on the relationship (see
//! [`holds_on_resolution`]): of two cards that share a creature type, one that has left its
//! zone is compared using its last known information, so the one still in the graveyard
//! is returned as long as it has a creature type the other had as it left; targets that
//! must have equal toughness or a total mana value or power are compared among the
//! targets that are still legal.

use crate::ability::{TargetGroup, TargetSpec, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::{Characteristics, GameObject};
use crate::types::*;

fn creature_types(c: &Characteristics) -> impl Iterator<Item = &Subtype> {
    c.subtypes.iter().filter(|s| is_creature_type(s))
}

fn objects<'a>(g: &'a Game, targets: &[Entity]) -> Vec<&'a GameObject> {
    targets
        .iter()
        .filter_map(|e| e.object())
        .map(|o| g.obj(o))
        .collect()
}

/// Whether every two of them have the relationship `pair`.
fn pairwise(objs: &[&GameObject], pair: impl Fn(&GameObject, &GameObject) -> bool) -> bool {
    objs.iter()
        .enumerate()
        .all(|(i, a)| objs.iter().skip(i + 1).all(|b| pair(a, b)))
}

/// The bound of a "total ... N or less" relationship.
fn bound(g: &Game, v: &Value, ctx: &Ctx) -> i64 {
    g.eval_value(v, ctx)
}

/// The values a "total" relationship adds up, for each object.
fn stat(g: &Game, group: &TargetGroup, o: &GameObject) -> i64 {
    match group {
        TargetGroup::TotalManaValueAtMost(_) => g.mana_value_of(o.id) as i64,
        TargetGroup::TotalPowerAtMost(_) => o.power() as i64,
        _ => 0,
    }
}

/// Whether these targets, taken together, have the relationship. Objects that left their
/// zone are compared using their last known information.
pub fn group_ok(g: &Game, group: &TargetGroup, targets: &[Entity], ctx: &Ctx) -> bool {
    let objs = objects(g, targets);
    let every = crate::kw::changeling::every_creature_type;
    match group {
        TargetGroup::SameOwner => objs.iter().all(|o| o.owner == objs[0].owner),
        TargetGroup::SameController => objs.iter().all(|o| o.controller == objs[0].controller),
        TargetGroup::DifferentControllers => pairwise(&objs, |a, b| a.controller != b.controller),
        // CR 201.2b: each has a name, and no two have a name in common.
        TargetGroup::DifferentNames => {
            objs.iter().all(|o| o.chars.has_a_name())
                && pairwise(&objs, |a, b| !a.chars.shares_name_with(&b.chars))
        }
        TargetGroup::DifferentManaValues => {
            pairwise(&objs, |a, b| g.mana_value_of(a.id) != g.mana_value_of(b.id))
        }
        TargetGroup::EqualToughness => objs.iter().all(|o| o.toughness() == objs[0].toughness()),
        TargetGroup::TotalManaValueAtMost(v) | TargetGroup::TotalPowerAtMost(v) => {
            objs.iter().map(|o| stat(g, group, o)).sum::<i64>() <= bound(g, v, ctx)
        }
        TargetGroup::ShareCreatureType => {
            if objs.len() < 2 {
                return true;
            }
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
        TargetGroup::ShareNoCreatureType => pairwise(&objs, |a, b| {
            let (a, b) = (&a.chars, &b.chars);
            let a_any = every(a) || creature_types(a).next().is_some();
            let b_any = every(b) || creature_types(b).next().is_some();
            !((every(a) && b_any)
                || (every(b) && a_any)
                || creature_types(a).any(|t| b.has_subtype(t)))
        }),
        TargetGroup::ShareCardType => {
            objs.len() < 2
                || objs[0]
                    .chars
                    .card_types
                    .iter()
                    .any(|t| objs.iter().all(|o| o.chars.is(t)))
        }
        TargetGroup::SharePermanentType => {
            objs.len() < 2
                || objs[0]
                    .chars
                    .card_types
                    .iter()
                    .filter(|t| t.is_permanent_type())
                    .any(|t| objs.iter().all(|o| o.chars.is(t)))
        }
    }
}

/// Whether the targets of one instance of the word "target" still have the relationship
/// as the spell or ability resolves (CR 608.2b). `all` are the targets chosen and
/// `legal` those of them that are still legal on their own. Targets that must share a
/// type or be in one graveyard are compared including the ones that left their zone, by
/// their last known information (Raise the Draugr); the others are compared among the
/// legal targets only ("the legal targets no longer all have equal toughness", V.A.T.S.).
pub fn holds_on_resolution(
    g: &Game,
    group: &TargetGroup,
    all: &[Entity],
    legal: &[Entity],
    ctx: &Ctx,
) -> bool {
    match group {
        TargetGroup::SameOwner
        | TargetGroup::SameController
        | TargetGroup::ShareCreatureType
        | TargetGroup::ShareNoCreatureType
        | TargetGroup::ShareCardType
        | TargetGroup::SharePermanentType => group_ok(g, group, all, ctx),
        TargetGroup::DifferentControllers
        | TargetGroup::DifferentNames
        | TargetGroup::DifferentManaValues
        | TargetGroup::EqualToughness
        | TargetGroup::TotalManaValueAtMost(_)
        | TargetGroup::TotalPowerAtMost(_) => group_ok(g, group, legal, ctx),
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
    if cands.len() < n {
        return None;
    }
    // A total is smallest with the smallest values (a value may be negative, so partial
    // totals don't rule anything out).
    if matches!(
        group,
        TargetGroup::TotalManaValueAtMost(_) | TargetGroup::TotalPowerAtMost(_)
    ) {
        let mut v: Vec<Entity> = cands.to_vec();
        v.sort_by_key(|e| e.object().map_or(0, |o| stat(g, group, g.obj(o))));
        v.truncate(n);
        return group_ok(g, group, &v, ctx).then_some(v);
    }
    // The other relationships hold for every part of a group that has them.
    let mut chosen = Vec::new();
    search(g, group, cands, n, &mut chosen, ctx).then_some(chosen)
}

fn search(
    g: &Game,
    group: &TargetGroup,
    cands: &[Entity],
    n: usize,
    chosen: &mut Vec<Entity>,
    ctx: &Ctx,
) -> bool {
    if chosen.len() == n {
        return true;
    }
    for (i, c) in cands.iter().enumerate() {
        chosen.push(*c);
        if group_ok(g, group, chosen, ctx) && search(g, group, &cands[i + 1..], n, chosen, ctx) {
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

/// Whether the targets chosen for `spec` have the relationship it requires with the
/// targets of the slot it's related to (`TargetSpec::related_to`), given every slot's
/// targets. With no targets in either slot there's nothing to compare.
pub fn related_ok(
    g: &Game,
    spec: &TargetSpec,
    mine: &[Entity],
    slots: &[Vec<Entity>],
    ctx: &Ctx,
) -> bool {
    let Some((other, group)) = &spec.related_to else {
        return true;
    };
    let Some(theirs) = slots.get(*other as usize) else {
        return true;
    };
    mine.iter().all(|m| {
        theirs.iter().all(|t| {
            let pair = [*t, *m];
            group_ok(g, group, &pair, ctx)
        })
    })
}
