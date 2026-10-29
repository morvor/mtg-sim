//! "if that spell was kicked" in a "Whenever you cast a spell" trigger (Bloodstone
//! Goblin): "that spell" is the spell that caused the ability to trigger, and it was
//! kicked if any of its kicker costs was paid (CR 702.33d). As an intervening "if" clause
//! it's checked as the ability triggers and as it resolves (CR 603.4), from the spell's
//! last known information if it has left the stack.

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;

fn that_spell_was_kicked(c: &str) -> Option<Condition> {
    let c = c.trim().trim_end_matches(['.', ',']);
    matches!(c, "that spell was kicked" | "that spell is kicked").then(|| {
        Condition::SelMatches(
            Sel::TriggerSpell,
            Filter::CastWithCost(crate::kw::kicker::KICKED.into()),
        )
    })
}

inventory::submit! {
    ConditionPattern {
        name: "kicker: that spell was kicked",
        priority: 100,
        parse: that_spell_was_kicked,
    }
}
