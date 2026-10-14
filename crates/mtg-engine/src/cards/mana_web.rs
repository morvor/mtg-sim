//! Mana Web: "Whenever a land an opponent controls is tapped for mana, tap all lands that
//! player controls that could produce any type of mana that land could produce."
//! (CR 106.7: the mana a permanent could produce.)

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Filter, Sel};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const SHARES: &str = "card:Mana Web:could produce a type of mana that land could produce";
const TEXT: &str = "Whenever a land an opponent controls is tapped for mana, tap all lands that player controls that could produce any type of mana that land could produce.";

inventory::submit! { ManualAbility {
    card: "Mana Web",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever a land an opponent controls is tapped for mana, tap all lands that player controls.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Tap { what: Sel::All(f) } => Effect::Tap {
                        what: Sel::All(Filter::And(vec![f, Filter::Custom(SHARES.into())])),
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "taps the lands that could produce a type of mana the tapped land could produce: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Any of its types of mana (using any of its mana abilities, not only the one used)
    /// is one the land that was tapped could produce.
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != SHARES {
            return None;
        }
        let Some(land) = g.eval_sel_objects(&Sel::TriggerObject, ctx).first().copied() else {
            return Some(false);
        };
        let types = crate::mana_abilities::could_produce(g, land);
        let mine = crate::mana_abilities::could_produce(g, id);
        Some(mine.iter().any(|t| types.contains(t)))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
