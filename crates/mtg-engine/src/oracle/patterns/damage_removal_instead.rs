//! "If [condition], [effect] instead." after a sentence: the effect of the previous
//! sentence is replaced when the condition holds as the spell or ability resolves (CR
//! 608.2c, 614.1a's "instead" in a one-shot effect):
//!
//! - "Target creature gets -2/-2 until end of turn. If ~ was kicked, that creature gets
//!   -5/-5 until end of turn instead."
//! - "~ deals 2 damage to any target. If ~ was kicked, it deals 4 damage instead." (only
//!   the amount changes; the recipients stay the same)
//! - "Draw a card. If you control a Wizard, draw two cards instead."
//! - "Exile the top two cards of your library. If ~'s additional cost was paid, exile the
//!   top three cards instead." (of the same library)
//! - "Create a token that's a copy of target creature. If ~ was kicked, create five of
//!   those tokens instead." (the same tokens, more of them)
//!
//! Only a previous sentence that is a single effect is replaced, and the replacement
//! can't introduce targets of its own.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_condition;

/// Conditions that can be parsed without knowing what "it"/"that" refer to.
fn pronoun_free(c: &str) -> bool {
    if matches!(
        c,
        "it's your turn" | "it's not your turn" | "it's night" | "it's day" | "it was kicked"
    ) {
        return true;
    }
    // "that spell" in a "whenever you cast" trigger is the spell that caused it to trigger
    // (opus: "if five or more mana was spent to cast that spell").
    if crate::oracle::patterns::opus::refers_to_the_trigger_spell(c) {
        return true;
    }
    !c.split(' ')
        .any(|w| matches!(w, "it" | "its" | "it's" | "that" | "they" | "their" | "them" | "those"))
}

/// "[source] deals N damage" with the recipients of the previous damage effect.
fn damage_amount(x: &str, prev: &Effect) -> Option<Effect> {
    let Effect::DealDamage { source, to, .. } = prev else {
        return None;
    };
    let r = ["it deals ", "~ deals ", "this creature deals ", "he deals ", "she deals "]
        .iter()
        .find_map(|p| x.strip_prefix(p))?;
    let (n, r) = parse_number(r)?;
    if r.trim() != "damage" {
        return None;
    }
    Some(Effect::DealDamage {
        source: source.clone(),
        amount: n,
        to: to.clone(),
    })
}

/// "exile the top N cards" of the library the previous effect exiled the top cards of
/// ("Exile the top two cards of your library. If ..., exile the top three cards
/// instead.").
fn top_cards_amount(x: &str, prev: &Effect) -> Option<Effect> {
    let Effect::Exile {
        what: Sel::TopOfLibrary(who, _),
        face_down,
        link,
    } = prev
    else {
        return None;
    };
    let (n, r) = parse_number(x.strip_prefix("exile the top ")?)?;
    if !matches!(r.trim(), "cards" | "card") {
        return None;
    }
    Some(Effect::Exile {
        what: Sel::TopOfLibrary(who.clone(), n),
        face_down: *face_down,
        link: *link,
    })
}

/// "create N of those tokens" with the tokens the previous effect creates ("Create a token
/// that's a copy of target creature. If ~ was kicked, create five of those tokens
/// instead.").
fn token_amount(x: &str, prev: &Effect) -> Option<Effect> {
    let (n, r) = parse_number(x.strip_prefix("create ")?)?;
    if r.trim() != "of those tokens" {
        return None;
    }
    let mut e = prev.clone();
    match &mut e {
        Effect::CreateToken { count, .. }
        | Effect::CreateTokenAttached { count, .. }
        | Effect::CreateTokenCopy { count, .. } => *count = n,
        _ => return None,
    }
    Some(e)
}

/// Whether an effect refers to the objects the previous instruction produced (`vars::IT`).
fn mentions_it(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|s| s.contains(&format!("{{\"Var\":{}}}", vars::IT)))
}

/// The target slots an effect refers to (`Sel::Target(i)`, also inside other selectors
/// and player references).
pub fn targets_mentioned(e: &Effect) -> Vec<usize> {
    let Ok(s) = serde_json::to_string(e) else {
        return Vec::new();
    };
    s.match_indices("\"Target\":")
        .filter_map(|(i, m)| {
            let digits: String = s[i + m.len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            digits.parse().ok()
        })
        .collect()
}

/// Whether `replacement` refers only to target slots `prev` refers to: a replacement for
/// the previous sentence that acts on a target only an earlier sentence acted on restates
/// that earlier sentence too.
pub fn targets_within(replacement: &Effect, prev: &Effect) -> bool {
    let before = targets_mentioned(prev);
    targets_mentioned(replacement)
        .iter()
        .all(|i| before.contains(i))
}

fn f_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    // "[effect] instead if [condition]" is "if [condition], [effect] instead" ("Put three
    // +1/+1 counters on that creature instead if there are four or more card types among
    // cards in your graveyard.").
    let reordered;
    let l = match l.split_once(" instead if ") {
        Some((x, c)) if !l.starts_with("if ") && !c.contains(", ") => {
            reordered = format!("if {c}, {x} instead");
            reordered.as_str()
        }
        _ => l,
    };
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    // "..., [effect] instead" or "..., instead [effect]".
    let Some(x) = x
        .strip_suffix(" instead")
        .or_else(|| x.strip_prefix("instead "))
    else {
        return false;
    };
    if matches!(prev, Effect::Seq(_) | Effect::Noop) || !pronoun_free(c) {
        return false;
    }
    let Some(cond) = parse_condition(c, b.ctx) else {
        return false;
    };
    // "If ~ was bargained, it deals twice X damage to that permanent instead": the subject
    // "it" is the condition's (the source), not the object the previous sentence affects.
    // Likewise "~ deals 2 damage to target creature. It deals 4 damage to that creature
    // instead if ...": "it" is the subject of the damage sentence it replaces, ~.
    let prev_source_is_this = matches!(
        prev,
        Effect::DealDamage {
            source: Sel::This,
            ..
        }
    );
    let subject_is_source;
    let x = match x.strip_prefix("it ") {
        Some(r) if c.starts_with("~ ") || (prev_source_is_this && r.starts_with("deals ")) => {
            subject_is_source = format!("~ {r}");
            subject_is_source.as_str()
        }
        _ => x,
    };
    let targets = b.targets.len();
    let replacement = match parse_clause(x, b) {
        Some(e) if b.targets.len() == targets => Some(e),
        _ => {
            b.targets.truncate(targets);
            damage_amount(x, prev)
                .or_else(|| top_cards_amount(x, prev))
                .or_else(|| token_amount(x, prev))
        }
    };
    let Some(e) = replacement else {
        return false;
    };
    // "... put that card onto the battlefield instead": a replacement that acts on what
    // the previous sentence produced ("it", "that card") can't stand in for it.
    if mentions_it(&e) {
        b.targets.truncate(targets);
        return false;
    }
    // "Return target card ... to your hand. Put up to one other target card ... on top of
    // your library. Exile ~. Adamant — If ..., instead return those cards to your hand and
    // exile ~.": a replacement that acts on a target the previous sentence doesn't restates
    // earlier sentences too, so it can't stand in for the previous sentence alone.
    if !targets_within(&e, prev) {
        b.targets.truncate(targets);
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    let e = restated_modify(&old, e);
    *prev = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(old),
    };
    true
}

/// "Until end of turn, target artifact or creature becomes an artifact creature with base
/// power and toughness 4/3. If evidence was collected, it has base power and toughness 1/1
/// until end of turn instead.": a replacement that restates only some of the
/// characteristics the previous sentence changes for the same objects (here the base power
/// and toughness) replaces just those; the rest of that effect still happens.
fn restated_modify(old: &Effect, new: Effect) -> Effect {
    let (
        Effect::Modify {
            what,
            mods,
            duration,
        },
        Effect::Modify {
            what: w2,
            mods: m2,
            duration: d2,
        },
    ) = (old, &new)
    else {
        return new;
    };
    let same_what = serde_json::to_string(what).ok() == serde_json::to_string(w2).ok();
    let same_duration = serde_json::to_string(duration).ok() == serde_json::to_string(d2).ok();
    if !same_what || !same_duration {
        return new;
    }
    let kinds: Vec<_> = m2.iter().map(std::mem::discriminant).collect();
    if !kinds
        .iter()
        .all(|k| mods.iter().any(|m| std::mem::discriminant(m) == *k))
    {
        return new;
    }
    let mut merged: Vec<Modification> = mods
        .iter()
        .filter(|m| !kinds.contains(&std::mem::discriminant(*m)))
        .cloned()
        .collect();
    merged.extend(m2.iter().cloned());
    Effect::Modify {
        what: what.clone(),
        mods: merged,
        duration: duration.clone(),
    }
}

inventory::submit! { FollowupPattern { name: "damage_removal: if [condition], [effect] instead", priority: 60, apply: f_instead } }
