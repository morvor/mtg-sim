//! "If a Saproling was sacrificed this way" (Thallid Omnivore), "If an outlaw was
//! sacrificed this way" (Boneyard Desecrator): whether a permanent the ability's cost (or
//! an earlier instruction) sacrificed matched, using the permanent as it last existed on
//! the battlefield (CR 608.2h, 400.7: the recorded object is the battlefield one).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn sacrificed_this_way(c: &str) -> Option<Condition> {
    let c = end(c);
    // "If you sacrificed an Angel this way" (Shilgengar, Sire of Famine).
    // "If you sacrifice an Island this way" (Serendib Djinn): the same, in the present.
    let r = match c
        .strip_prefix("you sacrificed ")
        .or_else(|| c.strip_prefix("you sacrifice "))
    {
        Some(r) => r.strip_suffix(" this way")?,
        None => c.strip_suffix(" was sacrificed this way")?,
    };
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::CountSel(Box::new(Sel::Matching(
            Box::new(Sel::Var(vars::SACRIFICED)),
            f,
        ))),
        Cmp::Ge,
        Value::c(1),
    ))
}

inventory::submit! { ConditionPattern { name: "a [object] was sacrificed this way", priority: 100, parse: sacrificed_this_way } }

/// "Sacrifice another creature: Create a Blood token. If you sacrificed an Angel this way,
/// create a number of Blood tokens equal to its toughness instead." (Shilgengar, Sire of
/// Famine): the new instruction replaces the previous one if the condition holds; "it" is
/// the sacrificed permanent as it last existed on the battlefield.
fn sacrificed_instead(l: &str, prev: &mut Effect, b: &mut crate::oracle::effects::Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(" this way, ") else {
        return false;
    };
    let Some(x) = x.strip_suffix(" instead") else {
        return false;
    };
    let Some(cond) = sacrificed_this_way(&format!("{c} this way")) else {
        return false;
    };
    if matches!(prev, Effect::Seq(_) | Effect::Noop) {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::Var(vars::SACRIFICED);
    // "equal to its toughness": the sacrificed permanent's (not the tokens').
    let x = x.replace("equal to its ", "equal to the sacrificed creature's ");
    let e = crate::oracle::effects::parse_clause(&x, b);
    // Targets of its own would be chosen whether or not it happens.
    let Some(e) = e.filter(|_| b.targets.len() == saved.0) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    if std::mem::discriminant(&e) != std::mem::discriminant(&*prev) {
        b.it = saved.1;
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

inventory::submit! { super::FollowupPattern { name: "if [a permanent] was sacrificed this way, [effect] instead", priority: 55, apply: sacrificed_instead } }
