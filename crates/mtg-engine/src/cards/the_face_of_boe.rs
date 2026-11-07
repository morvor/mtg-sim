//! The Face of Boe: "{T}: You may cast a spell with suspend from your hand. If you do, pay
//! its suspend cost rather than its mana cost. Activate only as a sorcery." The spell is
//! cast as the ability resolves (ruling), for its suspend cost as an alternative cost
//! (CR 118.9).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::CastMethod;
use crate::types::ObjectId;

const CAST_FOR_SUSPEND: &str = "card:The Face of Boe:cast a spell with suspend for its suspend cost";
const TEXT: &str = "{T}: You may cast a spell with suspend from your hand. If you do, pay its suspend cost rather than its mana cost. Activate only as a sorcery.";

inventory::submit! { ManualAbility {
    card: "The Face of Boe",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}: Draw a card. Activate only as a sorcery.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(CAST_FOR_SUSPEND.into())))
            .collect()
    },
    reason: "casts a spell with suspend from hand for its suspend cost: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CAST_FOR_SUSPEND {
            return false;
        }
        let p = ctx.controller;
        let suspended: Vec<(ObjectId, crate::ability::Cost)> = g
            .player(p)
            .hand
            .iter()
            .filter_map(|c| {
                let kw = g
                    .obj(*c)
                    .chars
                    .keywords()
                    .find(|k| k.kind == KeywordKind::Suspend)?
                    .clone();
                Some((*c, kw.cost.unwrap_or_default()))
            })
            .collect();
        let cards: Vec<ObjectId> = suspended.iter().map(|(c, _)| *c).collect();
        let Some(card) = g
            .ask_objects(p, ctx.source, "Cast a spell with suspend for its suspend cost?", cards, 0, 1)
            .first()
            .copied()
        else {
            return true;
        };
        let cost = suspended
            .into_iter()
            .find(|(c, _)| *c == card)
            .map(|(_, cost)| cost)
            .unwrap_or_default();
        let saved = g.resolution_alt_cost.replace(cost);
        let _ = crate::casting::cast_during_resolution_as(g, p, card, CastMethod::Normal, None);
        g.resolution_alt_cost = saved;
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
