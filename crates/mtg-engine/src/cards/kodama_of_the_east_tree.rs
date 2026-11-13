//! Kodama of the East Tree: "Whenever another permanent you control enters, if it wasn't put
//! onto the battlefield with this ability, you may put a permanent card with equal or
//! lesser mana value from your hand onto the battlefield." The permanents this ability (of
//! this object) put onto the battlefield are recorded as it does.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const PUT: &str = "card:Kodama of the East Tree:permanents put onto the battlefield with this ability";
const NOT_PUT: &str = "card:Kodama of the East Tree:it wasn't put onto the battlefield with this ability";
const TEXT: &str = "Whenever another permanent you control enters, if it wasn't put onto the battlefield with this ability, you may put a permanent card with equal or lesser mana value from your hand onto the battlefield.";

fn record_after_move(e: Effect) -> Effect {
    match e {
        Effect::May { who, effect } => Effect::May {
            who,
            effect: Box::new(Effect::Seq(vec![*effect, Effect::Custom(PUT.into())])),
        },
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Kodama of the East Tree",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever another permanent you control enters, you may put a permanent card with equal or lesser mana value from your hand onto the battlefield.")
            .iter()
            .map(|a| {
                let a = map_effect(a, TEXT, record_after_move);
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.intervening_if = Some(Condition::Custom(NOT_PUT.into()));
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "\"if it wasn't put onto the battlefield with this ability\": the ability must remember what it put there; only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (Kodama, permanent) for what the move put onto the battlefield.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PUT {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let put: Vec<Entity> = ctx.vars.get(&vars::IT).cloned().unwrap_or_default();
        for e in put {
            if let Entity::Object(o) = e {
                g.cards.push(PUT, vec![i64::from(src.0), i64::from(o.0)]);
            }
        }
        true
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != NOT_PUT {
            return None;
        }
        let (Some(src), Some(o)) = (ctx.source, ctx.event.as_ref().and_then(|e| e.object)) else {
            return Some(true);
        };
        Some(
            !g.cards
                .get(PUT)
                .iter()
                .any(|r| r[0] == i64::from(src.0) && r[1] == i64::from(o.0)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
