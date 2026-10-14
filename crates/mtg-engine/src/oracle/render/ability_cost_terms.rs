//! Cost changes of an ability that depend on what it targets (CR 601.2f, 602.2b: the
//! total cost is determined after targets are chosen): "This ability costs {1} less to
//! activate if it targets a creature with power 3 or less", "Equip {10}. This ability
//! costs {X} less to activate, where X is the power of the creature it targets". Also
//! numbers remembered as an ability resolves that are about its targets, said in a
//! delayed ability ("add X mana ..., where X is that spell's mana value").

use super::*;

/// The slots `Sel::Target(i)` a node mentions.
fn target_slots(dbg: &str) -> Vec<usize> {
    (0..8)
        .filter(|i| dbg.contains(&format!("Target({i})")))
        .collect()
}

impl Renderer<'_> {
    /// A remembered number, with the targets of an outer context when it's about those
    /// (the current context has fewer).
    pub(crate) fn outer_value(&mut self, v: &Value) -> String {
        let slots = target_slots(&format!("{v:?}"));
        let beyond = slots.iter().any(|i| *i >= self.targets.len());
        let Some((t, intro)) = self.target_stack.last().cloned().filter(|_| beyond) else {
            return self.value(v);
        };
        if slots.iter().any(|i| *i >= t.len()) {
            return self.value(v);
        }
        let saved_t = std::mem::replace(&mut self.targets, t);
        let saved_i = std::mem::replace(&mut self.introduced, intro);
        let s = self.value(v);
        self.targets = saved_t;
        self.introduced = saved_i;
        s
    }

    /// "it targets a creature with power 3 or less", "it targets two creatures you
    /// control": a condition on the ability's own targets.
    pub(crate) fn targets_condition(&mut self, c: &Condition) -> Option<String> {
        match c {
            Condition::SelMatches(Sel::Target(_), f) => {
                let n = self.noun_det(f, super::nouns::Det::A);
                Some(format!("it targets {n}"))
            }
            Condition::And(v) if !v.is_empty() => {
                let mut f0: Option<&Filter> = None;
                for (i, x) in v.iter().enumerate() {
                    let Condition::SelMatches(Sel::Target(t), f) = x else {
                        return None;
                    };
                    if *t as usize != i || f0.is_some_and(|g| format!("{g:?}") != format!("{f:?}"))
                    {
                        return None;
                    }
                    f0 = Some(f);
                }
                let n = self.noun(f0?, Num::Many);
                Some(format!("it targets {} {n}", number_word(v.len() as i32)))
            }
            _ => None,
        }
    }

    /// "Equip {10}. This ability costs {X} less to activate, where X is the power of the
    /// creature it targets.": a keyword ability and the change to its own cost, one line
    /// (the next ability, `next`, changes the cost of the keyword's abilities of this
    /// object).
    pub(crate) fn keyword_with_own_cost(&mut self, a: &Ability, next: &Ability) -> Option<String> {
        let AbilityKind::Keyword(k) = &a.kind else {
            return None;
        };
        let AbilityKind::Static(s) = &next.kind else {
            return None;
        };
        let StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ActivatedAbilities(scope),
            change: CostChange::ReduceGeneric(v),
            ..
        }) = &s.effect
        else {
            return None;
        };
        if s.condition.is_some()
            || !matches!(scope.sources, Filter::Source)
            || !matches!(scope.class, AbilityClass::Keyword(c) if c == k.kind)
            || target_slots(&format!("{v:?}")).is_empty()
        {
            return None;
        }
        let saved = self.targeted_noun.replace("the creature it targets");
        let (amt, tail) = match v {
            Value::PowerOf(_) => (
                "{X}".to_string(),
                ", where X is the power of the creature it targets".to_string(),
            ),
            Value::CountersOn(_, Some(c)) => (
                "{1}".to_string(),
                format!(" for each {} counter on the creature it targets", c),
            ),
            Value::DistinctAmong(Among::Colors, _) => (
                "{1}".to_string(),
                " for each color of the creature it targets".to_string(),
            ),
            other => {
                let w = self.value(other);
                ("{X}".to_string(), format!(", where X is {w}"))
            }
        };
        self.targeted_noun = saved;
        let line = self.ability(a);
        Some(format!(
            "{line}. This ability costs {amt} less to activate{tail}"
        ))
    }
}
