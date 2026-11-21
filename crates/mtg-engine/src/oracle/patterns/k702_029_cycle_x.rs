//! "When you cycle ~, [effect with X]" (Shark Typhoon: "When you cycle this card, create
//! an X/X blue Shark creature token with flying."): X is the X of the cycling ability's
//! cost (CR 107.3e, 702.29c), so the text defines X for the trigger's effect.

use super::AbilityPattern;
use crate::oracle::CompileContext;

fn cycle_x_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<crate::ability::Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let eff = lower.strip_prefix("when you cycle ~, ")?;
    if !eff
        .split(|c: char| !c.is_alphanumeric() && c != '/')
        .any(|w| w == "x" || w.starts_with("x/") || w.ends_with("/x"))
    {
        return None;
    }
    // Text that defines X itself ("you may pay {X}", "where X is ...") compiles as it is.
    if crate::oracle::triggers::parse_triggered(t, ctx).is_some() {
        return None;
    }
    let a = super::value_grammar::with_x_defined(true, || {
        crate::oracle::triggers::parse_triggered(t, ctx)
    })?;
    Some(vec![a])
}

inventory::submit! { AbilityPattern { name: "k702.29 when you cycle with x", priority: 60, parse: cycle_x_trigger } }
