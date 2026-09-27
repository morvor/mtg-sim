//! CR 702.175 Offspring: "Offspring [cost]" means "You may pay an additional [cost] as you
//! cast this spell" and "When this permanent enters, if its offspring cost was paid,
//! create a token that's a copy of it, except it's 1/1." (CR 702.175a). The cost is an
//! optional additional cost (CR 601.2b, 601.2f–h).
//!
//! Each instance is paid separately and its trigger looks only at the payment made for it
//! (CR 702.175b): the `i`th offspring ability of a spell (counting from 1) is recorded as
//! `"offspring#i"` in `CastInfo::paid`, and the `i`th "Offspring" triggered ability of the
//! permanent checks that name. The permanent sees how the spell it was cast as was paid
//! for (CR 400.7d); a token copy of it wasn't cast, so its own offspring abilities don't
//! trigger.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: the offspring cost of the resolving (or triggering) offspring
/// ability's instance was paid.
pub const PAID: &str = "offspring:its offspring cost was paid";

/// The name recorded in `CastInfo::paid` for the payment of the `i`th (0-based) offspring
/// cost of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("offspring#{}", i + 1))
}

pub struct Offspring;

impl KeywordRules for Offspring {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Offspring]
    }

    /// "When this permanent enters, if its offspring cost was paid, create a token that's
    /// a copy of it, except it's 1/1."
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::CreateTokenCopy {
                of: Sel::This,
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
                mods: vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))],
            }),
        );
        t.intervening_if = Some(Condition::Custom(PAID.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Offspring.name(),
        )])
    }

    /// "You may pay an additional [cost] as you cast this spell", once for each instance
    /// (CR 702.175b).
    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Offspring)
            .enumerate()
            .map(|(i, k)| (cost_name(i), k.cost.clone().unwrap_or_default(), false))
            .collect()
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != PAID {
            return None;
        }
        // Which of the permanent's offspring triggered abilities this is.
        let index = ctx.source.and_then(|s| {
            g.obj(s)
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(a.kind, AbilityKind::Triggered(_))
                        && a.text == KeywordKind::Offspring.name()
                })
                .position(|a| a.uid == ctx.ability_uid)
        });
        let Some(i) = index else {
            return Some(false);
        };
        let paid = cost_name(i);
        Some(
            g.cast_info(ctx)
                .is_some_and(|c| c.was_cast && c.paid.iter().any(|p| *p == paid)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Offspring) }
