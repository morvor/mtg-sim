//! CR 702.56 Replicate. "Replicate [cost]" means "As an additional cost to cast this
//! spell, you may pay [cost] any number of times" and "When you cast this spell, if a
//! replicate cost was paid for it, copy it for each time its replicate cost was paid. If
//! the spell has any targets, you may choose new targets for any of the copies."
//! (CR 702.56a). The cost is an optional additional cost (CR 601.2b, 601.2f–h).
//!
//! Each instance is paid separately and its trigger copies the spell for the payments
//! made for it (CR 702.56b): the `i`th replicate ability of a spell (counting from 1) is
//! recorded as `"replicate#i"` in `CastInfo::paid` once per payment, and the `i`th
//! "Replicate" triggered ability of the spell counts those.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Value::Custom`: the number of times the replicate cost of the resolving (or
/// triggering) replicate ability's instance was paid.
pub const TIMES_PAID: &str = "replicate:times its replicate cost was paid";

/// The name recorded in `CastInfo::paid` for a payment of the `i`th (0-based) replicate
/// cost of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("replicate#{}", i + 1))
}

/// A replicate cost "equal to its mana cost" (Hatchery Sliver): the mana cost of the
/// spell that has the keyword.
pub fn its_mana_cost() -> Cost {
    Cost::free().with(CostPart::PayManaCostOf(Box::new(Sel::This)))
}

fn is_its_mana_cost(c: &Cost) -> bool {
    c.mana.is_none()
        && matches!(c.parts.as_slice(), [CostPart::PayManaCostOf(s)] if matches!(**s, Sel::This))
}

pub struct Replicate;

impl KeywordRules for Replicate {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Replicate]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let times = Value::Custom(TIMES_PAID.into());
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            // Copies are made from the spell as it last existed on the stack if it has
            // left it.
            Body::effect(Effect::CopySpell {
                what: Sel::This,
                count: times.clone(),
                new_targets: true,
            }),
        );
        t.zone = FunctionZone::Stack;
        t.intervening_if = Some(Condition::Compare(times, Cmp::Gt, Value::c(0)));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Replicate.name(),
        )])
    }

    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        let chars = &g.obj(spell).chars;
        chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Replicate)
            .enumerate()
            .filter_map(|(i, k)| {
                let cost = k.cost.clone().unwrap_or_default();
                let cost = if is_its_mana_cost(&cost) {
                    // The spell's mana cost, {X} included: X has the value chosen for
                    // the spell (see Djinn Illuminatus). Without a mana cost, it can't be
                    // paid.
                    Cost::mana(chars.mana_cost.clone()?)
                } else {
                    cost
                };
                Some((cost_name(i), cost, true))
            })
            .collect()
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != TIMES_PAID {
            return None;
        }
        // Which of the spell's replicate triggered abilities this is.
        let index = ctx.source.and_then(|s| {
            g.obj(s)
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(a.kind, AbilityKind::Triggered(_))
                        && a.text == KeywordKind::Replicate.name()
                })
                .position(|a| a.uid == ctx.ability_uid)
        });
        let Some(i) = index else {
            return Some(0);
        };
        let paid = cost_name(i);
        Some(g.cast_info(ctx).map_or(0, |c| {
            c.paid.iter().filter(|p| **p == paid).count() as i64
        }))
    }
}

inventory::submit! { KeywordRegistration(&Replicate) }
