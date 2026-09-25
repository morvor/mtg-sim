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
    *prev = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "damage_removal: if [condition], [effect] instead", priority: 60, apply: f_instead } }
