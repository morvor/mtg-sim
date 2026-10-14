//! "[Creatures with flying] ... and their activated abilities with {T} in their costs
//! can't be activated." (Katabatic Winds): a static ability (`StaticEffect::Custom` named
//! [`PREFIX`] plus the JSON of the objects' filter) that stops the activated abilities of
//! those objects whose costs include the tap symbol, mana abilities included (CR 602.5,
//! 605.3).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{ObjectId, PlayerId};

/// `StaticEffect::Custom` prefix, followed by the JSON of the objects' `Filter`.
pub const PREFIX: &str = "tap abilities can't be activated:";

/// The static ability's effect for the objects `f` describes.
pub fn effect(f: &Filter) -> Option<StaticEffect> {
    let json = serde_json::to_string(f).ok()?;
    Some(StaticEffect::Custom(format!("{PREFIX}{json}").into()))
}

pub struct TapAbilitiesProhibited;

impl KeywordRules for TapAbilitiesProhibited {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn activation_allowed(&self, g: &Game, _p: PlayerId, src: ObjectId, a: &Ability) -> bool {
        let AbilityKind::Activated(act) = &a.kind else {
            return true;
        };
        if !act.cost.parts.iter().any(|c| matches!(c, CostPart::Tap)) {
            return true;
        }
        !g.statics.customs.iter().any(|(s, c, n)| {
            n.strip_prefix(PREFIX)
                .and_then(|j| serde_json::from_str::<Filter>(j).ok())
                .is_some_and(|f| g.matches(src, &f, &Ctx::new(Some(*s), *c)))
        })
    }
}

inventory::submit! { KeywordRegistration(&TapAbilitiesProhibited) }
