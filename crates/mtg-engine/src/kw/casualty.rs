//! CR 702.153 Casualty: "Casualty N" means "As an additional cost to cast this spell, you
//! may sacrifice a creature with power N or greater," and "When you cast this spell, if a
//! casualty cost was paid for it, copy it. If the spell has any targets, you may choose new
//! targets for the copy." (CR 702.153a). Paying the casualty cost follows the rules for
//! additional costs (CR 601.2b, 601.2f–h): one creature is sacrificed, and the spell is
//! copied once. The copy is created on the stack, not cast (CR 707.10), and it resolves
//! first.
//!
//! Each instance is paid separately and triggers based on its own payment (CR 702.153b):
//! paying the `i`th casualty cost of a spell (counting from 1) is recorded as
//! `"casualty#i"` in `CastInfo::paid`, and the `i`th "Casualty" triggered ability of the
//! spell checks it. If the spell loses that instance before its triggered ability resolves
//! (e.g. it loses all abilities), the ability still copies it (CR 113.7a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: the casualty cost of the resolving (or triggering) casualty
/// ability's instance was paid.
pub const PAID: &str = "casualty:a casualty cost was paid for it";

/// The name recorded in `CastInfo::paid` for the `i`th (0-based) casualty cost of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("casualty#{}", i + 1))
}

/// "Sacrifice a creature with power N or greater."
pub fn casualty_cost(n: i32) -> Cost {
    Cost::free().with(CostPart::Sacrifice {
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::Power(Cmp::Ge, Box::new(Value::c(n))),
        ]),
        count: Value::c(1),
    })
}

pub struct Casualty;

impl KeywordRules for Casualty {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Casualty]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            // Copied from the spell as it last existed on the stack if it has left it.
            Body::effect(Effect::CopySpell {
                what: Sel::This,
                count: Value::c(1),
                new_targets: true,
            }),
        );
        t.zone = FunctionZone::Stack;
        t.intervening_if = Some(Condition::Custom(PAID.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Casualty.name(),
        )])
    }

    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Casualty)
            .enumerate()
            // "Casualty X" (a variable N) isn't offered.
            .filter_map(|(i, k)| {
                let n = k.n.filter(|n| *n >= 0)?;
                Some((cost_name(i), casualty_cost(n), false))
            })
            .collect()
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != PAID {
            return None;
        }
        // Which of the spell's casualty triggered abilities this is.
        let index = ctx.source.and_then(|s| {
            g.obj(s)
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(a.kind, AbilityKind::Triggered(_))
                        && a.text == KeywordKind::Casualty.name()
                })
                .position(|a| a.uid == ctx.ability_uid)
        });
        let Some(i) = index else {
            // The spell no longer has this ability (e.g. it lost all abilities). Once
            // triggered, the ability exists independently of its source (CR 113.7a); it
            // triggered because this instance's casualty cost was paid, and that doesn't
            // change: as it resolves (CR 603.4), it still was.
            return Some(ctx.stack_obj.is_some());
        };
        let paid = cost_name(i);
        Some(
            g.cast_info(ctx)
                .is_some_and(|c| c.paid.iter().any(|p| *p == paid)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Casualty) }
