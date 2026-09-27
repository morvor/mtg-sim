//! CR 702.157 Squad: "Squad [cost]" means "As an additional cost to cast this spell, you
//! may pay [cost] any number of times" and "When this creature enters, if its squad cost
//! was paid, create a token that's a copy of it for each time its squad cost was paid."
//! (CR 702.157a). The cost is an optional additional cost (CR 601.2b, 601.2f–h); the
//! permanent's enters ability sees how its spell was cast (its `GameObject::cast`).
//!
//! Each instance is paid separately, and each triggered ability counts only the payments
//! made for its own instance (CR 702.157b): a payment of the `i`th squad cost of a spell
//! (counting from 1) is recorded as `"squad#i"` in `CastInfo::paid`, and the `i`th "Squad"
//! triggered ability of the permanent counts those.
//!
//! The tokens are copies of the permanent as it last existed on the battlefield if it has
//! left; they weren't cast, so their own squad abilities don't trigger.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Value::Custom`: the number of times the squad cost of the resolving (or triggering)
/// squad ability's instance was paid.
pub const TIMES_PAID: &str = "squad:times its squad cost was paid";

/// The name recorded in `CastInfo::paid` for a payment of the `i`th (0-based) squad cost
/// of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("squad#{}", i + 1))
}

pub struct Squad;

impl KeywordRules for Squad {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Squad]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let times = Value::Custom(TIMES_PAID.into());
        let mut t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::CreateTokenCopy {
                of: Sel::This,
                count: times.clone(),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
                mods: vec![],
            }),
        );
        t.intervening_if = Some(Condition::Compare(times, Cmp::Gt, Value::c(0)));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Squad.name(),
        )])
    }

    /// One repeatable optional additional cost per instance (CR 702.157b).
    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Squad)
            .enumerate()
            .filter_map(|(i, k)| Some((cost_name(i), k.cost.clone()?, true)))
            .collect()
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != TIMES_PAID {
            return None;
        }
        // Which of the permanent's squad triggered abilities this is.
        let index = ctx.source.and_then(|s| {
            g.obj(s)
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(a.kind, AbilityKind::Triggered(_))
                        && a.text == KeywordKind::Squad.name()
                })
                .position(|a| a.uid == ctx.ability_uid)
        });
        let Some(i) = index else {
            return Some(0);
        };
        let paid = cost_name(i);
        Some(
            g.cast_info(ctx)
                .map_or(0, |c| c.paid.iter().filter(|p| **p == paid).count() as i64),
        )
    }
}

inventory::submit! { KeywordRegistration(&Squad) }
