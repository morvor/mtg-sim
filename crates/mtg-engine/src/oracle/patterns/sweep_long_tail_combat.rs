//! Combat long tail:
//! - "You may have ~ assign its combat damage as though it weren't blocked." (Thorn
//!   Elemental, Pride of Lions), implemented by the `kw::assign_as_though_unblocked` hook
//!   (CR 510.1c).
//! - "Remove [creature] from combat" (CR 506.4): "untap it and remove it from combat"
//!   (Gustcloak Runner), "remove target attacking or blocking creature from combat".

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::kw::assign_as_though_unblocked::MAY_ASSIGN_UNBLOCKED;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

inventory::submit! {
    StaticPattern { name: "sweep: may assign combat damage as though unblocked", priority: 50, parse: may_assign_unblocked }
}
inventory::submit! {
    EffectPattern { name: "sweep: remove from combat", priority: 95, parse: remove_from_combat }
}

fn may_assign_unblocked(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell()
        || end(l) != "you may have ~ assign its combat damage as though it weren't blocked"
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

/// "remove it from combat", "remove target attacking or blocking creature from combat".
fn remove_from_combat(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove ")?;
    let (what, tail) = object_ref(r, b)?;
    if tail.trim() != "from combat" {
        return None;
    }
    Some(Effect::RemoveFromCombat { what })
}
