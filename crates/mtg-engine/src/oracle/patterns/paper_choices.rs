//! Choices that older Oracle wordings leave implicit (CR 608.2d: a choice an effect
//! offers without naming who makes it is made by the controller as the effect resolves):
//!
//! - a keyword list joined by "or" without "your choice of": "~ gains flying, first
//!   strike, or trample" (Flowstone Sculpture), "Target creature loses first strike or
//!   swampwalk until end of turn" (Urborg): read as "your choice of";
//! - two instructions joined by "or": "Put a +1/+1 counter on ~ or ~ gains flying, first
//!   strike, or trample" (Flowstone Sculpture), "Put a +1/+1 counter on target creature or
//!   that creature gains banding, first strike, or trample" (Nature's Blessing);
//! - two objects of one verb joined by "or": "Until end of turn, target creature has base
//!   power 1 or base toughness 1" (Vhati il-Dal).
//!
//! Each becomes [`Effect::ChooseOne`] with the controller choosing on resolution.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{
    duration_suffix, keyword_mods, parse_clause, parse_simple, player_ref, Builder,
};
use crate::oracle::phrases::*;

/// The items of "A, B, or C" / "A or B" (None for a list joined otherwise).
fn or_items(s: &str) -> Option<Vec<&str>> {
    let v: Vec<&str> = if let Some((a, c)) = s.split_once(", or ") {
        let mut v: Vec<&str> = a.split(", ").collect();
        v.push(c);
        v
    } else {
        let (a, c) = s.split_once(" or ")?;
        vec![a, c]
    };
    (v.len() >= 2 && v.iter().all(|i| !i.is_empty() && !i.contains(" or "))).then_some(v)
}

/// Whether `s` names exactly one keyword ability without a cost or a quality list that
/// the "or" could belong to ("protection from black or red").
fn single_keyword(s: &str) -> bool {
    !s.starts_with("protection")
        && !s.contains(" and ")
        && keyword_mods(s).is_some_and(|m| {
            m.len() == 1 && matches!(&m[0], Modification::AddKeyword(k) if k.cost.is_none())
        })
}

/// "[object] gains A, B, or C [until end of turn]" / "[object] loses A or B ...": the
/// keyword list joined by "or" is the controller's choice ("your choice of").
fn implicit_keyword_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.contains("your choice of") || l.contains('"') {
        return None;
    }
    let (verb, at) = [" gains ", " gain ", " loses ", " lose "]
        .iter()
        .find_map(|v| l.find(v).map(|i| (*v, i)))?;
    // The subject is one object phrase ("put a counter on ~ or ~ gains ..." is two
    // instructions: `or_instructions`).
    if l[..at].contains(" or ") {
        return None;
    }
    let r = &l[at + verb.len()..];
    let (_, list) = duration_suffix(r);
    let tail = &r[list.len()..];
    let items = or_items(list)?;
    if !items.iter().all(|i| single_keyword(i)) {
        return None;
    }
    let text = format!("{}{verb}your choice of {list}{tail}", &l[..at]);
    match parse_clause(&text, b)? {
        e @ Effect::ChooseOne { .. } => Some(e),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "gains A, B, or C (implicit choice)", priority: 61, parse: implicit_keyword_choice } }

/// Whether a clause starts with a subject that names a player (whose own choice an "or"
/// after it would be: "that player sacrifices a creature or loses 3 life").
fn player_subject(s: &str) -> bool {
    [
        "you ",
        "target player",
        "target opponent",
        "each ",
        "that player",
        "they ",
        "its controller",
        "an opponent",
        "defending player",
    ]
    .iter()
    .any(|p| s.starts_with(p))
}

/// Whether a clause starts with a subject that names an object ("~ gains ...", "that
/// creature gains ...", "it gets ...").
fn object_subject(s: &str) -> bool {
    ["~ ", "it ", "that creature ", "that permanent ", "the creature "]
        .iter()
        .any(|p| s.starts_with(p))
}

/// "[instruction] or [instruction with its own object subject]": the controller chooses
/// one as the ability resolves. The second instruction may refer to the first's target
/// ("Put a +1/+1 counter on target creature or that creature gains banding, first strike,
/// or trample.").
fn or_instructions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if player_subject(l) || l.contains('"') {
        return None;
    }
    let mut from = 0;
    while let Some(i) = l[from..].find(" or ").map(|i| i + from) {
        from = i + 4;
        let (a, c) = (&l[..i], &l[i + 4..]);
        if !object_subject(c) {
            continue;
        }
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let Some(ea) = parse_simple(a, b) else {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
            continue;
        };
        let before_c = b.targets.len();
        match parse_simple(c, b) {
            // The second instruction adds no targets of its own: every target is chosen
            // as the ability is put on the stack, whichever instruction is chosen.
            Some(ec) if b.targets.len() == before_c => {
                return Some(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options: vec![(a.to_string(), ea), (c.to_string(), ec)],
                });
            }
            _ => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
            }
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "[instruction] or [instruction]", priority: 400, parse: or_instructions } }

/// "[until end of turn, ][object] has A or B [duration]" (and "gets"): one verb with two
/// objects joined by "or", each of which reads as a whole instruction with the verb
/// ("target creature has base power 1 or base toughness 1").
fn or_objects(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.contains('"') || l.contains("your choice of") {
        return None;
    }
    let (lead, body) = match l.strip_prefix("until end of turn, ") {
        Some(r) => ("until end of turn, ", r),
        None => ("", l),
    };
    let (verb, at) = [" has ", " gets "]
        .iter()
        .find_map(|v| body.find(v).map(|i| (*v, i)))?;
    let subject = &body[..at];
    if player_subject(subject) {
        return None;
    }
    let r = &body[at + verb.len()..];
    let (_, list) = duration_suffix(r);
    let tail = &r[list.len()..];
    let (x, y) = list.split_once(" or ")?;
    if x.contains(", ") || y.contains(" or ") {
        return None;
    }
    let first = b.targets.len();
    let saved = (b.it.clone(), b.it_player.clone());
    let mut options = Vec::new();
    for o in [x, y] {
        b.targets.truncate(first);
        (b.it, b.it_player) = (saved.0.clone(), saved.1.clone());
        let text = format!("{lead}{subject}{verb}{o}{tail}");
        let Some(e) = parse_simple(&text, b) else {
            b.targets.truncate(first);
            (b.it, b.it_player) = saved;
            return None;
        };
        // Each option is an effect on the same object.
        if !matches!(e, Effect::Modify { .. }) {
            b.targets.truncate(first);
            (b.it, b.it_player) = saved;
            return None;
        }
        options.push((o.to_string(), e));
    }
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options,
    })
}

inventory::submit! { EffectPattern { name: "[object] has A or B", priority: 401, parse: or_objects } }

/// "sacrifice a land of an opponent's choice" (Demonic Hordes): you sacrifice it; an
/// opponent you choose (CR 601.7a's procedure for "an opponent") chooses which.
fn sacrifice_of_an_opponents_choice(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("sacrifice ")?;
    let noun = r.strip_suffix(" of an opponent's choice")?;
    let noun = noun.strip_prefix("a ").or_else(|| noun.strip_prefix("an "))?;
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        },
        Effect::SacrificeObjects {
            what: Sel::Choose {
                chooser: PlayerRef::ChosenOpponent,
                filter: Filter::and(vec![
                    f,
                    Filter::InZone(ZoneKind::Battlefield),
                    Filter::ControlledBy(PlayerRel::You),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
    ]))
}

inventory::submit! { EffectPattern { name: "sacrifice a [permanent] of an opponent's choice", priority: 300, parse: sacrifice_of_an_opponents_choice } }

/// "[player] may attach ~ to a land of their choice" (Kudzu): that player chooses
/// whether to and the object (untargeted, CR 701.3a; one it can legally be attached to,
/// CR 701.3b).
fn may_attach_of_their_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subj, r) = l.split_once(" may attach ")?;
    let (what, r) = r.split_once(" to ")?;
    let noun = r.strip_suffix(" of their choice")?;
    let noun = noun.strip_prefix("a ").or_else(|| noun.strip_prefix("an "))?;
    let first = b.targets.len();
    let (who, rest) = player_ref(subj, b)?;
    if !rest.trim().is_empty() || b.targets.len() != first || matches!(who, PlayerRef::You) {
        b.targets.truncate(first);
        return None;
    }
    let (what, tail) = crate::oracle::effects::object_ref(what, b)?;
    if !end(&tail).is_empty() || !matches!(what, Sel::This) {
        return None;
    }
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::Attach {
                what,
                to: Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
            }),
        }),
    })
}

inventory::submit! { EffectPattern { name: "[player] may attach ~ to a [permanent] of their choice", priority: 300, parse: may_attach_of_their_choice } }

/// "Target opponent chooses one." after revealing cards (Intuition): one of them.
fn chooses_one(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let subj = l.strip_suffix(" chooses one")?;
    parse_clause(&format!("{subj} chooses one of them"), b)
}

inventory::submit! { EffectPattern { name: "[player] chooses one (of the revealed cards)", priority: 300, parse: chooses_one } }
