//! Exemptions from the legend rule (CR 704.5j) beyond `legend_rule_exempt.rs`: for a
//! duration ("The "legend rule" doesn't apply to permanents you control this turn.", Hall
//! of Echoes) and under a condition ("If there are exactly two permanents named ~ on the
//! battlefield, the "legend rule" doesn't apply to them.", Brothers Yamazaki).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn legend_rule(l: &str) -> Option<&str> {
    l.strip_prefix("the \"legend rule\" doesn't apply")
        .or_else(|| l.strip_prefix("the “legend rule” doesn't apply"))
        .or_else(|| l.strip_prefix("the legend rule doesn't apply"))
}

fn not_this_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    if legend_rule(end(l))? != " to permanents you control this turn" {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::Custom(crate::legend_rule::NOT_FOR_YOUR_PERMANENTS.into()),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "game rules: the legend rule doesn't apply to permanents you control this turn", priority: 100, parse: not_this_turn } }

fn exactly_two_named(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix(
        "if there are exactly two permanents named ~ on the battlefield, ",
    )?;
    if legend_rule(r)? != " to them" {
        return None;
    }
    let named = Filter::Named(ctx.card_name.into());
    let mut s = StaticAbility::new(StaticEffect::LegendRuleExempt(Filter::and(vec![
        Filter::Permanent,
        named.clone(),
    ])));
    s.condition = Some(Condition::Compare(
        Value::Count(Filter::and(vec![named, Filter::Permanent])),
        Cmp::Eq,
        Value::c(2),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "game rules: exactly two named ~, the legend rule doesn't apply to them", priority: 100, parse: exactly_two_named } }
