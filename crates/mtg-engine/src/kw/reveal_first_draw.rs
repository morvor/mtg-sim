//! "Reveal the first card you draw each turn." (Primitive Etchings, Rowen) and "Reveal the
//! first card you draw on each of your turns." (Keranos, God of Storms): a static ability
//! (Rowen's rulings: "The first sentence is a continuous ability. You reveal the card when
//! you draw it, during the resolution of any draw effect."). The card is revealed as it's
//! drawn (CR 701.20a), once however many such abilities apply (Primitive Etchings'
//! rulings). The triggered abilities "Whenever you reveal a [card] this way" are compiled
//! next to it by `oracle/patterns/draw_reveal_first.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, StaticEffect};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{ObjectId, PlayerId};

/// `StaticEffect::Custom` name: "Reveal the first card you draw each turn."
pub const EACH_TURN: &str = "reveal the first card you draw each turn";
/// `StaticEffect::Custom` name: "Reveal the first card you draw on each of your turns."
pub const YOUR_TURNS: &str = "reveal the first card you draw on each of your turns";

pub struct RevealFirstDraw;

impl KeywordRules for RevealFirstDraw {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn after_draw(&self, g: &mut Game, p: PlayerId, card: ObjectId, nth: u32) {
        if nth != 1 {
            return;
        }
        if g.dirty {
            g.recompute();
        }
        let your_turn = g.active_player() == p;
        let source = g.permanents().find_map(|o| {
            (o.controller == p
                && o.chars.abilities.iter().any(|a| {
                    matches!(&a.kind, AbilityKind::Static(s)
                        if matches!(&s.effect, StaticEffect::Custom(n)
                            if n == EACH_TURN || (n == YOUR_TURNS && your_turn)))
                }))
            .then_some(o.id)
        });
        if let Some(s) = source {
            crate::reveal::reveal(g, p, &[card], Some(s));
        }
    }
}

inventory::submit! { KeywordRegistration(&RevealFirstDraw) }
