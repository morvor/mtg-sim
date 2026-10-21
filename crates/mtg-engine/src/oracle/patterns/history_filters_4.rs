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
    (" during their controller's last turn", CONTROLLERS_LAST_TURN),
    (" during its controller's last turn", CONTROLLERS_LAST_TURN),
];

/// The window "during [the object's] controller's last turn" (Halls of Mist), read as
/// "during your last turn" and then made about the tested object's controller.
const CONTROLLERS_LAST_TURN: &str = " during your last turn\u{1}";

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
    // "~" in the predicate is the card itself, not the object being tested: "each
    // creature that dealt damage to ~ this turn" (Giant Albatross).
    if p.contains('~') {
        let combat = match p.as_str() {
            "dealt damage to ~" => false,
            "dealt combat damage to ~" => true,
            _ => return None,
        };
        let t = TriggerCond::DealsDamage {
            source: Filter::In(Box::new(Sel::Var(vars::TESTED))),
            to: DamageRecipient::Object(Filter::Source),
            combat_only: combat,
        };
        let v = match window {
            " this turn" => Value::EventsThisTurn(Box::new(t), Tally::Events),
            _ => return None,
        };
        return Some(Condition::Compare(v, Cmp::Ge, Value::Const(1)));
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
        history_condition(&format!("~ {p}{}", window.trim_end_matches('\u{1}')))?
    };
    let cond = about(cond, &Sel::Var(vars::TESTED))?;
    let cond = if window == CONTROLLERS_LAST_TURN {
        controllers_last_turn(cond)?
    } else {
        cond
    };
    identity(cond, so_far)
}

/// The condition with "your last turn" as the tested object's controller's last turn.
fn controllers_last_turn(cond: Condition) -> Option<Condition> {
    let mut json = serde_json::to_value(&cond).ok()?;
    let whose = serde_json::to_value(PlayerRef::ControllerOf(Box::new(Sel::Var(
        vars::TESTED,
    ))))
    .ok()?;
    let mut n = 0;
    fn walk(v: &mut serde_json::Value, whose: &serde_json::Value, n: &mut usize) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m.iter_mut() {
                    if k == "PlayersLastTurn" {
                        *x = whose.clone();
                        *n += 1;
                    } else {
                        walk(x, whose, n);
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, whose, n)),
            _ => {}
        }
    }
    walk(&mut json, &whose, &mut n);
    if n == 0 {
        return None;
    }
    serde_json::from_value(json).ok()
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
/// "[was put there from a zone] this turn": the core's named quality
/// (`kw/hand_graveyard_actions.rs`); otherwise the history clause.
fn object_filter(pred: &str, window: &str, so_far: &Filter) -> Option<Filter> {
    use crate::kw::hand_graveyard_actions::PUT_THERE_THIS_TURN;
    for (neg, pos) in [("wasn't ", "was "), ("weren't ", "were ")] {
        if let Some(r) = pred.strip_prefix(neg) {
            return Some(Filter::not(object_filter(&format!("{pos}{r}"), window, so_far)?));
        }
    }
    if let Some((a, b)) = pred.split_once(" or ") {
        if let Some(aux) = ["was ", "were "].into_iter().find(|x| a.starts_with(x)) {
            if !b.starts_with("was ") && !b.starts_with("were ") {
                let fa = object_filter(a, window, so_far)?;
                let fb = object_filter(&format!("{aux}{b}"), window, so_far)?;
                return Some(Filter::Or(vec![fa, fb]));
            }
        }
    }
    if window == " this turn" && so_far.zone() == Some(ZoneKind::Graveyard) {
        let put = pred
            .strip_prefix("was put there")
            .or_else(|| pred.strip_prefix("were put there"));
        let from = match put.map(str::trim) {
            Some("" | "from anywhere") => Some(""),
            Some("from the battlefield") => Some("battlefield"),
            Some("from a library" | "from your library" | "from their library") => {
                Some("library")
            }
            _ => None,
        };
        if let Some(from) = from {
            return Some(Filter::Custom(
                format!("{PUT_THERE_THIS_TURN}{from}").into(),
            ));
        }
    }
    Some(Filter::History(Box::new(object_predicate(
        pred, window, so_far,
    )?)))
}

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
        if let Some(f) = object_filter(pred, window, so_far) {
            return Some((f, rest));
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
    // "~ attacked this turn" (Angel of Destiny): the player is the object of the clause.
    if let Some(r) = t.strip_prefix("~ attacked this turn") {
        if clause_end(r) {
            let c = iterated(history_condition("~ attacked you this turn")?)?;
            return Some((PlayerFilter::History(Box::new(c)), r));
        }
    }
    // "previously dealt damage by it" (Diseased Vermin): at any earlier time this game,
    // by the source ("it").
    if let Some(r) = t.strip_prefix("previously dealt damage by it") {
        if clause_end(r) {
            let c = player_predicate("was dealt damage by ~", " this game")?;
            return Some((PlayerFilter::History(Box::new(c)), r));
        }
    }
    let (body, participle) = if let Some(r) = t.strip_prefix("who ") {
        (r, false)
    } else if t.starts_with("dealt ") {
        (t, true)
    } else {
        return None;
    };
    for (pred, window, rest) in windowed(body) {
        // "dealt combat damage this game by ...": the window isn't the clause's end.
        if pred.is_empty() || rest.starts_with(" by ") {
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
    // "dealt combat damage this game by a creature named ~" (Gollum, Obsessed Stalker):
    // the window before the source.
    if participle {
        for (w, as_written) in WINDOWS {
            let Some(i) = body.find(&format!("{w} by ")) else {
                continue;
            };
            let (head, by) = (&body[..i], &body[i + w.len()..]);
            // The source phrase runs to the end of the clause: try each end.
            let words: Vec<&str> = by.split(' ').collect();
            for k in (2..=words.len()).rev() {
                let src = words[..k].join(" ");
                let consumed = src.len();
                let rest = &by[consumed..];
                if !clause_end(rest) {
                    continue;
                }
                if let Some(c) = player_predicate(&format!("was {head}{src}"), as_written) {
                    let n = rest.len();
                    return Some((PlayerFilter::History(Box::new(c)), &t[t.len() - n..]));
                }
            }
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

/// "controlled by a player who was dealt combat damage by three or more Pirates this
/// turn" (Admiral Beckett Brass), "controlled by an opponent who lost life this turn".
fn controlled_by_who<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let (base, r) = if let Some(r) = t.strip_prefix("controlled by a player") {
        (PlayerFilter::Any, r)
    } else if let Some(r) = t.strip_prefix("controlled by an opponent") {
        (PlayerFilter::Opponent, r)
    } else {
        return None;
    };
    if !r.starts_with(" who ") {
        return None;
    }
    let (q, rest) = player_qualifier(r)?;
    Some((
        Filter::ControllerMatches(Box::new(PlayerFilter::And(vec![base, q]))),
        rest,
    ))
}

inventory::submit! { super::FilterSuffixPattern { name: "history 4: controlled by a player who [did something] this turn", priority: 900, parse: controlled_by_who } }

/// A player qualifier of a static ability's subject: the history qualifiers, and "who
/// attacked you or a planeswalker you control this turn" (CR 508.6: declared attackers
/// attacking you or a planeswalker you control).
fn subject_qualifier(q: &str) -> Option<PlayerFilter> {
    for (w, planeswalkers) in [
        ("who attacked you or a planeswalker you control this turn", true),
        ("who attacked you this turn", false),
    ] {
        if q == w {
            return Some(PlayerFilter::AttackRelation {
                attacking: true,
                other: Box::new(PlayerFilter::You),
                past: true,
                planeswalkers,
            });
        }
    }
    let (f, rest) = player_qualifier(q)?;
    rest.trim().is_empty().then_some(f)
}

/// "Each opponent who attacked you or a planeswalker you control this turn can't cast
/// spells." (Sandswirl Wanderglyph), "Each opponent who lost life this turn can't gain
/// life.": the restriction of "each opponent can't ..." for those opponents only.
fn each_who_cant(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let l = crate::oracle::phrases::end(l).trim();
    let (subj, base) = if let Some(r) = l.strip_prefix("each opponent ") {
        ("each opponent", r)
    } else if let Some(r) = l.strip_prefix("each player ") {
        ("each player", r)
    } else {
        return None;
    };
    if !base.starts_with("who ") {
        return None;
    }
    let i = base.find(" can't ")?;
    let q = subject_qualifier(&base[..i])?;
    let plain = format!("{subj}{}.", &base[i..]);
    let abilities = crate::oracle::statics::parse_static(&plain, ctx)?;
    let base_json = serde_json::to_value(if subj == "each opponent" {
        PlayerFilter::Opponent
    } else {
        PlayerFilter::Any
    })
    .ok()?;
    let mut out = Vec::new();
    for a in abilities {
        let mut d = (*a).clone();
        let AbilityKind::Static(s) = &mut d.kind else {
            return None;
        };
        let StaticEffect::Restriction(r) = &s.effect else {
            return None;
        };
        // The restricted players: "who" of the restriction, made those who did it.
        let mut json = serde_json::to_value(r).ok()?;
        let who = json.as_object_mut()?.values_mut().next()?.get_mut("who")?;
        if *who != base_json {
            return None;
        }
        *who = serde_json::to_value(PlayerFilter::And(vec![
            serde_json::from_value(base_json.clone()).ok()?,
            q.clone(),
        ]))
        .ok()?;
        s.effect = StaticEffect::Restriction(serde_json::from_value(json).ok()?);
        d.text = text.to_string();
        out.push(std::sync::Arc::new(d));
    }
    Some(out)
}

inventory::submit! { super::StaticPattern { name: "history 4: each opponent who [did something] this turn can't ...", priority: 900, parse: each_who_cant } }

/// "[who] cast a spell this turn with the same name as that card" (Grim Reminder: the card
/// the instruction before revealed), for "you" (the player performing the instruction).
pub fn past_with_referent(
    cond: &str,
    b: &crate::oracle::effects::Builder,
) -> Option<Condition> {
    let r = cond.strip_prefix("cast a spell this turn with the same name as ")?;
    let sel = match r {
        "that card" | "it" => b.it.clone(),
        _ => return None,
    };
    if super::oracle_hardening_referents::is_no_referent(&sel) || matches!(sel, Sel::This) {
        return None;
    }
    Some(Condition::Compare(
        Value::EventsThisTurn(
            Box::new(TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::and(vec![Filter::Spell, Filter::SameNameAs(Box::new(sel))]),
            }),
            Tally::Events,
        ),
        Cmp::Ge,
        Value::Const(1),
    ))
}

/// "all creatures except those that share a creature type with a creature that convoked
/// ~" (Everything Comes to Dust, CR 702.51c).
fn except_sharing_type_with_convoker<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let rest = t
        .strip_prefix("except those that share a creature type with a creature that convoked ~")
        .or_else(|| {
            t.strip_prefix("except those that share a creature type with a creature that convoked this spell")
        })?;
    clause_end(rest).then(|| {
        (
            Filter::not(Filter::Custom(
                crate::kw::damage_history_4::SHARES_TYPE_WITH_CONVOKER.into(),
            )),
            rest,
        )
    })
}

inventory::submit! { super::FilterSuffixPattern { name: "history 4: except those that share a creature type with a creature that convoked ~", priority: 900, parse: except_sharing_type_with_convoker } }
