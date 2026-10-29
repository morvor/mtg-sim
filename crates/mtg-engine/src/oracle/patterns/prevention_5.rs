//! Prevention grammar V (cards4-costs-mana-replacements): damage that can't be prevented
//! or redirected (CR 615.12, 614.9).
//!
//! - "Damage that would be dealt to that creature this turn can't be prevented or dealt
//!   instead to another permanent or player." (Whippoorwill): a restriction on the damage
//!   dealt to that creature, locked onto it as the effect begins.
//! - "~ deals X damage to any target. If ~ would deal damage to a creature, that damage
//!   can't be prevented or dealt instead to another permanent or player." (Lava Burst):
//!   the spell's damage to creatures; created before the damage is dealt.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

const CANT: &str = " can't be prevented or dealt instead to another permanent or player";

/// "damage that would be dealt to [object] this turn can't be prevented or dealt instead to
/// another permanent or player".
fn damage_to_cant_be_prevented(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("damage that would be dealt to ")?;
    let who = r.strip_suffix(CANT)?.strip_suffix(" this turn")?;
    let saved = b.targets.len();
    let (sel, rest) = crate::oracle::effects::object_ref(who, b)?;
    if !rest.trim().is_empty() || b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::DamageToCantBePreventedOrRedirected(Filter::In(Box::new(sel))),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "prevention 5: damage that would be dealt to [object] this turn can't be prevented or redirected", priority: 60, parse: damage_to_cant_be_prevented } }

/// "If ~ would deal damage to [a creature], that damage can't be prevented or dealt instead
/// to another permanent or player." after the spell's damage instruction.
fn spell_damage_to_cant_be_prevented(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !b.ctx.is_spell() {
        return false;
    }
    let Some(r) = end(l).strip_prefix("if ~ would deal damage to ") else {
        return false;
    };
    let Some(to) = r.strip_suffix(&format!(", that damage{CANT}")) else {
        return false;
    };
    let to = to
        .strip_prefix("a ")
        .or_else(|| to.strip_prefix("an "))
        .unwrap_or(to);
    let Some((f, false, rest)) = parse_object_phrase(to) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let from_this = |e: &Effect| {
        matches!(
            e,
            Effect::DealDamage {
                source: Sel::This,
                ..
            }
        )
    };
    let ok = match &*prev {
        Effect::Seq(v) => !v.is_empty() && v.iter().all(from_this),
        e => from_this(e),
    };
    if !ok {
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        Effect::AddRestriction {
            restriction: Restriction::SourceDamageToCantBePreventedOrRedirected {
                source: Filter::In(Box::new(Sel::This)),
                to: f,
            },
            duration: Duration::EndOfTurn,
        },
        old,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "prevention 5: if ~ would deal damage to a creature, that damage can't be prevented or redirected", priority: 50, apply: spell_damage_to_cant_be_prevented } }
