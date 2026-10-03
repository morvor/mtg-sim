//! "If [noncombat] damage would be dealt to ~, prevent that damage. Put a +1/+1 counter on
//! ~ for each 1 damage prevented this way." (Stormwild Capridor; Phyrexian Hydra with
//! -1/-1 counters), "If damage would be dealt to another creature you control, prevent
//! that damage. Put a +1/+1 counter on that creature for each 1 damage prevented this
//! way." (Vigor): a static prevention effect whose second instruction is performed right
//! after the damage is prevented (CR 615.5), counting the damage prevented.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn prevent_then_counters(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (first, second) = l.split_once(". ")?;
    let (noncombat, r) = match first.strip_prefix("if noncombat damage would be dealt to ") {
        Some(r) => (true, r),
        None => (false, first.strip_prefix("if damage would be dealt to ")?),
    };
    // The recipients, and how the second sentence refers to the one dealt damage.
    let (recipients, on, what) = match r {
        "~, prevent that damage" => (Filter::Source, "~", Sel::This),
        // "another creature you control" (Vigor): "that creature" is the one the damage
        // would have been dealt to.
        "another creature you control, prevent that damage" => (
            Filter::and(vec![
                Filter::creature(),
                Filter::ControlledBy(PlayerRel::You),
                Filter::Not(Box::new(Filter::Source)),
            ]),
            "that creature",
            Sel::TriggerObject,
        ),
        _ => return None,
    };
    let kind = second
        .strip_prefix("put a ")?
        .strip_suffix(" for each 1 damage prevented this way")?
        .strip_suffix(&format!(" counter on {on}"))?;
    if !matches!(kind, "+1/+1" | "-1/-1") {
        return None;
    }
    let event = if noncombat {
        ReplacementEvent::NoncombatDamage {
            source: Filter::Any,
            to_players: None,
            to_objects: Some(recipients),
        }
    } else {
        ReplacementEvent::Damage {
            source: Filter::Any,
            to_players: None,
            to_objects: Some(recipients),
            combat_only: false,
        }
    };
    let def = ReplacementDef {
        event,
        action: ReplacementAction::PreventAndThen(
            None,
            Box::new(Effect::AddCounters {
                what,
                kind: kind.into(),
                n: Value::EventAmount,
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

inventory::submit! { StaticPattern { name: "if damage would be dealt to ~, prevent that damage and put counters on it", priority: 100, parse: prevent_then_counters } }
