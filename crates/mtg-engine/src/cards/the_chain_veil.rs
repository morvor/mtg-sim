//! The Chain Veil: "{4}, {T}: For each planeswalker you control, you may activate one of
//! its loyalty abilities once this turn as though none of its loyalty abilities have been
//! activated this turn." A rule for the rest of the turn: planeswalkers that come under
//! your control later count too, and each resolution adds one more activation (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, ObjectId, PlayerId};

const ONE_MORE: &str = "card:The Chain Veil:one more loyalty activation this turn";
const TEXT: &str = "{4}, {T}: For each planeswalker you control, you may activate one of its loyalty abilities once this turn as though none of its loyalty abilities have been activated this turn.";

inventory::submit! { ManualAbility {
    card: "The Chain Veil",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{4}, {T}: Draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(ONE_MORE.into())))
            .collect()
    },
    reason: "lets each planeswalker you control activate one more loyalty ability this turn: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, you).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != ONE_MORE {
            return false;
        }
        let turn = i64::from(g.turn.number);
        g.cards.retain(ONE_MORE, |r| r[0] == turn);
        g.cards.push(ONE_MORE, vec![turn, i64::from(ctx.controller.0)]);
        true
    }
    fn extra_loyalty_activations(&self, g: &Game, p: PlayerId, src: ObjectId) -> u32 {
        if !g.obj(src).is(CardType::Planeswalker) || g.obj(src).controller != p {
            return 0;
        }
        let turn = i64::from(g.turn.number);
        g.cards
            .get(ONE_MORE)
            .iter()
            .filter(|r| r[0] == turn && r[1] == i64::from(p.0))
            .count() as u32
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
