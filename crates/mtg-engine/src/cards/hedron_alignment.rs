//! Hedron Alignment: "At the beginning of your upkeep, you may reveal your hand. If you do,
//! you win the game if you own a card named ~ in exile, in your hand, in your graveyard,
//! and on the battlefield."

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Condition, Effect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;

const ALIGNED: &str = "card:Hedron Alignment:you own one in exile, in your hand, in your graveyard and on the battlefield";
const TEXT: &str = "At the beginning of your upkeep, you may reveal your hand. If you do, you win the game if you own a card named ~ in exile, in your hand, in your graveyard, and on the battlefield.";
const NAME: &str = "Hedron Alignment";

fn guard_win(e: Effect) -> Effect {
    match e {
        Effect::WinGame { who } => Effect::If {
            cond: Condition::Custom(ALIGNED.into()),
            then: Box::new(Effect::WinGame { who }),
            otherwise: Box::new(Effect::Noop),
        },
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(guard_win).collect()),
        Effect::If {
            cond,
            then,
            otherwise,
        } => Effect::If {
            cond,
            then: Box::new(guard_win(*then)),
            otherwise: Box::new(guard_win(*otherwise)),
        },
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Hedron Alignment",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of your upkeep, you may reveal your hand. If you do, you win the game.")
            .iter()
            .map(|a| map_effect(a, TEXT, guard_win))
            .collect()
    },
    reason: "wins if you own a card with its name in exile, your hand, your graveyard and on the battlefield at once: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != ALIGNED {
            return None;
        }
        let you = ctx.controller;
        let named = |id: &crate::types::ObjectId| {
            let o = g.obj(*id);
            o.owner == you && !o.face_down && o.chars.name == NAME
        };
        let p = g.player(you);
        Some(
            g.exile.iter().any(named)
                && p.hand.iter().any(named)
                && p.graveyard.iter().any(named)
                && g.battlefield
                    .iter()
                    .any(|id| named(id) && g.obj(*id).zone == Zone::Battlefield && !g.obj(*id).phased_out),
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
