//! CR 702.29 Cycling and typecycling.
//!
//! "Cycling [cost]" means "[Cost], Discard this card: Draw a card" (CR 702.29a), an
//! activated ability that functions only while the card is in a player's hand but exists
//! in every zone (CR 702.29b). "[Type]cycling [cost]" means "[Cost], Discard this card:
//! Search your library for a [type] card, reveal it, and put it into your hand. Then
//! shuffle your library" (CR 702.29e). Typecycling abilities are cycling abilities
//! (CR 702.29f): the derived ability is named "Cycling" either way, so effects and
//! triggers that look for cycling (`Event::Cycled`, "cycling abilities you activate",
//! [`crate::keyword_impls::ability_from_keyword`]) find both. Discarding the card to pay
//! the cost is cycling it (CR 702.29c; see `Game::activate_inner`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

/// The name of every cycling ability (typecycling included), as derived from the keyword.
pub const CYCLING: &str = "Cycling";

pub struct Cycling;

impl KeywordRules for Cycling {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Cycling]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let mut cost = kw.cost.clone().unwrap_or_default();
        cost.parts.push(CostPart::DiscardSelf);
        let effect = match &kw.filter {
            // CR 702.29e: typecycling searches for a card of the type instead of drawing.
            Some(ty) => Effect::Search {
                who: PlayerRef::You,
                whose: PlayerRef::You,
                filter: ty.clone(),
                count: Value::c(1),
                to: Destination::zone(ZoneKind::Hand),
                reveal: true,
                shuffle: true,
            },
            None => Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(1),
            },
        };
        let mut act = ActivatedAbility::new(cost, Body::effect(effect));
        // CR 702.29a–b: activated only while the card is in a player's hand.
        act.zone = FunctionZone::Hand;
        Some(vec![AbilityDef::new(AbilityKind::Activated(act), CYCLING)])
    }
}

inventory::submit! { KeywordRegistration(&Cycling) }
