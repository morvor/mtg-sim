//! An instruction repeated for a list of keywords: "put a flying counter on any creature
//! you control if a creature card in your graveyard has flying. Repeat this process for
//! first strike, double strike, ... and vigilance. Then put a +1/+1 counter on ~ for each
//! counter put on a creature this way." (Kathril, Aspect Warper). The repeated sentence
//! is expanded once per keyword (each a separate instruction, CR 608.2c) and the
//! counters put this way are counted (CR 122.1b keyword counters).

use super::{AbilityPattern, ConditionPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// "a creature card in your graveyard has flying": at least one such card has the keyword.
fn card_in_graveyard_has(s: &str) -> Option<Condition> {
    let s = end(s);
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (obj, kw) = r.rsplit_once(" has ")?;
    let kind = KeywordKind::from_name(kw)?;
    let obj = obj.strip_suffix(" in your graveyard")?;
    let (f, false, tail) = parse_object_phrase(obj)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        f,
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
        Filter::HasKeyword(kind),
    ])))
}

inventory::submit! { ConditionPattern { name: "a [card] in your graveyard has [keyword]", priority: 50, parse: card_in_graveyard_has } }

/// The keywords of "repeat this process for first strike, double strike, and vigilance".
fn keyword_list(s: &str) -> Option<Vec<String>> {
    let s = end(s);
    let mut out = Vec::new();
    for part in s.split(", ") {
        let part = part.strip_prefix("and ").unwrap_or(part).trim();
        KeywordKind::from_name(part)?;
        out.push(part.to_string());
    }
    (out.len() >= 2).then_some(out)
}

/// Splits `l` at the whole-word occurrences of `word`, for substituting another keyword.
fn substitute(l: &str, word: &str, with: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = l;
    let mut n = 0;
    while let Some(i) = rest.find(word) {
        let before_ok = i == 0 || !rest.as_bytes()[i - 1].is_ascii_alphanumeric();
        let after = &rest[i + word.len()..];
        let after_ok = !after.starts_with(|c: char| c.is_ascii_alphanumeric());
        out.push_str(&rest[..i]);
        if before_ok && after_ok {
            out.push_str(with);
            n += 1;
        } else {
            out.push_str(word);
        }
        rest = after;
    }
    out.push_str(rest);
    (n >= 2).then_some(out)
}

/// "When ~ enters, [sentence about flying]. Repeat this process for [keywords]. [rest]":
/// the sentence once for each keyword, then the rest.
fn repeat_this_process(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    if lower.contains('"') {
        return None;
    }
    let (head, tail) = lower.split_once(". repeat this process for ")?;
    let (list, rest) = tail.split_once(". ").unwrap_or((tail, ""));
    let kws = keyword_list(list)?;
    let (trigger, first) = head.split_once(", ")?;
    if !trigger.starts_with("when") {
        return None;
    }
    // "on any creature you control": one creature you choose.
    let first = first.replace(
        " on any creature you control ",
        " on a creature you control ",
    );
    let first = first.as_str();
    // The keyword the first sentence is about: the one it names (at least) twice.
    let base = ["flying", "first strike", "deathtouch"]
        .into_iter()
        .find(|k| substitute(first, k, k).is_some())?;
    let mut text = format!("{trigger}, {first}.");
    for k in &kws {
        text.push(' ');
        text.push_str(&substitute(first, base, k)?);
        text.push('.');
    }
    // "Then put a +1/+1 counter on ~ for each counter put on a creature this way."
    let rest = end(rest);
    let per_counter = match rest
        .strip_prefix("then ")
        .unwrap_or(rest)
        .strip_suffix(" for each counter put on a creature this way")
    {
        Some(head) => {
            let one = crate::oracle::parse_ability(&format!("{trigger}, {head}."), ctx)?;
            let [a] = one.as_slice() else {
                return None;
            };
            let AbilityKind::Triggered(t) = &a.kind else {
                return None;
            };
            let Effect::AddCounters {
                what,
                kind,
                n: Value::Const(1),
            } = &t.body.effect
            else {
                return None;
            };
            Some((what.clone(), kind.clone()))
        }
        None if rest.is_empty() => None,
        None => {
            text.push(' ');
            text.push_str(rest);
            text.push('.');
            None
        }
    };
    let mut abilities = crate::oracle::parse_ability(&text, ctx)?;
    if abilities.len() != 1 || matches!(abilities[0].kind, AbilityKind::Unsupported(_)) {
        return None;
    }
    let a = std::sync::Arc::make_mut(&mut abilities[0]);
    a.text = block.into();
    if let Some((what, kind)) = per_counter {
        let AbilityKind::Triggered(t) = &mut a.kind else {
            return None;
        };
        let mut counted = std::mem::take(&mut t.body.effect);
        if count_counters_put(&mut counted) < 2 {
            return None;
        }
        t.body.effect = Effect::seq(vec![
            Effect::StoreValue {
                var: PUT_THIS_WAY,
                value: Value::c(0),
            },
            counted,
            Effect::AddCounters {
                what,
                kind,
                n: Value::Var(PUT_THIS_WAY),
            },
        ]);
    }
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "repeat this process for [keywords]", priority: 120, parse: repeat_this_process } }

/// The number of counters "put on a creature this way".
const PUT_THIS_WAY: Var = vars::USER + 5106;

/// Makes each "if [condition], put a counter on [a creature]" of `e` also count the
/// counter it puts; how many instructions were counted.
fn count_counters_put(e: &mut Effect) -> usize {
    match e {
        Effect::Seq(v) => v.iter_mut().map(count_counters_put).sum(),
        Effect::If { then, .. } if matches!(**then, Effect::AddCounters { .. }) => {
            let Effect::AddCounters { what, kind, n } = std::mem::take(&mut **then) else {
                return 0;
            };
            let chosen = matches!(what, Sel::Choose { .. });
            **then = Effect::seq(vec![
                Effect::AddCounters {
                    what: what.clone(),
                    kind,
                    n: n.clone(),
                },
                Effect::StoreValue {
                    var: PUT_THIS_WAY,
                    value: Value::Sum(vec![
                        Value::Var(PUT_THIS_WAY),
                        if chosen {
                            // On the one creature chosen, if there was one.
                            Value::If(
                                Box::new(Condition::Exists(Filter::and(vec![
                                    Filter::Type(crate::types::CardType::Creature),
                                    Filter::ControlledBy(PlayerRel::You),
                                    Filter::InZone(ZoneKind::Battlefield),
                                ]))),
                                Box::new(n),
                                Box::new(Value::c(0)),
                            )
                        } else {
                            n
                        },
                    ]),
                },
            ]);
            1
        }
        _ => 0,
    }
}
