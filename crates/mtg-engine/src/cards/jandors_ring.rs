//! Jandor's Ring: "{2}, {T}, Discard the last card you drew this turn: Draw a card."
//!
//! The cost can be paid only while that card is still in your hand (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const LAST_DRAWN: &str = "card:Jandor's Ring:the last card you drew this turn";
/// One row per player: `[player, card, turn]`, the last card they drew.
const DRAWN: &str = "card:Jandor's Ring:last card drawn";
const TEXT: &str = "{2}, {T}, Discard the last card you drew this turn: Draw a card.";

inventory::submit! { ManualAbility {
    card: "Jandor's Ring",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{2}, {T}, Discard a card: Draw a card.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Activated(act) = &mut kind {
                    for part in &mut act.cost.parts {
                        if let CostPart::Discard { filter, .. } = part {
                            *filter = Filter::Custom(LAST_DRAWN.into());
                        }
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "cost: discard the last card you drew this turn: unique cost tracking",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::Drew { player, card, .. } = ev {
            let p = player.0 as i64;
            let turn = g.turn.number as i64;
            g.cards.retain(DRAWN, |r| r[0] != p);
            g.cards.push(DRAWN, vec![p, card.0 as i64, turn]);
        }
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != LAST_DRAWN {
            return None;
        }
        let turn = g.turn.number as i64;
        Some(g.cards.get(DRAWN).iter().any(|r| {
            r[0] == ctx.controller.0 as i64 && r[2] == turn && r[1] == id.0 as i64
        }))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
