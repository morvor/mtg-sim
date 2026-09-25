//! Oracle patterns for stickers (CR 123): "you may put a [name] sticker on [it / a
//! nonland permanent you own]", "you get {TK}", "put a +1/+1 counter on it for each
//! unique vowel on that sticker", "as long as you control a stickered permanent",
//! "whenever you place a sticker".

use super::{AbilityPattern, ConditionPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use crate::oracle::CompileContext;

fn sticker_kind(s: &str) -> Option<(Option<StickerType>, &str)> {
    for (p, k) in [
        ("a sticker", None),
        ("a name sticker", Some(StickerType::Name)),
        ("an art sticker", Some(StickerType::Art)),
        ("an ability sticker", Some(StickerType::Ability)),
        (
            "a power and toughness sticker",
            Some(StickerType::PowerToughness),
        ),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((k, r));
        }
    }
    None
}

/// "[you may] put a [name] sticker on [it / ~ / a nonland permanent you own]" (CR 123.3).
fn put_sticker(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (optional, l) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let rest = l.strip_prefix("put ")?;
    let (kind, rest) = sticker_kind(rest)?;
    let what_text = rest.strip_prefix(" on ")?;
    let what = match what_text {
        "it" | "~" | "this creature" => b.it.clone(),
        _ => {
            let s = what_text
                .strip_prefix("a ")
                .or_else(|| what_text.strip_prefix("an "))?;
            let (f, _, tail) = parse_object_phrase(s)?;
            if !end(tail).is_empty() {
                return None;
            }
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: f,
                count: Value::c(1),
                up_to: false,
                store: None,
            }
        }
    };
    let e = Effect::PutSticker {
        who: PlayerRef::You,
        what,
        kind,
        max_ticket: None,
        free: false,
    };
    Some(if optional {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

/// "you get {TK}{TK}": ticket counters (CR 107.17).
fn get_tickets(l: &str, _b: &mut Builder) -> Option<Effect> {
    let rest = end(l).strip_prefix("you get ")?.trim();
    let n = rest.matches("{tk}").count();
    if n == 0 || !rest.replace("{tk}", "").trim().is_empty() {
        return None;
    }
    Some(Effect::AddPlayerCounters {
        who: PlayerRef::You,
        kind: crate::types::counters::TICKET.into(),
        n: Value::c(n as i32),
    })
}

/// "put a +1/+1 counter on it for each unique vowel on that sticker" (CR 123.6e).
fn counters_per_vowel(l: &str, b: &mut Builder) -> Option<Effect> {
    let rest = end(l).strip_prefix("put a ")?;
    let (kind, rest) = rest.split_once(" counter on ")?;
    let what = match rest {
        "it for each unique vowel on that sticker" | "~ for each unique vowel on that sticker" => {
            b.it.clone()
        }
        _ => return None,
    };
    Some(Effect::AddCounters {
        what,
        kind: kind.into(),
        n: Value::Custom("sticker unique vowels".into()),
    })
}

/// "you gain x life, where x is the number of unique vowels on that sticker" (CR 123.6e).
fn life_per_vowel(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "you gain x life, where x is the number of unique vowels on that sticker" {
        return None;
    }
    Some(Effect::GainLife {
        who: PlayerRef::You,
        n: Value::Custom("sticker unique vowels".into()),
    })
}

/// "you control a stickered permanent" (CR 123.4).
fn controls_stickered(c: &str) -> Option<Condition> {
    let rest = c.strip_prefix("you control ")?;
    let (n, rest) = parse_number(rest)?;
    let Value::Const(n) = n else {
        return None;
    };
    let rest = rest.trim_start();
    let rest = rest
        .strip_prefix("stickered permanents")
        .or_else(|| rest.strip_prefix("stickered permanent"))
        .or_else(|| rest.strip_prefix("more stickered permanents"))?;
    let rest = rest.trim_start().strip_prefix("or more").unwrap_or(rest);
    if !end(rest).is_empty() {
        return None;
    }
    let f = Filter::and(vec![
        Filter::Permanent,
        Filter::ControlledBy(PlayerRel::You),
        Filter::HasSticker(None),
    ]);
    Some(if n <= 1 {
        Condition::Exists(f)
    } else {
        Condition::Compare(Value::Count(f), Cmp::Ge, Value::c(n))
    })
}

/// "~ has [keyword] as long as you control a stickered permanent" (CR 123.4).
fn has_while_stickered(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let (head, cond) = end(&lower).split_once(" as long as ")?;
    let cond = controls_stickered(cond)?;
    let head_text = &t[..head.len()];
    let mut out = crate::oracle::statics::parse_static(head_text, ctx)?;
    for a in out.iter_mut() {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let mut s = s.clone();
        s.condition = Some(cond.clone());
        *a = AbilityDef::new(AbilityKind::Static(s), t);
    }
    Some(out)
}

/// "whenever you place a sticker", "whenever you put a sticker on ~".
fn place_sticker_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let which = match end(r) {
        "whenever you place a sticker" | "you place a sticker" => "you",
        "whenever you put a sticker on ~" | "you put a sticker on ~" => "self",
        _ => return None,
    };
    Some((
        TriggerCond::Custom(format!("sticker placed:{which}").into()),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { EffectPattern { name: "put a sticker", priority: 0, parse: put_sticker } }
inventory::submit! { EffectPattern { name: "you get tickets", priority: 0, parse: get_tickets } }
inventory::submit! { EffectPattern { name: "counters per unique vowel", priority: 0, parse: counters_per_vowel } }
inventory::submit! { EffectPattern { name: "life per unique vowel", priority: 0, parse: life_per_vowel } }
inventory::submit! { ConditionPattern { name: "you control a stickered permanent", priority: 0, parse: controls_stickered } }
inventory::submit! { AbilityPattern { name: "has ... as long as stickered", priority: 0, parse: has_while_stickered } }
inventory::submit! { TriggerPattern { name: "place a sticker", priority: 0, parse: place_sticker_trigger } }
