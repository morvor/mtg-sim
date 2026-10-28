//! Copy grammar V (cards4-referents-statics, family `copies`).
//!
//! - "Enchanted creature is a copy of the chosen creature." (Metamorphic Alteration),
//!   "Equipped creature is a copy of the last chosen card." (Paleontologist's Pick-Axe //
//!   Dinosaur Headdress): a static ability's copy effect on the permanent the source is
//!   attached to (CR 707.2, 613.2a), copying the object the source's linked choice noted
//!   ([`Modification::CopyOf`], `static_copy.rs`).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// Whether the card notes a choice of one object for its other abilities: "As ~ enters,
/// choose a creature." or "..., choose an exiled creature card used to craft ~."
fn card_notes_one_choice(what: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    match what {
        "the chosen creature" => raw.contains(" enters, choose a creature.")
            || raw.contains(" enters, choose another creature"),
        "the last chosen card" => raw.contains(", choose an exiled creature card"),
        _ => false,
    }
}

fn attached_is_copy_of_chosen(l: &str, orig: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let rest = l
        .strip_prefix("enchanted creature is a copy of ")
        .or_else(|| l.strip_prefix("equipped creature is a copy of "))?;
    if !ctx.is_permanent() || !card_notes_one_choice(rest) {
        return None;
    }
    let effect = StaticEffect::Continuous {
        affected: Filter::AttachedToSource,
        mods: vec![Modification::CopyOf(Box::new(Sel::LinkedNoted))],
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        orig,
    )])
}

inventory::submit! { StaticPattern { name: "copies 5: enchanted/equipped creature is a copy of the chosen object", priority: 100, parse: attached_is_copy_of_chosen } }
