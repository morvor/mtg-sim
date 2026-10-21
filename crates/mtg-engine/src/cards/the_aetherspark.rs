//! The Aetherspark: "As long as ~ is attached to a creature, ~ can't be attacked and has
//! "Whenever equipped creature deals combat damage during your turn, put that many loyalty
//! counters on ~."" A planeswalker that's no attack target while attached (CR 506.3c,
//! 508.1b); one already being attacked stays attacked (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const UNATTACKABLE: &str = "card:The Aetherspark:can't be attacked while attached to a creature";
const TEXT: &str = "As long as ~ is attached to a creature, ~ can't be attacked and has \"Whenever equipped creature deals combat damage during your turn, put that many loyalty counters on ~.\"";

inventory::submit! { ManualAbility {
    card: "The Aetherspark",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let granted = parse(ctx, "As long as ~ is attached to a creature, ~ has \"Whenever equipped creature deals combat damage during your turn, put that many loyalty counters on ~.\"");
        let cond = granted.iter().find_map(|a| match &a.kind {
            AbilityKind::Static(s) => s.condition.clone(),
            _ => None,
        });
        let mut v: Vec<Ability> = granted
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect();
        if let Some(c) = cond {
            v.push(super::marker_if(UNATTACKABLE, c, TEXT));
        }
        v
    },
    reason: "a planeswalker that can't be attacked while it's attached to a creature: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn cant_be_attacked(&self, g: &Game, o: ObjectId) -> bool {
        super::active(g, UNATTACKABLE).iter().any(|(s, _)| *s == o)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
