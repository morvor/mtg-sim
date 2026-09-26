//! CR 702.45 Bushido. Template for keyword implementations: a `KeywordRules` impl
//! registered with `inventory::submit!`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};

/// `Value::Custom`: the total of N over the bushido abilities of the object a continuous
/// effect is being applied to ("for each point of bushido it has").
pub const BUSHIDO_POINTS: &str = "bushido:points of bushido it has";

pub struct Bushido;

impl KeywordRules for Bushido {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Bushido]
    }

    /// CR 702.45a: "Bushido N" means "Whenever this creature blocks or becomes blocked,
    /// it gets +N/+N until end of turn."
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::BlocksOrBecomesBlocked(Filter::Source),
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(Value::c(n), Value::c(n))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            format!("Bushido {n}"),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != BUSHIDO_POINTS {
            return None;
        }
        let it = ctx.var_objects(vars::AFFECTED).first().copied();
        Some(it.map_or(0, |o| {
            g.obj(o)
                .chars
                .keywords()
                .filter(|k| k.kind == KeywordKind::Bushido)
                .map(|k| k.n.unwrap_or(0).max(0) as i64)
                .sum()
        }))
    }
}

inventory::submit! { KeywordRegistration(&Bushido) }
