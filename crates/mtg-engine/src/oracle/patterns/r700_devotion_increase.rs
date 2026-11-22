//! "Your devotion to each color and each combination of colors is increased by one"
//! (Altar of the Pantheon, CR 700.5): see [`crate::game_terms::devotion_bonus`].

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

inventory::submit! { StaticPattern { name: "r700 devotion increased by one", priority: 100, parse: devotion_increased } }

fn devotion_increased(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "your devotion to each color and each combination of colors is increased by one" {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Custom(
        crate::game_terms::DEVOTION_INCREASED.into(),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}
