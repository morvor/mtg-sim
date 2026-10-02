//! Spy Network: "Look at target player's hand, the top card of that player's library, and
//! any face-down creatures they control. Look at the top four cards of your library, then
//! put them back in any order."
//!
//! Looking gives its controller information only (CR 701.16, 708.5): the cards aren't
//! revealed.

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, PlayerRef};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const LOOK: &str = "card:Spy Network:look at the top card of their library and their face-down creatures";
const TEXT: &str = "Look at target player's hand, the top card of that player's library, and any face-down creatures they control. Look at the top four cards of your library, then put them back in any order.";

inventory::submit! { ManualAbility {
    card: "Spy Network",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Look at target player's hand. Look at the top four cards of your library, then put them back in any order.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Seq(mut v) if !v.is_empty() => {
                        v.insert(1, Effect::Custom(LOOK.into()));
                        Effect::Seq(v)
                    }
                    e => e,
                })
            })
            .collect()
    },
    reason: "looks at a player's hand, top card of library and face-down creatures: unique combination",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != LOOK {
            return false;
        }
        let viewer = ctx.controller;
        for p in g.eval_players(&PlayerRef::Target(0), ctx) {
            let top = g.library_top(p);
            let face_down: Vec<_> = g
                .permanents_controlled_by(p)
                .into_iter()
                .filter(|id| g.obj(*id).face_down && g.obj(*id).is_creature())
                .collect();
            g.log(|g| {
                let name = |id| {
                    g.obj(id)
                        .card
                        .as_ref()
                        .map_or_else(|| g.describe(id), |c| format!("{} {id}", c.name))
                };
                format!(
                    "{viewer} looks at the top card of {p}'s library ({}) and their face-down creatures ({})",
                    top.map_or_else(|| "none".to_string(), name),
                    face_down.iter().map(|id| name(*id)).collect::<Vec<_>>().join(", ")
                )
            });
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
