//! Fireball: "~ deals X damage divided evenly, rounded down, among any number of targets."
//! The division happens as it resolves, among the targets still legal then (its cost
//! increase for each target beyond the first compiles).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const SHARE: &str = "card:Fireball:X divided evenly among the legal targets";
const TEXT: &str = "~ deals X damage divided evenly, rounded down, among any number of targets.";

inventory::submit! { ManualAbility {
    card: "Fireball",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "~ deals X damage to each of any number of targets.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::DealDamage { source, to, .. } => Effect::DealDamage {
                        source,
                        amount: Value::Custom(SHARE.into()),
                        to,
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "X damage divided evenly, rounded down, among the targets legal as it resolves: unique division",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_value(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != SHARE {
            return None;
        }
        // During resolution the slot holds only the targets still legal (CR 608.2b).
        let n = ctx.targets.first().map_or(0, Vec::len) as i64;
        Some(if n == 0 { 0 } else { i64::from(ctx.x.max(0)) / n })
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
