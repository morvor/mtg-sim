//! Irencrag Feat: "Add seven {R}. You can cast only one more spell this turn." The limit
//! counts from the moment it resolves, not from the start of the turn (ruling).

use super::{parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Characteristics;
use crate::types::{ObjectId, PlayerId};

const LIMIT: &str = "card:Irencrag Feat:you can cast only one more spell this turn";
const TEXT: &str = "Add seven {R}. You can cast only one more spell this turn.";

inventory::submit! { ManualAbility {
    card: "Irencrag Feat",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Add seven {R}.")
            .iter()
            .map(|a| {
                super::map_effect(a, TEXT, |e| Effect::Seq(vec![e, Effect::Custom(LIMIT.into())]))
            })
            .collect()
    },
    reason: "\"you can cast only one more spell this turn\": a limit counted from when it resolves; only this card",
} }

fn spells_cast(g: &Game, p: PlayerId) -> i64 {
    g.history.spells_cast.iter().filter(|(q, _)| *q == p).count() as i64
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != LIMIT {
            return false;
        }
        let p = ctx.controller;
        let turn = i64::from(g.turn.number);
        let row = vec![turn, i64::from(p.0), spells_cast(g, p)];
        g.cards.retain(LIMIT, |r| r[0] == turn);
        g.cards.push(LIMIT, row);
        true
    }
    fn cast_prohibited(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        _chars: &Characteristics,
    ) -> bool {
        let turn = i64::from(g.turn.number);
        let n = spells_cast(g, p);
        g.cards
            .get(LIMIT)
            .iter()
            .any(|r| r[0] == turn && r[1] == i64::from(p.0) && n > r[2])
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
