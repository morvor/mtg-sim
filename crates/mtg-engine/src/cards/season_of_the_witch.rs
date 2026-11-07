//! Season of the Witch: "At the beginning of the end step, destroy all untapped creatures
//! that didn't attack this turn, except for creatures that couldn't attack." Destroys the
//! creatures that could have been declared as attackers during the turn's declare attackers
//! step but weren't (ruling): the creatures that could attack are recorded as attackers are
//! declared (CR 508.1a).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Filter, Sel};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::turn::Step;
use crate::types::ObjectId;

const COULD_ATTACK: &str = "card:Season of the Witch:could have attacked this turn";
const TEXT: &str = "At the beginning of the end step, destroy all untapped creatures that didn't attack this turn, except for creatures that couldn't attack.";

inventory::submit! { ManualAbility {
    card: "Season of the Witch",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of the end step, destroy all untapped creatures that didn't attack this turn.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Destroy { what: Sel::All(f), no_regen } => Effect::Destroy {
                        what: Sel::All(Filter::And(vec![f, Filter::Custom(COULD_ATTACK.into())])),
                        no_regen,
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "\"except for creatures that couldn't attack\": needs which creatures could have been declared as attackers; only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// As the declare attackers step begins (the creatures that didn't attack are as they
    /// were when attackers were declared), records the creatures that could have been
    /// declared as attackers.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::StepBegan {
            step: Step::DeclareAttackers,
            ..
        } = ev
        else {
            return;
        };
        let turn = i64::from(g.turn.number);
        g.cards.retain(COULD_ATTACK, |r| r[0] == turn);
        let mut could: Vec<ObjectId> = crate::combat::attack_options(g)
            .into_iter()
            .map(|(c, _)| c)
            .collect();
        if let Some(c) = &g.combat {
            could.extend(c.attackers.iter().map(|a| a.id));
        }
        for id in could {
            g.cards.push(COULD_ATTACK, vec![turn, i64::from(id.0)]);
        }
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name != COULD_ATTACK {
            return None;
        }
        let turn = i64::from(g.turn.number);
        Some(
            g.cards
                .get(COULD_ATTACK)
                .iter()
                .any(|r| r[0] == turn && r[1] == i64::from(id.0)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
