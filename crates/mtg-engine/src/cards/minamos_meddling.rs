//! Minamo's Meddling: "Counter target spell. That spell's controller reveals their hand,
//! then discards each card with the same name as a card spliced onto that spell." The
//! cards spliced onto a spell are recorded as they're spliced (CR 702.47a, `splice.rs`).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const DISCARD: &str = "card:Minamo's Meddling:discard each card with the name of a card spliced onto that spell";
const SPLICED: &str = "card:Minamo's Meddling:cards spliced onto spells";
const TEXT: &str = "Counter target spell. That spell's controller reveals their hand, then discards each card with the same name as a card spliced onto that spell.";

inventory::submit! { ManualAbility {
    card: "Minamo's Meddling",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Counter target spell. That spell's controller reveals their hand.")
            .iter()
            .map(|a| map_effect(a, TEXT, |e| Effect::Seq(vec![e, Effect::Custom(DISCARD.into())])))
            .collect()
    },
    reason: "discards each card with the same name as a card spliced onto the countered spell: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (spell, spliced card) as cards are spliced onto spells.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::Custom {
            name,
            obj: Some(card),
            amount,
            ..
        } = ev
        {
            if name == crate::splice::SPLICED {
                // Spells that have left the stack are new objects (CR 400.7).
                let stale: Vec<i64> = g
                    .cards
                    .get(SPLICED)
                    .iter()
                    .map(|r| r[0])
                    .filter(|s| !g.is_live(ObjectId(*s as u32)))
                    .collect();
                g.cards.retain(SPLICED, |r| !stale.contains(&r[0]));
                g.cards.push(SPLICED, vec![i64::from(*amount), i64::from(card.0)]);
            }
        }
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != DISCARD {
            return false;
        }
        let Some(Entity::Object(spell)) = ctx.targets.first().and_then(|t| t.first()).copied()
        else {
            return true;
        };
        let names: Vec<_> = g
            .cards
            .get(SPLICED)
            .iter()
            .filter(|r| r[0] == i64::from(spell.0))
            .map(|r| g.obj(ObjectId(r[1] as u32)).chars.name.clone())
            .collect();
        if names.is_empty() {
            return true;
        }
        let p = g.obj(spell).controller;
        let hand: Vec<ObjectId> = g
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| names.contains(&g.obj(*c).chars.name))
            .collect();
        for c in hand {
            g.discard(p, c, ctx.source);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
