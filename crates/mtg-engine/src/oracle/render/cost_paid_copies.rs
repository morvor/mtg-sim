//! "As an additional cost to cast ~, you may sacrifice one or more creatures. When you do,
//! copy ~ for each creature sacrificed this way." (Plumb the Forbidden): compiled as the
//! optional additional cost, paid any number of times, and a cast trigger that copies the
//! spell once for each time it was paid if it was paid at least once (see the
//! interpretation register, `cost-paid-reflexive-trigger`).

use super::*;

impl Renderer<'_> {
    pub(crate) fn sacrifice_cost_with_copies(
        &mut self,
        a: &Ability,
        next: &Ability,
    ) -> Option<String> {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            who: PlayerRel::You,
            change: CostChange::AdditionalCost(c),
        }) = &s.effect
        else {
            return None;
        };
        if s.condition.is_some() || c.mana.is_some() {
            return None;
        }
        let [CostPart::Repeated { cost, times }] = c.parts.as_slice() else {
            return None;
        };
        let Value::Custom(name) = times else {
            return None;
        };
        if !name.starts_with("spell cost:times paid:") || cost.mana.is_some() {
            return None;
        }
        let [CostPart::Sacrifice {
            filter,
            count: Value::Const(1),
        }] = cost.parts.as_slice()
        else {
            return None;
        };
        let AbilityKind::Triggered(t) = &next.kind else {
            return None;
        };
        let (
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            FunctionZone::Stack,
            Some(Condition::Compare(paid, Cmp::Gt, Value::Const(0))),
            Effect::CopySpell {
                what: Sel::This,
                count,
                new_targets: false,
            },
        ) = (&t.trigger, &t.zone, &t.intervening_if, &t.body.effect)
        else {
            return None;
        };
        let same = |v: &Value| matches!(v, Value::Custom(n) if n == name);
        if !same(paid)
            || !same(count)
            || !t.body.targets.is_empty()
            || t.body.modal.is_some()
            || t.once_per_turn
            || t.limit.is_some()
        {
            return None;
        }
        let f = effects::strip_controller(filter);
        let many = self.noun(&f, Num::Many);
        let one = self.noun(&f, Num::One);
        Some(format!(
            "As an additional cost to cast ~, you may sacrifice one or more {many}. When you do, copy ~ for each {one} sacrificed this way"
        ))
    }
}
