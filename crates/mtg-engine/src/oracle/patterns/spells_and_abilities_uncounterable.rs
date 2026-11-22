//! "Spells and abilities can't be countered." (Spider-Punk): nothing on the stack can be
//! countered while the ability applies (CR 701.6). Spells and abilities that would counter
//! them can still target them; they just don't counter them as they resolve.

use crate::ability::*;
use crate::oracle::patterns::StaticPattern;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn spells_and_abilities_cant_be_countered(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell() || end(l) != "spells and abilities can't be countered" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
            Restriction::CantBeCountered(Filter::Any),
        ))),
        text,
    )])
}

inventory::submit! {
    StaticPattern {
        name: "statics: spells and abilities can't be countered",
        priority: 50,
        parse: spells_and_abilities_cant_be_countered,
    }
}
