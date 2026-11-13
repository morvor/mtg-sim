//! Common Cause: "Nonartifact creatures get +2/+2 as long as they all share a color."

use super::{parse, with_condition, ManualAbility};
use crate::ability::Condition;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, ColorSet};

const SHARE: &str = "card:Common Cause:all nonartifact creatures share a color";
const TEXT: &str = "Nonartifact creatures get +2/+2 as long as they all share a color.";

inventory::submit! { ManualAbility {
    card: "Common Cause",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Nonartifact creatures get +2/+2.")
            .iter()
            .map(|a| with_condition(a, Condition::Custom(SHARE.into()), TEXT))
            .collect()
    },
    reason: "anthem conditional on all nonartifact creatures sharing a color: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// All nonartifact creatures on the battlefield have a color in common (at least one
    /// color, all of them have it).
    fn custom_condition(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
        if name != SHARE {
            return None;
        }
        let mut common = ColorSet::ALL;
        for o in g.permanents() {
            if o.is_creature() && !o.is(CardType::Artifact) {
                common = ColorSet(common.0 & o.chars.colors.0);
            }
        }
        Some(!common.is_colorless())
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
