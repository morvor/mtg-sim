//! Name-changing statics: "has all names of nonlegendary creature cards in addition to
//! its name" (Spy Kit, CR 612.7).

use crate::ability::*;
use crate::oracle::patterns::AbilityPattern;
use crate::oracle::statics::parse_static;
use crate::oracle::CompileContext;

const ALL_NAMES: &str = " and has all names of nonlegendary creature cards in addition to its name";

fn all_creature_names(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let head = lower.trim_end_matches('.').strip_suffix(ALL_NAMES)?;
    let mut abilities = parse_static(head, ctx)?;
    for a in abilities.iter_mut() {
        if let AbilityKind::Static(s) = &a.kind {
            if let StaticEffect::Continuous { affected, mods } = &s.effect {
                let mut s2 = s.clone();
                let mut mods = mods.clone();
                mods.push(Modification::AllCreatureNames);
                s2.effect = StaticEffect::Continuous {
                    affected: affected.clone(),
                    mods,
                };
                *a = AbilityDef::new(AbilityKind::Static(s2), text);
                return Some(abilities);
            }
        }
    }
    None
}

inventory::submit! { AbilityPattern { name: "all names of nonlegendary creature cards", priority: 100, parse: all_creature_names } }
