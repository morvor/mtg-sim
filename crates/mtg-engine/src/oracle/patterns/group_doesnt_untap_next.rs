//! "Lands you control don't untap during your next untap step." (the "Last" cards of
//! Amonkhet: Bontu's Last Reckoning, Kefnet's Last Word, ...): a rule-modifying effect
//! on a group of permanents, not locked to the permanents there as it resolves (CR 611.2c
//! applies to effects that modify characteristics or controllers): lands you control
//! during that untap step don't untap, including lands that weren't tapped or weren't on
//! the battlefield when the spell resolved. It lasts until your next untap step has
//! passed (CR 502.3).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};

fn group_doesnt_untap_next(l: &str, _b: &mut Builder) -> Option<Effect> {
    let subject = end(l).strip_suffix(" don't untap during your next untap step")?;
    let (f, plural, tail) = parse_object_phrase(subject)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    // Only groups of permanents you control ("lands you control", "creatures you
    // control"): "your next untap step" is the one in which they'd untap.
    let yours = match &f {
        Filter::And(v) => v
            .iter()
            .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You))),
        _ => false,
    };
    if !yours {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::DoesntUntap(f),
        duration: Duration::ThroughYourNextUntapStep,
    })
}

inventory::submit! { EffectPattern { name: "[permanents you control] don't untap during your next untap step", priority: 100, parse: group_doesnt_untap_next } }
