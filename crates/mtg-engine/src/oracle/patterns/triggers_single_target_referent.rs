//! "Whenever you cast a spell that targets only a single creature, gain control of that
//! creature until end of turn. If it's your turn, untap that creature and it gains haste
//! until end of turn." (Loki, God of Lies): the trigger condition names the spell's one
//! target (CR 115.9c), and the effect's "that creature" and "it" are that target, not the
//! spell. (Effects about the spell itself — "copy that spell. The copy targets ~." — keep
//! the spell as their referent; see `copy_targets_source.rs`.)

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::parse_trigger_body;
use crate::oracle::CompileContext;

fn single_target_referent(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cond, body) = text.trim().split_once(", ")?;
    let cond = cond.to_lowercase();
    let r = cond.strip_prefix("whenever ")?;
    let (trigger, _, player) = super::copy_targets_source::cast_targeting_only_a_single(r)?;
    // The noun the effect names the target by ("creature" in "a single creature you
    // control"); an effect that names the spell is left to the usual reading.
    let (_, target) = r.split_once(" that targets only a single ")?;
    let noun = target.split(' ').next()?;
    let lower = body.to_lowercase();
    if !lower.contains(&format!("that {noun}"))
        || lower.contains("that spell")
        || lower.contains("copy")
    {
        return None;
    }
    let it = Sel::All(Filter::TargetOf(Box::new(Sel::TriggerSpell)));
    let body = parse_trigger_body(body, ctx, it, player)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(trigger, body)),
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "whenever you cast a spell that targets only a single [object], ... that [object]", priority: 100, parse: single_target_referent } }
