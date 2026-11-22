//! Booby Trap: "The chosen player reveals each card they draw." and "When the chosen player
//! draws a card with the chosen name, sacrifice ~. If you do, ~ deals 10 damage to that
//! player." (Its "As ~ enters, choose an opponent and a card name ..." compiles.) Each card
//! is revealed as it's drawn; the trigger waits until the drawing is done, and only the
//! first to resolve can sacrifice it (rulings).

use super::{active, marker, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{ObjectId, PlayerId};

const REVEALS: &str = "card:Booby Trap:the chosen player reveals each card they draw";
const NAMED: &str = "card:Booby Trap:the drawn card has the chosen name";
const REVEAL_TEXT: &str = "The chosen player reveals each card they draw.";
const TRAP_TEXT: &str = "When the chosen player draws a card with the chosen name, sacrifice ~. If you do, ~ deals 10 damage to that player.";

inventory::submit! { ManualAbility {
    card: "Booby Trap",
    face: 0,
    text: REVEAL_TEXT,
    build: |_| vec![marker(REVEALS, REVEAL_TEXT)],
    reason: "the chosen player reveals each card they draw: only this card",
} }

inventory::submit! { ManualAbility {
    card: "Booby Trap",
    face: 0,
    text: TRAP_TEXT,
    build: |ctx| {
        parse(ctx, "Whenever an opponent draws a card, sacrifice ~. If you do, ~ deals 10 damage to that player.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.trigger = TriggerCond::Where {
                        trigger: Box::new(TriggerCond::Draws {
                            who: PlayerRel::Chosen,
                        }),
                        cond: Condition::Custom(NAMED.into()),
                    };
                }
                AbilityDef::new(kind, TRAP_TEXT)
            })
            .collect()
    },
    reason: "a trigger on the chosen player drawing a card with the chosen name: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// The drawn card (revealed as it was drawn, so it's known) has the chosen name.
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != NAMED {
            return None;
        }
        let Some(card) = ctx.event.as_ref().and_then(|e| e.object) else {
            return Some(false);
        };
        Some(g.matches(card, &Filter::ChosenName, ctx))
    }
    fn after_draw(&self, g: &mut Game, p: PlayerId, card: ObjectId, _nth: u32) {
        let traps: Vec<ObjectId> = active(g, REVEALS)
            .into_iter()
            .filter(|(s, _)| g.obj(*s).choices.player == Some(p))
            .map(|(s, _)| s)
            .collect();
        if let Some(src) = traps.first() {
            crate::reveal::reveal(g, p, &[card], Some(*src));
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
