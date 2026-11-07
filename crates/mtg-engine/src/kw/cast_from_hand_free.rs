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
use crate::ability::vars;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{Entity, ObjectId};

const PREFIX: &str = "cast from hand free with mana value at most:";
const PREFIX_TARGET: &str = "cast from hand free with mana value at most that of target:";

/// The `Effect::Custom` name of "you may cast a spell with mana value `max` or less from
/// your hand without paying its mana cost".
pub fn effect_name(max: u32) -> String {
    format!("{PREFIX}{max}")
}

/// The `Effect::Custom` name of "you may cast a spell with equal or lesser mana value
/// from your hand without paying its mana cost" (Reinterpret): at most the mana value of
/// the object chosen in target slot `slot` (as it last existed, if it's gone).
pub fn effect_name_target(slot: u8) -> String {
    format!("{PREFIX_TARGET}{slot}")
}

pub struct CastFromHandFree;

impl KeywordRules for CastFromHandFree {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(slot) = name
            .strip_prefix(PREFIX_TARGET)
            .and_then(|n| n.parse::<usize>().ok())
        {
            let target = ctx
                .targets
                .get(slot)
                .and_then(|t| t.iter().find_map(|e| e.object()));
            // The spell may have left the stack: its last known mana value.
            let Some(max) = target.map(|o| g.mana_value_of(o)) else {
                return true;
            };
            cast_from_hand(g, ctx, max);
            return true;
        }
        let Some(max) = name.strip_prefix(PREFIX).and_then(|n| n.parse::<u32>().ok()) else {
            return false;
        };
        cast_from_hand(g, ctx, max);
        true
    }
}

/// The controller may cast a card from their hand as a spell with mana value `max` or
/// less without paying its mana cost: they choose a card (or none), then, if it could be
/// cast as more than one such spell (each half of a split card, each door of a Room),
/// which one. "If you do" after it asks whether a spell was cast.
fn cast_from_hand(g: &mut Game, ctx: &mut Ctx, max: u32) {
    let p = ctx.controller;
    ctx.prev_happened = false;
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
    let mut cards: Vec<ObjectId> = Vec::new();
    for (card, _, _) in &options {
        if !cards.contains(card) {
            cards.push(*card);
        }
    }
    // Choosing none (the default) is declining: it's "you may".
    let prompt =
        format!("Cast a spell with mana value {max} or less from your hand for free (or none)");
    let Some(card) = g
        .ask_objects(p, ctx.source, &prompt, cards, 0, 1)
        .into_iter()
        .next()
    else {
        return;
    };
    let mut ways: Vec<_> = options
        .into_iter()
        .filter(|(c, _, _)| *c == card)
        .map(|(_, label, opt)| (label, opt))
        .collect();
    if ways.is_empty() {
        return;
    }
    let i = if ways.len() > 1 {
        let labels = ways
            .iter()
            .map(|(label, _)| format!("Cast {label}"))
            .collect();
        g.ask_option(p, Some(card), "Choose which spell to cast", labels)
    } else {
        0
    };
    let (_, opt) = ways.swap_remove(i.min(ways.len() - 1));
    if let Ok(spell) = g.cast_with_option(p, card, opt) {
        ctx.prev_happened = true;
        // CR 400.7h: other parts of the effect can find the spell cast this way.
        ctx.set_var(vars::IT, vec![Entity::Object(spell)]);
    }
}

inventory::submit! { KeywordRegistration(&CastFromHandFree) }
