//! CR 702.16 Protection.
//!
//! A protection keyword carries its quality as a filter (`Keyword::filter`; `Filter::Any`
//! for "protection from everything", 702.16j; `ControlledBy(..)` for protection from a
//! player, 702.16k). "From A and from B" and "from each color" compile to one keyword per
//! quality (702.16g–i). Players get protection through `PlayerModification::ProtectionFrom`.
//!
//! Each part of protection ("DEBT") is enforced where the action happens:
//! * **D**amage (702.16e) — here, as a prevention effect (CR 615) of the permanent or
//!   player with protection;
//! * **E**nchanting/**E**quipping/fortifying (702.16c–d) — `attach.rs` and the SBAs of
//!   CR 704.5m–n;
//! * **B**locking (702.16f) — `combat.rs`;
//! * **T**argeting (702.16b) — `stack.rs`.
//!
//! The quality is matched against the source's characteristics; a card type, subtype, or
//! supertype quality matches permanents and objects in other zones alike (702.16a).

use super::{KeywordRegistration, KeywordRules, KeywordShield};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

pub struct Protection;

impl KeywordRules for Protection {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Protection]
    }

    /// CR 702.16e: "Any damage that would be dealt by sources that have the stated
    /// quality to a permanent or player with protection is prevented." Multiple
    /// instances from the same quality are redundant (702.16m), so one effect per
    /// permanent or player is enough.
    fn damage_prevention(&self, g: &Game, source: ObjectId, target: Entity) -> Vec<KeywordShield> {
        match target {
            Entity::Object(o) => {
                let ob = g.obj(o);
                if ob.zone != Zone::Battlefield {
                    return vec![];
                }
                let ctx = Ctx::new(Some(o), ob.controller);
                ob.chars
                    .abilities
                    .iter()
                    .find_map(|a| match &a.kind {
                        AbilityKind::Keyword(k)
                            if k.kind == KeywordKind::Protection
                                && k.filter.as_ref().is_none_or(|f| g.matches(source, f, &ctx)) =>
                        {
                            Some(KeywordShield {
                                holder: target,
                                id: a.uid,
                                controller: ob.controller,
                                text: format!("{}: {}", ob.chars.name, a.text),
                            })
                        }
                        _ => None,
                    })
                    .into_iter()
                    .collect()
            }
            Entity::Player(p) => g
                .player(p)
                .mods
                .iter()
                .enumerate()
                .find_map(|(i, m)| match m {
                    PlayerModification::ProtectionFrom(f)
                        if g.matches(source, f, &Ctx::new(None, p)) =>
                    {
                        Some(KeywordShield {
                            holder: target,
                            id: i as u64,
                            controller: p,
                            text: format!("{p}: protection"),
                        })
                    }
                    _ => None,
                })
                .into_iter()
                .collect(),
        }
    }
}

inventory::submit! { KeywordRegistration(&Protection) }

// ---------------------------------------------------------------------------
// "This effect doesn't remove ..." (CR 702.16n, 702.16p)
// ---------------------------------------------------------------------------

/// Marker text on a granted protection keyword: "This effect doesn't remove Auras"
/// (CR 702.16n) — no Aura falls off because of this instance.
pub const DOESNT_REMOVE_AURAS: &str = "doesn't remove auras";

/// Marker text on a granted protection keyword: "This effect doesn't remove Auras and
/// Equipment you control that are already attached to it" (CR 702.16p). Bound to the
/// granting object when applied (see [`bind_marker`]).
pub const DOESNT_REMOVE_ATTACHED: &str = "doesn't remove attached";

/// The bound form of a marker for the object `src` granting the protection, if the text
/// is a marker that needs binding.
pub fn bind_marker(text: &str, src: ObjectId) -> Option<smol_str::SmolStr> {
    match text {
        crate::choices::DOESNT_REMOVE_SOURCE => Some(crate::choices::doesnt_remove_marker(src)),
        DOESNT_REMOVE_ATTACHED => Some(format!("{DOESNT_REMOVE_ATTACHED} #{}", src.0).into()),
        _ => None,
    }
}

/// Whether this protection instance of `t` leaves `obj` attached to it (or lets it stay
/// attached) because the effect granting it doesn't remove it.
fn exempts(g: &Game, kw: &crate::keywords::Keyword, t: ObjectId, obj: ObjectId) -> bool {
    let Some(text) = kw.text.as_deref() else {
        return false;
    };
    let o = g.obj(obj);
    // CR 702.16n: "doesn't remove [this Aura]" / "doesn't remove Auras".
    if text == crate::choices::doesnt_remove_marker(obj) {
        return true;
    }
    if text == DOESNT_REMOVE_AURAS {
        return o.chars.has_subtype("Aura");
    }
    // CR 702.16p: objects the granting object's controller controls that were already
    // attached when the effect started to apply — those attached no later than the
    // granting object got its timestamp by entering or becoming attached (CR 613.7e).
    // An object becoming attached now isn't already attached.
    if let Some(src) = text
        .strip_prefix(DOESNT_REMOVE_ATTACHED)
        .and_then(|r| r.trim().strip_prefix('#'))
        .and_then(|n| n.parse::<u32>().ok())
    {
        let src = g.obj(ObjectId(src));
        return o.attached_to == Some(Entity::Object(t))
            && o.controller == src.controller
            && o.timestamp <= src.timestamp;
    }
    false
}

/// Whether permanent `t`'s protection keeps `obj` (an Aura, Equipment, or Fortification)
/// from being attached to it (CR 702.16c–d), apart from effects that don't remove it.
pub fn prevents_attachment(g: &Game, t: ObjectId, obj: ObjectId) -> bool {
    let ob = g.obj(t);
    let ctx = Ctx::new(Some(t), ob.controller);
    ob.chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .filter(|k| k.filter.as_ref().is_none_or(|f| g.matches(obj, f, &ctx)))
        .any(|k| !exempts(g, k, t, obj))
}
