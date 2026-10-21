//! Autumn's Veil: "Spells you control can't be countered by blue or black spells this turn,
//! and creatures you control can't be the targets of blue or black spells this turn."
//! Both halves are rules for the rest of the turn about any spells and creatures you
//! control then (ruling). A blue or black spell's color is checked as it would counter
//! (ruling); abilities aren't affected.

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{ObjKind, Zone};
use crate::types::{Color, ObjectId};

const SHIELD: &str = "card:Autumn's Veil:spells you control can't be countered by blue or black spells this turn";
const TEXT: &str = "Spells you control can't be countered by blue or black spells this turn, and creatures you control can't be the targets of blue or black spells this turn.";

inventory::submit! { ManualAbility {
    card: "Autumn's Veil",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Creatures you control can't be the targets of blue or black spells this turn.")
            .iter()
            .map(|a| map_effect(a, TEXT, |e| Effect::Seq(vec![Effect::Custom(SHIELD.into()), e])))
            .collect()
    },
    reason: "\"can't be countered by blue or black spells\": the only restriction on what counters a spell",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, you).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != SHIELD {
            return false;
        }
        let turn = i64::from(g.turn.number);
        g.cards.retain(SHIELD, |r| r[0] == turn);
        g.cards.push(SHIELD, vec![turn, i64::from(ctx.controller.0)]);
        true
    }
    fn cant_be_countered_by(&self, g: &Game, id: ObjectId, cause: Option<ObjectId>) -> bool {
        let Some(c) = cause else {
            return false;
        };
        let spell = g.obj(id);
        if spell.kind == ObjKind::StackAbility {
            return false;
        }
        let counterer = g.obj(c);
        let is_spell = counterer.zone == Zone::Stack && counterer.kind != ObjKind::StackAbility;
        let blue_or_black = counterer.chars.colors.contains(Color::Blue)
            || counterer.chars.colors.contains(Color::Black);
        let turn = i64::from(g.turn.number);
        is_spell
            && blue_or_black
            && g.cards
                .get(SHIELD)
                .iter()
                .any(|r| r[0] == turn && r[1] == i64::from(spell.controller.0))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
