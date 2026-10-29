//! "If damage would be dealt to ~, prevent that damage. Remove a +1/+1 counter from ~."
//! (the Phantom creatures of Judgment): a static prevention effect whose second
//! instruction is performed right after the damage is prevented (CR 615.5). It prevents
//! the damage even if ~ has no +1/+1 counters; applied to simultaneous damage from several
//! sources, it removes one counter (see `prevention_followups`); and if the damage can't
//! be prevented, the counter is still removed (CR 615.12).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn prevent_then_remove_counter(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (first, second) = l.split_once(". ")?;
    if first != "if damage would be dealt to ~, prevent that damage" {
        return None;
    }
    let kind = second
        .strip_prefix("remove a ")?
        .strip_suffix(" counter from ~")?;
    if !matches!(kind, "+1/+1" | "-1/-1") {
        return None;
    }
    let def = ReplacementDef {
        event: ReplacementEvent::Damage {
            source: Filter::Any,
            to_players: None,
            to_objects: Some(Filter::Source),
            combat_only: false,
        },
        action: ReplacementAction::PreventAndThen(
            None,
            Box::new(Effect::RemoveCounters {
                what: Sel::This,
                kind: Some(kind.into()),
                n: Value::c(1),
            }),
        ),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "if damage would be dealt to ~, prevent that damage and remove a counter from it", priority: 100, parse: prevent_then_remove_counter } }
