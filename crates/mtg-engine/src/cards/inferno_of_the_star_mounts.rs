//! Inferno of the Star Mounts: "{R}: ~ gets +1/+0 until end of turn. When its power becomes
//! 20 this way, it deals 20 damage to any target." The reflexive ability triggers only if
//! this ability makes its power exactly 20 (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const BEFORE: &str = "card:Inferno of the Star Mounts:remember its power";
const POWER_BEFORE: Var = vars::USER + 4430;
const TEXT: &str = "{R}: ~ gets +1/+0 until end of turn. When its power becomes 20 this way, it deals 20 damage to any target.";

fn became_twenty(e: Effect) -> Effect {
    match e {
        Effect::Seq(v) => {
            let mut out = vec![Effect::Custom(BEFORE.into())];
            for e in v {
                out.push(match e {
                    Effect::If { then, otherwise, .. } => Effect::If {
                        cond: Condition::And(vec![
                            Condition::Compare(
                                Value::PowerOf(Box::new(Sel::This)),
                                Cmp::Eq,
                                Value::Const(20),
                            ),
                            Condition::Compare(Value::Var(POWER_BEFORE), Cmp::Ne, Value::Const(20)),
                        ]),
                        then,
                        otherwise,
                    },
                    e => e,
                });
            }
            Effect::Seq(out)
        }
        e => e,
    }
}

inventory::submit! { KeywordRegistration(&Rules) }

inventory::submit! { ManualAbility {
    card: "Inferno of the Star Mounts",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{R}: ~ gets +1/+0 until end of turn. When you do, ~ deals 20 damage to any target.")
            .iter()
            .map(|a| map_effect(a, TEXT, became_twenty))
            .collect()
    },
    reason: "a reflexive trigger only when this ability makes its power exactly 20: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != BEFORE {
            return false;
        }
        if g.dirty {
            g.recompute();
        }
        let p = ctx
            .source
            .filter(|s| g.is_live(*s))
            .map_or(0, |s| g.obj(s).power());
        ctx.nums.insert(POWER_BEFORE, i64::from(p));
        true
    }
}
