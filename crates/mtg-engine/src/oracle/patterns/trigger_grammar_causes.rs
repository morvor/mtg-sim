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

/// `Condition::Custom`: the trigger event was caused by a spell or ability an opponent of
/// the ability's controller controls (`EventInfo::caused_by`, `kw/tap_causes.rs`).
pub const CAUSED_BY_OPPONENT: &str = "event cause: a spell or ability an opponent controls";

/// Adds `cond` to the trigger event `t` (inside a batch, to each event of it).
fn qualified(t: TriggerCond, cond: Condition) -> TriggerCond {
    match t {
        TriggerCond::Batched { trigger, per } => TriggerCond::Batched {
            trigger: Box::new(qualified(*trigger, cond)),
            per,
        },
        other => TriggerCond::Where {
            trigger: Box::new(other),
            cond,
        },
    }
}

/// "a spell or ability an opponent controls causes you to discard a card / cards"
/// (Spiritual Focus, Pure Intentions): the discard events record the controller of the
/// spell or ability whose effect made the player discard (`Event::Discarded::by`); a card
/// discarded to pay a cost or by a game rule wasn't (Pure Intentions's rulings: "only the
/// controller of the spell or ability matters").
fn caused_by_opponent(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(t).strip_prefix("a spell or ability an opponent controls causes you to discard ")?;
    if !matches!(r, "a card" | "cards" | "one or more cards") {
        return None;
    }
    let what = if r == "cards" { "one or more cards" } else { r };
    let (base, it, p) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever you discard {what}"))?;
    if !matches!(
        &base,
        TriggerCond::Discards { .. } | TriggerCond::Batched { .. }
    ) {
        return None;
    }
    Some((
        qualified(base, Condition::Custom(CAUSED_BY_OPPONENT.into())),
        it,
        p,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger causes: a spell or ability an opponent controls causes you to discard", priority: 100, parse: caused_by_opponent } }
