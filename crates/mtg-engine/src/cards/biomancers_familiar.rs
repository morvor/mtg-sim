//! Biomancer's Familiar: "{T}: The next time target creature adapts this turn, it adapts as
//! though it had no +1/+1 counters on it." It adds or removes no counters itself (ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const NEXT_ADAPT: &str = "card:Biomancer's Familiar:the next adapt ignores +1/+1 counters";
const TEXT: &str = "{T}: The next time target creature adapts this turn, it adapts as though it had no +1/+1 counters on it.";

inventory::submit! { ManualAbility {
    card: "Biomancer's Familiar",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}: Tap target creature.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(NEXT_ADAPT.into())))
            .collect()
    },
    reason: "the next time the creature adapts this turn, it ignores its +1/+1 counters: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, creature).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != NEXT_ADAPT {
            return false;
        }
        let turn = i64::from(g.turn.number);
        g.cards.retain(NEXT_ADAPT, |r| r[0] == turn);
        if let Some(Entity::Object(c)) = ctx.targets.first().and_then(|v| v.first()) {
            g.cards.push(NEXT_ADAPT, vec![turn, i64::from(c.0)]);
        }
        true
    }
    fn adapts_ignoring_counters(&self, g: &mut Game, obj: ObjectId) -> bool {
        let turn = i64::from(g.turn.number);
        let row = vec![turn, i64::from(obj.0)];
        if !g.cards.get(NEXT_ADAPT).contains(&row) {
            return false;
        }
        // Used up (each such effect applies to that adapt).
        g.cards.retain(NEXT_ADAPT, |r| *r != row);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
