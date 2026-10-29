//! Call Damage Control: "Choose up to two. Return those cards from your graveyard to your
//! hand. / • Target artifact card. / • Target creature card. / • Target enchantment card. /
//! • Target land card." Modes that are target descriptions for one shared instruction: each
//! chosen mode returns its target.

use super::{parse, ManualAbility};
use crate::ability::AbilityDef;

const TEXT: &str = "Choose up to two. Return those cards from your graveyard to your hand.\n• Target artifact card.\n• Target creature card.\n• Target enchantment card.\n• Target land card.";

inventory::submit! { ManualAbility {
    card: "Call Damage Control",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Choose up to two —\n• Return target artifact card from your graveyard to your hand.\n• Return target creature card from your graveyard to your hand.\n• Return target enchantment card from your graveyard to your hand.\n• Return target land card from your graveyard to your hand.")
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect()
    },
    reason: "modes that are only target descriptions for an instruction stated once before them: only this card",
} }
