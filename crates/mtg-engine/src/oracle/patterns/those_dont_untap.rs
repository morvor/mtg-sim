//! "Tap all nonblue creatures. Those creatures don't untap during their controllers' next
//! untap steps." (CR 502.3): the plural pronoun refers to the objects the previous
//! sentence tapped.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;

/// The objects the effect's last part tapped.
fn tapped(e: &Effect) -> Option<Sel> {
    match e {
        Effect::Tap { what } => Some(what.clone()),
        Effect::Seq(v) => v.last().and_then(tapped),
        _ => None,
    }
}

fn f_those_dont_untap(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(subject) = [
        " don't untap during their controllers' next untap steps",
        " don't untap during their controller's next untap step",
    ]
    .iter()
    .find_map(|s| l.strip_suffix(s)) else {
        return false;
    };
    if !matches!(subject, "those creatures" | "those permanents" | "they") {
        return false;
    }
    let Some(what) = tapped(prev) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::AddRestriction {
            restriction: Restriction::DoesntUntap(Filter::In(Box::new(what))),
            duration: Duration::ThroughNextUntapStep,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "those creatures don't untap (after tapping them)", priority: 0, apply: f_those_dont_untap } }
