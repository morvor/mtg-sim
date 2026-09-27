//! Parts of an effect that happen only under a condition, in the word orders the core
//! "if [condition], [effect]" doesn't cover:
//!
//! * "[effect] unless [condition]": "Draw three cards. Then discard a card unless this
//!   spell was cast using teamwork." (Timeline Inquiry), "sacrifice ~ unless a nonland
//!   permanent left the battlefield this turn or a spell was warped this turn";
//! * "[effect] if [condition]": "Also put a +1/+1 counter on that creature if this spell
//!   was cast using teamwork." (Beast Mode);
//! * "If [condition], instead [effect]" (or "..., [effect] instead") where the replacement
//!   has targets of its own: "Exile target creature with mana value 3 or less. If this
//!   spell was cast using teamwork, instead exile target creature and you gain 3 life."
//!   (Cruel Alliance), "Exile target artifact or enchantment. If this spell was kicked,
//!   exile target nonland permanent instead." (Tear Asunder). Only for conditions settled
//!   by choices made as the spell was cast (an optional cost paid or not): its controller
//!   then chooses the targets of the part that will have its effect only (CR 601.2c,
//!   702.194c), and "the chosen card" in a later sentence is whichever was chosen.
//!
//! Conditions that mention "it", "that", "they" and the like aren't taken: parsed on their
//! own, those pronouns would mean the source, not what the effect is about ("destroy
//! target creature if it's black").

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::patterns::damage_removal_instead::targets_mentioned;
use crate::oracle::phrases::end;
use crate::oracle::statics::parse_condition;

/// Whether a condition's words can be parsed without a referent.
fn pronoun_free(c: &str) -> bool {
    !c.split(' ')
        .any(|w| matches!(w, "it" | "its" | "it's" | "that" | "they" | "their" | "them" | "those"))
}

/// Whether a condition is settled by what was chosen as the spell was cast (optional costs
/// paid or not), so the targets of a part it governs can be chosen accordingly (CR
/// 601.2c).
fn cast_choice(c: &Condition) -> bool {
    match c {
        Condition::CostPaid(_) => true,
        Condition::Not(c) => cast_choice(c),
        Condition::And(v) | Condition::Or(v) => v.iter().all(cast_choice),
        _ => false,
    }
}

/// The targets an effect introduced (slots `from..`) are chosen only if `cond` holds.
fn condition_targets(b: &mut Builder, from: usize, cond: &Condition) {
    if !cast_choice(cond) {
        return;
    }
    for spec in &mut b.targets[from..] {
        spec.condition = Some(match spec.condition.take() {
            Some(c) => Condition::And(vec![c, cond.clone()]),
            None => cond.clone(),
        });
    }
}

/// "[effect] unless [condition]" and "[effect] if [condition]".
fn trailing_condition(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (x, c, negate) = match (l.rsplit_once(" unless "), l.rsplit_once(" if ")) {
        (Some((x, c)), _) => (x, c, true),
        (None, Some((x, c))) => (x, c, false),
        _ => return None,
    };
    // "unless [player] pays ...", "if able", "if you do": not conditions of this kind.
    // Several conditional instructions joined together ("A if {U} was spent to cast this
    // spell, and B if {R} was spent to cast this spell") are split first.
    if x.is_empty()
        || x.ends_with(',')
        || x.contains(" if ")
        || x.contains(" unless ")
        || !pronoun_free(c)
    {
        return None;
    }
    let cond = parse_condition(c, b.ctx)?;
    let cond = if negate {
        Condition::Not(Box::new(cond))
    } else {
        cond
    };
    let from = b.targets.len();
    let e = parse_clause(x, b)?;
    condition_targets(b, from, &cond);
    Some(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "[effect] unless/if [condition]", priority: 400, parse: trailing_condition } }

/// "If [condition], instead [effect]" / "If [condition], [effect] instead" where the
/// replacement has targets of its own (see the module docs).
fn instead_with_own_targets(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let Some(x) = x
        .strip_prefix("instead ")
        .or_else(|| x.strip_suffix(" instead"))
    else {
        return false;
    };
    let Some(cond) = parse_condition(c, b.ctx) else {
        return false;
    };
    if !cast_choice(&cond) {
        return false;
    }
    // The previous sentence introduced every target so far: they're its alone. ("Choose
    // target creature card ..." as the first sentence only chooses its target.)
    let before = b.targets.len();
    let prev_targets = targets_mentioned(prev);
    let only_chooses = matches!(prev, Effect::Noop) && b.sentences == 1;
    if before == 0 || (0..before).any(|i| !prev_targets.contains(&i) && !only_chooses) {
        return false;
    }
    // "~ deals 5 damage to target creature. If ~ was kicked, it deals 10 damage divided as
    // you choose among any number of targets instead.": the subject "it" is the source.
    let source_deals = matches!(
        prev,
        Effect::DealDamage {
            source: Sel::This,
            ..
        }
    );
    let subject_is_source;
    let x = match x.strip_prefix("it ") {
        Some(r) if c.starts_with("~ ") || (source_deals && r.starts_with("deals ")) => {
            subject_is_source = format!("~ {r}");
            subject_is_source.as_str()
        }
        _ => x,
    };
    let old_it = b.it.clone();
    let Some(e) = parse_clause(x, b) else {
        return false;
    };
    // Without targets of its own, it's the core "instead" patterns' case.
    if b.targets.len() == before {
        return false;
    }
    // CR 601.2c, 702.194c: the targets of the part that won't have its effect aren't
    // chosen.
    let kept = targets_mentioned(&e);
    let not = Condition::Not(Box::new(cond.clone()));
    for (i, spec) in b.targets.iter_mut().enumerate() {
        let c = if i >= before {
            cond.clone()
        } else if !kept.contains(&i) {
            not.clone()
        } else {
            continue;
        };
        spec.condition = Some(match spec.condition.take() {
            Some(old) => Condition::And(vec![old, c]),
            None => c,
        });
    }
    // "Return the chosen card to the battlefield.": whichever target was chosen.
    if let (Sel::Target(o), Sel::Target(n)) = (&old_it, &b.it) {
        if o != n {
            b.it = Sel::Union(vec![old_it.clone(), b.it.clone()]);
        }
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "if [cast choice], instead [effect with its own targets]", priority: 400, apply: instead_with_own_targets } }
