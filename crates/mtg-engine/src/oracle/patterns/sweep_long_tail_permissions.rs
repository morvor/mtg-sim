//! "You may play lands from your graveyard." (Crucible of Worlds, Ramunap Excavator): a
//! permission to play land cards from another zone. Playing a land this way is still the
//! player's land play for the turn, following the normal timing and limits (CR 305.2,
//! 305.3, 601.3).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CardType;

inventory::submit! {
    StaticPattern { name: "sweep: play lands from your graveyard", priority: 50, parse: play_lands_from_graveyard }
}

fn play_lands_from_graveyard(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || end(l) != "you may play lands from your graveyard" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayPermission(
            PlayPermission {
                who: PlayerRel::You,
                zone: ZoneKind::Graveyard,
                top_only: false,
                what: Filter::Type(CardType::Land),
                lands: true,
                spells: false,
                cost: None,
                flash: false,
                terms: Default::default(),
            },
        ))),
        text,
    )])
}
