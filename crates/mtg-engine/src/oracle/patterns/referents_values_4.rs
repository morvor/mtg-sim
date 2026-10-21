//! Referent and value grammar IV (results "this way", referents to earlier objects,
//! computed values; CR 608.2c, 608.2h):
//!
//! - "If a player is dealt damage this way, they discard a card." (Sonic Shrieker), "If
//!   an opponent is dealt damage this way, ...": the players the previous damage
//!   instruction actually dealt damage to (after prevention, CR 120.4b), recorded as
//!   [`vars::DAMAGED_PLAYERS`]; "they"/"that player" in the effect are those players.
//! - "Players dealt damage this way can't cast noncreature spells this turn." (Aurelia's
//!   Fury): the same players as the subject of an instruction.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// The players an earlier damage instruction of the effect dealt damage to, snapshotted
/// right after the condition reads them (a later damage instruction records its own).
pub const DAMAGED_PLAYERS_SNAPSHOT: Var = vars::USER + 14_001;

/// Instructions after a damage instruction that don't deal damage (so "this way" still
/// means that instruction).
fn keeps_damage(e: &Effect) -> bool {
    matches!(
        e,
        Effect::GainLife { .. }
            | Effect::LoseLife { .. }
            | Effect::Store { .. }
            | Effect::StoreValue { .. }
            | Effect::Tap { .. }
    )
}

fn is_damage(e: &Effect) -> bool {
    matches!(
        e,
        Effect::DealDamage { .. } | Effect::DealDividedDamage { .. }
    )
}

/// Whether the effect's last damage instruction is followed only by instructions that keep
/// it the one "this way" refers to.
pub fn ends_with_damage(e: &Effect) -> bool {
    let parts: Vec<&Effect> = match e {
        Effect::Seq(v) => v.iter().collect(),
        other => vec![other],
    };
    match parts.iter().rposition(|e| is_damage(e)) {
        Some(i) => parts[i + 1..].iter().all(|e| keeps_damage(e)),
        None => false,
    }
}

/// "a player" / "an opponent" / "a player or planeswalker"... as the players dealt
/// damage: the player filter.
fn damaged_player_filter(s: &str) -> Option<PlayerFilter> {
    Some(match s {
        "a player" | "one or more players" | "players" => PlayerFilter::Any,
        "an opponent" | "one or more opponents" | "opponents" => PlayerFilter::Opponent,
        _ => return None,
    })
}

/// The players dealt damage by the previous instruction that match `f`.
fn damaged_players(f: PlayerFilter, var: Var) -> PlayerRef {
    let inv = PlayerFilter::Ref(Box::new(PlayerRef::Var(var)));
    match f {
        PlayerFilter::Any => PlayerRef::Var(var),
        f => PlayerRef::Each(PlayerFilter::And(vec![f, inv])),
    }
}

/// Parses `x` with "they"/"that player" meaning `who`.
fn with_players(x: &str, who: PlayerRef, b: &mut Builder) -> Option<Effect> {
    let saved = b.it_player.clone();
    b.it_player = who;
    let x = super::choice_grammar::they_as_that_player(x).unwrap_or_else(|| x.to_string());
    let e = parse_clause(&x, b);
    b.it_player = saved;
    e
}

/// "If a player is dealt damage this way, [effect]."
fn if_player_dealt_damage(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(" this way, ") else {
        return false;
    };
    let Some(subject) = c
        .strip_suffix(" is dealt damage")
        .or_else(|| c.strip_suffix(" was dealt damage"))
        .or_else(|| c.strip_suffix(" are dealt damage"))
        .or_else(|| c.strip_suffix(" were dealt damage"))
    else {
        return false;
    };
    let Some(f) = damaged_player_filter(subject) else {
        return false;
    };
    if x.ends_with(" instead") || !ends_with_damage(prev) {
        return false;
    }
    let who = damaged_players(f, DAMAGED_PLAYERS_SNAPSHOT);
    let first_new = b.targets.len();
    let Some(e) = with_players(x, who.clone(), b) else {
        b.targets.truncate(first_new);
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::Store {
        var: DAMAGED_PLAYERS_SNAPSHOT,
        sel: Sel::Players(PlayerRef::Var(vars::DAMAGED_PLAYERS)),
    });
    v.push(Effect::If {
        cond: Condition::SelNonEmpty(Sel::Players(who)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if a player is dealt damage this way, [effect]", priority: 140, apply: if_player_dealt_damage } }

/// "Players dealt damage this way [effect]." / "Each opponent dealt damage this way
/// [effect]." as an instruction after a damage instruction.
fn players_dealt_damage_subject(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((subject, x)) = l.split_once(" dealt damage this way ") else {
        return false;
    };
    let (f, singular) = match subject {
        "players" => (PlayerFilter::Any, false),
        "opponents" => (PlayerFilter::Opponent, false),
        "each player" => (PlayerFilter::Any, true),
        "each opponent" => (PlayerFilter::Opponent, true),
        _ => return false,
    };
    if !ends_with_damage(prev) {
        return false;
    }
    let who = damaged_players(f, DAMAGED_PLAYERS_SNAPSHOT);
    // The verb agrees with "that player".
    let x = if singular {
        format!("that player {x}")
    } else {
        let (v, rest) = x.split_once(' ').unwrap_or((x, ""));
        let v = super::choice_grammar::third_person_verb(v);
        format!("that player {v} {rest}")
    };
    let first_new = b.targets.len();
    let Some(e) = with_players(x.trim_end(), who, b) else {
        b.targets.truncate(first_new);
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::Store {
        var: DAMAGED_PLAYERS_SNAPSHOT,
        sel: Sel::Players(PlayerRef::Var(vars::DAMAGED_PLAYERS)),
    });
    v.push(e);
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "players dealt damage this way [effect]", priority: 140, apply: players_dealt_damage_subject } }

/// "each creature dealt damage this way", "a creature dealt damage this way" (after the
/// noun): the objects the most recent damage instruction actually dealt damage to.
fn dealt_damage_this_way<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    // "each creature tapped this way": the permanents the most recent tap instruction
    // tapped (not those already tapped).
    for (p, var) in [
        ("dealt damage this way", vars::DAMAGED),
        ("tapped this way", vars::TAPPED),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with([' ', ',', '.']) {
                return Some((Filter::In(Box::new(Sel::Var(var))), rest));
            }
        }
    }
    None
}

inventory::submit! { super::FilterSuffixPattern { name: "referents iv: dealt damage this way", priority: 50, parse: dealt_damage_this_way } }

thread_local! {
    static ANOTHER_REWORDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// [`Filter::Custom`]: not the card a permanent the effect sacrificed became ("If you
/// sacrificed a permanent this way, you may return another permanent card from your
/// graveyard to your hand.": the sacrificed permanent is a new object in the graveyard,
/// CR 400.7, and "another" means other than it).
pub const NOT_SACRIFICED_CARD: &str = "not-the-sacrificed-card";

/// Adds `f` to the filter of the first object choice in `e`.
fn restrict_first_choice(e: &mut Effect, f: &Filter) -> bool {
    fn in_sel(s: &mut Sel, f: &Filter) -> bool {
        match s {
            Sel::Choose { filter, .. } | Sel::AtRandom { filter, .. } => {
                *filter = Filter::and(vec![filter.clone(), f.clone()]);
                true
            }
            _ => false,
        }
    }
    match e {
        Effect::Move { what, .. } => in_sel(what, f),
        Effect::May { effect, .. } => restrict_first_choice(effect, f),
        Effect::Seq(v) => v.iter_mut().any(|x| restrict_first_choice(x, f)),
        _ => false,
    }
}

/// "If you sacrificed a permanent this way, you may return another permanent card from
/// your graveyard to your hand." (Deadly Brew, Rise of the Witch-king): the sentence as
/// "this way" conditions read it, with "another" meaning other than the sacrificed card.
fn another_after_sacrificed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if ANOTHER_REWORDING.with(|r| r.get()) {
        return false;
    }
    let l = end(l);
    let Some(r) = l.strip_prefix("if you sacrificed ") else {
        return false;
    };
    let Some((_, x)) = r.split_once(" this way, ") else {
        return false;
    };
    let x = x.strip_prefix("you may ").unwrap_or(x);
    if !(x.starts_with("return another ") || x.starts_with("put another ")) {
        return false;
    }
    let t = l.replacen(" another ", " a ", 1);
    let mut trial = prev.clone();
    ANOTHER_REWORDING.with(|r| r.set(true));
    let ok = crate::oracle_ext::apply_followup_ext(&t, &mut trial, b);
    ANOTHER_REWORDING.with(|r| r.set(false));
    if !ok {
        return false;
    }
    // The conditional instruction the reading added last.
    let Effect::Seq(v) = &mut trial else {
        return false;
    };
    let Some(Effect::If { then, .. }) = v.last_mut() else {
        return false;
    };
    if !restrict_first_choice(then, &Filter::Custom(NOT_SACRIFICED_CARD.into())) {
        return false;
    }
    *prev = trial;
    true
}

inventory::submit! { FollowupPattern { name: "if you sacrificed [a permanent] this way, return another [card]", priority: 140, apply: another_after_sacrificed } }

/// "the greatest power among creatures you controlled as you cast ~" (Lifestream's
/// Blessing): the value as the spell became cast (CR 601.2i), recorded then (see
/// `kw/referents_values_4.rs`). `s` is a value phrase that ends with "as you cast ~";
/// "you controlled" is read as what you controlled then.
pub fn as_cast_snapshot(s: &str, b: &mut Builder) -> Option<Value> {
    if !b.ctx.is_spell() {
        return None;
    }
    let s = end(s);
    let v = s
        .strip_suffix(" as you cast ~")
        .or_else(|| s.strip_suffix(" as you cast this spell"))?;
    let v = v.replace(" you controlled", " you control");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let read = super::value_grammar::parse_value(&v, b);
    // A target can't be read as the spell becomes cast (it's chosen before, CR 601.2c),
    // and nothing it refers to exists then.
    let ok = b.targets.len() == saved.0;
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    let (value, tail) = read?;
    (ok && end(&tail).trim().is_empty())
        .then(|| crate::kw::referents_values_4::as_cast_value(&value))
}

/// Value phrases of this grammar for the value grammar (`value_results::atom_ext`):
/// "[value] as you cast ~" as the whole rest of the phrase.
pub fn atom(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = keywords_among(s) {
        return Some(v);
    }
    let i = s.find(" as you cast ~")?;
    let head_end = i + " as you cast ~".len();
    let rest = &s[head_end..];
    if !(rest.is_empty() || rest.starts_with([',', '.', ' '])) {
        return None;
    }
    let v = as_cast_snapshot(&s[..head_end], b)?;
    Some((v, rest.to_string()))
}

/// "the number of abilities from among flying, first strike, ... and vigilance found among
/// creatures you control" (Odric, Blood-Cursed): how many of the listed keyword abilities
/// at least one of the objects has, each counted once.
fn keywords_among(s: &str) -> Option<(Value, String)> {
    let r = s.strip_prefix("the number of abilities from among ")?;
    let (list, r) = r.split_once(" found among ")?;
    let list = list.replace(", and ", ", ").replace(" and ", ", ");
    let kinds: Option<Vec<crate::keywords::KeywordKind>> = list
        .split(", ")
        .map(crate::keywords::KeywordKind::from_name)
        .collect();
    let kinds = kinds?;
    if kinds.len() < 2 {
        return None;
    }
    let (f, true, rest) = crate::oracle::phrases::parse_object_phrase(r)? else {
        return None;
    };
    let rest = rest.trim_start();
    if !(rest.is_empty() || rest.starts_with([',', '.'])) {
        return None;
    }
    let parts = kinds
        .into_iter()
        .map(|k| {
            Value::If(
                Box::new(Condition::SelNonEmpty(Sel::All(Filter::and(vec![
                    f.clone(),
                    Filter::HasKeyword(k),
                ])))),
                Box::new(Value::c(1)),
                Box::new(Value::c(0)),
            )
        })
        .collect();
    Some((Value::Sum(parts), rest.to_string()))
}

/// The creature the most recent bolster instruction put counters on ("The chosen creature
/// gains trample until end of turn.").
pub const BOLSTERED: Var = vars::USER + 14_002;

/// Whether `e` ends with a bolster instruction.
fn ends_with_bolster(e: &Effect) -> bool {
    match e {
        Effect::KeywordAction {
            action: KeywordAction::Bolster,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_bolster),
        _ => false,
    }
}

/// "Bolster 2. The chosen creature gains trample until end of turn." (Optimus Prime,
/// Hero): bolster chooses the creature it puts counters on (CR 701.39a); "the chosen
/// creature" and a later "that creature" are it.
fn chosen_after_bolster(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.starts_with("the chosen creature ") || !ends_with_bolster(prev) {
        return false;
    }
    let sel = Sel::Var(BOLSTERED);
    for p in ["the chosen creature", "that creature"] {
        b.named.push((p.to_string(), sel.clone()));
    }
    let saved = b.it.clone();
    b.it = sel.clone();
    let Some(e) = crate::oracle::effects::parse_sentence(l, b) else {
        b.named
            .retain(|(p, _)| !(p == "the chosen creature" || p == "that creature"));
        b.it = saved;
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::Store {
        var: BOLSTERED,
        sel: Sel::Var(vars::IT),
    });
    v.push(e);
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "the chosen creature, after bolster", priority: 140, apply: chosen_after_bolster } }

thread_local! {
    static NAMING_TARGETS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// "Put a +1/+1 counter on target creature you control, then it fights target creature
/// you don't control. ..., the creature you control gains indestructible until end of
/// turn." (Outmuscle): "the [description]" names the one earlier target described as
/// "target [description]" (only a description that tells it apart: with a controller).
fn the_described_target(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if NAMING_TARGETS.with(|c| c.get()) {
        return false;
    }
    let l = end(l);
    let mut pushed = Vec::new();
    for (i, t) in b.targets.iter().enumerate() {
        let Some(desc) = t.text.strip_prefix("target ") else {
            continue;
        };
        let distinguishing = [
            " you control",
            " you don't control",
            " an opponent controls",
        ]
        .iter()
        .any(|q| desc.ends_with(q));
        let phrase = format!("the {desc}");
        let unique = b
            .targets
            .iter()
            .filter(|u| u.text.strip_prefix("target ") == Some(desc))
            .count()
            == 1;
        if distinguishing
            && unique
            && l.contains(&phrase)
            && !b.named.iter().any(|(p, _)| *p == phrase)
        {
            pushed.push((phrase, Sel::Target(i as u8)));
        }
    }
    if pushed.is_empty() {
        return false;
    }
    let n = pushed.len();
    b.named.extend(pushed);
    NAMING_TARGETS.with(|c| c.set(true));
    let e = crate::oracle::effects::parse_sentence(l, b);
    NAMING_TARGETS.with(|c| c.set(false));
    let keep = b.named.len() - n;
    b.named.truncate(keep);
    let Some(e) = e else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = match old {
        Effect::Seq(mut v) => {
            v.push(e);
            Effect::Seq(v)
        }
        other => Effect::seq(vec![other, e]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "the [description] of an earlier target", priority: 160, apply: the_described_target } }

/// The cards an earlier exile instruction of the effect exiled, recorded right after it
/// ("Then put all cards exiled this way into their owners' hands.").
pub const EXILED_SNAPSHOT: Var = vars::USER + 14_003;

thread_local! {
    /// Set while a sentence about the cards an earlier exile instruction exiled is read
    /// (so "exiled this way" after a noun names [`EXILED_SNAPSHOT`]).
    static EXILED_RECORDED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn is_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile { .. } => true,
        Effect::May { effect, .. } => is_exile(effect),
        _ => false,
    }
}

fn mentions_exile(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|j| j.contains("\"Exile\""))
}

/// "[cards] exiled this way" after the noun, while [`EXILED_RECORDED`] is set.
fn exiled_this_way_suffix<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    if !EXILED_RECORDED.with(|c| c.get()) {
        return None;
    }
    let rest = s.strip_prefix("exiled this way")?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((Filter::In(Box::new(Sel::Var(EXILED_SNAPSHOT))), rest))
}

inventory::submit! { super::FilterSuffixPattern { name: "referents iv: exiled this way", priority: 50, parse: exiled_this_way_suffix } }

/// A sentence about "[cards] exiled this way" after an exile instruction, possibly with
/// other instructions in between that exile nothing (Worlds Within Worlds): the exiled
/// cards are recorded right after the exile.
fn about_exiled_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains(" exiled this way") || EXILED_RECORDED.with(|c| c.get()) {
        return false;
    }
    let parts: Vec<Effect> = match &*prev {
        Effect::Seq(v) => v.clone(),
        other => vec![other.clone()],
    };
    let Some(i) = parts.iter().rposition(is_exile) else {
        return false;
    };
    if parts[i + 1..].iter().any(mentions_exile) {
        return false;
    }
    EXILED_RECORDED.with(|c| c.set(true));
    let e = crate::oracle::effects::parse_sentence(l, b);
    EXILED_RECORDED.with(|c| c.set(false));
    let Some(e) = e else {
        return false;
    };
    let mut v = parts;
    v.insert(
        i + 1,
        Effect::Store {
            var: EXILED_SNAPSHOT,
            sel: Sel::Var(vars::IT),
        },
    );
    v.push(e);
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "[cards] exiled this way, after an exile", priority: 970, apply: about_exiled_this_way } }

/// "Counter target spell. If that spell is countered this way, its controller may ..."
/// (Fold into Aether): whether the counter instruction countered it (a spell that can't
/// be countered isn't, CR 701.6a).
fn if_countered_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = [
        "if that spell is countered this way, ",
        "if it's countered this way, ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    // "..., exile it instead of putting it into its owner's graveyard": a replacement
    // (read by the counterspell patterns).
    if x.contains("instead") {
        return false;
    }
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::CounterSpell { what }) = last else {
        return false;
    };
    let what = what.clone();
    let saved = b.it.clone();
    b.it = what;
    let first_new = b.targets.len();
    let e = parse_clause(x, b);
    let Some(e) = e else {
        b.it = saved;
        b.targets.truncate(first_new);
        return false;
    };
    let cond_effect = Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = match old {
        Effect::Seq(mut v) => {
            v.push(cond_effect);
            Effect::Seq(v)
        }
        other => Effect::seq(vec![other, cond_effect]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "if that spell is countered this way, [effect]", priority: 140, apply: if_countered_this_way } }
