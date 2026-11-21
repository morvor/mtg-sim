//! "Can't have [ability]" and resolving spells and abilities (CR 113.11): "If a resolving
//! spell or ability creates a continuous effect that would add the specified ability to
//! such an object, that part of that continuous effect does not apply; however, other
//! parts of that continuous effect will still apply". That part never applies, even after
//! the "can't have" effect ends (Archetype of Courage leaving the battlefield doesn't give
//! first strike to a creature an earlier Sure Strike targeted).
//!
//! Static abilities that add the ability are handled by the layer system (the "can't
//! have" modification is applied after every other layer 6 effect, `layers.rs`); this is
//! only for effects created by a resolving spell or ability.

use crate::ability::*;
use crate::game::{Affected, Game};
use crate::keywords::KeywordKind;
use crate::types::*;

/// The objects of the continuous effects `g.effects[from..]` (just created by a resolving
/// spell or ability) that can't have an ability one of them adds lose that part of the
/// effect: after the effects apply, an object without the keyword the effect adds can't
/// have it (the new effect has the latest timestamp, so nothing applied later removed
/// it).
pub fn drop_ungrantable_keywords(g: &mut Game, from: usize) {
    let adds_keyword = |e: &crate::game::ContinuousEffect| {
        e.mods
            .iter()
            .any(|m| matches!(m, Modification::AddKeyword(_)))
    };
    if !g.effects[from..].iter().any(adds_keyword) {
        return;
    }
    g.recompute();
    let mut i = from;
    while i < g.effects.len() {
        let e = &g.effects[i];
        let Affected::Objects(objs) = &e.affected else {
            i += 1;
            continue;
        };
        let kinds: Vec<KeywordKind> = e
            .mods
            .iter()
            .filter_map(|m| match m {
                Modification::AddKeyword(k) => Some(k.kind),
                _ => None,
            })
            .collect();
        if kinds.is_empty() {
            i += 1;
            continue;
        }
        // For each object, the kinds it can't have.
        let blocked: Vec<(ObjectId, Vec<KeywordKind>)> = objs
            .iter()
            .map(|o| {
                let ks = kinds
                    .iter()
                    .copied()
                    .filter(|k| g.is_live(*o) && !g.obj(*o).chars.has_keyword(*k))
                    .collect();
                (*o, ks)
            })
            .collect();
        if blocked.iter().all(|(_, ks)| ks.is_empty()) {
            i += 1;
            continue;
        }
        // Split the effect: objects that can have every keyword keep it as it is; each
        // other object gets the effect without the keywords it can't have.
        let e = g.effects[i].clone();
        let keep: Vec<ObjectId> = blocked
            .iter()
            .filter(|(_, ks)| ks.is_empty())
            .map(|(o, _)| *o)
            .collect();
        let mut split = Vec::new();
        for (o, ks) in blocked.into_iter().filter(|(_, ks)| !ks.is_empty()) {
            let mut part = e.clone();
            part.id = g.new_effect_id();
            part.affected = Affected::Objects(vec![o]);
            part.mods
                .retain(|m| !matches!(m, Modification::AddKeyword(k) if ks.contains(&k.kind)));
            split.push(part);
        }
        if keep.is_empty() {
            g.effects.remove(i);
        } else {
            g.effects[i].affected = Affected::Objects(keep);
            i += 1;
        }
        g.effects.extend(split.into_iter().filter(|p| !p.mods.is_empty()));
    }
    g.dirty = true;
}
