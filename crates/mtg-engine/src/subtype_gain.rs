//! Subtypes that a resolving spell or ability gives an object that can't have them as the
//! effect begins (CR 205.3d, 611.2c; register
//! `subtype-gained-only-if-it-corresponds-as-the-effect-begins`, decided by the user Nov 19).
//!
//! CR 205.3d: "An object can't gain a subtype that doesn't correspond to one of that
//! object's types." A continuous effect from a resolving spell or ability that makes an
//! object a [subtype] "in addition to its other types" gives it only the subtypes that
//! correspond to one of its types as the effect begins (its types with the effect applied:
//! the card types the effect itself, or an earlier effect, gives it count). The others are
//! never gained by that effect, for its whole duration, even if the object gets that card
//! type later (a noncreature Vehicle crewed). Shadow Puppeteers (2023-09-01): "Since it's
//! not a creature, it won't gain the Dragon subtype, and it won't have base power and
//! toughness 4/4." The effect's other parts (its color) apply. The same for what a resolving
//! spell or ability says a permanent it puts onto the battlefield "is" (CR 611.2e: "That
//! creature is a black Zombie in addition to its other colors and types", Rise from the
//! Grave): that effect begins as the permanent enters, so its types then decide
//! (`actions.rs`, the entering permanent's `with_mods`).
//!
//! The effect's base power and toughness aren't set aside: CR 208.3a "If an effect would be
//! created that sets the base power and/or toughness of a noncreature permanent, or
//! otherwise modifies its power and/or toughness, that effect is created even though it
//! doesn't do anything unless that permanent becomes a creature." (register
//! `base-pt-set-on-a-noncreature-applies-once-it-is-a-creature`, provisional).
//!
//! An object that could have the subtype as the effect began keeps following 205.3d
//! continuously (layer 4 in `layers.rs`): Terror of Towashi (2023-04-14) "If the creature
//! stops being a creature after the ability has resolved, it will also stop being a
//! Phyrexian until it becomes a creature again, if applicable." Static abilities and
//! characteristic-defining abilities aren't affected: only effects a resolving spell or
//! ability creates. (A copy effect's exceptions are handled as the copy applies, see
//! `layers.rs`, `drop_ungainable_subtypes`.)

use crate::ability::*;
use crate::game::{Affected, ContinuousEffect, Game};
use crate::object::Characteristics;
use crate::types::*;

/// Whether `m` is a part of an effect this module may set aside for an object.
fn lockable(m: &Modification) -> bool {
    matches!(
        m,
        Modification::AddSubtypes(_) | Modification::AllCreatureTypes
    )
}

/// The parts of `mods` an object with characteristics `c` (as the effect begins) can't
/// gain: `mods` with those parts removed or narrowed, or `None` if it can gain them all.
fn gainable_mods(mods: &[Modification], c: &Characteristics) -> Option<Vec<Modification>> {
    let types = c.card_types;
    let creature_types = types.contains(CardType::Creature) || types.contains(CardType::Kindred);
    let mut changed = false;
    let mut out = Vec::with_capacity(mods.len());
    for m in mods {
        match m {
            Modification::AddSubtypes(ss) => {
                let keep: Vec<Subtype> = ss
                    .iter()
                    .filter(|s| crate::layers::subtype_still_valid(s, types))
                    .cloned()
                    .collect();
                if keep.len() != ss.len() {
                    changed = true;
                    if !keep.is_empty() {
                        out.push(Modification::AddSubtypes(keep));
                    }
                } else {
                    out.push(m.clone());
                }
            }
            Modification::AllCreatureTypes if !creature_types => changed = true,
            _ => out.push(m.clone()),
        }
    }
    changed.then_some(out)
}

/// The objects of the continuous effects `g.effects[from..]` (just created by a resolving
/// spell or ability) lose the parts of them they can't gain as the effects begin (see the
/// module docs): an effect is split so that each such object gets it without those parts.
pub fn lock_ungainable_parts(g: &mut Game, from: usize) {
    let has_lockable = |e: &ContinuousEffect| {
        e.layer1.is_none()
            && matches!(e.affected, Affected::Objects(_))
            && e.mods.iter().any(lockable)
    };
    if !g.effects[from..].iter().any(has_lockable) {
        return;
    }
    // The objects' characteristics as the effects begin: with them applied (their own
    // card types count) and nothing later.
    g.recompute();
    let mut i = from;
    let end = g.effects.len();
    let mut split: Vec<ContinuousEffect> = Vec::new();
    while i < end {
        if !has_lockable(&g.effects[i]) {
            i += 1;
            continue;
        }
        let Affected::Objects(objs) = &g.effects[i].affected else {
            i += 1;
            continue;
        };
        let narrowed: Vec<(ObjectId, Option<Vec<Modification>>)> = objs
            .iter()
            .map(|o| {
                let n = if g.is_live(*o) {
                    gainable_mods(&g.effects[i].mods, &g.obj(*o).chars)
                } else {
                    None
                };
                (*o, n)
            })
            .collect();
        if narrowed.iter().all(|(_, n)| n.is_none()) {
            i += 1;
            continue;
        }
        let e = g.effects[i].clone();
        let keep: Vec<_> = narrowed
            .iter()
            .filter(|(_, n)| n.is_none())
            .map(|(o, _)| *o)
            .collect();
        for (o, n) in narrowed {
            let Some(mods) = n else { continue };
            if mods.is_empty() {
                continue;
            }
            let mut part = e.clone();
            part.id = g.new_effect_id();
            part.affected = Affected::Objects(vec![o]);
            part.mods = mods;
            split.push(part);
        }
        g.effects[i].affected = Affected::Objects(keep);
        i += 1;
    }
    // Effects left with no object are dropped (only among the new ones).
    let mut idx = 0;
    g.effects.retain(|e| {
        let new = idx >= from && idx < end;
        idx += 1;
        !(new && matches!(&e.affected, Affected::Objects(v) if v.is_empty()))
    });
    g.effects.extend(split);
    g.dirty = true;
}
