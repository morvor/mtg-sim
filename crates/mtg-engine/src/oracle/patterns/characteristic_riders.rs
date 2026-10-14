//! Characteristic riders on permanents put onto the battlefield: "Return it to the
//! battlefield ... It's a Spirit Detective." (Otherworldly Escort), "It's a land named
//! Moon. It gains \"{T}: Add {C}.\"" (Princess Yue), "It has flying and is an Angel in
//! addition to its other types." (Valkyrie's Call), "The creature is a Vampire in
//! addition to its other types." (Drana), "They're black Zombies in addition to their
//! other colors and types and they gain decayed." (Ghouls' Night Out), "They are Food
//! artifacts with \"...\"" (Supper for Spiders), "It's an artifact." (Tom, Bert, and
//! William).
//!
//! The sentence describes what the permanent is as it enters: a continuous effect of the
//! resolving instruction that applies to it from the moment it's on the battlefield
//! (CR 611.2c, 611.3c, 614.1c). It's read as the "becomes" sentence it amounts to ("it
//! becomes a Spirit Detective", CR 205.1a: new subtypes replace the old ones of their
//! kind) and kept as the destination's modifications (`Destination::with_mods`).
//! A sentence that makes the permanent an Aura isn't read: an Aura put onto the
//! battlefield is attached as it enters (CR 303.4f), which these modifications don't do.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use super::statics::Subject;
use crate::types::CardType;

/// The subjects that name the object an earlier instruction put onto the battlefield.
const SUBJECTS: [(&str, &str); 8] = [
    ("it's ", "is "),
    ("it is ", "is "),
    ("it has ", "has "),
    ("the creature is ", "is "),
    ("that creature is ", "is "),
    ("they're ", "is "),
    ("they are ", "is "),
    ("the permanent is ", "is "),
];

/// The last instruction of `e`, if it puts objects onto the battlefield.
fn last_battlefield_move(e: &mut Effect) -> Option<&mut Destination> {
    match e {
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(to),
        Effect::Seq(v) => v.last_mut().and_then(last_battlefield_move),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_battlefield_move(then),
        Effect::May { effect, .. } => last_battlefield_move(effect),
        Effect::Reflexive { body } => last_battlefield_move(&mut body.effect),
        _ => None,
    }
}

/// "is a Spirit in addition to its other types and gains decayed" → "becomes a Spirit
/// in addition to its other types and gains decayed": the predicates of a "becomes"
/// sentence about one object.
fn as_becomes(pred: &str) -> String {
    let p = pred
        .replace("their other colors and types", "its other colors and types")
        .replace("their other types", "its other types")
        .replace(" and they gain ", " and gains ")
        .replace(" and they lose ", " and loses ")
        .replace(" and they have ", " and gains ")
        .replace(", and it loses ", " and loses ")
        .replace(" and it loses ", " and loses ")
        .replace(" and is ", " and becomes ")
        .replace(", is ", ", becomes ");
    if let Some(r) = p.strip_prefix("is ") {
        format!("becomes {r}")
    } else if let Some(r) = p.strip_prefix("has ") {
        format!("gains {r}")
    } else {
        p
    }
}

/// Plural type words of a group ("black Zombies", "Food artifacts", "4/4 creatures") as
/// one object's ("a black Zombies": type words read plurals as their singulars).
fn singular_types(p: &str) -> String {
    match p.strip_prefix("becomes ") {
        Some(r) if !r.starts_with("a ") && !r.starts_with("an ") => format!("becomes a {r}"),
        _ => p.to_string(),
    }
}

/// The modifications of a rider about the object `b.it` refers to.
fn rider_mods(l: &str, b: &mut Builder) -> Option<Vec<Modification>> {
    let l = end(l);
    let (subj, verb) = SUBJECTS.iter().find(|(s, _)| l.starts_with(s))?;
    let pred = format!("{verb}{}", &l[subj.len()..]);
    let mut sentence = as_becomes(&pred);
    if subj.starts_with("they") {
        sentence = singular_types(&sentence);
    }
    let saved = (b.targets.len(), b.it.clone());
    if !sentence.contains('"') {
        // The object put onto the battlefield (a creature card, or one the rider makes a
        // creature).
        let subj = Subject {
            filter: Filter::Any,
            it: Some(b.it.clone()),
            hint: CardType::Creature,
            lands: false,
            creatures: true,
        };
        let mut mods = Vec::new();
        for p in super::becomes_grammar::split_predicates(&sentence) {
            match super::becomes_grammar::predicate_mods(p, &subj, b) {
                Some(m) => mods.extend(m),
                None => {
                    b.targets.truncate(saved.0);
                    b.it = saved.1;
                    return None;
                }
            }
        }
        if b.targets.len() != saved.0 || mods.is_empty() || makes_aura(&mods) {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
        return Some(mods);
    }
    let e = crate::oracle::effects::parse_sentence(&format!("it {sentence}"), b);
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
    };
    let Some(Effect::Modify {
        mods,
        duration: Duration::Permanent,
        ..
    }) = e
    else {
        restore(b);
        return None;
    };
    if b.targets.len() != saved.0 || mods.is_empty() || makes_aura(&mods) {
        restore(b);
        return None;
    }
    b.it = saved.1;
    Some(mods)
}

/// Whether the modifications make the object an Aura (CR 303.4f: it must be attached as
/// it enters).
fn makes_aura(mods: &[Modification]) -> bool {
    mods.iter().any(|m| match m {
        Modification::SetTypes { subtypes, .. } | Modification::AddSubtypes(subtypes) => {
            subtypes.iter().any(|s| s == "Aura")
        }
        _ => false,
    })
}

/// "[return it to the battlefield]. It's [characteristics]."
fn rider_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    {
        let Some(to) = last_battlefield_move(prev) else {
            return false;
        };
        if !to.with_mods.is_empty() {
            return false;
        }
    }
    let Some(mods) = rider_mods(l, b) else {
        return false;
    };
    let Some(to) = last_battlefield_move(prev) else {
        return false;
    };
    to.with_mods = mods;
    true
}

inventory::submit! { FollowupPattern { name: "characteristic riders: [put it onto the battlefield]. it's [characteristics]", priority: 1150, apply: rider_followup } }

/// A rider that adds to the one before it: "She's a land named Moon. She gains \"{T}: Add
/// {C}.\"" (Princess Yue): "it gains [abilities]" after a rider is part of what the
/// permanent is as it enters.
fn rider_gains_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.starts_with("it gains ") {
        return false;
    }
    {
        let Some(to) = last_battlefield_move(prev) else {
            return false;
        };
        if to.with_mods.is_empty() {
            return false;
        }
    }
    let saved = (b.targets.len(), b.it.clone());
    let e = crate::oracle::effects::parse_sentence(l, b);
    let Some(Effect::Modify {
        mods,
        duration: Duration::Permanent,
        ..
    }) = e
    else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    if b.targets.len() != saved.0 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    }
    let Some(to) = last_battlefield_move(prev) else {
        return false;
    };
    to.with_mods.extend(mods);
    true
}

inventory::submit! { FollowupPattern { name: "characteristic riders: ... it gains [abilities] (as it enters)", priority: 1150, apply: rider_gains_followup } }

/// "Return it to the battlefield and it's a 3/3 Robot artifact creature with flying."
/// (Brilliance Unleashed): the rider joined to the instruction in one sentence.
fn and_its_rider(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, rider) = l.split_once(" and it's ")?;
    if !head.contains(" to the battlefield") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = crate::oracle::effects::parse_clause(head, b)?;
    let Some(mods) = rider_mods(&format!("it's {rider}"), b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    let to = last_battlefield_move(&mut e)?;
    if !to.with_mods.is_empty() {
        return None;
    }
    to.with_mods = mods;
    Some(e)
}

inventory::submit! { EffectPattern { name: "characteristic riders: return it to the battlefield and it's [characteristics]", priority: 1150, parse: and_its_rider } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    #[test]
    fn rider_predicates() {
        let tl = TypeLine::default();
        let ctx = CompileContext {
            card_name: "X",
            full_name: "X",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let mut b = Builder::new(&ctx);
        let s = singular_types(&as_becomes(
            "is black zombies in addition to their other colors and types and they gain decayed",
        ));
        assert_eq!(s, "becomes a black zombies in addition to its other colors and types and gains decayed");
        let subj = Subject {
            filter: Filter::Any,
            it: Some(Sel::This),
            hint: CardType::Creature,
            lands: false,
            creatures: true,
        };
        let parts = super::super::becomes_grammar::split_predicates(&s);
        assert_eq!(parts.len(), 2, "{parts:?}");
        for p in parts {
            assert!(super::super::becomes_grammar::predicate_mods(p, &subj, &mut b).is_some(), "{p}");
        }
    }
}
