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
use crate::game::Game;
use crate::object::Characteristics;
use crate::types::*;

fn creature_types(c: &Characteristics) -> impl Iterator<Item = &Subtype> {
    c.subtypes.iter().filter(|s| is_creature_type(s))
}

/// Whether these targets, taken together, have the relationship. Objects that left their
/// zone are compared using their last known information.
pub fn group_ok(g: &Game, group: TargetGroup, targets: &[Entity]) -> bool {
    let objs: Vec<&crate::object::GameObject> = targets
        .iter()
        .filter_map(|e| e.object())
        .map(|o| g.obj(o))
        .collect();
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
    }
}

/// A group of `n` of the candidates that has the relationship, if there is one.
pub fn find_group(g: &Game, group: TargetGroup, cands: &[Entity], n: usize) -> Option<Vec<Entity>> {
    if n == 0 {
        return Some(vec![]);
    }
    let mut chosen = Vec::new();
    search(g, group, cands, n, &mut chosen).then_some(chosen)
}

fn search(
    g: &Game,
    group: TargetGroup,
    cands: &[Entity],
    n: usize,
    chosen: &mut Vec<Entity>,
) -> bool {
    if chosen.len() == n {
        return true;
    }
    for (i, c) in cands.iter().enumerate() {
        chosen.push(*c);
        if group_ok(g, group, chosen) && search(g, group, &cands[i + 1..], n, chosen) {
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
    group: TargetGroup,
    picked: Vec<Entity>,
    cands: &[Entity],
    min: usize,
) -> Option<Vec<Entity>> {
    if group_ok(g, group, &picked) {
        return Some(picked);
    }
    let mut kept: Vec<Entity> = Vec::new();
    for e in picked {
        kept.push(e);
        if !group_ok(g, group, &kept) {
            kept.pop();
        }
    }
    if kept.len() >= min {
        return Some(kept);
    }
    find_group(g, group, cands, min)
}
