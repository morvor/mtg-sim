//! "This spell can't be copied." (Display of Power, See Double) and "This ability can't be
//! copied." (Gogo, Master of Mimicry): copy effects don't copy them (CR 707.10). An
//! ability that states a spell can't be copied functions on the stack (CR 113.6g); the
//! one about an ability is a static ability linked to it (see `copy::cant_be_copied`).

use super::AbilityPattern;
use crate::ability::*;
use crate::copy::{ABILITY_CANT_BE_COPIED, CANT_BE_COPIED};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use std::sync::Arc;

/// "~ can't be copied." on an instant or sorcery.
fn spell_cant_be_copied(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    if !ctx.is_spell() || end(&text.to_lowercase()) != "~ can't be copied" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(CANT_BE_COPIED.into()));
    s.zone = FunctionZone::Stack;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "r707 this spell can't be copied", priority: 50, parse: spell_cant_be_copied } }

/// The `link` shared by an ability and its "This ability can't be copied."
const UNCOPIABLE_LINK: u16 = 0x7c0;

/// "[cost]: [effect]. This ability can't be copied[ and X can't be 0]." — the ability
/// without that sentence, linked to a static ability saying it can't be copied.
fn ability_cant_be_copied(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_ascii_lowercase();
    let (sentence, keep) = [
        (" this ability can't be copied and x can't be 0.", " X can't be 0."),
        (" this ability can't be copied.", ""),
    ]
    .into_iter()
    .find(|(s, _)| lower.contains(s))?;
    let at = lower.find(sentence)?;
    let rewritten = format!("{}{keep}{}", &text[..at], &text[at + sentence.len()..]);
    let mut out = crate::oracle::parse_ability(&rewritten, ctx)?;
    let [a] = &mut out[..] else {
        return None;
    };
    if !matches!(a.kind, AbilityKind::Activated(_) | AbilityKind::Triggered(_)) || a.link != 0 {
        return None;
    }
    let a = Arc::make_mut(a);
    a.text = text.to_string();
    a.link = UNCOPIABLE_LINK;
    out.push(AbilityDef::with_link(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            ABILITY_CANT_BE_COPIED.into(),
        ))),
        "This ability can't be copied.",
        UNCOPIABLE_LINK,
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "r707 this ability can't be copied", priority: 50, parse: ability_cant_be_copied } }
