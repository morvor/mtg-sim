//! Oracle patterns for CR 603 wording: reflexive triggered abilities ("When you do, ...",
//! CR 603.12), abilities that count their own resolutions ("When this ability resolves for
//! the third time this turn, ...", CR 603.7h), "Do this only once each turn" (CR 603.2h),
//! and "sacrifice another [object]".

use super::{AbilityPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, parse_sentence, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Parses the text of a reflexive (or similar immediately-created) triggered ability into
/// its own body: it has its own targets. "That creature" refers to the objects the
/// preceding instruction acted on.
pub(crate) fn reflexive_body(text: &str, b: &Builder) -> Option<Body> {
    reflexive_body_about(text, b, Sel::Var(vars::IT))
}

/// [`reflexive_body`] where "it" is `it`.
pub(crate) fn reflexive_body_about(text: &str, b: &Builder, it: Sel) -> Option<Body> {
    let text = text
        .replace("that creature's power", "its power")
        .replace("that creature's toughness", "its toughness");
    // "When you do, she deals 4 damage to target creature." (Elektra, Femme Fatale), "When
    // you do, if you control a red permanent other than ~, he deals damage ..." (Ajani,
    // Nacatl Avenger): a character's pronoun is the card itself.
    let text = match ["she ", "he "].iter().find_map(|p| text.strip_prefix(p)) {
        Some(r) => format!("~ {r}"),
        None => text.replace(", she ", ", ~ ").replace(", he ", ", ~ "),
    };
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = it;
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
    when_you_do_about(l, b, Sel::Var(vars::IT))
}

/// [`when_you_do`] where the reflexive ability's "it" is `it`.
fn when_you_do_about(l: &str, b: &mut Builder, it: Sel) -> Option<Effect> {
    let (r, did) = if let Some(r) = l.strip_prefix("when you do, ") {
        (r, true)
    } else if let Some(r) = l.strip_prefix("when you don't, ") {
        (r, false)
    } else {
        return None;
    };
    let body = reflexive_body_about(r, b, it)?;
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

/// "Create a 2/1 ... Inkling creature token with flying. When you do, return up to one
/// target Aura or Equipment card ... attached to that token." (Forum Filibuster), "create
/// a colorless Equipment artifact token named Axe .... When you do, attach it to target
/// creature you control." (Dain Ironfoot): when what "you do" is creating tokens, the
/// reflexive ability's "it" and "that token" are the tokens created (CR 603.12).
fn f_when_you_do_after_creating(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(
        last_instruction(prev),
        Effect::CreateToken { .. }
            | Effect::CreateTokenWithPT { .. }
            | Effect::CreateTokenCopy { .. }
    ) {
        // (Not a Role token created attached to a creature: "When you do, that creature
        // fights ..." is about the creature, Curse of the Werefox.)
        return false;
    }
    let Some(e) = when_you_do_about(end(l), b, Sel::Var(vars::CREATED)) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

/// The last instruction of an effect (looking into sequences).
fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        _ => e,
    }
}

/// "When you do, create a token that's a copy of target permanent you control, except
/// .... Sacrifice it at the beginning of the next end step." (Saheeli, Radiant Creator):
/// a sentence following a reflexive triggered ability that creates tokens and refers to
/// them ("it", "them", "the token") continues that triggered ability's effect, after the
/// tokens were created (CR 603.12).
fn f_reflexive_continues(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let refers = l
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| matches!(w, "it" | "them" | "token" | "tokens"));
    if !refers || l.contains("target") {
        return false;
    }
    // (After a payment, the reflexive ability is the last of a sequence; see
    // `reflexive_after_payment`.)
    let prev = match prev {
        Effect::Seq(v) => match v.last_mut() {
            Some(last) => last,
            None => return false,
        },
        e => e,
    };
    let Effect::If {
        cond,
        then,
        otherwise,
    } = prev
    else {
        return false;
    };
    let did = match &*cond {
        Condition::PrevHappened => true,
        Condition::Not(c) => matches!(**c, Condition::PrevHappened),
        _ => false,
    };
    if !did || !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let Effect::Reflexive { body } = &mut **then else {
        return false;
    };
    if !matches!(
        last_instruction(&body.effect),
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. }
    ) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::CREATED);
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    // A sentence that modifies the body's last instruction ("Sacrifice it at the
    // beginning of the next end step." after "create a token ..."), or another
    // instruction about the tokens.
    if crate::oracle_ext::apply_followup_ext(l, &mut body.effect, &mut sub) {
        return sub.targets.is_empty();
    }
    let Some(e) = parse_sentence(l, &mut sub) else {
        return false;
    };
    if !sub.targets.is_empty() {
        return false;
    }
    let old = std::mem::replace(&mut body.effect, Effect::Noop);
    body.effect = Effect::seq(vec![old, e]);
    true
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
    // The triggered ability before it, as the compiler reads it on its own (including
    // whole-ability patterns: "you may draw that many cards", Terrasymbiosis).
    let a = match crate::oracle::triggers::parse_triggered(body, ctx) {
        Some(a) => a,
        None => {
            let mut v = crate::oracle::parse_ability(body, ctx)?;
            if v.len() != 1 {
                return None;
            }
            v.pop()?
        }
    };
    let AbilityKind::Triggered(mut tr) = a.kind.clone() else {
        return None;
    };
    tr.do_once_per_turn = true;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { EffectPattern { name: "reflexive: when you do", priority: 0, parse: when_you_do } }
inventory::submit! { FollowupPattern { name: "reflexive: continues after creating tokens", priority: 0, apply: f_reflexive_continues } }
inventory::submit! { FollowupPattern { name: "reflexive: when you do, after creating tokens", priority: 0, apply: f_when_you_do_after_creating } }
inventory::submit! { EffectPattern { name: "resolves for the nth time", priority: 0, parse: resolves_for_the_nth_time } }
inventory::submit! { EffectPattern { name: "pay any number of times", priority: 0, parse: pay_any_number_of_times } }
inventory::submit! { EffectPattern { name: "reflexive: when you pay this cost", priority: 0, parse: when_you_pay_this_cost } }
inventory::submit! { EffectPattern { name: "sacrifice another", priority: 0, parse: sacrifice_another } }
inventory::submit! { AbilityPattern { name: "do this only once each turn", priority: 0, parse: do_this_only_once } }

/// "this is the third time this ability has resolved this turn" (Inner-Flame Igniter; the
/// count includes this resolution).
fn nth_time_resolved(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("this is the ")?;
    let (w, rest) = split_word(r);
    let n = match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    };
    if end(rest) != "time this ability has resolved this turn" {
        return None;
    }
    Some(Condition::Compare(
        Value::TimesResolvedThisTurn,
        Cmp::Eq,
        Value::c(n),
    ))
}

inventory::submit! { super::ConditionPattern { name: "this is the nth time this ability has resolved this turn", priority: 0, parse: nth_time_resolved } }
