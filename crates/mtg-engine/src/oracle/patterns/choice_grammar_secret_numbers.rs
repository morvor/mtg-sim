//! Numbers players choose (usually secretly) for the same object, and what the text says
//! about them once they're revealed (`choices.rs`: [`PlayerFilter::ChoseNumber`] and the
//! values [`HIGHEST_NUMBER`], [`LOWEST_NUMBER`], [`OWN_NUMBER`]):
//!
//! - "choose a number 0 or greater"; "each player hides at least one item" (a secret
//!   number of at least one, Goblin Game's rulings: "write numbers on a piece of paper and
//!   reveal the numbers");
//! - "each player with the highest number loses that much life", "~ deals damage equal
//!   to the highest number to each player who chose that number", "each player who didn't
//!   choose the lowest number discards their hand", "each player loses life equal to the
//!   number of items they revealed", "the player who revealed the fewest items then loses
//!   half their life, rounded up" (with ties, each of them: the next sentence says so);
//! - "if you are one of those players, [instruction]" after an instruction for a group of
//!   players; "if they match, [instruction]" (the chosen numbers are all the same).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::choices::{HIGHEST_NUMBER, LOWEST_NUMBER, NUMBERS_MATCH, OWN_NUMBER, SECRET_NEXT};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

fn custom_value(name: &str) -> Value {
    Value::Custom(SmolStr::new(name))
}

/// "choose a number 0 or greater", "choose a number 1 or greater".
fn choose_number_or_greater(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose a number ")?;
    let (n, r) = parse_number(r)?;
    let Value::Const(min) = n else {
        return None;
    };
    (end(r) == "or greater").then_some(Effect::Choose {
        who: PlayerRef::You,
        kind: ChoiceKind::Number { min, max: 1000 },
    })
}

inventory::submit! { EffectPattern { name: "choose a number N or greater", priority: 120, parse: choose_number_or_greater } }

/// "each player hides at least one item, then all players reveal them simultaneously":
/// each player secretly chooses a number of at least one (Goblin Game's rulings).
fn hide_items(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player hides at least one item")?;
    let reveal = match r {
        "" => false,
        ", then all players reveal them simultaneously" => true,
        _ => return None,
    };
    let mut v = vec![Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::seq(vec![
                Effect::Custom(SmolStr::new(SECRET_NEXT)),
                Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::Number { min: 1, max: 1000 },
                },
            ])),
        }),
    }];
    if reveal {
        v.push(Effect::Custom(SmolStr::new(crate::choices::REVEAL_CHOICES)));
    }
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "each player hides at least one item", priority: 60, parse: hide_items } }

/// The players a phrase about the chosen numbers names.
const GROUPS: [(&str, Cmp, &str, bool); 5] = [
    ("each player with the highest number", Cmp::Eq, HIGHEST_NUMBER, true),
    ("each player who chose that number", Cmp::Eq, HIGHEST_NUMBER, true),
    ("each player who didn't choose the lowest number", Cmp::Ne, LOWEST_NUMBER, true),
    // "The player who revealed the fewest items ... If two or more players are tied for
    // fewest, each ...": each of them.
    ("the player who revealed the fewest items then", Cmp::Eq, LOWEST_NUMBER, false),
    ("the player who revealed the fewest items", Cmp::Eq, LOWEST_NUMBER, false),
];

/// Amounts that are chosen numbers, as worded, and the value.
const AMOUNTS: [(&str, &str, &str); 4] = [
    ("loses that much life", "loses x life", HIGHEST_NUMBER),
    ("damage equal to the highest number", "x damage", HIGHEST_NUMBER),
    (
        "loses life equal to the number of items they revealed",
        "loses x life",
        OWN_NUMBER,
    ),
    (
        "loses life equal to the number of items you revealed",
        "loses x life",
        OWN_NUMBER,
    ),
];

/// An instruction about the players who chose some number, with the chosen numbers as
/// amounts: performed by (or to) each of those players in turn, "x" being the amount.
fn chosen_number_instruction(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let group = GROUPS.iter().find(|(p, ..)| l.contains(p));
    let amount = AMOUNTS.iter().find(|(p, ..)| l.contains(p));
    if group.is_none() && amount.is_none() {
        return None;
    }
    // "each player loses life equal to the number of items they revealed": everyone.
    let (text, who) = match group {
        Some((p, c, v, _)) => (
            l.replacen(p, "each player", 1),
            PlayerFilter::ChoseNumber(*c, Box::new(custom_value(v))),
        ),
        None if l.starts_with("each player ") => (l.to_string(), PlayerFilter::Any),
        None => return None,
    };
    let (text, value) = match amount {
        Some((p, x, v)) => (text.replacen(p, x, 1), Some(custom_value(v))),
        None => (text, None),
    };
    // The instruction for one of those players ("you"), performed by each of them.
    let one = match text.strip_prefix("each player ") {
        Some(pred) => {
            let pred = super::player_subjects::as_you(pred)
                .or_else(|| (!pred.contains("your")).then(|| pred.to_string()))?;
            format!("you {pred}")
        }
        None => {
            // "~ deals x damage to each player": to each of them.
            text.replacen("each player", "you", 1)
        }
    };
    if one.contains("each player") || one.contains(" x ") && value.is_none() {
        return None;
    }
    let targets = b.targets.len();
    let Some(mut e) = parse_clause(&one, b) else {
        b.targets.truncate(targets);
        return None;
    };
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    if let Some(v) = value {
        // The amount for each player as they perform it (their own number, or the
        // highest one).
        let from = serde_json::to_value(Value::X).ok()?;
        let to = serde_json::to_value(v).ok()?;
        let mut n = 0;
        let j = replace_json(serde_json::to_value(&e).ok()?, &from, &to, &mut n);
        if n == 0 {
            b.targets.truncate(targets);
            return None;
        }
        e = serde_json::from_value(j).ok()?;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Each(who),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    })
}

inventory::submit! { EffectPattern { name: "instructions about the chosen numbers", priority: 60, parse: chosen_number_instruction } }

fn replace_json(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
    n: &mut usize,
) -> serde_json::Value {
    use serde_json::Value as J;
    if v == *from {
        *n += 1;
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| replace_json(x, from, to, n)).collect()),
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, replace_json(x, from, to, n)))
                .collect(),
        ),
        x => x,
    }
}

/// "If two or more players are tied for fewest, each loses half their life, rounded up.":
/// what the instruction before already does for each of the tied players.
fn tied_for_fewest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "if two or more players are tied for fewest, each loses half their life, rounded up" {
        return false;
    }
    matches!(prev, Effect::ForEachPlayer { who: PlayerRef::Each(PlayerFilter::ChoseNumber(Cmp::Eq, v)), .. }
        if matches!(v.as_ref(), Value::Custom(n) if n == LOWEST_NUMBER))
}

inventory::submit! { FollowupPattern { name: "tied for fewest (each of them)", priority: 60, apply: tied_for_fewest } }

/// "If you are one of those players, [instruction]" after an instruction for a group of
/// players: whether you're among them.
fn if_you_are_one_of_those(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you are one of those players, ") else {
        return false;
    };
    let who = match &*prev {
        Effect::ForEachPlayer {
            who: PlayerRef::Each(f),
            ..
        } => f.clone(),
        _ => return false,
    };
    let targets = b.targets.len();
    let Some(e) = parse_clause(r, b) else {
        b.targets.truncate(targets);
        return false;
    };
    let first = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        first,
        Effect::If {
            cond: Condition::PlayerMatches(PlayerRef::You, who),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if you are one of those players", priority: 60, apply: if_you_are_one_of_those } }

/// "If they match, [instruction]" after numbers were chosen: all the same.
fn if_they_match(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if they match, ")?;
    let targets = b.targets.len();
    let Some(e) = parse_clause(r, b) else {
        b.targets.truncate(targets);
        return None;
    };
    Some(Effect::If {
        cond: Condition::Custom(SmolStr::new(NUMBERS_MATCH)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if they match (the chosen numbers)", priority: 60, parse: if_they_match } }
