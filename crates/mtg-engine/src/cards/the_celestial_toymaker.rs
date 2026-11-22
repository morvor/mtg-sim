//! The Celestial Toymaker: "At the beginning of each end step, each opponent loses 2 life
//! for each spell or ability that caused one or more players to guess or to group cards or
//! permanents into a pile this turn." Grouping into piles (CR 700.3) is reported by the
//! pile instructions (`piles.rs`), guessing by the guessing instructions (`choices.rs`);
//! a spell or ability that did both, or did either several times, counts once.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const PILERS: &str = "card:The Celestial Toymaker:spells and abilities that caused guessing or grouping into piles this turn";
const TEXT: &str = "At the beginning of each end step, each opponent loses 2 life for each spell or ability that caused one or more players to guess or to group cards or permanents into a pile this turn.";

inventory::submit! { ManualAbility {
    card: "The Celestial Toymaker",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of each end step, each opponent loses 2 life.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::LoseLife { who, .. } => Effect::LoseLife {
                        who,
                        n: Value::Mul(Box::new(Value::Const(2)), Box::new(Value::Custom(PILERS.into()))),
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "counts the spells and abilities that made players group cards into piles this turn: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, spell or ability), once each.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::Custom {
            name,
            obj: Some(o),
            ..
        } = ev
        else {
            return;
        };
        if name != crate::piles::GROUPED_INTO_PILES && name != crate::choices::GUESSED {
            return;
        }
        let turn = i64::from(g.turn.number);
        let row = vec![turn, i64::from(o.0)];
        g.cards.retain(PILERS, |r| r[0] == turn);
        if !g.cards.get(PILERS).contains(&row) {
            g.cards.push(PILERS, row);
        }
    }
    fn custom_value(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<i64> {
        if name != PILERS {
            return None;
        }
        let turn = i64::from(g.turn.number);
        Some(g.cards.get(PILERS).iter().filter(|r| r[0] == turn).count() as i64)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
