//! "Put a +1/+1 counter on target creature, two +1/+1 counters on another target
//! creature, and three +1/+1 counters on a third target creature." (Incremental Growth),
//! "~ deals 2 damage to target creature, 3 damage to another target creature, and 4
//! damage to a third target creature." (Serpentine Spike), "~ deals 1 damage to any
//! target, 2 damage to another target, and 3 damage to a third target." (Cone of Flame):
//! three instances of the word "target", each a different object or player from the
//! others (CR 115.3: "another target" and "a third target" can't be one already chosen),
//! so the spell needs three different legal targets to be cast. The first two are read
//! as "[A] and [B]" (the second omitting the verb), the third with the first's verb.
//! The damage is dealt by one instruction, so "a creature dealt damage this way" in a
//! later sentence means any of the three.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_simple, Builder};
use crate::oracle::phrases::end;

fn third_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first_two, third) = l.rsplit_once(", and ")?;
    if !third.contains("a third target") {
        return None;
    }
    let (first, second) = first_two.split_once(", ")?;
    if !second.contains("another target") || second.contains(", ") {
        return None;
    }
    let before = b.targets.len();
    let two = parse_clause(&format!("{first} and {second}"), b)?;
    // Exactly two new instances of "target", the second different from the first.
    if b.targets.len() != before + 2 {
        return None;
    }
    // The third part shares the first's verb ("put", "~ deals").
    let verb = if first.starts_with("put ") {
        "put ".to_string()
    } else {
        let (subject, _) = first.split_once(" deals ")?;
        format!("{subject} deals ")
    };
    let third = third.replacen("a third target", "another target", 1);
    let last = parse_simple(&format!("{verb}{third}"), b)?;
    if b.targets.len() != before + 3 {
        return None;
    }
    // Different from both earlier targets.
    let spec = b.targets.last_mut()?;
    for s in [before as u8, before as u8 + 1] {
        if !spec.distinct_from.contains(&s) {
            spec.distinct_from.push(s);
        }
    }
    spec.text = spec.text.replacen("another target", "a third target", 1);
    let mut parts = match two {
        Effect::Seq(v) => v,
        e => vec![e],
    };
    parts.push(last);
    Some(Effect::Seq(all_dealt_damage(parts)))
}

/// Accumulates what the damage parts were dealt to.
const DAMAGED_SO_FAR: Var = vars::USER + 3118;

/// The damage of the three parts is dealt by one instruction: "a creature dealt damage
/// this way" afterwards is any of the three, not only the last one.
fn all_dealt_damage(parts: Vec<Effect>) -> Vec<Effect> {
    if !parts.iter().all(|e| matches!(e, Effect::DealDamage { .. })) {
        return parts;
    }
    let n = parts.len();
    let so_far = Sel::Union(vec![Sel::Var(DAMAGED_SO_FAR), Sel::Var(vars::DAMAGED)]);
    let mut out = Vec::new();
    for (i, e) in parts.into_iter().enumerate() {
        out.push(e);
        out.push(Effect::Store {
            var: if i + 1 == n {
                vars::DAMAGED
            } else {
                DAMAGED_SO_FAR
            },
            sel: if i == 0 {
                Sel::Var(vars::DAMAGED)
            } else {
                so_far.clone()
            },
        });
    }
    out
}

inventory::submit! { EffectPattern { name: "[A target], [another target], and [a third target]", priority: 80, parse: third_target } }
