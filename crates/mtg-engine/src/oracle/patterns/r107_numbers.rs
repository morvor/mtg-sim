//! Numbers (CR 107.1–107.3): "half ..., rounded up/down" (CR 107.1a), "where X is ..."
//! (CR 107.3c) with negative results treated as 0 (CR 107.1b), and "+X/+Y" with two
//! defined variables (CR 107.3p).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "half the number of forests you control, rounded down", "half their life, rounded up",
/// or a value phrase understood by the core compiler. Returns the value and the rest.
pub fn value_phrase(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("half ") {
        let (inner, rest) = if let Some(x) = r.strip_prefix("the number of ") {
            let (f, _, tail) = parse_object_phrase(x)?;
            (Value::Count(f), tail.to_string())
        } else if let Some(x) = r.strip_prefix("your life total") {
            (Value::LifeTotal(PlayerRef::You), x.to_string())
        } else {
            let (v, tail) = crate::oracle::statics::parse_value_phrase(r, b)?;
            (v, tail)
        };
        let rest = rest.trim_start();
        let (up, rest) = if let Some(x) = rest.strip_prefix(", rounded up") {
            (true, x)
        } else if let Some(x) = rest.strip_prefix(", rounded down") {
            (false, x)
        } else {
            // CR 107.1a: the text says how to round.
            return None;
        };
        return Some((Value::Div(Box::new(inner), 2, up), rest.to_string()));
    }
    crate::oracle::statics::parse_value_phrase(s, b)
}

/// CR 107.1b: a calculation that determines the result of an effect uses 0 instead of a
/// negative number.
fn nonnegative(v: Value) -> Value {
    Value::Max(Box::new(v), Box::new(Value::c(0)))
}

/// Replaces `Value::X` (and optionally `Value::Y` written as the variable `y`) in an
/// effect.
fn substitute_x(e: &Effect, x: &Value) -> Option<Effect> {
    substitute_x_in(e, x)
}

/// Replaces `Value::X` in any part of an ability (an effect, a target's number or
/// division).
fn substitute_x_in<T: serde::Serialize + serde::de::DeserializeOwned>(
    t: &T,
    x: &Value,
) -> Option<T> {
    let json = serde_json::to_value(t).ok()?;
    let xv = serde_json::to_value(x).ok()?;
    let out = subst(json, &serde_json::Value::String("X".into()), &xv);
    serde_json::from_value(out).ok()
}

fn subst(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::Value as J;
    if &v == from {
        return to.clone();
    }
    match v {
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, subst(x, from, to)))
                .collect(),
        ),
        J::Array(a) => J::Array(a.into_iter().map(|x| subst(x, from, to)).collect()),
        other => other,
    }
}

/// "[effect], where X is [value]" (CR 107.3c: the text defines X, so the controller
/// doesn't choose it; the value is determined as the effect is performed).
fn where_x_is(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, value_s) = l.rsplit_once(", where x is ")?;
    let (v, tail) = value_phrase(value_s, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let first_target = b.targets.len();
    let e = crate::oracle::effects::parse_clause(clause, b)?;
    let x = nonnegative(v);
    // "Return up to X target permanents ..., where X is ...": the defined X is also the
    // number of targets (or the amount divided among them).
    for i in first_target..b.targets.len() {
        b.targets[i] = substitute_x_in(&b.targets[i], &x)?;
    }
    substitute_x(&e, &x)
}

inventory::submit! { EffectPattern { name: "r107 where x is", priority: 70, parse: where_x_is } }

/// "Whenever you cast a spell with {X} in its mana cost, [effect with X]" (Zaxara): the X
/// in the effect is that spell's X (CR 107.3e).
fn cast_x_spell_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let eff = lower.strip_prefix("whenever you cast a spell with {x} in its mana cost, ")?;
    let x = Value::XOf(Box::new(Sel::TriggerSpell));
    // "create [token], then put X +1/+1 counters on it": "it" is the created token.
    let body = if let Some((create, counters)) = end(eff).split_once(", then put x ") {
        let first = crate::oracle::effects::parse_trigger_body(
            create,
            ctx,
            Sel::TriggerSpell,
            PlayerRef::You,
        )?;
        if !matches!(first.effect, Effect::CreateToken { .. }) {
            return None;
        }
        let (kind, rest) = crate::oracle::costs::counter_kind(counters)?;
        if end(rest) != "counters on it" {
            return None;
        }
        Body {
            effect: Effect::seq(vec![
                first.effect,
                Effect::AddCounters {
                    what: Sel::Var(vars::CREATED),
                    kind,
                    n: x.clone(),
                },
            ]),
            ..first
        }
    } else {
        let body = crate::oracle::effects::parse_trigger_body(
            eff,
            ctx,
            Sel::TriggerSpell,
            PlayerRef::You,
        )?;
        let effect = substitute_x(&body.effect, &x)?;
        Body { effect, ..body }
    };
    let tr = TriggeredAbility::new(
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::and(vec![Filter::Spell, Filter::HasX]),
        },
        body,
    );
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "r107 cast spell with x", priority: 70, parse: cast_x_spell_trigger } }

/// "−X: [effect]" loyalty abilities (CR 107.7: [−X] means "Remove X loyalty counters from
/// this permanent"; X is announced as it's activated, CR 107.3a).
fn minus_x_loyalty(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().replace('\u{2212}', "-");
    let eff = t.strip_prefix("-X: ")?;
    let body = crate::oracle::effects::parse_body(eff, ctx)?;
    let mut act = ActivatedAbility::new(
        Cost {
            mana: None,
            parts: vec![CostPart::RemoveCounters {
                kind: crate::types::counters::LOYALTY.into(),
                count: Value::X,
            }],
        },
        body,
    );
    act.is_loyalty = true;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "r107 minus x loyalty", priority: 70, parse: minus_x_loyalty } }

/// "[trigger], you may pay [cost]. If you do, [effect]. [If you don't, [effect].]" (e.g.
/// Flameblast Dragon: "you may pay {X}{R}. If you do, it deals X damage to any target").
/// An X in the cost isn't defined by the ability, so the controller chooses it as they pay
/// (CR 107.3f).
fn may_pay_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    if !(lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at ")) {
        return None;
    }
    let idx = lower.find(", you may pay ")?;
    let cond_s = &lower[..idx];
    let rest = &lower[idx + ", you may pay ".len()..];
    let (cost_s, then_s) = rest.split_once(". if you do, ")?;
    // "If you don't, ..." is what happens when the cost isn't paid. An "otherwise" may
    // refer to another condition in the text, so leave such text to the core compiler.
    let (then_s, else_s) = match then_s.split_once(". if you don't, ") {
        Some((a, b)) => (a, Some(b)),
        None => (then_s, None),
    };
    if then_s.contains("otherwise") || else_s.is_some_and(|e| e.contains("otherwise")) {
        return None;
    }
    let (trigger, it, it_player) = crate::oracle::triggers::parse_trigger_condition(cond_s)?;
    let (cost, loyalty) = crate::oracle::costs::parse_cost(cost_s)?;
    if loyalty {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it.clone();
    b.it_player = it_player.clone();
    let then = crate::oracle::effects::parse_effect_text(then_s, &mut b)?;
    let otherwise = match else_s {
        Some(e) => {
            b.it = it;
            b.it_player = it_player;
            crate::oracle::effects::parse_effect_text(e, &mut b)?
        }
        None => Effect::Noop,
    };
    let body = Body {
        targets: b.targets,
        effect: Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        },
        modal: None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(trigger, body)),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "r107 may pay trigger", priority: 70, parse: may_pay_trigger } }

/// "sacrifice ~ unless you pay its mana cost" (CR 107.3h: an {X} in it is 0 for a
/// permanent).
fn sacrifice_unless_mana_cost(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "sacrifice ~ unless you pay its mana cost" {
        return None;
    }
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::PayManaCostOf(Box::new(Sel::This))],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
    })
}

inventory::submit! { EffectPattern { name: "r107 sacrifice unless mana cost", priority: 70, parse: sacrifice_unless_mana_cost } }

/// "[player] loses half their life, rounded up" (CR 107.1a).
fn loses_half_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, up) = if let Some(s) = l.strip_suffix(" loses half their life, rounded up") {
        (s, true)
    } else if let Some(s) = l.strip_suffix(" loses half their life, rounded down") {
        (s, false)
    } else if let Some(s) = l.strip_suffix(" lose half your life, rounded up") {
        (s, true)
    } else if let Some(s) = l.strip_suffix(" lose half your life, rounded down") {
        (s, false)
    } else {
        return None;
    };
    let who = match subject {
        "that player" => b.it_player.clone(),
        "you" => PlayerRef::You,
        "each player" => PlayerRef::EachPlayer,
        "each opponent" => PlayerRef::EachOpponent,
        "target player" | "target opponent" => {
            let filter = if subject == "target player" {
                PlayerFilter::Any
            } else {
                PlayerFilter::Opponent
            };
            let slot = b.add_target(TargetSpec::player(filter, subject), subject);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::LoseLife {
            who: PlayerRef::Iterated,
            n: Value::Div(Box::new(Value::LifeTotal(PlayerRef::Iterated)), 2, up),
        }),
    })
}

inventory::submit! { EffectPattern { name: "r107 half life", priority: 70, parse: loses_half_life } }

/// "Enchanted creature gets +X/+Y, where X is [value], and Y is [value]." (CR 107.3p:
/// Y follows the same rules as X.)
fn enchanted_gets_xy(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, affected) = if let Some(r) = l.strip_prefix("enchanted creature gets ") {
        (r, Filter::AttachedToSource)
    } else if let Some(r) = l.strip_prefix("equipped creature gets ") {
        (r, Filter::AttachedToSource)
    } else {
        return None;
    };
    let (pt, rest) = subject.split_once(", where x is ")?;
    let tl = crate::types::TypeLine::default();
    let cctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let mut b = Builder::new(&cctx);
    let (x, tail) = value_phrase(rest, &mut b)?;
    let (p, t) = match pt {
        "+x/+y" => {
            let y_s = tail.trim_start().strip_prefix(", and y is ")?;
            let (y, tail2) = value_phrase(y_s, &mut b)?;
            if !end(&tail2).is_empty() {
                return None;
            }
            (nonnegative(x), nonnegative(y))
        }
        "+x/+x" => {
            if !end(&tail).is_empty() {
                return None;
            }
            (nonnegative(x.clone()), nonnegative(x))
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: vec![Modification::ModifyPT(p, t)],
        })),
        text,
    )])
}

fn enchanted_gets_xy_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    enchanted_gets_xy(end(&lower), block, ctx)
}

inventory::submit! { AbilityPattern { name: "r107 gets +x/+y", priority: 70, parse: enchanted_gets_xy_block } }
