//! Oracle patterns for CR 603 wording: reflexive triggered abilities ("When you do, ...",
//! CR 603.12), abilities that count their own resolutions ("When this ability resolves for
//! the third time this turn, ...", CR 603.7h), "Do this only once each turn" (CR 603.2h),
//! and "sacrifice another [object]".

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Parses the text of a reflexive (or similar immediately-created) triggered ability into
/// its own body: it has its own targets. "That creature" refers to the objects the
/// preceding instruction acted on.
fn reflexive_body(text: &str, b: &Builder) -> Option<Body> {
    let text = text
        .replace("that creature's power", "its power")
        .replace("that creature's toughness", "its toughness");
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    let effect = parse_effect_text(&text, &mut sub)?;
    Some(Body {
        targets: sub.targets,
        effect,
        modal: None,
    })
}

/// "When you do, [effect]" / "When you don't, [effect]" (CR 603.12).
fn when_you_do(l: &str, b: &mut Builder) -> Option<Effect> {
    let (r, did) = if let Some(r) = l.strip_prefix("when you do, ") {
        (r, true)
    } else if let Some(r) = l.strip_prefix("when you don't, ") {
        (r, false)
    } else {
        return None;
    };
    let body = reflexive_body(r, b)?;
    let reflexive = Effect::Reflexive {
        body: Box::new(body),
    };
    let cond = if did {
        Condition::PrevHappened
    } else {
        Condition::Not(Box::new(Condition::PrevHappened))
    };
    Some(Effect::If {
        cond,
        then: Box::new(reflexive),
        otherwise: Box::new(Effect::Noop),
    })
}

/// "When this ability resolves for the Nth time this turn, [effect]" (CR 603.7h): a
/// delayed triggered ability created only during that resolution, triggering as it ends.
fn resolves_for_the_nth_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("when this ability resolves for the ")?;
    let (w, rest) = split_word(r);
    let n = match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    };
    let rest = rest.strip_prefix("time this turn, ")?;
    let body = reflexive_body(rest, b)?;
    Some(Effect::If {
        cond: Condition::Compare(Value::TimesResolvedThisTurn, Cmp::Eq, Value::c(n)),
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

/// Variables used while paying a cost any number of times.
const PAID_COUNT: Var = vars::USER + 90;
const PAID_STOP: Var = vars::USER + 91;

/// "pay {cost} any number of times": the player pays it as many times as they choose
/// (and can); "that many" is the number of payments (bound to X).
fn pay_any_number_of_times(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("pay ")?;
    let c = r.strip_suffix(" any number of times")?;
    let cost = crate::oracle::keywords::parse_keyword_cost(c)?;
    let pay_once = Effect::If {
        cond: Condition::Compare(Value::Var(PAID_STOP), Cmp::Eq, Value::c(0)),
        then: Box::new(Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::StoreValue {
                var: PAID_COUNT,
                value: Value::Sum(vec![Value::Var(PAID_COUNT), Value::c(1)]),
            }),
            otherwise: Box::new(Effect::StoreValue {
                var: PAID_STOP,
                value: Value::c(1),
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    Some(Effect::Seq(vec![
        Effect::StoreValue {
            var: PAID_COUNT,
            value: Value::c(0),
        },
        Effect::StoreValue {
            var: PAID_STOP,
            value: Value::c(0),
        },
        Effect::Repeat {
            times: Value::c(30),
            effect: Box::new(pay_once),
        },
        Effect::SetX {
            value: Value::Var(PAID_COUNT),
        },
    ]))
}

/// "When you pay this cost one or more times, [effect]": a reflexive triggered ability
/// that triggers only once however many times the cost was paid (CR 603.12a).
fn when_you_pay_this_cost(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("when you pay this cost one or more times, ")?;
    // "..., then create twice that many [tokens]": X doubles for the rest.
    let body = match r.split_once(", then ") {
        Some((first, second)) if second.contains("twice that many") => {
            let mut sub = Builder::new(b.ctx);
            sub.in_trigger = true;
            sub.it = Sel::Var(vars::IT);
            sub.it_player = b.it_player.clone();
            let e1 = parse_effect_text(&first.replace("that many", "x"), &mut sub)?;
            let e2 = parse_effect_text(&second.replace("twice that many", "x"), &mut sub)?;
            Body {
                targets: sub.targets,
                effect: Effect::Seq(vec![
                    e1,
                    Effect::SetX {
                        value: Value::Sum(vec![Value::X, Value::X]),
                    },
                    e2,
                ]),
                modal: None,
            }
        }
        _ => reflexive_body(&r.replace("that many", "x"), b)?,
    };
    Some(Effect::If {
        cond: Condition::Compare(Value::X, Cmp::Ge, Value::c(1)),
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

/// "sacrifice another creature" — the controller of the effect sacrifices one.
fn sacrifice_another(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("sacrifice ")?;
    if !r.starts_with("another ") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Sacrifice {
        who: PlayerRef::You,
        filter: f,
        count: Value::c(1),
    })
}

/// A triggered ability ending in "Do this only once each turn." (CR 603.2h).
fn do_this_only_once(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    if !(lower.starts_with("when") || lower.starts_with("at ")) {
        return None;
    }
    let body = t.strip_suffix("Do this only once each turn.")?.trim_end();
    let a = crate::oracle::triggers::parse_triggered(body, ctx)?;
    let AbilityKind::Triggered(mut tr) = a.kind.clone() else {
        return None;
    };
    tr.do_once_per_turn = true;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { EffectPattern { name: "reflexive: when you do", priority: 0, parse: when_you_do } }
inventory::submit! { EffectPattern { name: "resolves for the nth time", priority: 0, parse: resolves_for_the_nth_time } }
inventory::submit! { EffectPattern { name: "pay any number of times", priority: 0, parse: pay_any_number_of_times } }
inventory::submit! { EffectPattern { name: "reflexive: when you pay this cost", priority: 0, parse: when_you_pay_this_cost } }
inventory::submit! { EffectPattern { name: "sacrifice another", priority: 0, parse: sacrifice_another } }
inventory::submit! { AbilityPattern { name: "do this only once each turn", priority: 0, parse: do_this_only_once } }
