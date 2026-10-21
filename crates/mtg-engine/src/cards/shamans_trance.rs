//! Shaman's Trance: "Other players can't play lands or cast spells from their graveyards
//! this turn. You may play lands and cast spells from other players' graveyards this turn
//! as though those cards were in your graveyard."
//!
//! The permission is the "as though" effect of CR 609.4 (`as_though.rs`): it lets
//! graveyard keywords such as flashback work on other players' cards (rulings). It doesn't
//! let you activate abilities of those cards.

use super::{spell, ManualAbility};
use crate::ability::*;

const TEXT: &str = "Other players can't play lands or cast spells from their graveyards this turn. You may play lands and cast spells from other players' graveyards this turn as though those cards were in your graveyard.";

inventory::submit! { ManualAbility {
    card: "Shaman's Trance",
    face: 0,
    text: TEXT,
    build: |_ctx| {
        let in_graveyard = || Filter::InZone(ZoneKind::Graveyard);
        vec![spell(
            vec![],
            Effect::Seq(vec![
                Effect::AddRestriction {
                    restriction: Restriction::CantPlayLandCards {
                        who: PlayerFilter::NotYou,
                        what: in_graveyard(),
                    },
                    duration: Duration::ThisTurn,
                },
                Effect::AddRestriction {
                    restriction: Restriction::CantCast {
                        who: PlayerFilter::NotYou,
                        what: in_graveyard(),
                    },
                    duration: Duration::ThisTurn,
                },
                Effect::AddPlayerEffect {
                    who: PlayerRef::You,
                    effect: PlayerModification::Custom(
                        crate::as_though::OTHER_GRAVEYARDS_AS_YOURS.into(),
                    ),
                    duration: Duration::ThisTurn,
                },
            ]),
            TEXT,
        )]
    },
    reason: "another player's graveyard treated as yours for playing cards: only this card",
} }
