//! "Reveal the first card you draw each turn." (Primitive Etchings, Rowen) and "Reveal the
//! first card you draw on each of your turns." (Keranos, God of Storms): a static ability
//! (Rowen's rulings: "The first sentence is a continuous ability. You reveal the card when
//! you draw it, during the resolution of any draw effect."). The card is revealed as it's
//! drawn (CR 701.20a), once however many such abilities apply (Primitive Etchings'
//! rulings). The triggered abilities "Whenever you reveal a [card] this way" are compiled
//! next to it by `oracle/patterns/draw_reveal_first.rs`.
//!
//! "You may reveal the first card you draw each turn as you draw it." (Inquisitor
//! Eisenhorn, God-Eternal Kefnet): the player looks at the card as they draw it and
//! chooses then whether to reveal it (their rulings), for each such ability separately
//! (Kefnet's ruling: "you may reveal a card you draw for any number of their abilities");
//! its "Whenever you reveal ... this way" triggers only if they did for that ability. A
//! card revealed by a "Reveal the first card ..." ability is revealed without a choice,
//! by each such ability at once (Primitive Etchings' rulings: "only one card is revealed
//! (both of them reveal the same card). Both of them will trigger").
//!
//! "Whenever you reveal a [card] this way" ([`REVEALED_FIRST_DRAW`]) triggers on the
//! reveal itself, which this module performs as the card is drawn (CR 701.20a), or, for
//! a card drawn while a spell is being cast or an ability activated, after the spell
//! becomes cast or the ability becomes activated (CR 121.8: "If an effect allows or
//! instructs a player to reveal the card as it's being drawn, it's revealed after the
//! spell becomes cast or the ability becomes activated."; `draw_rules::finish_casting`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, StaticEffect};
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::EventInfo;
use crate::types::{ObjectId, PlayerId};

/// `StaticEffect::Custom` name: "Reveal the first card you draw each turn."
pub const EACH_TURN: &str = "reveal the first card you draw each turn";
/// `StaticEffect::Custom` name: "Reveal the first card you draw on each of your turns."
pub const YOUR_TURNS: &str = "reveal the first card you draw on each of your turns";
/// `StaticEffect::Custom` name: "You may reveal the first card you draw each turn as you
/// draw it."
pub const MAY_EACH_TURN: &str = "you may reveal the first card you draw each turn as you draw it";
/// `TriggerCond::Custom`: "Whenever you reveal a [card] this way" after one of the
/// abilities above: the first card its controller draws is revealed by that ability of
/// the same permanent. Also the name of the `Event::Custom` this module emits as it
/// reveals such a card.
pub const REVEALED_FIRST_DRAW: &str = "reveal first draw:you reveal the first card you draw this way";

/// The permanents `p` controls with one of the abilities above, named by `names`.
fn sources(g: &Game, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| {
            o.controller == p
                && o.chars.abilities.iter().any(|a| {
                    matches!(&a.kind, AbilityKind::Static(s)
                        if matches!(&s.effect, StaticEffect::Custom(n) if names.contains(&n.as_str())))
                })
        })
        .map(|o| o.id)
        .collect()
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
        let mut by = sources(g, p, mandatory);
        // The player looks at the card as they draw it, then chooses for each ability.
        for s in sources(g, p, &[MAY_EACH_TURN]) {
            if by.contains(&s) {
                continue;
            }
            let name = g.obj(card).chars.name.clone();
            if g.ask_yes_no(p, Some(s), &format!("Reveal {name} as you draw it?"), true) {
                by.push(s);
            }
        }
        let Some(first) = by.first().copied() else {
            return;
        };
        g.history
            .first_draws_revealed
            .extend(by.iter().map(|s| (card, *s)));
        crate::reveal::reveal(g, p, &[card], Some(first));
        g.emit(Event::Custom {
            name: REVEALED_FIRST_DRAW.into(),
            player: Some(p),
            obj: Some(card),
            amount: 0,
        });
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != REVEALED_FIRST_DRAW {
            return None;
        }
        let Event::Custom {
            name: ev_name,
            player: Some(p),
            obj: Some(card),
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        if ev_name != REVEALED_FIRST_DRAW
            || *p != ctl
            || !g.history.first_draws_revealed.contains(&(*card, src))
        {
            return Some(vec![]);
        }
        Some(vec![EventInfo {
            object: Some(*card),
            player: Some(*p),
            ..Default::default()
        }])
    }
}

inventory::submit! { KeywordRegistration(&RevealFirstDraw) }
