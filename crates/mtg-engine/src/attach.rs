//! Attaching Auras, Equipment, and Fortifications (CR 301.5, 301.6, 303.4, 701.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::{MoveEv, ReplEvent};
use crate::types::*;

/// `Filter::Custom` name: an object the source could legally be attached to right now
/// (CR 301.5c, 303.4k, 701.3).
pub const SOURCE_CAN_ATTACH: &str = "source_can_attach";

/// Custom filters about attaching. Returns `None` if `name` isn't one.
pub fn custom_filter(g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
    (name == SOURCE_CAN_ATTACH).then(|| {
        ctx.source
            .is_some_and(|s| can_attach(g, s, Entity::Object(id)))
    })
}

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
    // CR 310.10: a battle can't be attached to players or permanents, even if it's also an
    // Aura, Equipment, or Fortification.
    if chars.is(CardType::Battle) {
        return false;
    }
    // CR 801.8, 801.9: not to an object or player outside its controller's range of
    // influence.
    if !crate::multiplayer::range::attachment_in_range(g, obj, to) {
        return false;
    }
    match to {
        Entity::Player(p) => {
            if !g.player(p).in_game() {
                return false;
            }
            if chars.has_subtype("Aura") && !o.is_creature() {
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
                // CR 303.4d: an Aura that's also a creature can't enchant anything.
                if o.is_creature() {
                    return false;
                }
                let Some(f) = enchant_filter(chars) else {
                    return false;
                };
                if !g.matches(t, &f, &Ctx::new(Some(obj), o.controller)) {
                    return false;
                }
                // CR 702.16c: can't be enchanted by Auras with the protected quality.
                !aura_protection_applies(g, t, obj)
            } else if chars.has_subtype("Equipment") {
                // CR 301.5c: Equipment can be attached only to creatures, and an Equipment
                // that's also a creature can't equip one unless it has reconfigure;
                // 702.16d protection.
                (target.is_creature() || as_creature)
                    && (!o.is_creature() || o.has_keyword(KeywordKind::Reconfigure))
                    && !crate::kw::protection::prevents_attachment(g, t, obj)
                    && crate::keyword_impls::equip_restriction_ok(g, obj, t)
            } else if chars.has_subtype("Fortification") {
                // CR 301.6: Fortifications attach to lands; one that's also a creature can't
                // fortify a land.
                target.chars.is_land()
                    && !o.is_creature()
                    && !crate::kw::protection::prevents_attachment(g, t, obj)
            } else {
                false
            }
        }
    }
}

/// What kind of attachment a permanent entering the battlefield can have.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Aura,
    /// Equipment or Fortification.
    Equipment,
    Other,
}

/// Whether the object could be an Aura as it enters (a cheap check before computing it as
/// it would exist on the battlefield): any face of its card, what it's entering as a copy
/// of, or its current characteristics.
fn might_enter_as_aura(g: &Game, mv: &MoveEv) -> bool {
    let o = g.obj(mv.obj);
    o.chars.has_subtype("Aura")
        || o.card
            .as_ref()
            .is_some_and(|c| c.faces.iter().any(|f| f.chars.has_subtype("Aura")))
        || mv
            .etb
            .copy_of
            .is_some_and(|s| g.obj(s).copiable.has_subtype("Aura"))
}

/// Decides what a permanent entering the battlefield is attached to, adjusting the move
/// (CR 301.5e, 303.4f–i, 310.10). Returns false if it can't enter: an Aura with no legal
/// object or player to enchant, or one put onto the battlefield attached to something it
/// can't legally enchant or that's undefined, stays in its current zone (CR 303.4g,
/// 303.4i). An Aura spell resolving is attached to its target (CR 303.4a, 608.3).
pub(crate) fn entry_attachment(g: &mut Game, mv: &mut MoveEv) -> bool {
    let specified = mv.etb.attach_specified || mv.etb.attach_to.is_some();
    if !specified && !might_enter_as_aura(g, mv) {
        return true;
    }
    let o = g.obj(mv.obj);
    let aura_spell = o.zone == Zone::Stack
        && mv.cause == MoveCause::Resolve
        && o.chars.has_subtype("Aura")
        && !o.face_down;
    if aura_spell {
        return true;
    }
    let id = mv.obj;
    let target = mv.etb.attach_to;
    let (kind, legal, candidates) = g.with_hypothetical_entry(mv, |g| {
        let o = g.obj(id);
        let kind = if o.is(CardType::Battle) {
            EntryKind::Other
        } else if o.chars.has_subtype("Aura") && o.is(CardType::Enchantment) {
            EntryKind::Aura
        } else if o.chars.has_subtype("Equipment") || o.chars.has_subtype("Fortification") {
            EntryKind::Equipment
        } else {
            EntryKind::Other
        };
        let legal = target.is_some_and(|t| can_attach(g, id, t));
        let candidates: Vec<Entity> = if kind == EntryKind::Aura && !specified {
            g.permanent_ids()
                .into_iter()
                .map(Entity::Object)
                .chain(g.players_in_game().into_iter().map(Entity::Player))
                .filter(|e| can_attach(g, id, *e))
                .collect()
        } else {
            vec![]
        };
        (kind, legal, candidates)
    });
    match kind {
        // CR 303.4i: attached to something it can't enchant, or to something undefined.
        EntryKind::Aura if specified => legal,
        EntryKind::Aura => {
            // CR 303.4f: the player it enters under the control of chooses a legal object
            // or player to enchant; CR 303.4g: with none, it can't enter.
            let Some(first) = candidates.first().copied() else {
                return false;
            };
            let controller = g.entry_controller(mv);
            let chosen = g.ask_entities(
                controller,
                Some(id),
                "Choose what the Aura will enchant",
                candidates,
                1,
                1,
            );
            mv.etb.attach_to = Some(chosen.first().copied().unwrap_or(first));
            mv.etb.attach_specified = true;
            true
        }
        // CR 301.5e: an Equipment or Fortification that can't be attached to it enters
        // unattached.
        EntryKind::Equipment => {
            if !legal {
                mv.etb.attach_to = None;
            }
            true
        }
        // CR 303.4h, 310.10: other permanents (and battles) enter unattached.
        EntryKind::Other => {
            mv.etb.attach_to = None;
            true
        }
    }
}

/// CR 303.4g, 303.4i: an Aura that can't enter the battlefield stays in its current zone,
/// unless that zone is the stack: then it's put into its owner's graveyard instead. (A
/// resolving spell is put there by its resolution, CR 608.3e.) Returns the replaced events
/// of that move to the graveyard, if any.
pub(crate) fn aura_left_on_stack(g: &mut Game, mv: &MoveEv) -> Vec<ReplEvent> {
    let o = g.obj(mv.obj);
    if o.zone != Zone::Stack || mv.cause == MoveCause::Resolve {
        return vec![];
    }
    let owner = o.owner;
    g.replace(ReplEvent::Move(MoveEv {
        obj: mv.obj,
        to: Zone::Graveyard(owner),
        pos: LibraryPosition::Top,
        cause: mv.cause,
        by: mv.by,
        etb: Default::default(),
        source: mv.source,
    }))
}
