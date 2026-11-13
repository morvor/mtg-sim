//! "If [condition], ~ can't be countered." on an instant or sorcery (Exquisite Firecraft's
//! spell mastery): a static ability that functions while the spell is on the stack, as
//! long as the condition holds. The condition is checked when something tries to counter
//! the spell; a card counts in a zone only if it's there then (the spell itself is still
//! on the stack, not in its owner's graveyard).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn conditional_cant_be_countered(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let text = block.trim();
    let lower = text.to_lowercase();
    let c = end(&lower)
        .strip_prefix("if ")?
        .strip_suffix(", ~ can't be countered")?;
    let cond = crate::oracle::statics::parse_condition(c, ctx)?;
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantBeCountered(
        Filter::Source,
    )));
    s.zone = FunctionZone::Stack;
    s.condition = Some(cond);
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! {
    AbilityPattern {
        name: "if [condition], ~ can't be countered",
        priority: 50,
        parse: conditional_cant_be_countered,
    }
}
