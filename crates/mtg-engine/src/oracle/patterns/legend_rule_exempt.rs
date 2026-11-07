//! "The "legend rule" doesn't apply." (Mirror Gallery), "The "legend rule" doesn't apply
//! to permanents you control." (Mirror Box), "The "legend rule" doesn't apply to tokens
//! you control." (Cadric, Soul Kindler): an exemption from the legend rule (CR 704.5j;
//! see `legend_rule.rs`).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn legend_rule_exempt(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l).trim();
    let r = l
        .strip_prefix("the \"legend rule\" doesn't apply")
        .or_else(|| l.strip_prefix("the “legend rule” doesn't apply"))
        .or_else(|| l.strip_prefix("the legend rule doesn't apply"))?;
    let affected = if r.is_empty() {
        Filter::Permanent
    } else {
        let (f, _, tail) = parse_object_phrase(r.strip_prefix(" to ")?)?;
        if !end(tail).trim().is_empty() {
            return None;
        }
        Filter::and(vec![Filter::Permanent, f])
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::LegendRuleExempt(affected))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "the legend rule doesn't apply", priority: 100, parse: legend_rule_exempt } }
