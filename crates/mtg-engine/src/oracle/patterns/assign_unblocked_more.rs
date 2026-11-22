//! "You may have ~ assign his combat damage as though he weren't blocked." (Wolverine,
//! Claws Out): a character's own pronoun in Thorn Elemental's ability (CR 510.1c; see
//! `kw::assign_as_though_unblocked`). (Gurzigost's "... this turn ..." is chosen as its
//! ability resolves, which that hook doesn't model: not understood.)

use super::StaticPattern;
use crate::ability::*;
use crate::kw::assign_as_though_unblocked::MAY_ASSIGN_UNBLOCKED;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn pronoun_wording(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell()
        || !matches!(
            end(l).trim(),
            "you may have ~ assign his combat damage as though he weren't blocked"
                | "you may have ~ assign her combat damage as though she weren't blocked"
                | "you may have ~ assign their combat damage as though they weren't blocked"
        )
    {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            MAY_ASSIGN_UNBLOCKED.into(),
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "may have ~ assign his/her combat damage as though unblocked", priority: 120, parse: pronoun_wording } }
