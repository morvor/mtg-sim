//! Two static abilities in one line, the second about the first's objects: "Creatures you
//! control of the chosen type get +1/+1. As long as you have the city's blessing, they
//! also have vigilance." (Radiant Destiny; CR 702.131 city's blessing).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::statics::parse_static;
use crate::oracle::CompileContext;

fn they_also_have(l: &str, _text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (first, second) = end(l).split_once(". as long as ")?;
    let (cond, rest) = second.split_once(", ")?;
    let what = rest.strip_prefix("they also have ")?;
    let (subject, _) = first
        .split_once(" get ")
        .or_else(|| first.split_once(" have "))?;
    let mut out = parse_static(first, ctx)?;
    out.extend(parse_static(
        &format!("as long as {cond}, {subject} have {what}"),
        ctx,
    )?);
    Some(out)
}

inventory::submit! { StaticPattern { name: "game rules: [static]. as long as [condition], they also have [ability]", priority: 100, parse: they_also_have } }

/// "As long as there are four or more card types among cards in your graveyard, each
/// opponent's maximum hand size is equal to seven minus the number of those card types."
/// (Winter, Misanthropic Guide; CR 402.2): "those card types" are the ones the condition
/// counts; the maximum is recomputed continuously (CR 611.3a).
fn max_hand_size_minus(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as long as ")?;
    let (cond_s, rest) = r.split_once(", ")?;
    let (who, r) = [
        ("each opponent's maximum hand size is equal to ", PlayerFilter::Opponent),
        ("your maximum hand size is equal to ", PlayerFilter::You),
    ]
    .into_iter()
    .find_map(|(p, w)| rest.strip_prefix(p).map(|r| (w, r)))?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    n.as_const()?;
    let r = r.trim_start().strip_prefix("minus ")?;
    const THOSE: &str = "the number of those card types";
    const AMONG: &str = "card types among cards in your graveyard";
    let r = if r == THOSE && cond_s.contains(AMONG) {
        format!("the number of {AMONG}")
    } else {
        r.to_string()
    };
    let cond = crate::oracle::statics::parse_condition(cond_s, ctx)?;
    let mut b = crate::oracle::effects::Builder::new(ctx);
    let (v, tail) = crate::oracle::statics::parse_value_phrase(&r, &mut b)?;
    if !end(&tail).is_empty() || !b.targets.is_empty() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::PlayerEffect {
        affected: who,
        effect: PlayerModification::MaxHandSize(Some(Value::Max(
            Box::new(Value::Diff(Box::new(n), Box::new(v))),
            Box::new(Value::c(0)),
        ))),
    });
    s.condition = Some(cond);
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "game rules: maximum hand size is N minus [value]", priority: 100, parse: max_hand_size_minus } }
