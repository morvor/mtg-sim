//! "You may play two additional lands on each of your turns" (Azusa, Lost but Seeking;
//! CR 305.2): like the core "an additional land" static, for any number. Effects allowing
//! additional land plays are cumulative (CR 305.2a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn additional_lands(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may play ")?;
    let (w, r) = r.split_once(" additional lands")?;
    if r != " on each of your turns" {
        return None;
    }
    let n = match w {
        "two" => 2,
        "three" => 3,
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

inventory::submit! { StaticPattern { name: "r305 N additional lands on each of your turns", priority: 100, parse: additional_lands } }
