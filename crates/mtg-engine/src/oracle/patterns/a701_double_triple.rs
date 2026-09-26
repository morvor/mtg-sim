//! Doubling and tripling (CR 701.10, 701.11):
//!
//! * power and/or toughness: "Double the power of target creature until end of turn.",
//!   "Double ~'s power and toughness until end of turn.", "Triple target creature's power
//!   and toughness until end of turn." — each creature gets +X/+Y where X and Y are its
//!   power and toughness (times two when tripling) as the effect is created, which is
//!   negative for a negative value (CR 701.10a–c, 701.11a–c);
//! * a life total: "Double target player's life total." (CR 701.10d);
//! * damage: "If a source you control would deal damage to a permanent or player, it deals
//!   double that damage to that permanent or player instead.", "Double all damage
//!   equipped creature would deal." — replacement effects (CR 701.10g).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The variable bound to each creature whose power/toughness is doubled.
const EACH: Var = vars::USER + 710;

/// "double" → 2, "triple" → 3.
fn factor(verb: &str) -> Option<i32> {
    match verb {
        "double" => Some(2),
        "triple" => Some(3),
        _ => None,
    }
}

/// "power", "toughness", "power and toughness" → (power?, toughness?).
fn stats(s: &str) -> Option<(bool, bool)> {
    match s {
        "power" => Some((true, false)),
        "toughness" => Some((false, true)),
        "power and toughness" => Some((true, true)),
        _ => None,
    }
}

/// The effect that multiplies the P/T of each selected creature by `k`.
fn multiply_pt(what: Sel, k: i32, (p, t): (bool, bool), duration: Duration) -> Effect {
    let each = || Box::new(Sel::Var(EACH));
    let delta = |v: Value, on: bool| {
        if on {
            Value::Mul(Box::new(v), Box::new(Value::c(k - 1)))
        } else {
            Value::c(0)
        }
    };
    Effect::ForEach {
        sel: what,
        var: EACH,
        effect: Box::new(Effect::Modify {
            what: Sel::Var(EACH),
            mods: vec![Modification::ModifyPT(
                delta(Value::PowerOf(each()), p),
                delta(Value::ToughnessOf(each()), t),
            )],
            duration,
        }),
    }
}

/// "double the power of target creature until end of turn", "double target creature's
/// power and toughness until end of turn", "triple ~'s power until end of turn".
fn double_pt(l: &str, b: &mut Builder) -> Option<Effect> {
    let (verb, r) = end(l).split_once(' ')?;
    let k = factor(verb)?;
    let (duration, r) = duration_suffix(r);
    if matches!(duration, Duration::Permanent) {
        return None;
    }
    // "the power [and toughness] of [objects]"
    if let Some(r) = r.strip_prefix("the ") {
        for s in ["power and toughness", "power", "toughness"] {
            if let Some(objs) = r.strip_prefix(s).and_then(|x| x.strip_prefix(" of ")) {
                let (what, tail) = object_ref(objs, b)?;
                if !end(&tail).is_empty() {
                    return None;
                }
                return Some(multiply_pt(what, k, stats(s)?, duration));
            }
        }
        return None;
    }
    // "[object]'s power [and toughness]"
    let (obj, s) = r.rsplit_once("'s ")?;
    let which = stats(s)?;
    let (what, tail) = object_ref(obj, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(multiply_pt(what, k, which, duration))
}

inventory::submit! { EffectPattern { name: "a701 double power and toughness", priority: 100, parse: double_pt } }

/// "double target player's life total", "double your life total": the player gains or
/// loses the life needed for twice their current total (CR 701.10d).
fn double_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let (verb, r) = end(l).split_once(' ')?;
    let k = factor(verb)?;
    let who = r.strip_suffix(" life total")?;
    let who = who
        .strip_suffix("'s")
        .or_else(|| (who == "your").then_some("you"))?;
    let (p, tail) = player_ref(who, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let change = Value::Mul(
        Box::new(Value::LifeTotal(p.clone())),
        Box::new(Value::c(k - 1)),
    );
    Some(Effect::If {
        cond: Condition::Compare(Value::LifeTotal(p.clone()), Cmp::Ge, Value::c(0)),
        then: Box::new(Effect::GainLife {
            who: p.clone(),
            n: change.clone(),
        }),
        otherwise: Box::new(Effect::LoseLife {
            who: p,
            n: Value::Mul(Box::new(change), Box::new(Value::c(-1))),
        }),
    })
}

inventory::submit! { EffectPattern { name: "a701 double life total", priority: 100, parse: double_life } }

fn damage_replacement(source: Filter, k: i32, text: &str) -> Vec<Ability> {
    vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Damage {
                    source,
                    to_players: Some(PlayerFilter::Any),
                    to_objects: Some(Filter::Any),
                    combat_only: false,
                },
                action: ReplacementAction::Multiply(k),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )]
}

/// "If a source [you control] would deal damage to a permanent or player, it deals
/// double that damage to that permanent or player instead." and "Double all damage
/// [equipped creature / sources you control] would deal." (CR 701.10g).
fn double_damage(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("if a source ") {
        let (who, r) = r.split_once("would deal damage to a permanent or player, it deals ")?;
        let source = match who.trim() {
            "" => Filter::Any,
            "you control" => Filter::ControlledBy(PlayerRel::You),
            "an opponent controls" => Filter::ControlledBy(PlayerRel::Opponent),
            _ => return None,
        };
        let (verb, r) = r.split_once(' ')?;
        let k = factor(verb)?;
        if r != "that damage to that permanent or player instead" {
            return None;
        }
        return Some(damage_replacement(source, k, text));
    }
    let (verb, r) = l.split_once(' ')?;
    let k = factor(verb)?;
    let who = r.strip_prefix("all damage ")?.strip_suffix(" would deal")?;
    let source = match who {
        "equipped creature" | "enchanted creature" => Filter::AttachedToSource,
        "~" => Filter::Source,
        _ => {
            let who = who.strip_prefix("that ").unwrap_or(who);
            let (f, _, tail) = parse_object_phrase(who)?;
            if !end(tail).is_empty() {
                return None;
            }
            f
        }
    };
    Some(damage_replacement(source, k, text))
}

inventory::submit! { StaticPattern { name: "a701 double damage", priority: 100, parse: double_damage } }
