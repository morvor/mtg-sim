//! Draft abilities, grammar V (cards5-game-rules-dice; CR 905.2): "Reveal ~ as you draft
//! it. After you draft ~, you may add a booster pack to the draft." (Lore Seeker) and
//! "Each player passes the last card from each booster pack to a player who drafted a card
//! named ~." (Canal Dredger). `crate::draft` applies them while drafting; they do nothing
//! during the game.

use super::AbilityPattern;
use crate::ability::*;
use crate::draft::{DRAFT_ADD_BOOSTER, DRAFT_LAST_CARD_TO_DRAFTER};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn draft_abilities_5(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = text.trim().to_lowercase();
    let name = match end(&l) {
        "reveal ~ as you draft it. after you draft ~, you may add a booster pack to the draft" => {
            DRAFT_ADD_BOOSTER
        }
        "each player passes the last card from each booster pack to a player who drafted a card named ~" => {
            DRAFT_LAST_CARD_TO_DRAFTER
        }
        _ => return None,
    };
    let mut s = StaticAbility::new(StaticEffect::Custom(name.into()));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "draft 5: add a booster pack, last cards to a drafter", priority: 100, parse: draft_abilities_5 } }
