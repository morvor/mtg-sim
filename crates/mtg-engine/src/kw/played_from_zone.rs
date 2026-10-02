//! "a card played from exile" for lands: `Filter::Custom` names `came from:<zone>`, true
//! of a permanent whose previous incarnation (CR 400.7) was in that zone, e.g. a land that
//! was played from exile ("Whenever you play a card from exile", Prosper, Tome-Bound).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::ZoneKind;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// The `Filter::Custom` name for a permanent that came from `zone`.
pub fn came_from(zone: ZoneKind) -> String {
    format!("{PREFIX}{zone:?}")
}

const PREFIX: &str = "came from:";

pub struct PlayedFromZone;

impl KeywordRules for PlayedFromZone {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        name.strip_prefix(PREFIX)?;
        let prev = g.obj(id).prev.and_then(|p| g.try_obj(p));
        Some(
            prev.and_then(|o| o.zone.kind())
                .is_some_and(|z| came_from(z) == name),
        )
    }
}

inventory::submit! { KeywordRegistration(&PlayedFromZone) }
