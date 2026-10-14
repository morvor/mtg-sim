//! Approach of the Second Sun: "If ~ was cast from your hand and you've cast another
//! spell named ~ this game, you win the game. Otherwise, put ~ into its owner's library
//! seventh from the top and you gain 7 life."
//!
//! Everything but "you've cast another spell named ~ this game" compiles; that condition
//! is checked against a record of the spells each player cast this game (copies aren't
//! cast, CR 707.10).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Condition, Effect};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const ANOTHER: &str = "card:Approach of the Second Sun:you've cast another spell with its name this game";
/// Rows `[player, spell]`: every spell cast this game.
const CAST: &str = "card:Approach of the Second Sun:spells cast this game";
const TEXT: &str = "If ~ was cast from your hand and you've cast another spell named ~ this game, you win the game. Otherwise, put ~ into its owner's library seventh from the top and you gain 7 life.";

inventory::submit! { ManualAbility {
    card: "Approach of the Second Sun",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "If ~ was cast from your hand, you win the game. Otherwise, put ~ into its owner's library seventh from the top and you gain 7 life.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::If { cond, then, otherwise } => Effect::If {
                        cond: Condition::And(vec![cond, Condition::Custom(ANOTHER.into())]),
                        then,
                        otherwise,
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "win if cast from hand and another spell with its name was cast this game: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::SpellCast { spell, player, .. } = ev {
            g.cards.push(CAST, vec![player.0 as i64, spell.0 as i64]);
        }
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != ANOTHER {
            return None;
        }
        let me = ctx.source?;
        let my_name = g.obj(me).chars.name.clone();
        Some(g.cards.get(CAST).iter().any(|r| {
            let spell = ObjectId(r[1] as u32);
            r[0] == ctx.controller.0 as i64 && spell != me && g.obj(spell).chars.name == my_name
        }))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
