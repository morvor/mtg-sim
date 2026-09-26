//! CR 702.22 Banding and "bands with other".
//!
//! A "bands with other [quality]" ability is a banding keyword whose `filter` is the
//! quality (so an effect that removes banding removes it too, CR 702.22b). Plain banding
//! has no filter.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::types::*;
use smol_str::SmolStr;

/// Whether the keyword instance is plain banding (not "bands with other").
pub fn is_plain_banding(kw: &Keyword) -> bool {
    kw.kind == KeywordKind::Banding && kw.filter.is_none()
}

/// Whether the object has banding (a "bands with other" ability isn't banding, CR
/// 702.22c).
pub fn has_banding(g: &Game, id: ObjectId) -> bool {
    g.obj(id).chars.keywords().any(is_plain_banding)
}

/// The qualities of the object's "bands with other" abilities.
pub fn bands_with_other(g: &Game, id: ObjectId) -> Vec<Filter> {
    g.obj(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Banding)
        .filter_map(|k| k.filter.clone())
        .collect()
}

/// Whether the object is a [quality] creature.
fn is_quality(g: &Game, id: ObjectId, q: &Filter) -> bool {
    let o = g.obj(id);
    o.is_creature() && g.matches(id, q, &Ctx::new(Some(id), o.controller))
}

/// The quality of "bands with other [quality]" ("legendary creatures", "Dinosaurs",
/// "creatures named Wolves of the Hunt"). `raw` keeps the original case for names.
pub fn quality_filter(lower: &str, raw: &str) -> Option<Filter> {
    let lower = end(lower.trim());
    if let Some(i) = lower.find(" named ") {
        let (f, _, tail) = parse_object_phrase(&lower[..i])?;
        if !end(tail).is_empty() {
            return None;
        }
        let name = raw.trim().get(i + " named ".len()..)?.trim().trim_end_matches('.');
        return Some(Filter::and(vec![f, Filter::Named(SmolStr::new(name))]));
    }
    let (f, _, tail) = parse_object_phrase(lower)?;
    end(tail).is_empty().then_some(f)
}

/// Whether the creatures can be declared as one attacking band (CR 702.22c): creatures
/// with banding and at most one without (even if it has "bands with other"); or [quality]
/// creatures, at least one with "bands with other [quality]".
pub fn legal_band(g: &Game, members: &[ObjectId]) -> bool {
    if members.len() < 2 {
        return false;
    }
    let without = members.iter().filter(|m| !has_banding(g, **m)).count();
    if without <= 1 && without < members.len() {
        return true;
    }
    members.iter().any(|m| {
        bands_with_other(g, *m)
            .iter()
            .any(|q| members.iter().all(|x| is_quality(g, *x, q)))
    })
}

/// CR 508.1e, 702.22c–d: as attackers are declared, the active player announces bands.
/// For each attacking creature with banding or "bands with other" not yet in a band, they
/// choose other creatures attacking the same player, planeswalker, or battle to band with
/// it; an illegal choice forms no band. Returns (creature, band id) pairs.
pub fn announce_bands(
    g: &mut Game,
    ap: PlayerId,
    declared: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, u32)> {
    let mut out: Vec<(ObjectId, u32)> = Vec::new();
    let mut next = 1u32;
    for (a, t) in declared {
        let a = *a;
        let can_lead = has_banding(g, a)
            || bands_with_other(g, a)
                .iter()
                .any(|q| is_quality(g, a, q));
        if !can_lead || out.iter().any(|(x, _)| *x == a) {
            continue;
        }
        // CR 702.22d: all attack the same player, planeswalker, or battle.
        let cands: Vec<ObjectId> = declared
            .iter()
            .filter(|(x, u)| *x != a && u == t && !out.iter().any(|(y, _)| y == x))
            .map(|(x, _)| *x)
            .collect();
        if cands.is_empty() {
            continue;
        }
        let name = g.obj(a).chars.name.clone();
        let chosen: Vec<ObjectId> = g
            .ask_objects(
                ap,
                Some(a),
                &format!("Choose attacking creatures to band with {name}"),
                cands.clone(),
                0,
                cands.len() as u32,
            )
            .into_iter()
            .filter(|x| cands.contains(x))
            .collect();
        if chosen.is_empty() {
            continue;
        }
        let mut members = vec![a];
        members.extend(chosen);
        members.dedup();
        if !legal_band(g, &members) {
            continue;
        }
        out.extend(members.into_iter().map(|m| (m, next)));
        next += 1;
    }
    out
}

/// The other creatures in the attacking creature's band (CR 702.22f: creatures removed
/// from combat aren't in it any more).
pub fn band_mates(g: &Game, attacker: ObjectId) -> Vec<ObjectId> {
    let Some(c) = &g.combat else { return vec![] };
    let Some(band) = c.attacker(attacker).and_then(|a| a.band) else {
        return vec![];
    };
    c.attackers
        .iter()
        .filter(|x| x.id != attacker && x.band == Some(band))
        .map(|x| x.id)
        .collect()
}

/// Whether the creatures include a creature with banding, or both a [quality] creature
/// with "bands with other [quality]" and another [quality] creature (CR 702.22j–k).
fn banding_group(g: &Game, creatures: &[ObjectId]) -> bool {
    creatures.iter().any(|c| has_banding(g, *c))
        || creatures.iter().any(|c| {
            bands_with_other(g, *c).iter().any(|q| {
                is_quality(g, *c, q)
                    && creatures
                        .iter()
                        .any(|o| o != c && is_quality(g, *o, q))
            })
        })
}

pub struct Banding;

impl KeywordRules for Banding {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Banding]
    }

    /// CR 702.22j: an attacking creature blocked by a creature with banding (or by a
    /// banding group of "bands with other") has its damage assigned by the defending
    /// player. CR 702.22k: a blocking creature blocking a creature with banding (or such a
    /// group) has its damage assigned by the active player.
    fn combat_damage_assigner(&self, g: &Game, creature: ObjectId) -> Option<PlayerId> {
        let c = g.combat.as_ref()?;
        if let Some(ai) = c.attacker(creature) {
            let blockers: Vec<ObjectId> = ai
                .blockers
                .iter()
                .copied()
                .filter(|b| g.is_live(*b) && g.is_blocking(*b))
                .collect();
            if blockers.is_empty() || !banding_group(g, &blockers) {
                return None;
            }
            return Some(g.obj(blockers[0]).controller);
        }
        let blocked: Vec<ObjectId> = c
            .blocking(creature)
            .into_iter()
            .filter(|a| g.is_live(*a) && g.is_attacking(*a))
            .collect();
        if blocked.is_empty() || !banding_group(g, &blocked) {
            return None;
        }
        Some(g.obj(blocked[0]).controller)
    }

    /// CR 702.22h–i: when a member of a band becomes blocked, the whole band does.
    fn also_blocked(&self, g: &Game, attacker: ObjectId) -> Vec<ObjectId> {
        band_mates(g, attacker)
    }
}

inventory::submit! { KeywordRegistration(&Banding) }
