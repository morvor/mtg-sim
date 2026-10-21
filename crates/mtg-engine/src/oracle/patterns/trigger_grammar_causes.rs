//! Trigger events qualified by their cause, as grammar:
//!
//! * "Whenever you tap an untapped creature an opponent controls" (Solitary Sanctuary,
//!   Icewrought Sentry, Hylda of the Icy Crown), "Whenever you tap one or more untapped
//!   creatures your opponents control" (Sharae of Numbing Depths): the permanent becomes
//!   tapped (CR 603.2e: an actual change) and you tapped it — the effect of a spell or
//!   ability you control tapped it, or you tapped it to pay a cost or by declaring it as
//!   an attacker (CR 701.26a, 508.1f). The event records who tapped it
//!   (`Event::Tapped::by`).
//! * "if it isn't being declared as an attacker" (Rhoda, Geist Avenger, Verity Circle):
//!   it didn't become tapped by being declared as an attacker (CR 508.1f; Verity Circle's
//!   rulings).

use super::{ConditionPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};

/// `Filter::Custom` on the trigger event's permanent: it didn't become tapped by being
/// declared as an attacker (`kw/tap_causes.rs`).
pub const NOT_TAPPED_ATTACKING: &str = "tap cause: not declared as an attacker";

fn you_tap(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (batch, r) = match t.strip_prefix("you tap one or more untapped ") {
        Some(r) => (true, r),
        None => (false, t.strip_prefix("you tap an untapped ")?),
    };
    let (f, plural, rest) = parse_object_phrase(r)?;
    if !rest.trim().is_empty() || plural != batch || f.zone().is_some() {
        return None;
    }
    let tapped_by_you = TriggerCond::Where {
        trigger: Box::new(TriggerCond::BecomesTapped(f)),
        cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::You),
    };
    let trigger = if batch {
        TriggerCond::Batched {
            trigger: Box::new(tapped_by_you),
            per: BatchPer::Batch,
        }
    } else {
        tapped_by_you
    };
    Some((trigger, Sel::TriggerObject, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "trigger causes: you tap an untapped creature an opponent controls", priority: 100, parse: you_tap } }

fn not_declared_attacking(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "it isn't being declared as an attacker" | "that creature isn't being declared as an attacker"
    )
    .then(|| {
        Condition::SelMatches(
            Sel::TriggerObject,
            Filter::Custom(NOT_TAPPED_ATTACKING.into()),
        )
    })
}

inventory::submit! { ConditionPattern { name: "trigger causes: it isn't being declared as an attacker", priority: 100, parse: not_declared_attacking } }
