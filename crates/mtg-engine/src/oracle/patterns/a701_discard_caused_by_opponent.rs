//! "When a spell or ability an opponent controls causes you to discard ~, [effect]."
//! (Guerrilla Tactics, Psychic Purge): parsed as "When you discard ~, [effect]." (which
//! functions from wherever the card went, CR 113.6k) with the trigger condition narrowed
//! to discards an opponent's spell or ability caused; "that player" is that opponent. See
//! `kw/discarded_by_opponent.rs`.

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::discarded_by_opponent::DISCARDED_BY_OPPONENT;
use crate::oracle::CompileContext;

fn discard_caused_by_opponent(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest =
        t.strip_prefix("When a spell or ability an opponent controls causes you to discard ~, ")?;
    let a = crate::oracle::triggers::parse_triggered(&format!("When you discard ~, {rest}"), ctx)?;
    let AbilityKind::Triggered(mut tr) = a.kind.clone() else {
        return None;
    };
    if !matches!(
        tr.trigger,
        TriggerCond::Discards {
            filter: Filter::Source,
            ..
        }
    ) {
        return None;
    }
    tr.trigger = TriggerCond::Custom(DISCARDED_BY_OPPONENT.into());
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "a701 when a spell or ability an opponent controls causes you to discard ~", priority: 100, parse: discard_caused_by_opponent } }
