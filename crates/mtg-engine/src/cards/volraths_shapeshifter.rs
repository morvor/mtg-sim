//! Volrath's Shapeshifter: "As long as the top card of your graveyard is a creature card,
//! ~ has the full text of that card and has the text \"{2}: Discard a card.\"" A
//! text-changing effect (CR 612.6, layer 3): it takes the card's name, mana cost, color
//! indicator, type line, rules text and power and toughness, and keeps the discard
//! ability; its own characteristics come back when a noncreature card is on top (rulings).
//! A copy of it copies the printed card, shapeshifting ability included (CR 707.2).

use super::{parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "As long as the top card of your graveyard is a creature card, ~ has the full text of that card and has the text \"{2}: Discard a card.\"";

inventory::submit! { ManualAbility {
    card: "Volrath's Shapeshifter",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let top = Sel::TopOfGraveyard(PlayerRef::You);
        let discard = parse(ctx, "{2}: Discard a card.");
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![
                Modification::FullTextOf(Box::new(top.clone())),
                Modification::AddText {
                    abilities: discard,
                    text: "{2}: Discard a card.".into(),
                },
            ],
        });
        s.condition = Some(Condition::SelMatches(top, Filter::creature()));
        vec![AbilityDef::new(AbilityKind::Static(s), TEXT)]
    },
    reason: "has the full text of another card (CR 612.6): only this card",
} }
