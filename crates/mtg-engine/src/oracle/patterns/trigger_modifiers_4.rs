//! Triggering an additional time for a scoped set of sources or events (CR 603.2d):
//!
//! - an event phrase the other patterns don't read: "If turning a face-down permanent face
//!   up causes a triggered ability of a permanent you control to trigger, ..." (Panoptic
//!   Projektor): abilities whose trigger event is a permanent being turned face up
//!   (CR 702.37e, 708.8);
//! - sources named by their relation to the source: "If a triggered ability of ~ or an
//!   Equipment attached to it triggers, ..." (Cloud, Midgar Mercenary, after "As long as
//!   ~ is equipped,"), "If a creature dying causes a triggered ability of ~ or an emblem
//!   you own to trigger, ..." (the ability The Masamune grants the equipped creature):
//!   the source itself, Equipment attached to it, emblems its controller owns (CR 114).

use super::StaticPattern;
use crate::ability::*;
use crate::kw::trigger_modifiers::EMBLEM;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

const ADDITIONAL: &str = ", that ability triggers an additional time";

/// One description of the objects whose triggered abilities trigger again.
fn source_item(s: &str) -> Option<Filter> {
    let s = s.trim();
    match s {
        "~" => return Some(Filter::Source),
        "an emblem you own" => {
            return Some(Filter::and(vec![
                Filter::Custom(EMBLEM.into()),
                Filter::OwnedBy(PlayerRel::You),
            ]))
        }
        "a permanent you control" => {
            return Some(Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(PlayerRel::You),
            ]))
        }
        _ => {}
    }
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    // "an Equipment attached to it": attached to the source.
    let (s, attached) = match s.strip_suffix(" attached to it") {
        Some(r) => (r, true),
        None => (s, false),
    };
    let (f, plural, tail) = parse_object_phrase(s)?;
    if plural || !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let mut parts = vec![f, Filter::Permanent];
    if attached {
        parts.push(Filter::AttachedTo(Box::new(Sel::This)));
    }
    Some(Filter::and(parts))
}

/// "~ or an Equipment attached to it": at least one item names the source or one of its
/// relations (the plainer lists are read in `basic_effects_additional_trigger.rs`).
fn sources(s: &str) -> Option<Filter> {
    let items: Vec<&str> = s.split(" or ").collect();
    if !items
        .iter()
        .any(|i| i.trim() == "~" || i.contains(" attached to it") || i.contains("emblem"))
    {
        return None;
    }
    let mut fs: Vec<Filter> = items.iter().map(|i| source_item(i)).collect::<Option<_>>()?;
    Some(if fs.len() == 1 {
        fs.pop().expect("one")
    } else {
        Filter::Or(fs)
    })
}

/// The trigger event an event phrase describes, for the events
/// `basic_effects_additional_trigger.rs` doesn't read.
fn event(ev: &str) -> Option<TriggerCond> {
    if ev == "turning a face-down permanent face up" {
        // The permanent is face up once the event has happened.
        return Some(TriggerCond::TurnedFaceUp(Filter::Permanent));
    }
    let np = ev.strip_suffix(" dying")?;
    let np = np.strip_prefix("a ").or_else(|| np.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(np)?;
    if plural || !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    Some(TriggerCond::Dies(f))
}

fn additional_time(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l).strip_prefix("if ")?.strip_suffix(ADDITIONAL)?;
    let (srcs, cause) = if let Some(r) = l.strip_prefix("a triggered ability of ") {
        (r.strip_suffix(" triggers")?, None)
    } else {
        let (ev, r) = l.split_once(" causes a triggered ability of ")?;
        (r.strip_suffix(" to trigger")?, Some(event(ev)?))
    };
    // Plain "a permanent you control" with a plain event is read elsewhere.
    let sources = if srcs == "a permanent you control" {
        if !matches!(cause, Some(TriggerCond::TurnedFaceUp(_))) {
            return None;
        }
        source_item(srcs)?
    } else {
        sources(srcs)?
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
            sources,
            cause: cause.map(Box::new),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r603.2d triggers an additional time: face up, source relations", priority: 105, parse: additional_time } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_scoped_sources_and_events() {
        let f = sources("~ or an equipment attached to it").expect("cloud");
        assert!(matches!(f, Filter::Or(ref v) if v.len() == 2 && v[0] == Filter::Source));
        assert!(sources("~ or an emblem you own").is_some());
        assert!(sources("a shaman or another wizard you control").is_none());
        assert!(matches!(
            event("turning a face-down permanent face up"),
            Some(TriggerCond::TurnedFaceUp(_))
        ));
        assert!(matches!(event("a creature dying"), Some(TriggerCond::Dies(_))));
    }
}
