//! Smaller basic-effect constructs:
//!
//! - "shuffle target nontoken permanent you control into its owner's library" (CR 701.24):
//!   the object is shuffled into its owner's library.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// "shuffle [object] into its owner's library", "shuffle [objects] into their owners'
/// libraries".
fn shuffle_into_owners_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("shuffle ")?;
    let (what, tail) = object_ref(r, b)?;
    match tail.trim() {
        "into its owner's library" | "into their owners' libraries" => {
            Some(Effect::ShuffleInto { what })
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "basic effects: shuffle [object] into its owner's library", priority: 60, parse: shuffle_into_owners_library } }

/// "It gets an additional -1/-1 until end of turn for each Desert you control.", "Zombie
/// creatures you control get an additional +2/+2 until end of turn": "additional" only
/// says the change adds to an earlier one; it's the same change.
fn gets_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" an additional ")?;
    let head = &l[..i];
    if !(head.ends_with(" gets") || head.ends_with(" get")) {
        return None;
    }
    let text = format!("{head} {}", &l[i + " an additional ".len()..]);
    crate::oracle::effects::parse_clause(&text, b)
}

inventory::submit! { EffectPattern { name: "basic effects: gets an additional +N/+N", priority: 60, parse: gets_additional } }

/// "Each creature gets twice -X/-X until end of turn." (Nuclear Fallout): twice the amount.
fn gets_twice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" twice ")?;
    let head = &l[..i];
    if !(head.ends_with(" gets") || head.ends_with(" get")) {
        return None;
    }
    let text = format!("{head} {}", &l[i + " twice ".len()..]);
    let e = crate::oracle::effects::parse_clause(&text, b)?;
    let Effect::Modify {
        what,
        mods,
        duration,
    } = e
    else {
        return None;
    };
    let twice = |v: Value| Value::Mul(Box::new(Value::Const(2)), Box::new(v));
    let mods = mods
        .into_iter()
        .map(|m| match m {
            Modification::ModifyPT(p, t) => Some(Modification::ModifyPT(twice(p), twice(t))),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "basic effects: gets twice -X/-X", priority: 60, parse: gets_twice } }

/// "Until end of turn, double target creature's power X times." (Exponential Growth): the
/// power is doubled, then doubled again, X times in all (each doubling gives it +N/+0
/// where N is its power then, CR 701.10d).
fn double_n_times(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "Until end of turn, double ...": the duration belongs to the doubling.
    let moved;
    let l = match l.strip_prefix("until end of turn, ") {
        Some(r) => {
            let i = r.rfind(" times")?;
            let j = r[..i].rfind(' ')?;
            moved = format!("{} until end of turn{}", &r[..j], &r[j..]);
            moved.as_str()
        }
        None => l,
    };
    let i = l.rfind(" times")?;
    if !l[i..].trim_end().eq(" times") {
        return None;
    }
    let head = &l[..i];
    let j = head.rfind(' ')?;
    let (n, rest) = parse_number(&head[j + 1..])?;
    if !rest.is_empty() {
        return None;
    }
    let inner = &head[..j];
    if !(inner.contains("double ")) {
        return None;
    }
    let e = crate::oracle::effects::parse_clause(inner, b)?;
    Some(Effect::Repeat {
        times: n,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: double ... N times", priority: 60, parse: double_n_times } }

/// "That creature can block up to two additional creatures this turn", "target creature
/// can block an additional creature this turn", "... can block any number of creatures
/// this turn" (CR 509.1b).
fn can_block_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, l) = crate::oracle::effects::duration_suffix(end(l));
    if !matches!(dur, Duration::EndOfTurn) {
        return None;
    }
    let i = l.find(" can block ")?;
    let (what, tail) = object_ref(&l[..i], b)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let r = &l[i + " can block ".len()..];
    let n = match r {
        "an additional creature" => Some(1),
        "any number of creatures" => None,
        _ => {
            let r = r.strip_prefix("up to ")?.strip_suffix(" additional creatures")?;
            let (n, rest) = parse_number(r)?;
            if !rest.is_empty() {
                return None;
            }
            Some(n.as_const()? as u32)
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::ExtraBlocks {
            blocker: Filter::In(Box::new(what)),
            n,
        },
        duration: dur,
    })
}

inventory::submit! { EffectPattern { name: "basic effects: can block additional creatures this turn", priority: 60, parse: can_block_additional } }

/// "That creature gets an additional +4/+4 until end of turn unless any player pays {2}."
/// (Wild Might): any player may pay to stop it (CR 118.12a).
fn unless_any_player_pays(l: &str, b: &mut Builder) -> Option<Effect> {
    let (head, cost) = end(l).split_once(" unless any player pays ")?;
    let cost = crate::oracle::keywords::parse_keyword_cost(cost)?;
    let e = crate::oracle::effects::parse_clause(head, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::EachPlayer,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: unless any player pays", priority: 60, parse: unless_any_player_pays } }
