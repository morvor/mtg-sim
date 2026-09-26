//! CR 702.78 Conspire: "As an additional cost to cast this spell, you may tap two untapped
//! creatures you control that each share a color with it" and "When you cast this spell,
//! if its conspire cost was paid, copy it. If the spell has any targets, you may choose new
//! targets for the copy." (CR 702.78a). Paying the conspire cost follows the rules for
//! additional costs (CR 601.2b, 601.2f–h).
//!
//! Each instance is paid separately and triggers based on its own payment (CR 702.78b):
//! paying the `i`th conspire cost of a spell (counting from 1) is recorded as
//! `"conspire#i"` in `CastInfo::paid`, and the `i`th "Conspire" triggered ability of the
//! spell checks it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: the conspire cost of the resolving (or triggering) conspire
/// ability's instance was paid.
pub const PAID: &str = "conspire:its conspire cost was paid";

/// The name recorded in `CastInfo::paid` for the `i`th (0-based) conspire cost of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("conspire#{}", i + 1))
}

/// "Tap two untapped creatures you control that each share a color with it."
pub fn conspire_cost() -> Cost {
    Cost::free().with(CostPart::TapUntapped {
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::ControlledBy(PlayerRel::You),
            Filter::SharesColor(Box::new(Sel::This)),
        ]),
        count: Value::c(2),
    })
}

pub struct Conspire;

impl KeywordRules for Conspire {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Conspire]
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
            KeywordKind::Conspire.name(),
        )])
    }

    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Conspire)
            .enumerate()
            .map(|(i, _)| (cost_name(i), conspire_cost(), false))
            .collect()
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != PAID {
            return None;
        }
        // Which of the spell's conspire triggered abilities this is.
        let index = ctx.source.and_then(|s| {
            g.obj(s)
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(a.kind, AbilityKind::Triggered(_))
                        && a.text == KeywordKind::Conspire.name()
                })
                .position(|a| a.uid == ctx.ability_uid)
        });
        let Some(i) = index else {
            return Some(false);
        };
        let paid = cost_name(i);
        Some(
            g.cast_info(ctx)
                .is_some_and(|c| c.paid.iter().any(|p| *p == paid)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Conspire) }
