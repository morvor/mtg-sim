//! Mathemagics: "Target player draws 2ˣ cards."

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const POWER: &str = "card:Mathemagics:two to the X";
const TEXT: &str = "Target player draws 2ˣ cards.";

/// Draws past an empty library only matter as attempts (CR 121.4): the count saturates
/// here so a huge X doesn't loop for ever.
const MAX_EXPONENT: i32 = 16;

inventory::submit! { ManualAbility {
    card: "Mathemagics",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Target player draws X cards.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Draw { who, .. } => Effect::Draw {
                        who,
                        n: Value::Custom(POWER.into()),
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "draws two to the power of X cards: the only exponential value",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_value(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == POWER).then(|| 1i64 << ctx.x.clamp(0, MAX_EXPONENT))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
