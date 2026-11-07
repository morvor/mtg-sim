//! "Zombies you control get +2/+1 until end of turn. Other creatures you control get +1/+1
//! until end of turn." (In Oketra's Name): in an instant or sorcery, "other" in a sentence
//! that follows one affecting a group means "not in that group" — there's no permanent
//! for it to mean "other than this one" (the spell itself is never on the battlefield).
//! Both sets are locked in as the spell resolves (CR 611.2c).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::types::CardType;

/// The group the previous sentence's effect affects ("Zombies you control get ..."): the
/// filter of its last `Modify` of all matching objects.
fn previous_group(e: &Effect) -> Option<&Filter> {
    match e {
        Effect::Modify {
            what: Sel::All(f), ..
        } => Some(f),
        Effect::Seq(v) => v.last().and_then(previous_group),
        _ => None,
    }
}

/// Replaces the `Other` in a top-level conjunction with "not in `group`"; whether it did.
fn other_than(f: &mut Filter, group: &Filter) -> bool {
    match f {
        Filter::Other => {
            *f = Filter::Not(Box::new(group.clone()));
            true
        }
        Filter::And(v) => {
            let n = v.iter().filter(|x| matches!(x, Filter::Other)).count();
            if n != 1 {
                return false;
            }
            v.iter_mut().any(|x| other_than(x, group))
        }
        _ => false,
    }
}

fn other_than_previous_group(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let spell = b.ctx.type_line.card_types.contains(CardType::Instant)
        || b.ctx.type_line.card_types.contains(CardType::Sorcery);
    if !spell || b.in_trigger || !l.trim_start().to_lowercase().starts_with("other ") {
        return false;
    }
    let Some(group) = previous_group(prev).cloned() else {
        return false;
    };
    if matches!(group, Filter::And(ref v) if v.iter().any(|x| matches!(x, Filter::Other))) {
        return false;
    }
    let targets = b.targets.len();
    let Some(mut e) = parse_sentence(l, b) else {
        b.targets.truncate(targets);
        return false;
    };
    let ok = match &mut e {
        Effect::Modify {
            what: Sel::All(f), ..
        } => other_than(f, &group),
        _ => false,
    };
    if !ok {
        b.targets.truncate(targets);
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "[group] get ... Other [objects] get ... (other than that group)", priority: 100, apply: other_than_previous_group } }
