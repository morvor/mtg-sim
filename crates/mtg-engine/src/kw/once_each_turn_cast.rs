//! "Once during each of your turns, you may cast a creature spell from your graveyard."
//! (Karador, Ghost Chieftain; Danitha, New Benalia's Light; Lurrus of the Dream-Den): a
//! permission to cast one spell with that quality from the graveyard each of its
//! controller's turns (CR 601.3). It's a static ability granting a
//! `StaticEffect::PlayPermission` whose condition is "it's your turn and you haven't used
//! it this turn" ([`ONCE_UNUSED`]). The spell is cast normally: its costs are paid, or an
//! alternative cost instead (CR 118.9); the usual timing rules apply.
//!
//! A spell cast from its controller's graveyard by a way the permission is needed for
//! uses the permission of the first such object that allows it: normally, or for an
//! alternative cost such as evoke — but not with a keyword that is its own permission to
//! cast the card from a graveyard ([`OWN_GRAVEYARD_PERMISSION`], e.g. flashback), nor
//! with a permission a resolved effect gave for that card ("You may cast that card this
//! turn").

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::CastMethod;
use crate::types::ObjectId;

/// `Condition::Custom`: this object's once-each-turn permission hasn't been used this turn.
pub const ONCE_UNUSED: &str = "once each turn: permission unused";

/// Keywords that let a card be cast from a graveyard by themselves (CR 702.34a, 702.81a,
/// 702.127a, 702.133a, 702.138a, 702.146a, 702.180a, 702.187a): a spell cast with one of
/// them doesn't use another permission. (Other alternative costs, such as evoke,
/// are paid for a spell cast with that permission.)
const OWN_GRAVEYARD_PERMISSION: &[KeywordKind] = &[
    KeywordKind::Flashback,
    KeywordKind::Retrace,
    KeywordKind::Aftermath,
    KeywordKind::JumpStart,
    KeywordKind::Escape,
    KeywordKind::Disturb,
    KeywordKind::Harmonize,
    KeywordKind::Mayhem,
];

/// Whether the static ability is a once-each-turn permission to cast spells from a
/// graveyard; its permission if so.
fn once_permission(s: &StaticAbility) -> Option<&PlayPermission> {
    fn has_marker(c: &Condition) -> bool {
        match c {
            Condition::Custom(n) => n == ONCE_UNUSED,
            Condition::And(v) => v.iter().any(has_marker),
            _ => false,
        }
    }
    match (&s.condition, &s.effect) {
        (Some(c), StaticEffect::PlayPermission(p)) if has_marker(c) => Some(p),
        _ => None,
    }
}

pub struct OnceEachTurnCast;

impl KeywordRules for OnceEachTurnCast {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != ONCE_UNUSED {
            return None;
        }
        let src = ctx.source?;
        Some(!g.history.once_permissions_used.contains(&src))
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::SpellCast {
            spell,
            player,
            from: Some(ZoneKind::Graveyard),
        } = ev
        else {
            return;
        };
        let (spell, p) = (*spell, *player);
        if !g.is_live(spell) {
            return;
        }
        // A keyword that is its own permission to cast the card from a graveyard
        // (flashback, escape, ...) doesn't use this one.
        let Some(cast) = g.obj(spell).stack.as_ref().map(|si| &si.cast) else {
            return;
        };
        if let CastMethod::Keyword(k) = cast.method {
            if OWN_GRAVEYARD_PERMISSION.contains(&k) {
                return;
            }
        }
        // Nor does a card an effect let its controller cast ("You may cast that card this
        // turn").
        if cast
            .paid
            .iter()
            .any(|x| x.as_str() == crate::casting::CAST_WITH_GRANT)
        {
            return;
        }
        let mut sources: Vec<ObjectId> = Vec::new();
        for id in g.battlefield.clone() {
            let o = g.obj(id);
            if o.controller != p || g.history.once_permissions_used.contains(&id) {
                continue;
            }
            let ctx = Ctx::new(Some(id), p);
            let allows = o.chars.abilities.iter().any(|a| match &a.kind {
                AbilityKind::Static(s) => once_permission(s).is_some_and(|perm| {
                    perm.spells
                        && g.matches(spell, &crate::casting::as_spell_filter(&perm.what), &ctx)
                }),
                _ => false,
            });
            if allows {
                sources.push(id);
            }
        }
        if let Some(first) = sources.first() {
            g.history.once_permissions_used.push(*first);
            g.dirty = true;
        }
    }
}

inventory::submit! { KeywordRegistration(&OnceEachTurnCast) }
