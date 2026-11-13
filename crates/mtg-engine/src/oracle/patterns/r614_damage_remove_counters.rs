//! "If ~ would be dealt damage, remove that many [kind] counters from it instead. If you
//! can't, sacrifice it." (Underdark Beholder): a replacement effect (CR 614.1a) — the
//! damage isn't dealt; instead that many counters of the kind are removed from the
//! permanent, or, when it doesn't have that many, it's sacrificed.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn damage_remove_counters(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (first, second) = l.split_once(". ")?;
    if second != "if you can't, sacrifice it" {
        return None;
    }
    let kind = first
        .strip_prefix("if ~ would be dealt damage, remove that many ")?
        .strip_suffix(" counters from it instead")?;
    if kind.is_empty()
        || !kind
            .chars()
            .all(|c| c.is_alphabetic() || matches!(c, '+' | '-' | '/'))
    {
        return None;
    }
    let has_that_many = Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(kind.into())),
        Cmp::Ge,
        Value::EventAmount,
    );
    let def = ReplacementDef {
        event: ReplacementEvent::Damage {
            source: Filter::Any,
            to_players: None,
            to_objects: Some(Filter::Source),
            combat_only: false,
        },
        action: ReplacementAction::Instead(Box::new(Effect::If {
            cond: has_that_many,
            then: Box::new(Effect::RemoveCounters {
                what: Sel::This,
                kind: Some(kind.into()),
                n: Value::EventAmount,
            }),
            otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
        })),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "if ~ would be dealt damage, remove that many counters from it instead", priority: 100, parse: damage_remove_counters } }
