//! Rootwater Shaman: "You may cast Aura spells with enchant creature as though they had
//! flash." (CR 702.5a: the enchant keyword's parameter, as for searching for "an Aura card
//! with enchant creature".)

use super::{parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "You may cast Aura spells with enchant creature as though they had flash.";

inventory::submit! { ManualAbility {
    card: "Rootwater Shaman",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "You may cast Aura spells as though they had flash.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(s) = &mut kind {
                    if let StaticEffect::FlashPermission { what, .. } = &mut s.effect {
                        *what = Filter::And(vec![
                            what.clone(),
                            Filter::Custom(crate::search_rules::ENCHANT_CREATURE.into()),
                        ]);
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "flash for Aura spells with enchant creature (a spell filter on the enchant keyword's parameter): only this card",
} }
