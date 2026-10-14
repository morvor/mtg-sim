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
