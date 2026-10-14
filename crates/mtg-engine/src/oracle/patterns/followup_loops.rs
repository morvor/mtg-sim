//! Loops over the instructions before them (CR 608.2c): the process is the text's
//! instructions so far, performed again.
//!
//! - "Then repeat this process X more times." (Another Round), "Repeat this process two
//!   more times." (Research // Development), "Repeat this process six more times."
//!   (Professor Onyx): the process again, that many times. Each repetition is performed
//!   separately, with new choices (no state-based actions in between: they're part of
//!   one resolution, CR 117.5).
//! - "Then repeat this process for instant cards with mana values 2 and 1." (Firemind's
//!   Foresight), "Then repeat this process for an enchantment and a planeswalker."
//!   (Invoke Despair), "Repeat this process for artifacts and creatures." (Equipoise),
//!   "..., then do the same for creature cards with mana value 2 and 3" (Gruesome
//!   Menagerie), "..., then repeats this process for an artifact, an enchantment, and a
//!   land" (Linessa, Zephyr Mage), "Do the same for flying, first strike, ..."
//!   (Super-Adaptoid): the process again for each of the other things named, in order,
//!   with the thing it was about (a card type, a mana value, a keyword) replaced.
//! - "Repeat the following process X times. [instructions]" (Torment of Hailfire): the
//!   instructions after it, X times.
//! - "Then target opponent does the same." (The Wedding of River Song): that player
//!   performs the process as "you".
//! - "That opponent may repeat this process as many times as they choose." (Trade
//!   Secrets): after each pass, that player decides whether it's performed again.

use super::FollowupPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;
use serde_json::Value as J;

// ---------------------------------------------------------------------------
// The process with one thing it's about replaced by another
// ---------------------------------------------------------------------------

/// One of the things the process is about, as it appears in the compiled process.
#[derive(Clone, Debug, PartialEq)]
enum Param {
    CardType(String),
    ManaValue(i64),
    Keyword(KeywordKind),
}

fn card_type_word(w: &str) -> Option<&'static str> {
    let w = w.strip_prefix("a ").or_else(|| w.strip_prefix("an ")).unwrap_or(w);
    Some(match w {
        "artifact" | "artifacts" => "Artifact",
        "creature" | "creatures" => "Creature",
        "enchantment" | "enchantments" => "Enchantment",
        "land" | "lands" => "Land",
        "planeswalker" | "planeswalkers" => "Planeswalker",
        "battle" | "battles" => "Battle",
        "instant" | "instants" => "Instant",
        "sorcery" | "sorceries" => "Sorcery",
        _ => return None,
    })
}

/// The items of a list: "a, b, and c", "a and b".
fn items(list: &str) -> Vec<String> {
    let list = list.replace(", and ", ", ").replace(" and ", ", ");
    list.split(", ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The things a "for [list]" names.
fn params(list: &str) -> Option<Vec<Param>> {
    // "instant cards with mana values 2 and 1", "creature cards with mana value 2 and 3".
    for sep in [" with mana values ", " with mana value "] {
        if let Some((_, nums)) = list.split_once(sep) {
            let v: Option<Vec<Param>> = items(nums)
                .iter()
                .map(|n| n.parse::<i64>().ok().map(Param::ManaValue))
                .collect();
            return v.filter(|v| !v.is_empty());
        }
    }
    let its = items(list);
    if its.is_empty() {
        return None;
    }
    if let Some(v) = its
        .iter()
        .map(|i| card_type_word(i).map(|t| Param::CardType(t.to_string())))
        .collect::<Option<Vec<_>>>()
    {
        return Some(v);
    }
    its.iter()
        .map(|i| KeywordKind::from_name(i).map(Param::Keyword))
        .collect()
}

fn keyword_json(k: KeywordKind) -> Option<String> {
    match serde_json::to_value(k).ok()? {
        J::String(s) => Some(s),
        _ => None,
    }
}

/// The values of the kind of `p` the compiled process mentions.
fn mentioned(j: &J, p: &Param, out: &mut Vec<Param>) {
    match j {
        J::Object(m) => {
            for (k, v) in m {
                match (p, k.as_str(), v) {
                    (Param::CardType(_), "Type", J::String(t))
                        if card_type_word(&t.to_lowercase()).is_some() =>
                    {
                        let x = Param::CardType(t.clone());
                        if !out.contains(&x) {
                            out.push(x);
                        }
                    }
                    (Param::ManaValue(_), "ManaValue", J::Array(a)) if a.len() == 2 => {
                        if let (Some("Eq"), Some(n)) = (
                            a[0].as_str(),
                            a[1].get("Const").and_then(J::as_i64),
                        ) {
                            let x = Param::ManaValue(n);
                            if !out.contains(&x) {
                                out.push(x);
                            }
                        }
                    }
                    _ => {}
                }
                mentioned(v, p, out);
            }
        }
        J::Array(a) => a.iter().for_each(|x| mentioned(x, p, out)),
        J::String(s) => {
            if let Param::Keyword(_) = p {
                if let Some(k) = KeywordKind::from_name(&s.to_lowercase()) {
                    let x = Param::Keyword(k);
                    if keyword_json(k).as_deref() == Some(s.as_str()) && !out.contains(&x) {
                        out.push(x);
                    }
                }
            }
        }
        _ => {}
    }
}

/// `j` with `from` replaced by `to`.
fn replace(j: &J, from: &Param, to: &Param) -> J {
    match j {
        J::Object(m) => J::Object(
            m.iter()
                .map(|(k, v)| {
                    let v = match (from, to, k.as_str(), v) {
                        (Param::CardType(a), Param::CardType(b), "Type", J::String(t))
                            if t == a =>
                        {
                            J::String(b.clone())
                        }
                        (Param::ManaValue(a), Param::ManaValue(b), "ManaValue", J::Array(arr))
                            if arr.len() == 2
                                && arr[0].as_str() == Some("Eq")
                                && arr[1].get("Const").and_then(J::as_i64) == Some(*a) =>
                        {
                            serde_json::json!(["Eq", { "Const": b }])
                        }
                        _ => replace(v, from, to),
                    };
                    (k.clone(), v)
                })
                .collect(),
        ),
        J::Array(a) => J::Array(a.iter().map(|x| replace(x, from, to)).collect()),
        J::String(s) => match (from, to) {
            (Param::Keyword(a), Param::Keyword(b)) => {
                let (ja, jb) = (keyword_json(*a), keyword_json(*b));
                if ja.as_deref() == Some(s.as_str()) {
                    J::String(jb.unwrap_or_default())
                } else if *s == a.name() {
                    // A counter named after the keyword ("a haste counter").
                    J::String(b.name().to_string())
                } else {
                    j.clone()
                }
            }
            _ => j.clone(),
        },
        other => other.clone(),
    }
}

/// The process again for each thing in `list`, with the one thing of that kind it's
/// about replaced.
fn varied(process: &Effect, list: &str) -> Option<Vec<Effect>> {
    let ps = params(list)?;
    let j = serde_json::to_value(process).ok()?;
    let mut found = vec![];
    mentioned(&j, &ps[0], &mut found);
    // "for instant cards with mana values 2 and 1" after "an instant card with mana value
    // 3": the mana value; with card types, only one other than those in all the items.
    let [from] = found.as_slice() else {
        return None;
    };
    if ps.contains(from) {
        return None;
    }
    ps.iter()
        .map(|to| serde_json::from_value(replace(&j, from, to)).ok())
        .collect()
}

/// The text after "repeat this process for " / "do the same for ".
fn for_list(l: &str) -> Option<&str> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    [
        "repeat this process for ",
        "repeats this process for ",
        "do the same for ",
        "does the same for ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p))
}

/// "..., then do the same for [list]" in one clause: the first half is the process.
fn do_the_same_for(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(list) = for_list(end(l)) else {
        return false;
    };
    if matches!(prev, Effect::Noop) {
        return false;
    }
    let Some(more) = varied(prev, list) else {
        return false;
    };
    let first = std::mem::take(prev);
    *prev = Effect::seq(std::iter::once(first).chain(more).collect());
    true
}

inventory::submit! { FollowupPattern { name: "followup loops: then do the same for [other things]", priority: 120, apply: do_the_same_for } }

// ---------------------------------------------------------------------------
// Sentences about the whole process so far
// ---------------------------------------------------------------------------

/// "two more times", "X more times", "six more times".
fn more_times(r: &str) -> Option<Value> {
    let r = r.strip_suffix(" more times")?;
    let (n, rest) = parse_number(r)?;
    rest.trim().is_empty().then_some(n)
}

/// A sentence about the instructions before it (`effects`), which become its process.
/// Returns true if it understood the sentence.
pub fn sentence(s: &str, effects: &mut Vec<Effect>, b: &mut Builder) -> bool {
    let lower = s.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    // "Repeat the following process X times.", "repeat the following process for each
    // opponent in turn order.": a header for the instructions after it.
    if let Some(r) = l.strip_prefix("repeat the following process ") {
        if r == "for each opponent in turn order" {
            effects.push(Effect::InTurnOrder {
                first: TurnOrderStart::NextOpponent,
                who: PlayerFilter::Opponent,
                effect: Box::new(Effect::Noop),
                direction: SeatSide::default(),
            });
            b.it_player = PlayerRef::Iterated;
            return true;
        }
        if let Some((n, rest)) = r.strip_suffix(" times").and_then(parse_number) {
            if rest.trim().is_empty() {
                effects.push(Effect::Repeat {
                    times: n,
                    effect: Box::new(Effect::Noop),
                });
                return true;
            }
        }
        return false;
    }
    if effects.is_empty() || effects.iter().any(is_header) {
        return false;
    }
    let process = Effect::seq(effects.clone());
    if let Some(r) = l.strip_prefix("repeat this process ") {
        if let Some(n) = more_times(r) {
            effects.push(Effect::Repeat {
                times: n,
                effect: Box::new(process),
            });
            return true;
        }
    }
    if let Some(list) = for_list(l) {
        if let Some(more) = varied(&process, list) {
            effects.extend(more);
            return true;
        }
        return false;
    }
    // "Then target opponent does the same.": that player performs the process.
    if let Some(who) = l.strip_suffix(" does the same") {
        if !who.starts_with("target ") || process_has_targets(&process) {
            return false;
        }
        let Some((p, rest)) = player_ref(who, b) else {
            return false;
        };
        if !rest.trim().is_empty() || !matches!(p, PlayerRef::Target(_)) {
            return false;
        }
        effects.push(Effect::AsPlayer {
            who: p,
            effect: Box::new(process),
        });
        return true;
    }
    // "That opponent may repeat this process as many times as they choose.".
    for who in ["that opponent", "that player"] {
        if l == format!("{who} may repeat this process as many times as they choose") {
            let p = b.it_player.clone();
            if !matches!(p, PlayerRef::Target(_)) {
                return false;
            }
            let body = Effect::seq(vec![
                process,
                Effect::May {
                    who: p,
                    effect: Box::new(Effect::RepeatThisProcess),
                },
            ]);
            effects.clear();
            effects.push(Effect::RepeatProcess {
                body: Box::new(body),
            });
            return true;
        }
    }
    false
}

fn is_header(e: &Effect) -> bool {
    matches!(e, Effect::Repeat { effect, .. } | Effect::InTurnOrder { effect, .. } if matches!(**effect, Effect::Noop))
}

fn process_has_targets(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|s| s.contains("\"Target\""))
}

/// At the end of the text: the instructions after a "repeat the following process"
/// header are its process.
pub fn finish(effects: &mut Vec<Effect>) {
    let Some(i) = effects.iter().position(is_header) else {
        return;
    };
    let body = Effect::seq(effects.split_off(i + 1));
    match &mut effects[i] {
        Effect::Repeat { effect, .. } | Effect::InTurnOrder { effect, .. } => **effect = body,
        _ => {}
    }
}

/// Whether a header was left without a process (the text wasn't understood).
pub fn has_empty_header(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.iter().any(has_empty_header),
        e => is_header(e),
    }
}

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn loops() {
        for (ty, text) in [
            ("Sorcery", "Search your library for an instant card with mana value 3, reveal it, and put it into your hand. Then repeat this process for instant cards with mana values 2 and 1. Then shuffle."),
            ("Sorcery", "Target opponent sacrifices a creature of their choice. If they can't, they lose 2 life and you draw a card. Then repeat this process for an enchantment and a planeswalker."),
            ("Sorcery", "Each opponent may discard a card. If they don't, they lose 3 life. Repeat this process six more times."),
            ("Sorcery", "Repeat the following process X times. Each opponent loses 3 life."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
        // Nothing the process is about to replace.
        assert!(compiled("Sorcery", "Draw a card. Then repeat this process for an enchantment.").is_none());
    }
}
