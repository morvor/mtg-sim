//! Objects and players described by their history (CR 603.4, 608.2c):
//!
//! * "a creature that fought this turn", "target nonblack creature that entered since your
//!   last turn ended", "target card in a graveyard that was put there from a library this
//!   turn", "a card in a graveyard that was discarded or put there from a library this
//!   turn", "target creature card in your graveyard that wasn't put there this combat",
//!   "creature that attacked you this turn": [`Filter::History`].
//! * "target player who lost life this turn", "each player who tapped a land for mana this
//!   turn", "a player who was dealt combat damage by three or more Pirates this turn",
//!   "target player dealt damage by ~ this turn": [`PlayerFilter::History`].
//!
//! The relative clause is read with the history clause grammar
//! (`history_conditions::history_condition`) about "~" (an object) or "you" (a player),
//! and then made about the object being tested ([`vars::TESTED`]) or the player being
//! tested (the iterated player).

use super::history_conditions::{about, history_condition};
use crate::ability::*;

/// Window phrases a relative clause may end with, and how the history grammar writes them.
const WINDOWS: &[(&str, &str)] = &[
    (" this turn", " this turn"),
    (" this combat", " this combat"),
    (" since your last turn ended", " since your last turn"),
    (" this game", " this game"),
];

/// Whether `rest` ends the clause.
fn clause_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"'])
}

/// Splits "[predicate][window][rest]" at each window phrase, shortest first.
fn windowed(body: &str) -> Vec<(&str, &'static str, &str)> {
    let mut out = Vec::new();
    for (w, as_written) in WINDOWS {
        for (i, _) in body.match_indices(w) {
            let rest = &body[i + w.len()..];
            if clause_end(rest) {
                out.push((&body[..i], *as_written, rest));
            }
        }
    }
    out.sort_by_key(|(p, _, _)| p.len());
    out
}

/// Whether the words mention the ability's controller.
fn mentions_you(t: &str) -> bool {
    t.split([' ', ',', '.'])
        .any(|w| matches!(w, "you" | "your" | "you've" | "yours"))
}

/// "[object] [past predicate] [window]" about "~": the condition, with the predicate as the
/// noun phrase so far makes it read ("put there" for a card in a graveyard).
fn object_predicate(pred: &str, window: &str, so_far: &Filter) -> Option<Condition> {
    // "wasn't put there this combat": not "was put there".
    for (neg, pos) in [("wasn't ", "was "), ("weren't ", "were ")] {
        if let Some(r) = pred.strip_prefix(neg) {
            let c = object_predicate(&format!("{pos}{r}"), window, so_far)?;
            return Some(Condition::Not(Box::new(c)));
        }
    }
    // "was discarded or put there from a library": either.
    if let Some((a, b)) = pred.split_once(" or ") {
        let aux = ["was ", "were "].into_iter().find(|x| a.starts_with(x));
        if let Some(aux) = aux {
            if !b.starts_with("was ") && !b.starts_with("were ") {
                let ca = object_predicate(a, window, so_far)?;
                let cb = object_predicate(&format!("{aux}{b}"), window, so_far)?;
                return Some(Condition::Or(vec![ca, cb]));
            }
        }
    }
    let in_graveyard = so_far.zone() == Some(ZoneKind::Graveyard);
    let mut p = pred.to_string();
    if p.contains(" there") {
        // "put there": into the zone the noun phrase named. A card goes to its owner's
        // graveyard (CR 404.2), so "from their library" is "from a library".
        if !in_graveyard {
            return None;
        }
        p = p.replace(" there", " into a graveyard");
        for who in ["their library", "your library", "its owner's library"] {
            p = p.replace(&format!("from {who}"), "from a library");
        }
    }
    // "was discarded": a player discarded it (the card in its new zone).
    let cond = if matches!(p.as_str(), "was discarded" | "were discarded") {
        if !in_graveyard {
            return None;
        }
        let t = TriggerCond::Discards {
            who: PlayerRel::Any,
            filter: Filter::Source,
        };
        let v = match window {
            " this turn" => Value::EventsThisTurn(Box::new(t), Tally::Events),
            _ => return None,
        };
        Condition::Compare(v, Cmp::Ge, Value::Const(1))
    } else {
        history_condition(&format!("~ {p}{window}"))?
    };
    let cond = about(cond, &Sel::Var(vars::TESTED))?;
    identity(cond, so_far)
}

/// The condition with each trigger condition that judges the object as it was before the
/// event (leaving the battlefield or a graveyard, CR 603.10a) about what the tested
/// object was before its latest zone change ([`crate::relational::TESTED_PRIOR`]): the
/// object the event made it (CR 400.7). Such a clause describes an object in another zone
/// than the battlefield ("a card in a graveyard that was put there from the battlefield");
/// an object on the battlefield, or anywhere, isn't one that left it ("for each creature
/// that left the battlefield this turn" counts creatures that are gone). Events about
/// another object than the tested one or what it was (a spell cast, which became the
/// permanent) aren't read.
fn identity(cond: Condition, so_far: &Filter) -> Option<Condition> {
    use serde_json::Value as J;
    let mut json = serde_json::to_value(&cond).ok()?;
    let tested = serde_json::to_value(Filter::In(Box::new(Sel::Var(vars::TESTED)))).ok()?;
    let prior = serde_json::to_value(Filter::In(Box::new(Sel::Var(
        crate::relational::TESTED_PRIOR,
    ))))
    .ok()?;
    fn replace(v: &mut J, from: &J, to: &J) {
        if *v == *from {
            *v = to.clone();
            return;
        }
        match v {
            J::Array(a) => a.iter_mut().for_each(|x| replace(x, from, to)),
            J::Object(m) => m.values_mut().for_each(|x| replace(x, from, to)),
            _ => {}
        }
    }
    // Trigger conditions about the tested object: (kind, judged before the event).
    fn walk(v: &mut J, from: &J, to: &J, looked_back: &mut bool, bad: &mut bool) {
        match v {
            J::Object(m) => {
                for (k, x) in m.iter_mut() {
                    match k.as_str() {
                        "Dies" | "LeavesBattlefield" | "Sacrificed" | "YouSacrifice" => {
                            *looked_back = true;
                            replace(x, from, to);
                        }
                        "ZoneChange" => {
                            let back = matches!(
                                x.get("from").and_then(|f| f.as_str()),
                                Some("Battlefield" | "Graveyard")
                            ) && x.get("to").and_then(|t| t.as_str()) != Some("Battlefield");
                            if back {
                                *looked_back = true;
                                replace(x, from, to);
                            }
                        }
                        "CastSpell" | "NthSpellCast" | "SpellCopied" | "AbilityActivated"
                        | "AbilityResolved" | "AbilityTriggered" | "SpellCountered"
                        | "CounteredBy" | "BecomesTarget" => *bad = true,
                        _ => walk(x, from, to, looked_back, bad),
                    }
                }
            }
            J::Array(a) => a
                .iter_mut()
                .for_each(|x| walk(x, from, to, looked_back, bad)),
            _ => {}
        }
    }
    let (mut looked_back, mut bad) = (false, false);
    walk(&mut json, &tested, &prior, &mut looked_back, &mut bad);
    if bad {
        return None;
    }
    if looked_back && matches!(so_far.zone(), None | Some(ZoneKind::Battlefield)) {
        return None;
    }
    serde_json::from_value(json).ok()
}

/// "that was cast this turn", "that were cast this turn" (a spell, or a permanent that
/// was one): how it was cast (`kw/cast_history.rs`).
fn cast_status(pred: &str) -> Option<Filter> {
    use crate::kw::cast_history as ch;
    Some(match pred {
        "was cast this turn" | "were cast this turn" => Filter::Custom(ch::CAST_THIS_TURN.into()),
        _ => return None,
    })
}

/// "that [past predicate] [window]" after an object phrase.
fn that_clause<'a>(t: &'a str, so_far: &Filter) -> Option<(Filter, &'a str)> {
    let body = t.strip_prefix("that ")?;
    for w in ["was cast this turn", "were cast this turn"] {
        if let Some(rest) = body.strip_prefix(w).filter(|r| clause_end(r)) {
            return Some((cast_status(w)?, rest));
        }
    }
    for (pred, window, rest) in windowed(body) {
        if pred.is_empty() {
            continue;
        }
        if let Some(c) = object_predicate(pred, window, so_far) {
            return Some((Filter::History(Box::new(c)), rest));
        }
    }
    None
}

inventory::submit! { super::FilterSuffixPattern { name: "history 4: [objects] that [did something] this turn", priority: 900, parse: that_clause } }

/// A player's history as a condition about "you" ("lost life this turn" → "you lost life
/// this turn"), made about the iterated player.
fn player_predicate(pred: &str, window: &str) -> Option<Condition> {
    if mentions_you(pred) {
        return None;
    }
    let cond = history_condition(&format!("you {pred}{window}"))?;
    iterated(cond)
}

/// The condition with "you" as the player being tested (`Iterated`).
fn iterated(c: Condition) -> Option<Condition> {
    let mut json = serde_json::to_value(&c).ok()?;
    fn walk(v: &mut serde_json::Value, n: &mut usize) {
        match v {
            serde_json::Value::String(s) if s == "You" => {
                *s = "Iterated".into();
                *n += 1;
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, n)),
            serde_json::Value::Object(m) => m.values_mut().for_each(|x| walk(x, n)),
            _ => {}
        }
    }
    let mut n = 0;
    walk(&mut json, &mut n);
    if n == 0 {
        return None;
    }
    serde_json::from_value(json).ok()
}

/// A qualifier after "player" / "opponent": "who lost life this turn", "who was dealt
/// combat damage by three or more Pirates this turn", "dealt damage by ~ this turn".
/// Returns the filter and the rest.
pub fn player_qualifier(s: &str) -> Option<(PlayerFilter, &str)> {
    let t = s.trim_start();
    let (body, participle) = if let Some(r) = t.strip_prefix("who ") {
        (r, false)
    } else if t.starts_with("dealt ") {
        (t, true)
    } else {
        return None;
    };
    for (pred, window, rest) in windowed(body) {
        if pred.is_empty() {
            continue;
        }
        let pred = if participle {
            format!("was {pred}")
        } else {
            pred.to_string()
        };
        if let Some(c) = player_predicate(&pred, window) {
            return Some((PlayerFilter::History(Box::new(c)), rest));
        }
    }
    None
}

/// A player's history clause ("lost life this turn", "cast a spell this turn") as written
/// after "you", if the history grammar reads it so (for "each opponent who [did
/// something] this turn ..."; the condition is checked as each player performs the
/// instruction, so "you" is that player).
pub fn as_you_past(cond: &str) -> Option<String> {
    if mentions_you(cond) {
        return None;
    }
    history_condition(&format!("you {cond}"))?;
    Some(cond.to_string())
}

/// A player's history clause after its subject ("lost life this turn"), as a filter of
/// players who did it.
pub fn player_history(pred_and_window: &str) -> Option<PlayerFilter> {
    let (pred, window, rest) = windowed(pred_and_window)
        .into_iter()
        .find(|(p, _, r)| !p.is_empty() && r.trim().is_empty())?;
    let _ = rest;
    Some(PlayerFilter::History(Box::new(player_predicate(
        pred, window,
    )?)))
}
