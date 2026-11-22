//! Sands of Time: "At the beginning of each player's upkeep, that player simultaneously
//! untaps each tapped artifact, creature, and land they control and taps each untapped
//! artifact, creature, and land they control."

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, PlayerRef};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, ObjectId};

const TOGGLE: &str = "card:Sands of Time:that player untaps its tapped and taps its untapped permanents";
const TEXT: &str = "At the beginning of each player's upkeep, that player simultaneously untaps each tapped artifact, creature, and land they control and taps each untapped artifact, creature, and land they control.";

inventory::submit! { ManualAbility {
    card: "Sands of Time",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of each player's upkeep, that player draws a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(TOGGLE.into())))
            .collect()
    },
    reason: "simultaneously untaps the tapped and taps the untapped permanents: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != TOGGLE {
            return false;
        }
        let Some(p) = g.eval_player(&PlayerRef::TriggerPlayer, ctx) else {
            return true;
        };
        let mine: Vec<ObjectId> = g
            .permanents_controlled_by(p)
            .into_iter()
            .filter(|id| {
                let o = g.obj(*id);
                o.is(CardType::Artifact) || o.is_creature() || o.is(CardType::Land)
            })
            .collect();
        let (tapped, untapped): (Vec<ObjectId>, Vec<ObjectId>) =
            mine.into_iter().partition(|id| g.obj(*id).tapped);
        for id in tapped {
            g.untap(id);
        }
        for id in untapped {
            g.tap(id);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
