//! Referent grammar VI (round 5):
//!
//! * "Activated abilities of the chosen permanent can't be activated." (Scheming Fence):
//!   the permanent chosen as ~ entered (CR 607.2d, see `choice_grammar_etb`) is the
//!   static ability's object. A new object after it changes zones is no longer the chosen
//!   one (CR 400.7), and with nothing chosen the ability does nothing.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "Activated abilities of the chosen permanent can't be activated."
fn chosen_cant_activate(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("activated abilities of the chosen ")?;
    let noun = r.strip_suffix(" can't be activated")?;
    if !matches!(noun, "permanent" | "creature")
        || !ctx.is_permanent()
        || !super::choice_grammar_etb::card_chooses_object()
    {
        return None;
    }
    let r = Restriction::CantActivate {
        who: PlayerFilter::Any,
        sources: Filter::In(Box::new(Sel::LinkedNoted)),
        include_mana: true,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "referents 5: activated abilities of the chosen permanent can't be activated", priority: 40, parse: chosen_cant_activate } }
