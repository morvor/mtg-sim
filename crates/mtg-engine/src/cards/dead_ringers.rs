//! Dead Ringers: "Destroy two target nonblack creatures unless either one is a color the
//! other isn't. They can't be regenerated." Colors are compared on resolution.

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Condition, Effect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const SAME: &str = "card:Dead Ringers:the targets are exactly the same colors";
const TEXT: &str = "Destroy two target nonblack creatures unless either one is a color the other isn't. They can't be regenerated.";

inventory::submit! { ManualAbility {
    card: "Dead Ringers",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Destroy two target nonblack creatures. They can't be regenerated.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| Effect::If {
                    cond: Condition::Custom(SAME.into()),
                    then: Box::new(e),
                    otherwise: Box::new(Effect::Noop),
                })
            })
            .collect()
    },
    reason: "destroy unless either target is a color the other isn't: unique comparison",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// The (legal) targets all have exactly the same colors.
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != SAME {
            return None;
        }
        let colors: Vec<_> = ctx
            .targets
            .iter()
            .flatten()
            .filter_map(|e| match e {
                Entity::Object(o) => Some(g.obj(*o).chars.colors),
                Entity::Player(_) => None,
            })
            .collect();
        Some(colors.windows(2).all(|w| w[0] == w[1]))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
