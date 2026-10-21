//! Abandoned Sarcophagus: "If a card that has a cycling ability would be put into your
//! graveyard from anywhere and it wasn't cycled, exile it instead." A card discarded to pay
//! its own cycling cost was cycled (CR 702.29c) and goes to the graveyard; tokens aren't
//! cards (ruling). Typecycling abilities are cycling abilities (CR 702.29f).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const CYCLED: &str = "card:Abandoned Sarcophagus:it was cycled";
const TEXT: &str = "If a card that has a cycling ability would be put into your graveyard from anywhere and it wasn't cycled, exile it instead.";

inventory::submit! { ManualAbility {
    card: "Abandoned Sarcophagus",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "If a card would be put into your graveyard from anywhere, exile it instead.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(s) = &mut kind {
                    if let StaticEffect::Replacement(def) = &mut s.effect {
                        if let ReplacementEvent::ZoneChange { filter, .. } = &mut def.event {
                            *filter = Filter::And(vec![
                                filter.clone(),
                                Filter::HasKeyword(KeywordKind::Cycling),
                            ]);
                        }
                        def.event = ReplacementEvent::Where {
                            event: Box::new(def.event.clone()),
                            cond: Condition::Not(Box::new(Condition::Custom(CYCLED.into()))),
                        };
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "exiles cards with cycling unless they were cycled (needs the discard's cause): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// The card would be discarded to pay an ability's cost of its own (its cycling
    /// cost): "Discard this card" while the ability is being activated.
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CYCLED {
            return None;
        }
        let Some(ev) = &ctx.event else {
            return Some(false);
        };
        Some(
            ev.cause == Some(MoveCause::Discard)
                && ev.object.is_some()
                && ev.other == ev.object
                && g.special.casting > 0,
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
