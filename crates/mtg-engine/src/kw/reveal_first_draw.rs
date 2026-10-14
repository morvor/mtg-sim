//! "Reveal the first card you draw each turn." (Primitive Etchings, Rowen) and "Reveal the
//! first card you draw on each of your turns." (Keranos, God of Storms): a static ability
//! (Rowen's rulings: "The first sentence is a continuous ability. You reveal the card when
//! you draw it, during the resolution of any draw effect."). The card is revealed as it's
//! drawn (CR 701.20a), once however many such abilities apply (Primitive Etchings'
//! rulings). The triggered abilities "Whenever you reveal a [card] this way" are compiled
//! next to it by `oracle/patterns/draw_reveal_first.rs`.
//!
//! "You may reveal the first card you draw each turn as you draw it." (Inquisitor
//! Eisenhorn): the player looks at the card as they draw it and chooses then whether to
//! reveal it (its rulings); its "Whenever you reveal ... this way" triggers only if they
//! did ([`REVEALED_THIS_WAY`]). A card revealed by a "Reveal the first card ..." ability
//! is revealed without a choice.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, StaticEffect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{ObjectId, PlayerId};

/// `StaticEffect::Custom` name: "Reveal the first card you draw each turn."
pub const EACH_TURN: &str = "reveal the first card you draw each turn";
/// `StaticEffect::Custom` name: "Reveal the first card you draw on each of your turns."
pub const YOUR_TURNS: &str = "reveal the first card you draw on each of your turns";
/// `StaticEffect::Custom` name: "You may reveal the first card you draw each turn as you
/// draw it."
pub const MAY_EACH_TURN: &str = "you may reveal the first card you draw each turn as you draw it";
/// `Condition::Custom`, for a trigger condition about a card being drawn: it was revealed
/// as it was drawn by one of the abilities above.
pub const REVEALED_THIS_WAY: &str = "reveal first draw:the drawn card was revealed this way";

/// A permanent `p` controls with one of the abilities above, named by `names`.
fn source(g: &Game, p: PlayerId, names: &[&str]) -> Option<ObjectId> {
    g.permanents().find_map(|o| {
        (o.controller == p
            && o.chars.abilities.iter().any(|a| {
                matches!(&a.kind, AbilityKind::Static(s)
                    if matches!(&s.effect, StaticEffect::Custom(n) if names.contains(&n.as_str())))
            }))
        .then_some(o.id)
    })
}

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
        let mandatory: &[&str] = if your_turn {
            &[EACH_TURN, YOUR_TURNS]
        } else {
            &[EACH_TURN]
        };
        let src = match source(g, p, mandatory) {
            Some(s) => s,
            None => {
                let Some(s) = source(g, p, &[MAY_EACH_TURN]) else {
                    return;
                };
                // The player looks at the card as they draw it, then chooses.
                let name = g.obj(card).chars.name.clone();
                if !g.ask_yes_no(p, Some(s), &format!("Reveal {name} as you draw it?"), true) {
                    return;
                }
                s
            }
        };
        g.history.first_draws_revealed.push(card);
        crate::reveal::reveal(g, p, &[card], Some(src));
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != REVEALED_THIS_WAY {
            return None;
        }
        let card = ctx.event.as_ref().and_then(|e| e.object);
        Some(card.is_some_and(|c| g.history.first_draws_revealed.contains(&c)))
    }
}

inventory::submit! { KeywordRegistration(&RevealFirstDraw) }
