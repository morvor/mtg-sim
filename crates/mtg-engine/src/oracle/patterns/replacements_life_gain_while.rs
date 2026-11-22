//! "If you would gain life while you have 5 or less life, you gain twice that much life
//! instead." (Phial of Galadriel; CR 614.1a): a life gain replacement effect that applies
//! only while its controller's life total is at most N.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

fn gain_life_while(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if you would gain life while you have ")?;
    let (n, r) = parse_number(r)?;
    if !matches!(n, Value::Const(_)) {
        return None;
    }
    let r = r.trim_start().strip_prefix("or less life, ")?;
    let (event, action) = super::counters_resources_replace::life_gain_replacement(&format!(
        "if you would gain life, {r}"
    ))?;
    let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event,
        action,
        self_replacement: false,
        optional: false,
    }));
    s.condition = Some(Condition::Compare(
        Value::LifeTotal(PlayerRef::You),
        Cmp::Le,
        n,
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "replacements: if you would gain life while you have N or less life", priority: 100, parse: gain_life_while } }
