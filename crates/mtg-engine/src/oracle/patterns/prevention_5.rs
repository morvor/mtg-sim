//! Prevention grammar V (cards4-costs-mana-replacements): damage that can't be prevented
//! or redirected (CR 615.12, 614.9).
//!
//! - "Damage that would be dealt to that creature this turn can't be prevented or dealt
//!   instead to another permanent or player." (Whippoorwill): a restriction on the damage
//!   dealt to that creature, locked onto it as the effect begins.
//! - "~ deals X damage to any target. If ~ would deal damage to a creature, that damage
//!   can't be prevented or dealt instead to another permanent or player." (Lava Burst):
//!   the spell's damage to creatures; created before the damage is dealt.

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

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

/// "For each 1 damage that would be dealt to ~, if it has a +1/+1 counter on it, remove a
/// +1/+1 counter from it and prevent that 1 damage." (Rock Hydra): damage to it is
/// prevented up to the number of +1/+1 counters on it, and that many counters (as much as
/// the damage, at most the counters it has) are removed (CR 615.7). The counters go even if
/// the damage can't be prevented (CR 615.12: the effect's other parts still happen).
fn per_point_counter_prevention(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("for each 1 damage that would be dealt to ~, if it has a ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counter on it, remove a ")?;
    let (kind2, r) = crate::oracle::costs::counter_kind(r)?;
    if kind != kind2 || r.trim() != "counter from it and prevent that 1 damage" {
        return None;
    }
    let counters = Value::CountersOn(Box::new(Sel::This), Some(kind.clone()));
    let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::Damage {
            source: Filter::Any,
            to_players: None,
            to_objects: Some(Filter::Source),
            combat_only: false,
        },
        action: ReplacementAction::PreventAndThen(
            Some(counters.clone()),
            Box::new(Effect::RemoveCounters {
                what: Sel::This,
                kind: Some(kind),
                n: Value::Min(
                    Box::new(Value::Var(crate::prevention::THAT_DAMAGE)),
                    Box::new(counters.clone()),
                ),
            }),
        ),
        self_replacement: false,
        optional: false,
    }));
    s.condition = Some(Condition::Compare(counters, Cmp::Ge, Value::c(1)));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "prevention 5: for each 1 damage that would be dealt to ~, if it has a counter, remove one and prevent that 1 damage", priority: 60, parse: per_point_counter_prevention } }

/// "give each player a rad counter [for each ...]" as "each player gets ...".
fn give_each_player_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (then, l) = match l.strip_prefix("then ") {
        Some(r) => ("then ", r),
        None => ("", l),
    };
    let r = l.strip_prefix("give each player ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let r = r.trim_start();
    if !(r.starts_with("rad counter") || r.starts_with("poison counter")) {
        return None;
    }
    let count = match n {
        Value::Const(1) => "a".to_string(),
        Value::Const(k) => k.to_string(),
        _ => return None,
    };
    crate::oracle::effects::parse_clause(&format!("{then}each player gets {count} {r}"), b)
}

inventory::submit! { EffectPattern { name: "prevention 5: give each player N rad counters", priority: 100, parse: give_each_player_counters } }

/// "If [damage event], prevent that damage. [Instruction]." with the instruction as a
/// separate sentence ("~'s controller and that player each draw half that many cards,
/// rounded down.", Sokrates, Athenian Teacher's granted ability): the same as "prevent that
/// damage and [instruction]" (CR 615.5). "~'s controller" in it is "you".
fn prevent_then_sentence(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (head, rest) = l.split_once(", prevent that damage. ")?;
    if !head.starts_with("if ") || rest.contains(". ") {
        return None;
    }
    let rest = rest
        .strip_prefix("~'s controller and that player each ")
        .map(|r| format!("you and that player each {r}"))
        .unwrap_or_else(|| rest.to_string());
    let joined = format!("{head}, prevent that damage and {rest}");
    let mut out = crate::oracle::statics::parse_static(&joined, ctx)?;
    for a in out.iter_mut() {
        let a = std::sync::Arc::make_mut(a);
        if !matches!(
            &a.kind,
            AbilityKind::Static(StaticAbility {
                effect: StaticEffect::Replacement(ReplacementDef {
                    action: ReplacementAction::PreventAndThen(..),
                    ..
                }),
                ..
            })
        ) {
            return None;
        }
        a.text = text.into();
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "prevention 5: if ..., prevent that damage. [instruction].", priority: 100, parse: prevent_then_sentence } }

/// "If a creature would deal combat damage to you and/or one or more creatures you control,
/// prevent X of that damage, where X is [value]." (Cover of Winter): X of the damage each
/// such source deals at once, shared among its recipients ([`ReplacementAction::
/// PreventPerSource`]; Cover of Winter's rulings).
fn prevent_x_per_source(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (head, x) = end(l).split_once(", prevent x of that damage, where x is ")?;
    if !head.contains(" and/or ") {
        return None;
    }
    let mut b = Builder::new(ctx);
    let (v, rest) = crate::oracle::statics::parse_value_phrase(x, &mut b)?;
    if !rest.trim().is_empty() || !b.targets.is_empty() {
        return None;
    }
    let mut out =
        crate::oracle::statics::parse_static(&format!("{head}, prevent 1 of that damage"), ctx)?;
    let [a] = out.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Static(s) = &mut a.kind else {
        return None;
    };
    let StaticEffect::Replacement(def) = &mut s.effect else {
        return None;
    };
    let ReplacementEvent::Damage {
        to_players: Some(_),
        to_objects: Some(_),
        ..
    } = &def.event
    else {
        return None;
    };
    if !matches!(
        def.action,
        ReplacementAction::PreventAmount(Value::Const(1))
    ) {
        return None;
    }
    def.action = ReplacementAction::PreventPerSource(v);
    a.text = text.into();
    Some(out)
}

inventory::submit! { StaticPattern { name: "prevention 5: if a creature would deal combat damage to you and/or creatures you control, prevent X of that damage", priority: 60, parse: prevent_x_per_source } }
