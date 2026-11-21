//! Draws caused by a cycling ability (CR 702.29a: "[Cost], Discard this card: Draw a
//! card"), for "If a cycling ability of another nonland card would cause you to draw a
//! card, instead ..." (Unpredictable Cyclone): the draw happens while that ability
//! resolves (`Game::resolving`), and "the cycled card" is the source of that ability, as
//! it last existed in its owner's hand (CR 113.7a, 400.7).

use super::cycling::CYCLING;
use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;

/// `Condition::Custom` prefix: the event is caused by a cycling ability resolving, of a
/// card other than the effect's source (`other`) and nonland (`nonland`).
pub const CAUSED_BY_CYCLING: &str = "cycling draw:caused by a cycling ability";

/// `Filter::Custom`: shares a card type with the card whose cycling ability is resolving.
pub const SHARES_TYPE_WITH_CYCLED: &str = "cycling draw:shares a card type with the cycled card";

/// The `Condition::Custom` name.
pub fn caused_by_cycling(other: bool, nonland: bool) -> String {
    format!(
        "{CAUSED_BY_CYCLING}{}{}",
        if other { ":other" } else { "" },
        if nonland { ":nonland" } else { "" }
    )
}

/// The card whose cycling ability is resolving, as it last existed.
pub fn cycled_card(g: &Game) -> Option<ObjectId> {
    let r = g.resolving?;
    match &g.try_obj(r)?.stack.as_deref()?.kind {
        StackKind::Activated { source, ability } if ability.text == CYCLING => Some(*source),
        _ => None,
    }
}

struct CyclingDraw;

impl KeywordRules for CyclingDraw {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let quals = name.strip_prefix(CAUSED_BY_CYCLING)?;
        let Some(card) = cycled_card(g) else {
            return Some(false);
        };
        let o = g.try_obj(card)?;
        if quals.contains(":other")
            && ctx
                .source
                .is_some_and(|s| s == card || g.current(card) == s)
        {
            return Some(false);
        }
        if quals.contains(":nonland") && o.chars.card_types.contains(CardType::Land) {
            return Some(false);
        }
        Some(true)
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name != SHARES_TYPE_WITH_CYCLED {
            return None;
        }
        let Some(card) = cycled_card(g).and_then(|c| g.try_obj(c)) else {
            return Some(false);
        };
        let types = card.chars.card_types;
        Some(g.obj(id).chars.card_types.iter().any(|t| types.contains(t)))
    }
}

inventory::submit! { KeywordRegistration(&CyclingDraw) }
