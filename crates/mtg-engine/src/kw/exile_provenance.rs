//! Who exiled a card (round-4 card flow): "cards you exiled" (Haldan, Avid Arcanist),
//! "if it was exiled by an ability you controlled" (Evelyn, the Covetous), "cards they
//! exiled with ~" (Uba Mask, Shared Fate: each player's own set of the cards linked to the
//! source, CR 607.2a). The player who put a card into exile is the controller of the
//! effect that exiled it, or the player an effect has exile it ("that player exiles the
//! top card of their library instead"): kept on the object as `GameObject::moved_by`.
//! Also: "Each player may look at cards they exiled with ~" (CR 406.3), kept up to date
//! when state-based actions are checked, like `look_at_exiled_with.rs`.
//!
//! The text is parsed in `oracle/patterns/permissions_5.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, StaticEffect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// `Filter::Custom`: a card in exile that the evaluating player (the ability's controller)
/// exiled.
pub const EXILED_BY_YOU: &str = "a card in exile you exiled";
/// `Filter::Custom`: a card in exile that the player a permission is for
/// (`PlayerRel::Iterated`) exiled ("cards they exiled with ~").
pub const EXILED_BY_THEM: &str = "a card in exile the permitted player exiled";
/// `Filter::Custom`: a card put into exile this turn.
pub const EXILED_THIS_TURN: &str = "a card put into exile this turn";
/// `StaticEffect::Custom`: each player may look at the face-down cards linked to the
/// source that they exiled.
pub const EACH_MAY_LOOK_AT_OWN_EXILED: &str = "each player may look at cards they exiled with this";

/// Whether a `Filter::Custom` name is one of these (about where the card is and who put it
/// there, not a quality of the spell it becomes).
pub fn is_provenance(name: &str) -> bool {
    matches!(name, EXILED_BY_YOU | EXILED_BY_THEM | EXILED_THIS_TURN)
}

pub struct ExileProvenance;

impl KeywordRules for ExileProvenance {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let o = g.obj(id);
        let in_exile = o.zone == Zone::Exile;
        match name {
            EXILED_BY_YOU => Some(in_exile && o.moved_by == Some(ctx.controller)),
            EXILED_BY_THEM => Some(in_exile && o.moved_by.is_some() && o.moved_by == ctx.iter_player),
            EXILED_THIS_TURN => Some(in_exile && o.entered_turn == g.turn.number),
            _ => None,
        }
    }

    fn state_based_actions(&self, g: &mut Game) -> bool {
        let mut allow: Vec<(PlayerId, ObjectId)> = Vec::new();
        for o in g.permanents() {
            let has = o.chars.abilities.iter().any(|a| {
                matches!(&a.kind, AbilityKind::Static(s)
                    if matches!(&s.effect, StaticEffect::Custom(n) if n == EACH_MAY_LOOK_AT_OWN_EXILED))
            });
            if !has {
                continue;
            }
            for c in o.linked.values().flatten() {
                let c = g.current(*c);
                let e = g.obj(c);
                let Some(p) = e.moved_by else { continue };
                if e.zone == Zone::Exile && e.face_down && !crate::zones::may_look(g, p, c) {
                    allow.push((p, c));
                }
            }
        }
        for (p, c) in allow {
            crate::zones::allow_look(g, p, c);
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&ExileProvenance) }
