//! "You may cast a spell with mana value N or less from your hand without paying its mana
//! cost." (the Expertise cycle): as the spell resolves, its controller may cast a card
//! from their hand without paying its mana cost (an alternative cost, CR 118.9), ignoring
//! timing permissions, if the spell it would become has mana value N or less. The
//! permission looks only at the characteristics of the spell as it would be cast
//! (CR 601.3e): one half of a split card or one door of a Room, each judged by its own
//! mana value (CR 709.3a), with any X in its mana cost 0 (CR 107.3b).
//!
//! The effect is an `Effect::Custom` named by [`effect_name`]; the oracle phrase is parsed
//! in `oracle/patterns/r601_cast_from_hand_free.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

const PREFIX: &str = "cast from hand free with mana value at most:";

/// The `Effect::Custom` name of "you may cast a spell with mana value `max` or less from
/// your hand without paying its mana cost".
pub fn effect_name(max: u32) -> String {
    format!("{PREFIX}{max}")
}

pub struct CastFromHandFree;

impl KeywordRules for CastFromHandFree {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(max) = name.strip_prefix(PREFIX).and_then(|n| n.parse::<u32>().ok()) else {
            return false;
        };
        cast_from_hand(g, ctx.controller, max);
        true
    }
}

/// `p` may cast a card from their hand as a spell with mana value `max` or less without
/// paying its mana cost.
fn cast_from_hand(g: &mut Game, p: crate::types::PlayerId, max: u32) {
    // CR 702.61a: no spell can be cast while a spell with split second is on the stack.
    if g.split_second_on_stack() {
        return;
    }
    g.recompute();
    let hand = g.player(p).hand.clone();
    let mut options = Vec::new();
    for card in hand {
        for (label, opt) in crate::casting::free_cast_options(g, p, card, |mv| mv <= max) {
            options.push((card, label, opt));
        }
    }
    if options.is_empty() {
        return;
    }
    let mut labels: Vec<String> = options
        .iter()
        .map(|(_, label, _)| format!("Cast {label}"))
        .collect();
    labels.push("Don't cast a spell".into());
    let prompt = format!("Cast a spell with mana value {max} or less from your hand for free?");
    let i = g.ask_option(p, None, &prompt, labels);
    if i < options.len() {
        let (card, _, opt) = options.swap_remove(i);
        let _ = g.cast_with_option(p, card, opt);
    }
}

inventory::submit! { KeywordRegistration(&CastFromHandFree) }
