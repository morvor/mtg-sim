//! "Cast this spell only if [condition]" (Grim Wanderer's "Cast this spell only if a
//! creature died this turn"): a restriction on casting the card itself that functions
//! wherever the card could be cast from (CR 601.3, 604.6). Timing restrictions ("only
//! during combat") are parsed by other patterns.

use crate::ability::*;
use crate::oracle::patterns::{AbilityPattern, StaticPattern};
use crate::oracle::CompileContext;

fn cast_only_if(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l
        .strip_prefix("cast ~ only if ")
        .or_else(|| l.strip_prefix("cast this spell only if "))?;
    let cond = crate::oracle::statics::parse_condition(r, ctx)?;
    let mut s = StaticAbility::new(StaticEffect::CastOnlyIf(cond));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// The same on instants and sorceries, whose lines aren't parsed as static abilities.
fn cast_only_if_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    cast_only_if(lower.strip_suffix('.')?, t, ctx)
}

inventory::submit! { StaticPattern { name: "r601 cast only if a condition", priority: 110, parse: cast_only_if } }
inventory::submit! { AbilityPattern { name: "r601 cast only if a condition", priority: 110, parse: cast_only_if_block } }
