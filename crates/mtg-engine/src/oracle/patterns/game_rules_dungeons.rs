//! Dungeons (CR 309): "Room abilities of dungeons you own trigger an additional time."
//! (Hama Pashar, Ruin Seeker; granted by Dungeon Delver), CR 603.2d: the room abilities
//! (CR 309.4c) of dungeon cards the player owns in the command zone.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn room_abilities_additional_time(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "room abilities of dungeons you own trigger an additional time" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
            sources: Filter::and(vec![
                Filter::Type(CardType::Dungeon),
                Filter::OwnedBy(PlayerRel::You),
                Filter::InZone(ZoneKind::Command),
            ]),
            cause: None,
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "game rules: room abilities of dungeons you own trigger an additional time", priority: 100, parse: room_abilities_additional_time } }
