//! Vodalian War Machine: "When ~ dies, destroy all Merfolk tapped this turn to pay for its
//! abilities." The Merfolk tapped to pay the costs of its activated abilities this turn
//! (CR 602.2b) are recorded as they're activated.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{StackKind, Zone};
use crate::types::{Entity, ObjectId};

const TAPPED_FOR_IT: &str = "card:Vodalian War Machine:Merfolk tapped this turn to pay for its abilities";
const TEXT: &str = "When ~ dies, destroy all Merfolk tapped this turn to pay for its abilities.";

inventory::submit! { ManualAbility {
    card: "Vodalian War Machine",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ dies, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(TAPPED_FOR_IT.into())))
            .collect()
    },
    reason: "destroys the Merfolk tapped this turn to pay for its own abilities: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, War Machine, Merfolk) for the permanents tapped to pay the costs of
    /// its activated abilities.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::AbilityActivated {
            ability: Some(id),
            source,
            ..
        } = ev
        else {
            return;
        };
        let has_it = g.obj(*source).chars.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Triggered(t)
                if matches!(&t.body.effect, Effect::Custom(n) if n == TAPPED_FOR_IT))
        });
        if !has_it {
            return;
        }
        let from_it = matches!(
            g.obj(*id).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Activated { source: s, .. }) if s == source
        );
        let paid: Vec<ObjectId> = g
            .saved_ctx
            .get(id)
            .and_then(|c| c.vars.get(&(vars::USER + 91)))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(Entity::object)
            .filter(|o| g.obj(*o).tapped && g.obj(*o).chars.has_subtype("Merfolk"))
            .collect();
        if !from_it {
            return;
        }
        let turn = i64::from(g.turn.number);
        g.cards.retain(TAPPED_FOR_IT, |r| r[0] == turn);
        for m in paid {
            g.cards
                .push(TAPPED_FOR_IT, vec![turn, i64::from(source.0), i64::from(m.0)]);
        }
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != TAPPED_FOR_IT {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let turn = i64::from(g.turn.number);
        let merfolk: Vec<ObjectId> = g
            .cards
            .get(TAPPED_FOR_IT)
            .iter()
            .filter(|r| r[0] == turn && r[1] == i64::from(src.0))
            .map(|r| ObjectId(r[2] as u32))
            .filter(|m| {
                g.is_live(*m)
                    && g.obj(*m).zone == Zone::Battlefield
                    && g.obj(*m).chars.has_subtype("Merfolk")
            })
            .collect();
        for m in merfolk {
            g.destroy(m, Some(src));
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
