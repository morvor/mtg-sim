//! Conditional "instead" alternatives to the previous instruction (CR 608.2c), in the
//! word orders and with the referents the core "instead" patterns
//! (`damage_removal_instead`, `dig_grammar`, `conditional_parts`) don't read:
//!
//! - "[effect] instead if [condition]" and "If [condition], [effect] instead" are read as
//!   "If [condition], instead [effect]" when that's what the patterns know;
//! - "put each of those cards into your hand instead" is "put those cards into your hand";
//!   "that nonland permanent card" names the card "that card" names;
//! - "If you control a Gideon planeswalker, that player sacrifices two of those creatures
//!   of their choice instead." (Gideon's Triumph): the same instruction with another
//!   number of the objects it described;
//! - a sentence after "When you do, [effect]." that replaces that effect belongs to the
//!   reflexive triggered ability ("If the sacrificed creature was a Giant, ~ deals twice
//!   that much damage instead.", Surtland Flinger; CR 603.12).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use std::cell::Cell;

thread_local! {
    /// Set while a reworded sentence is being tried, so the rewording isn't applied again.
    static REWORDING: Cell<bool> = const { Cell::new(false) };
}

/// The sentence reworded for the core patterns, if any rewording applies.
fn reworded(l: &str) -> Option<String> {
    let mut t = l.to_string();
    // "[effect] instead if [condition]".
    if !t.starts_with("if ") {
        if let Some((x, c)) = t.rsplit_once(" instead if ") {
            if !c.contains(", ") {
                t = format!("if {c}, instead {x}");
            }
        }
    }
    // "If [condition], [effect] instead" → "If [condition], instead [effect]".
    if let Some(r) = t.strip_prefix("if ") {
        if let Some((c, x)) = r.split_once(", ") {
            if let Some(x) = x.strip_suffix(" instead") {
                if !x.starts_with("instead ") {
                    t = format!("if {c}, instead {x}");
                }
            }
        }
    }
    t = t.replace("each of those cards", "those cards");
    // "that nonland permanent card" → "that card".
    let words: Vec<&str> = t.split(' ').collect();
    let mut out: Vec<&str> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        if words[i] == "that" {
            if let Some(k) = (i + 2..words.len().min(i + 5)).find(|k| words[*k] == "card") {
                let between = &words[i + 1..k];
                if between
                    .iter()
                    .all(|w| matches!(*w, "nonland" | "permanent" | "creature" | "land" | "nontoken" | "noncreature" | "artifact" | "enchantment"))
                {
                    out.push("that");
                    out.push("card");
                    i = k + 1;
                    continue;
                }
            }
        }
        out.push(words[i]);
        i += 1;
    }
    let t = out.join(" ");
    (t != l).then_some(t)
}

fn instead_reworded(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if REWORDING.with(|r| r.get()) || !l.contains("instead") {
        return false;
    }
    let Some(t) = reworded(end(l)) else {
        return false;
    };
    // A condition about an object the text named other than a card ("If that land is a
    // Plains") may mean the trigger's object, which the pronoun grammar doesn't track
    // here.
    let cond = t.strip_prefix("if ").and_then(|r| r.split_once(", ")).map(|(c, _)| c);
    if cond.is_some_and(|c| {
        c.split(' ')
            .collect::<Vec<_>>()
            .windows(2)
            .any(|w| w[0] == "that" && w[1] != "card" && w[1] != "spell")
    }) {
        return false;
    }
    REWORDING.with(|r| r.set(true));
    let ok = crate::oracle_ext::apply_followup_ext(&t, prev, b);
    REWORDING.with(|r| r.set(false));
    if ok {
        all_looked_at(prev);
    }
    ok
}

/// "Look at the top three cards ... Put one of those cards into your hand and the rest on
/// the bottom ... If [condition], put each of those cards into your hand instead.": the
/// cards are still looked at; all of them go to the hand.
fn all_looked_at(e: &mut Effect) {
    let Effect::If {
        then, otherwise, ..
    } = e
    else {
        return;
    };
    let (Effect::Move { what: Sel::Var(v), to }, Effect::Dig { .. }) = (&**then, &**otherwise)
    else {
        return;
    };
    if *v != vars::IT {
        return;
    }
    let to = to.clone();
    let Effect::Dig {
        who,
        n,
        reveal,
        rest_to,
        ..
    } = (**otherwise).clone()
    else {
        return;
    };
    **then = Effect::Dig {
        who,
        n: n.clone(),
        reveal,
        filter: Filter::Any,
        take: n,
        take_up_to: false,
        take_to: to,
        rest_to,
    };
}

inventory::submit! { FollowupPattern { name: "followup instead: reworded for the core patterns", priority: 990, apply: instead_reworded } }

/// "If [condition], that player sacrifices two of those creatures of their choice
/// instead." after "[player] sacrifices a [creature] ...".
fn more_of_those(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let Some(x) = x.strip_suffix(" instead") else {
        return false;
    };
    let Effect::Sacrifice { who, filter, .. } = &*prev else {
        return false;
    };
    let Some(after) = x
        .strip_prefix("that player sacrifices ")
        .or_else(|| x.strip_prefix("they sacrifice "))
    else {
        return false;
    };
    let Some((n, rest)) = parse_number(after) else {
        return false;
    };
    let rest = rest.trim().strip_suffix(" of their choice").unwrap_or(rest.trim());
    if !matches!(rest, "of those creatures" | "of those permanents") || n.as_const().is_none() {
        return false;
    }
    if !matches!(who, PlayerRef::Target(_)) || format!("{:?}", b.it_player) != format!("{who:?}") {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let alt = Effect::Sacrifice {
        who: who.clone(),
        filter: filter.clone(),
        count: n,
    };
    let old = std::mem::take(prev);
    *prev = Effect::If {
        cond,
        then: Box::new(alt),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "followup instead: more of those objects", priority: 300, apply: more_of_those } }

/// The reflexive triggered ability `e` ends with ("[instruction]. When you do,
/// [effect].").
fn last_reflexive(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_reflexive),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_reflexive(then),
        Effect::Reflexive { body } => Some(body),
        _ => None,
    }
}

/// A sentence replacing the effect of "When you do, [effect]." is part of that reflexive
/// ability.
fn instead_in_reflexive(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !(l.ends_with(" instead") || l.contains(", instead ")) {
        return false;
    }
    let Some(body) = last_reflexive(prev) else {
        return false;
    };
    if body.modal.is_some() {
        return false;
    }
    let targets = b.targets.len();
    let ok = crate::oracle_ext::apply_followup_ext(l, &mut body.effect, b);
    if ok && b.targets.len() != targets {
        b.targets.truncate(targets);
        return false;
    }
    ok
}

inventory::submit! { FollowupPattern { name: "followup instead: in the reflexive ability", priority: 20, apply: instead_in_reflexive } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn instead_alternatives() {
        for (ty, text) in [
            ("Sorcery", "Target creature deals damage to itself equal to its power. If that creature has flying, it deals twice that much damage to itself instead."),
            ("Sorcery", "Target opponent sacrifices a creature of their choice that attacked or blocked this turn. If you control a Gideon planeswalker, that player sacrifices two of those creatures of their choice instead."),
            ("Sorcery", "Look at the top three cards of your library. Put one of those cards into your hand and the rest on the bottom of your library in any order. If this spell was kicked, put each of those cards into your hand instead."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
