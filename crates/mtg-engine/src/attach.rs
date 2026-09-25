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

/// The "enchant" restriction of an Aura as a filter over objects (None = can enchant a
/// player, handled via `enchant_player`).
pub fn enchant_filter(chars: &Characteristics) -> Option<Filter> {
    chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Enchant)
        .and_then(|k| k.filter.clone())
}

/// Whether the Aura's enchant ability allows enchanting players ("Enchant player",
/// "Enchant opponent").
pub fn enchant_player(chars: &Characteristics) -> Option<PlayerFilter> {
    let kw = chars.keywords().find(|k| k.kind == KeywordKind::Enchant)?;
    match kw.text.as_deref().map(|t| t.to_lowercase()) {
        Some(t) if t.ends_with("player") => Some(PlayerFilter::Any),
        Some(t) if t.ends_with("opponent") => Some(PlayerFilter::Opponent),
        _ => None,
    }
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
    if Entity::Object(obj) == to {
        return false; // an object can't be attached to itself
    }
    legal_attachment(g, obj, to)
}

/// Whether `obj` attached to `to` is legal (used by SBAs 704.5m/n).
pub fn legal_attachment(g: &Game, obj: ObjectId, to: Entity) -> bool {
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
                target.is_creature()
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
