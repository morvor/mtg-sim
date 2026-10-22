//! Oracle patterns for text box contents (CR 207): Cryptic Spires' colors circled as the
//! deck is created (CR 207.5). Circling colors is done by [`crate::deck::circle_colors`];
//! the circled mana symbols then are part of the card's printed rules text.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

inventory::submit! { StaticPattern { name: "r207 circle colors as you create your deck", priority: 100, parse: circle_colors } }
inventory::submit! { EffectPattern { name: "r207 add mana of the circled colors", priority: 100, parse: add_circled } }

/// "As you create your deck, circle two of the colors below." A deck-creation ability
/// (CR 207.5): it does nothing during the game.
fn circle_colors(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "as you create your deck, circle two of the colors below" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(crate::deck::CIRCLE_TWO_COLORS.into()));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// "Add one mana of either of the circled colors." With no colors circled (the card as
/// printed, or an ability gained without the circled colors), it adds no mana.
fn add_circled(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "add one mana of either of the circled colors" {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::Fixed(vec![]),
        restriction: None,
    })
}
