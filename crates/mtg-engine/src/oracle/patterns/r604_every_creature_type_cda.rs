//! "~ is every creature type (even if this card isn't on the battlefield)." (Mistform
//! Ultimus): a characteristic-defining ability (CR 604.3) — it applies in layer 4
//! (CR 613.1d) in every zone (CR 604.3), and, as a CDA, before other type-changing effects
//! (CR 613.3), while effects that remove abilities in layer 6 don't undo it (CR 613.6).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn every_creature_type_cda(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = block.trim().to_lowercase();
    // (The parenthetical is removed as reminder text, leaving "~ is every creature type .".)
    let l = l.trim_end_matches(['.', ' ']);
    let l = l
        .strip_suffix(" (even if this card isn't on the battlefield)")
        .unwrap_or(l)
        .trim_end();
    if l != "~ is every creature type" && l != "~ is all creature types" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::AllCreatureTypes],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block.trim())])
}

inventory::submit! { AbilityPattern { name: "r604.3: ~ is every creature type (a CDA)", priority: 100, parse: every_creature_type_cda } }
