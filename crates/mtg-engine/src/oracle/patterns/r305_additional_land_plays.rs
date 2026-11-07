//! Oracle patterns for additional land plays (CR 305.2): "You may play two additional
//! lands on each of your turns." (Azusa, Lost but Seeking). The core static parser
//! handles "an additional land".

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

inventory::submit! { StaticPattern { name: "N additional lands on each of your turns", priority: 100, parse: additional_lands_each_turn } }

/// "you may play N additional lands on each of your turns" (N a number word or digits).
fn additional_lands_each_turn(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may play ")?;
    let w = r.strip_suffix(" additional lands on each of your turns")?;
    let n: u32 = match w {
        "two" => 2,
        "three" => 3,
        "four" => 4,
        _ => w.parse().ok()?,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalLandPlays(
            PlayerRel::You,
            n,
        ))),
        text,
    )])
}
