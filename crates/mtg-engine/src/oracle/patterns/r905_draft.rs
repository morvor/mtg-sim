//! Abilities that function during a draft (CR 905.2): "Draft [this card] face up."
//! (CR 905.2c) and "Reveal [this card] as you draft it and note how many cards you've
//! drafted this draft round, including [this card]." / "... and note the player who
//! passed it to you." (CR 905.2b). They have no effect during the game; `crate::draft`
//! applies them while drafting.

use super::AbilityPattern;
use crate::ability::*;
use crate::draft::{DRAFT_FACE_UP, DRAFT_NOTE_COUNT, DRAFT_NOTE_PASSER};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// Tried before the standard classifier, so that an instant's or sorcery's draft ability
/// isn't read as part of its spell effect.
fn draft_ability(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = text.trim().to_lowercase();
    let name = match end(&l) {
        "draft ~ face up" => DRAFT_FACE_UP,
        "reveal ~ as you draft it and note how many cards you've drafted this draft round, including ~" => {
            DRAFT_NOTE_COUNT
        }
        "reveal ~ as you draft it and note the player who passed it to you" => DRAFT_NOTE_PASSER,
        _ => return None,
    };
    let mut s = StaticAbility::new(StaticEffect::Custom(name.into()));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "r905 draft abilities", priority: 100, parse: draft_ability } }
