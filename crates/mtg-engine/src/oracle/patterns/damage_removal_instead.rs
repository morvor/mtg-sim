//! "If [condition], [effect] instead." after a sentence: the effect of the previous
//! sentence is replaced when the condition holds as the spell or ability resolves (CR
//! 608.2c, 614.1a's "instead" in a one-shot effect):
//!
//! - "Target creature gets -2/-2 until end of turn. If ~ was kicked, that creature gets
//!   -5/-5 until end of turn instead."
//! - "~ deals 2 damage to any target. If ~ was kicked, it deals 4 damage instead." (only
//!   the amount changes; the recipients stay the same)
//! - "Draw a card. If you control a Wizard, draw two cards instead."
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

/// Whether an effect refers to the objects the previous instruction produced (`vars::IT`).
fn mentions_it(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|s| s.contains(&format!("{{\"Var\":{}}}", vars::IT)))
}

fn f_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let Some(x) = x.strip_suffix(" instead") else {
        return false;
    };
    if matches!(prev, Effect::Seq(_) | Effect::Noop) || !pronoun_free(c) {
        return false;
    }
    let Some(cond) = parse_condition(c, b.ctx) else {
        return false;
    };
    let targets = b.targets.len();
    let replacement = match parse_clause(x, b) {
        Some(e) if b.targets.len() == targets => Some(e),
        _ => {
            b.targets.truncate(targets);
            damage_amount(x, prev)
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
