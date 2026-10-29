//! Musician: "{T}: Put a music counter on target creature. If it doesn't have \"At the
//! beginning of your upkeep, destroy ~ unless you pay {1} for each music counter on it,\"
//! it gains that ability." The counter and the grant (with no duration: the creature keeps
//! the ability, CR 611.2a ruling) are the compiled grammar; what's hand-written is the
//! test of whether the creature already has that ability, so a creature gains it once
//! however many Musicians target it (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Condition, Effect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const TEXT: &str = "{T}: Put a music counter on target creature. If it doesn't have \"At the beginning of your upkeep, destroy ~ unless you pay {1} for each music counter on it,\" it gains that ability.";
const LACKS: &str = "card:Musician:it doesn't have that ability";
/// The text of the granted ability.
const GRANTED: &str = "At the beginning of your upkeep, destroy ~ unless you pay {1} for each music counter on it.";

inventory::submit! { ManualAbility {
    card: "Musician",
    face: 0,
    text: TEXT,
    build: |ctx| {
        // The quote names the creature that gets it ("this creature", raw text).
        crate::oracle::with_raw_text(
            "{T}: Put a music counter on target creature. It gains \"At the beginning of your upkeep, destroy this creature unless you pay {1} for each music counter on it.\"",
            || parse(ctx, &format!("{{T}}: Put a music counter on target creature. It gains \"{GRANTED}\"")),
        )
        .iter()
        .map(|a| {
            map_effect(a, TEXT, |e| match e {
                Effect::Seq(mut v) if v.len() == 2 => {
                    let gain = v.pop().unwrap_or_default();
                    v.push(Effect::If {
                        cond: Condition::Custom(LACKS.into()),
                        then: Box::new(gain),
                        otherwise: Box::new(Effect::Seq(vec![])),
                    });
                    Effect::Seq(v)
                }
                e => e,
            })
        })
        .collect()
    },
    reason: "a condition on having a particular quoted ability: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != LACKS {
            return None;
        }
        let it = g.eval_sel(&crate::ability::Sel::Target(0), ctx);
        Some(!it.iter().filter_map(|e| e.object()).any(|o| {
            g.obj(o).chars.abilities.iter().any(|a| a.text == GRANTED)
        }))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
