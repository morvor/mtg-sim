//! Attaching Auras, Equipment, and Fortifications (CR 301.5, 301.6, 303.4, 701.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

/// Whether `t` has protection that keeps the Aura from enchanting it (CR 702.16c),
/// ignoring protection from effects that say they don't remove it (CR 702.16n, 702.16p).
fn aura_protection_applies(g: &Game, t: ObjectId, aura: ObjectId) -> bool {
    crate::kw::protection::prevents_attachment(g, t, aura)
}

/// What one enchant ability allows enchanting players, if it's "Enchant player" or
/// "Enchant opponent" (CR 702.5d).
fn enchant_kw_player(k: &crate::keywords::Keyword) -> Option<PlayerFilter> {
    match k.text.as_deref().map(|t| t.to_lowercase()) {
        Some(t) if t.ends_with("player") => Some(PlayerFilter::Any),
        Some(t) if t.ends_with("opponent") => Some(PlayerFilter::Opponent),
        _ => None,
    }
}

/// The "enchant" restriction of an Aura as a filter over objects (None = it can't enchant
/// objects: it has no enchant ability, or one of them enchants players, handled via
/// `enchant_player`). With several enchant abilities, the Aura can enchant only objects
/// that match all of them (CR 702.5c).
pub fn enchant_filter(chars: &Characteristics) -> Option<Filter> {
    let mut fs = Vec::new();
    for k in chars.keywords().filter(|k| k.kind == KeywordKind::Enchant) {
        if enchant_kw_player(k).is_some() {
            return None; // CR 702.5d: can't enchant permanents.
        }
        fs.push(k.filter.clone()?);
    }
    if fs.is_empty() {
        return None;
    }
    Some(Filter::and(fs))
}

/// Whether the Aura's enchant abilities allow enchanting players ("Enchant player",
/// "Enchant opponent"). Every instance must allow it (CR 702.5c, 702.5d).
pub fn enchant_player(chars: &Characteristics) -> Option<PlayerFilter> {
    let mut out: Option<PlayerFilter> = None;
    for k in chars.keywords().filter(|k| k.kind == KeywordKind::Enchant) {
        let pf = enchant_kw_player(k)?;
        out = Some(match (out, pf) {
            (Some(PlayerFilter::Opponent), _) | (_, PlayerFilter::Opponent) => {
                PlayerFilter::Opponent
            }
            _ => PlayerFilter::Any,
        });
    }
    out
}

/// The target an Aura spell requires (CR 303.4a, 702.5a).
pub fn aura_target_spec(chars: &Characteristics) -> Option<TargetSpec> {
    if let Some(pf) = enchant_player(chars) {
        return Some(TargetSpec::player(pf, "player to enchant"));
    }
    let f = enchant_filter(chars)?;
    let f = match f.zone() {
        Some(_) => f,
        None => Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
    };
    Some(TargetSpec::object(f, "object to enchant"))
}

/// Whether `obj` could legally be attached to `to` right now (CR 301.5c, 303.4d, 701.3).
pub fn can_attach(g: &Game, obj: ObjectId, to: Entity) -> bool {
    can_attach_as(g, obj, to, false)
}

/// [`can_attach`], optionally treating `to` as though it were a creature ("equip
/// planeswalker", CR 702.6e).
pub fn can_attach_as(g: &Game, obj: ObjectId, to: Entity, as_creature: bool) -> bool {
    if Entity::Object(obj) == to {
        return false; // an object can't be attached to itself
    }
    legal_attachment_as(g, obj, to, as_creature)
}

/// Whether `obj` attached to `to` is legal (used by SBAs 704.5m/n).
pub fn legal_attachment(g: &Game, obj: ObjectId, to: Entity) -> bool {
    legal_attachment_as(g, obj, to, false)
}

fn legal_attachment_as(g: &Game, obj: ObjectId, to: Entity, as_creature: bool) -> bool {
    let o = g.obj(obj);
    let chars = &o.chars;
    if Entity::Object(obj) == to {
        return false;
    }
    match to {
        Entity::Player(p) => {
            if !g.player(p).in_game() {
                return false;
            }
            if chars.has_subtype("Aura") {
                let ok = enchant_player(chars).is_some_and(|pf| {
                    g.player_filter_matches(&pf, p, &Ctx::new(Some(obj), o.controller))
                });
                // Player protection from the Aura (CR 702.16c) handled through player mods.
                ok && !g.player(p).mods.iter().any(|m| matches!(m, PlayerModification::ProtectionFrom(f) if g.matches(obj, f, &Ctx::new(None, p))))
            } else {
                false
            }
        }
        Entity::Object(t) => {
            if !g.is_live(t) || g.obj(t).zone != Zone::Battlefield {
                return false;
            }
            let target = g.obj(t);
            if target.phased_out && !o.phased_out {
                return false;
            }
            if chars.has_subtype("Aura") {
                let Some(f) = enchant_filter(chars) else {
                    return false;
                };
                if !g.matches(t, &f, &Ctx::new(Some(obj), o.controller)) {
                    return false;
                }
                // CR 702.16c: can't be enchanted by Auras with the protected quality.
                !aura_protection_applies(g, t, obj)
            } else if chars.has_subtype("Equipment") {
                // CR 301.5c: Equipment can be attached only to creatures; 702.16d protection.
                (target.is_creature() || as_creature)
                    && !crate::kw::protection::prevents_attachment(g, t, obj)
                    && crate::keyword_impls::equip_restriction_ok(g, obj, t)
            } else if chars.has_subtype("Fortification") {
                // CR 301.6: Fortifications attach to lands.
                target.chars.is_land() && !crate::kw::protection::prevents_attachment(g, t, obj)
            } else {
                false
            }
        }
    }
}
