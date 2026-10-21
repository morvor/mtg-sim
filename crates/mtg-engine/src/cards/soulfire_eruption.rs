//! Soulfire Eruption: "Choose any number of target creatures, planeswalkers, and/or
//! players. For each of them, exile the top card of your library, then ~ deals damage
//! equal to that card's mana value to that permanent or player. You may play the exiled
//! cards until the end of your next turn."
//!
//! The library can't be exiled from several times at once, so the targets are taken one
//! at a time (CR 608.2f and its example): in APNAP order of the affected players (a target
//! player, or a target permanent's controller), and among the targets affecting the same
//! player in an order the spell's controller chooses. For each, the top card is exiled and
//! then the damage dealt. Illegal targets are skipped (CR 608.2b).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{vars, Effect, PlayerRef, Sel, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const EACH: &str = "card:Soulfire Eruption:for each target, exile the top card then deal damage";
const TEXT: &str = "Choose any number of target creatures, planeswalkers, and/or players. For each of them, exile the top card of your library, then ~ deals damage equal to that card's mana value to that permanent or player. You may play the exiled cards until the end of your next turn.";

/// The surrogate's "Exile the top card of your library" becomes the per-target loop.
fn replace_exile(e: Effect) -> Effect {
    match e {
        Effect::Exile { .. } => Effect::Custom(EACH.into()),
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(replace_exile).collect()),
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Soulfire Eruption",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Choose any number of target creatures, planeswalkers, and/or players. Exile the top card of your library. You may play the exiled cards until the end of your next turn.")
            .iter()
            .map(|a| map_effect(a, TEXT, replace_exile))
            .collect()
    },
    reason: "for each target in turn, exile the top card of your library and deal damage equal to its mana value to that target: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EACH {
            return false;
        }
        let mut remaining: Vec<Entity> = ctx.targets.first().cloned().unwrap_or_default();
        let mut exiled = Vec::new();
        let order = g.apnap();
        // The player a target affects: itself, or the permanent's controller.
        let affected = |g: &Game, e: &Entity| -> usize {
            let p = match e {
                Entity::Player(p) => *p,
                Entity::Object(o) => g.obj(*o).controller,
            };
            order.iter().position(|q| *q == p).unwrap_or(usize::MAX)
        };
        while !remaining.is_empty() {
            // APNAP order first (the first player in it with a remaining target) ...
            let first = remaining.iter().map(|e| affected(g, e)).min().unwrap_or(0);
            let options: Vec<Entity> = remaining
                .iter()
                .copied()
                .filter(|e| affected(g, e) == first)
                .collect();
            // ... then the target considered next among those: chosen by the controller.
            let next = if options.len() == 1 {
                options[0]
            } else {
                g.ask_entities(
                    ctx.controller,
                    ctx.source,
                    "Choose the next target",
                    options.clone(),
                    1,
                    1,
                )
                .first()
                .copied()
                .filter(|e| options.contains(e))
                .unwrap_or(options[0])
            };
            remaining.retain(|e| *e != next);
            g.exec(
                &Effect::Exile {
                    what: Sel::TopOfLibrary(PlayerRef::You, Value::c(1)),
                    face_down: false,
                    link: false,
                },
                ctx,
            );
            let card = ctx
                .vars
                .get(&vars::IT)
                .and_then(|v| v.first())
                .and_then(|e| e.object());
            let Some(card) = card else {
                continue;
            };
            exiled.push(Entity::Object(card));
            let mv = g.mana_value_of(card);
            if let Some(src) = ctx.source {
                if mv > 0 {
                    g.deal_damage(src, next, mv, false);
                }
            }
        }
        // "the exiled cards".
        ctx.set_var(vars::IT, exiled);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
