//! Homura, Human Ascendant // Homura's Essence: "When ~ dies, return it to the battlefield
//! flipped." The card enters the battlefield as a flipped permanent (CR 710.2): it has its
//! bottom half's characteristics from the start, under its owner's control.

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, LibraryPosition, Sel};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{FaceState, Zone};
use crate::replacement::{EtbInfo, MoveEv};

const RETURN_FLIPPED: &str = "card:Homura, Human Ascendant // Homura's Essence:return it to the battlefield flipped";
const TEXT: &str = "When ~ dies, return it to the battlefield flipped.";

inventory::submit! { ManualAbility {
    card: "Homura, Human Ascendant // Homura's Essence",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ dies, return it to the battlefield.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(RETURN_FLIPPED.into())))
            .collect()
    },
    reason: "returns a flip card to the battlefield already flipped (CR 710): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RETURN_FLIPPED {
            return false;
        }
        for card in g.resolve_objects(&Sel::TriggerObject, ctx) {
            // The card must still be the same object in its owner's graveyard (CR 400.7).
            let o = g.obj(card);
            if !g.is_live(card) || !matches!(o.zone, Zone::Graveyard(_)) {
                continue;
            }
            let owner = o.owner;
            if let Some(new) = g.move_object_ev(MoveEv {
                obj: card,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Return,
                by: Some(ctx.controller),
                etb: EtbInfo {
                    controller: Some(owner),
                    face: Some(FaceState::Flipped),
                    ..Default::default()
                },
                source: ctx.source,
            }) {
                if g.obj(new).zone == Zone::Battlefield && g.obj(new).face == FaceState::Flipped {
                    g.objects[new.0 as usize].flipped = true;
                    g.dirty = true;
                }
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
