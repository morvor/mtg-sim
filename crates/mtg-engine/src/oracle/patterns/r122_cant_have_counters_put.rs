//! "[Objects] can't have [kind] counters put on them." (Blightbeetle: "Creatures your
//! opponents control can't have +1/+1 counters put on them."): a static ability whose
//! effect stops those counters from being put on those objects — modeled as a replacement
//! effect that prevents the event (CR 614.1, 614.16; counters "put" include those a
//! permanent would enter with, CR 122.6). A player who'd be told to put such counters on
//! one of them can't (fabricate creates Servos instead, CR 702.123a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn cant_have_counters_put(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, rest) = end(l).split_once(" can't have ")?;
    let kind = rest.strip_suffix(" put on them")?;
    let kind = match kind {
        "counters" => None,
        _ => {
            let (k, tail) = crate::oracle::costs::counter_kind(kind)?;
            if tail.trim() != "counters" {
                return None;
            }
            Some(k)
        }
    };
    let (filter, _, tail) = parse_object_phrase(subject)?;
    if !end(tail).is_empty() || matches!(filter, Filter::Source) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::PutCounters {
                    on_objects: Some(filter),
                    on_players: None,
                    kind,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r122 [objects] can't have [kind] counters put on them", priority: 100, parse: cant_have_counters_put } }
