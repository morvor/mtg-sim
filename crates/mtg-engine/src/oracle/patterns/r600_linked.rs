//! Oracle patterns for static abilities linked to triggered abilities printed in the same
//! paragraph (CR 603.11, 607.2h): "You may exert this creature as it attacks. When you do,
//! [effect]."

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::exert::{EXERTED, EXERT_AS_ATTACKS};
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::CompileContext;

/// The link shared by an exert static ability and its "when you do" triggers.
const EXERT_LINK: u16 = 0x4001;

fn exert_paragraph(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let r = lower
        .strip_prefix("you may exert ~ as it attacks.")
        .or_else(|| lower.strip_prefix("you may exert ~ as he attacks."))
        .or_else(|| lower.strip_prefix("you may exert ~ as she attacks."))?
        .trim();
    let mut out = vec![AbilityDef::with_link(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            EXERT_AS_ATTACKS.into(),
        ))),
        t,
        EXERT_LINK,
    )];
    if r.is_empty() {
        return Some(out);
    }
    let effect_text = r.strip_prefix("when you do, ")?;
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    let effect = parse_effect_text(effect_text, &mut b)?;
    let body = Body {
        targets: b.targets,
        effect,
        modal: None,
    };
    out.push(AbilityDef::with_link(
        AbilityKind::Triggered(TriggeredAbility::new(
            TriggerCond::Custom(EXERTED.into()),
            body,
        )),
        t,
        EXERT_LINK,
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "exert as it attacks", priority: 0, parse: exert_paragraph } }
