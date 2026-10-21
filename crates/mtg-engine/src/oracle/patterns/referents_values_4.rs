//! Referent and value grammar IV (results "this way", referents to earlier objects,
//! computed values; CR 608.2c, 608.2h):
//!
//! - "If a player is dealt damage this way, they discard a card." (Sonic Shrieker), "If
//!   an opponent is dealt damage this way, ...": the players the previous damage
//!   instruction actually dealt damage to (after prevention, CR 120.4b), recorded as
//!   [`vars::DAMAGED_PLAYERS`]; "they"/"that player" in the effect are those players.
//! - "Players dealt damage this way can't cast noncreature spells this turn." (Aurelia's
//!   Fury): the same players as the subject of an instruction.
//! - "another permanent card" after "If you sacrificed a permanent this way" (Deadly
//!   Brew): other than the card the sacrificed permanent became (CR 400.7).
//! - Object phrases about results: "each creature dealt damage this way", "creatures
//!   tapped this way", "each artifact that became a creature this way", "all cards exiled
//!   this way" (recorded right after the exile, even with other instructions between).
//! - Values: "[value] as you cast ~" (recorded as the spell becomes cast, CR 601.2i), "the
//!   number of abilities from among [keywords] found among [objects]", "the difference
//!   between A and B", "the difference between [objects]' powers", "its total toxic
//!   value" (CR 702.164b), counts of objects affected "this way" (as they last existed).
//! - Referents: "the chosen creature" after bolster (CR 701.39a), "the [description]" of
//!   an earlier target ("the creature you control"), "the discarded card", "those cards"
//!   of "If two or more of those cards have the same name" (CR 201.2a).
//! - Results: "If that spell is countered this way", "If you didn't draw cards this way",
//!   "If those creatures have total power 8 or greater", "If that player discards a card
//!   this way" after "unless they [pay a cost]".
//! - [`lookback`]: a sentence about an instruction of an earlier sentence, read against
//!   all the instructions so far.

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
    // "each artifact that became a creature this way" (Rise and Shine): the objects the
    // most recent "becomes" instruction applied to that are creatures now.
    for p in [
        "that became a creature this way",
        "that became creatures this way",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with([' ', ',', '.']) {
                let f = Filter::and(vec![
                    Filter::Type(crate::types::CardType::Creature),
                    Filter::In(Box::new(Sel::Var(vars::MODIFIED))),
                ]);
                return Some((f, rest));
            }
        }
    }
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
    if let Some(v) = difference_between(s, b) {
        return Some(v);
    }
    if let Some(v) = total_toxic_value(s, b) {
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

/// The card "choose one of them [at random] and copy it" chose.
pub const CHOSEN_TO_COPY: Var = vars::USER + 14_004;

/// "If two or more cards are exiled this way, choose one of them at random and copy it."
/// (Mysterious Stranger): one of the objects "them" names, chosen (at random), copied
/// (CR 707.12).
fn choose_one_of_them_and_copy_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let random = match l {
        "choose one of them at random and copy it" => true,
        "choose one of them and copy it" => false,
        _ => return None,
    };
    let them = match &b.it {
        s @ Sel::Var(_) => s.clone(),
        _ => return None,
    };
    let filter = Filter::and(vec![Filter::In(Box::new(them)), Filter::Card]);
    let sel = if random {
        Sel::AtRandom {
            filter,
            count: Value::c(1),
            store: None,
        }
    } else {
        Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count: Value::c(1),
            up_to: false,
            store: None,
        }
    };
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN_TO_COPY,
            sel,
        },
        Effect::CopyCard {
            what: Sel::Var(CHOSEN_TO_COPY),
            named: None,
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "choose one of them (at random) and copy it", priority: 140, parse: choose_one_of_them_and_copy_it } }

/// The number of cards a draw instruction that a "didn't draw cards this way" condition
/// asks about was to draw, determined as it was performed.
pub const DRAW_COUNT: Var = vars::USER + 14_005;

/// "Draw cards equal to [value]. If you didn't draw cards this way, [effect]." (Mr.
/// Foxglove): the number is determined once, as the draw is performed (CR 608.2h); a
/// number of 0 or less draws nothing (CR 107.1b).
fn if_you_didnt_draw(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = [
        "if you didn't draw cards this way, ",
        "if you didn't draw a card this way, ",
        "if you drew no cards this way, ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::Draw {
        who: PlayerRef::You,
        n,
    }) = last
    else {
        return false;
    };
    let n = n.clone();
    let parsed = match x.strip_prefix("you may ") {
        Some(y) => parse_clause(y, b).map(|e| Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }),
        None => parse_clause(x, b),
    };
    let Some(e) = parsed else {
        return false;
    };
    let mut v = match std::mem::replace(prev, Effect::Noop) {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.pop();
    v.push(Effect::StoreValue {
        var: DRAW_COUNT,
        value: n,
    });
    v.push(Effect::Draw {
        who: PlayerRef::You,
        n: Value::Var(DRAW_COUNT),
    });
    v.push(Effect::If {
        cond: Condition::Compare(Value::Var(DRAW_COUNT), Cmp::Le, Value::c(0)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if you didn't draw cards this way, [effect]", priority: 140, apply: if_you_didnt_draw } }

/// A sentence about what an instruction of an earlier sentence (not just the previous
/// one) did, read against all the instructions so far: "Mill a card. You may put a land
/// card milled this way into your hand. You gain 2 life if a Lesson card is milled this
/// way.", "Exile all creatures. Each player may put any number of creature cards from
/// their hand onto the battlefield. Then put all cards exiled this way into their owners'
/// hands." Tried only for a sentence nothing else reads. The followups it uses take a
/// snapshot right after the instruction they're about (CR 608.2c).
pub fn lookback(s: &str, effects: &mut Vec<Effect>, b: &mut Builder) -> bool {
    if effects.len() < 2 {
        return false;
    }
    let lower = s.to_lowercase();
    let l = end(&lower);
    if super::referents_instead_4::instead_restates_all(l, effects, b) {
        return true;
    }
    if !lower.contains(" this way") && !lower.contains(" instead") {
        return false;
    }
    let mut whole = Effect::Seq(effects.clone());
    let saved = (
        b.targets.len(),
        b.it.clone(),
        b.it_player.clone(),
        b.named.len(),
    );
    let ok = super::conditions_this_way::if_this_way(l, &mut whole, b)
        || if_player_dealt_damage(l, &mut whole, b)
        || players_dealt_damage_subject(l, &mut whole, b)
        || about_exiled_this_way(l, &mut whole, b)
        || super::referents_instead_4::this_way_instead_pub(l, &mut whole, b)
        || super::referents_instead_4::double_that_damage_instead(l, &mut whole, b);
    if !ok {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        b.named.truncate(saved.3);
        return false;
    }
    let Effect::Seq(v) = whole else {
        return false;
    };
    *effects = v;
    true
}

/// The cards "those cards" names in a "two or more of those cards have the same name"
/// condition, recorded right after the instruction that affected them.
pub const SAME_NAME_CARDS: Var = vars::USER + 14_006;

/// Whether `e` ends with an instruction that exiles, mills or reveals cards from the top of
/// a library (its cards are `vars::IT`).
fn ends_with_top_cards(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(..),
            ..
        }
        | Effect::Mill { .. } => true,
        // "that player exiles the top four cards of their library"
        Effect::AsPlayer { effect, .. } => ends_with_top_cards(effect),
        Effect::Seq(v) => v.last().is_some_and(ends_with_top_cards),
        _ => false,
    }
}

/// "That player exiles the top four cards of their library. If two or more of those cards
/// have the same name, repeat this process." (Scalpelexis): some name is shared by two of
/// those cards (CR 201.2a).
fn if_two_have_the_same_name(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = l.strip_prefix("if two or more of those cards have the same name, ") else {
        return false;
    };
    if !ends_with_top_cards(prev) {
        return false;
    }
    let Some(e) = parse_clause(x, b) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::Store {
        var: SAME_NAME_CARDS,
        sel: Sel::Var(vars::IT),
    });
    v.push(Effect::If {
        cond: Condition::Compare(
            Value::LargestSameNameGroup(Filter::In(Box::new(Sel::Var(SAME_NAME_CARDS)))),
            Cmp::Ge,
            Value::c(2),
        ),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if two or more of those cards have the same name, [effect]", priority: 140, apply: if_two_have_the_same_name } }

/// "the difference between that spell's mana value and that nonland card's mana value"
/// (Lady Loki): the greater minus the lesser; "the difference between the chosen
/// creatures' powers" (Spry and Mighty): the greatest power among them minus the least.
fn difference_between(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let r = s.strip_prefix("the difference between ")?;
    for (w, stat) in [
        ("' powers", Stat::Power),
        ("' toughnesses", Stat::Toughness),
        ("' mana values", Stat::ManaValue),
    ] {
        if let Some(i) = r.find(w) {
            let saved = (b.targets.len(), b.it.clone());
            let (sel, tail) = crate::oracle::effects::object_ref(&r[..i], b)?;
            if !tail.trim().is_empty() {
                b.targets.truncate(saved.0);
                b.it = saved.1;
                return None;
            }
            let sel = Box::new(sel);
            let v = Value::Diff(
                Box::new(Value::Aggregate(AggOp::Max, stat.clone(), sel.clone())),
                Box::new(Value::Aggregate(AggOp::Min, stat, sel)),
            );
            return Some((v, r[i + w.len()..].to_string()));
        }
    }
    // "A and B": each split at an " and " where both sides are whole values.
    let mut from = 0;
    while let Some(k) = r[from..].find(" and ") {
        let at = from + k;
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let left = super::value_grammar::parse_value(&r[..at], b)
            .filter(|(_, tail)| tail.trim().is_empty());
        if let Some((x, _)) = left {
            if let Some((y, rest)) = super::value_grammar::parse_value(&r[at + 5..], b) {
                let rest_t = rest.trim_start();
                if rest_t.is_empty() || rest_t.starts_with([',', '.']) {
                    let v = Value::Max(
                        Box::new(Value::Diff(Box::new(x.clone()), Box::new(y.clone()))),
                        Box::new(Value::Diff(Box::new(y), Box::new(x))),
                    );
                    return Some((v, rest));
                }
            }
        }
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        from = at + 5;
    }
    None
}

/// "Attacking creatures you control gain indestructible until end of turn. If those
/// creatures have total power 8 or greater, convert ~." (Ultra Magnus, Tactician): the
/// group the previous instruction affected, their powers added up (CR 208).
fn if_those_have_total_power(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if those creatures have total power ") else {
        return false;
    };
    let Some((n, x)) = r.split_once(" or greater, ") else {
        return false;
    };
    let Some((Value::Const(n), tail)) = crate::oracle::phrases::parse_number(n) else {
        return false;
    };
    if !tail.trim().is_empty() {
        return false;
    }
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::Modify {
        what: Sel::All(_), ..
    }) = last
    else {
        return false;
    };
    // The objects it applied to (`vars::MODIFIED`).
    let group = Sel::Var(vars::MODIFIED);
    let Some(e) = parse_clause(x, b) else {
        return false;
    };
    let cond = Condition::Compare(
        Value::Aggregate(AggOp::Sum, Stat::Power, Box::new(group)),
        Cmp::Ge,
        Value::c(n),
    );
    let old = std::mem::replace(prev, Effect::Noop);
    let mut v = match old {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    v.push(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if those creatures have total power N or greater, [effect]", priority: 140, apply: if_those_have_total_power } }

/// The object a "each [object] deals damage to ..." instruction is about.
const EACH_DEALER: Var = vars::USER + 14_007;

/// "Each enchantment deals 2 damage to its controller", "each Aura attached to a creature
/// deals 2 damage to the creature it's attached to" (Aura Barbs): each of the objects is
/// the source of damage to a recipient named relative to it (CR 120.1).
fn each_deals_damage_to_its(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("each ")?;
    let (head, recipient) = r.split_once(" deals ")?;
    let (amount, to) = recipient.split_once(" damage to ")?;
    let (n, tail) = crate::oracle::phrases::parse_number(amount)?;
    if !tail.trim().is_empty() || n.as_const().is_none() {
        return None;
    }
    let (f, false, rest) = crate::oracle::phrases::parse_object_phrase(head)? else {
        return None;
    };
    if !end(rest).is_empty() {
        return None;
    }
    let me = Sel::Var(EACH_DEALER);
    let to = match to {
        "its controller" => Sel::Players(PlayerRef::ControllerOf(Box::new(me.clone()))),
        "its owner" => Sel::Players(PlayerRef::OwnerOf(Box::new(me.clone()))),
        "the creature it's attached to" | "the permanent it's attached to" => {
            Sel::HostOf(Box::new(me.clone()))
        }
        _ => return None,
    };
    let _ = b;
    Some(Effect::ForEach {
        sel: Sel::All(f),
        var: EACH_DEALER,
        effect: Box::new(Effect::DealDamage {
            source: me,
            amount: n,
            to,
        }),
    })
}

inventory::submit! { super::EffectPattern { name: "each [object] deals N damage to its controller / the creature it's attached to", priority: 140, parse: each_deals_damage_to_its } }

fn ends_with_discard(e: &Effect) -> bool {
    match e {
        Effect::Discard { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_discard),
        _ => false,
    }
}

/// "Draw a card, then discard a card. If there are four or more chorus counters on ~, you
/// may cast the discarded card without paying its mana cost." (Malcolm, Alluring
/// Scoundrel): the card the previous discard instruction discarded, where it went
/// (CR 400.7).
fn the_discarded_card(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains("the discarded card") || !ends_with_discard(prev) {
        return false;
    }
    let phrase = "the discarded card".to_string();
    if b.named.iter().any(|(p, _)| *p == phrase) {
        return false;
    }
    b.named
        .push((phrase.clone(), Sel::Var(crate::discard_rules::DISCARDED)));
    let e = crate::oracle::effects::parse_sentence(l, b);
    b.named.retain(|(p, _)| *p != phrase);
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

inventory::submit! { FollowupPattern { name: "the discarded card, after a discard", priority: 160, apply: the_discarded_card } }

/// "its total toxic value" (CR 702.164b): the sum of the N values of its toxic abilities
/// (Goliath Hatchery: the creature the text chose).
fn total_toxic_value(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let i = s.find(" total toxic value")?;
    let who = &s[..i];
    let rest = &s[i + " total toxic value".len()..];
    let sel = match who {
        "its" => {
            if super::oracle_hardening_referents::is_no_referent(&b.it) {
                return None;
            }
            b.it.clone()
        }
        w => {
            let w = w.strip_suffix("'s")?;
            let (sel, tail) = crate::oracle::effects::object_ref(w, b)?;
            if !tail.trim().is_empty() {
                return None;
            }
            sel
        }
    };
    Some((
        crate::kw::referents_values_4::total_toxic_value(&sel),
        rest.to_string(),
    ))
}

/// The creature a "for each creature ... with power greater than that creature's base
/// power" instruction is about.
const EACH_GREATER: Var = vars::USER + 14_008;

/// "For each creature you control with power greater than that creature's base power, put
/// a number of +1/+1 counters on that creature equal to the difference." (Sovereign
/// Okinec Ahau): each such creature gets its power minus its base power in counters, the
/// creatures and amounts determined as the instruction begins (CR 608.2h).
fn for_each_greater_than_base_power(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("for each ")?;
    let (group, x) = r.split_once(" with power greater than that creature's base power, ")?;
    let (counters, tail) = x
        .strip_prefix("put a number of ")?
        .split_once(" counters on that creature equal to the difference")?;
    if !tail.is_empty() {
        return None;
    }
    let kind = crate::oracle::costs::counter_kind(&format!("{counters} counters"))?.0;
    let (f, false, rest) = crate::oracle::phrases::parse_object_phrase(group)? else {
        return None;
    };
    if !end(rest).is_empty() {
        return None;
    }
    let _ = b;
    let tested = Sel::Var(vars::TESTED);
    let greater = Filter::ValueCmp(
        Box::new(Value::PowerOf(Box::new(tested.clone()))),
        Cmp::Gt,
        Box::new(Value::BasePowerOf(Box::new(tested))),
    );
    let me = Sel::Var(EACH_GREATER);
    let amounts: Var = vars::USER + 14_009;
    Some(Effect::ForEach {
        sel: Sel::All(Filter::and(vec![f, greater])),
        var: EACH_GREATER,
        effect: Box::new(Effect::seq(vec![
            Effect::StoreValue {
                var: amounts,
                value: Value::Diff(
                    Box::new(Value::PowerOf(Box::new(me.clone()))),
                    Box::new(Value::BasePowerOf(Box::new(me.clone()))),
                ),
            },
            Effect::AddCounters {
                what: me,
                kind,
                n: Value::Var(amounts),
            },
        ])),
    })
}

inventory::submit! { super::EffectPattern { name: "for each creature with power greater than its base power, counters equal to the difference", priority: 140, parse: for_each_greater_than_base_power } }

/// "the number of [objects] tapped this way" etc.: a count of the objects an earlier
/// instruction affected (the "this way" filters of this grammar), each judged as it is now
/// or as it last existed (CR 608.2h): `CountSel(Matching(var, rest))`.
pub fn count_this_way(f: &Filter) -> Option<Value> {
    let parts: Vec<Filter> = match f {
        Filter::And(v) => v.clone(),
        other => vec![other.clone()],
    };
    let is_var = |x: &Filter| match x {
        Filter::In(s) => match s.as_ref() {
            Sel::Var(v) => [vars::TAPPED, vars::DAMAGED, vars::MODIFIED, EXILED_SNAPSHOT]
                .contains(v)
                .then_some(*v),
            _ => None,
        },
        _ => None,
    };
    let var = parts.iter().find_map(is_var)?;
    let rest: Vec<Filter> = parts.into_iter().filter(|x| is_var(x).is_none()).collect();
    let rest = match rest.len() {
        0 => Filter::Any,
        1 => rest.into_iter().next().unwrap_or(Filter::Any),
        _ => Filter::And(rest),
    };
    Some(Value::CountSel(Box::new(Sel::Matching(
        Box::new(Sel::Var(var)),
        rest,
    ))))
}

/// "Target player discards a card unless they put a card from their hand on top of their
/// library. If that player discards a card this way, ~ deals 1 damage to each creature
/// and each player." (Tainted Specter): whether the discard the previous instruction may
/// perform (as the alternative to a cost the player didn't pay) happened.
fn if_that_player_discards_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = [
        "if that player discards a card this way, ",
        "if they discard a card this way, ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    fn nested_discard(e: &Effect) -> bool {
        match e {
            Effect::AsPlayer { effect, .. } => nested_discard(effect),
            Effect::PayOptional {
                then, otherwise, ..
            } => matches!(**then, Effect::Noop) && matches!(**otherwise, Effect::Discard { .. }),
            _ => false,
        }
    }
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    if !last.is_some_and(nested_discard) {
        return false;
    }
    let Some(e) = parse_clause(x, b) else {
        return false;
    };
    let discarded = crate::discard_rules::DISCARDED;
    let mut v = match std::mem::replace(prev, Effect::Noop) {
        Effect::Seq(v) => v,
        other => vec![other],
    };
    let i = v.len() - 1;
    // Nothing was discarded unless the instruction discards now.
    v.insert(
        i,
        Effect::Store {
            var: discarded,
            sel: Sel::None,
        },
    );
    v.push(Effect::If {
        cond: Condition::SelNonEmpty(Sel::Var(discarded)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if that player discards a card this way (unless a cost), [effect]", priority: 140, apply: if_that_player_discards_this_way } }

thread_local! {
    static REVEALED_REWORDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether `e` ends with revealing the top card of a library (which "that card" then
/// names).
fn ends_with_top_reveal(e: &Effect) -> bool {
    match e {
        Effect::Dig {
            reveal: true,
            n: Value::Const(1),
            ..
        } => true,
        Effect::AsPlayer { effect, .. } => ends_with_top_reveal(effect),
        Effect::Seq(v) => v.last().is_some_and(ends_with_top_reveal),
        _ => false,
    }
}

/// "Target opponent reveals the top card of their library. ~ deals damage equal to the
/// revealed card's mana value to that player ..." (Cerebral Eruption): "the revealed card"
/// is the card just revealed, as "that card" names it.
fn the_revealed_card(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if REVEALED_REWORDING.with(|c| c.get()) {
        return false;
    }
    let l = end(l);
    if !l.contains("the revealed card") || !ends_with_top_reveal(prev) {
        return false;
    }
    let t = l
        .replace("the revealed card's", "that card's")
        .replace("the revealed card", "that card");
    REVEALED_REWORDING.with(|c| c.set(true));
    let e = crate::oracle::effects::parse_sentence(&t, b);
    REVEALED_REWORDING.with(|c| c.set(false));
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

inventory::submit! { FollowupPattern { name: "the revealed card (the top card just revealed)", priority: 160, apply: the_revealed_card } }

/// "Whenever ~ deals combat damage to a player, exile up to that many target cards from
/// their graveyard." (Froghemoth): in an imperative instruction of an ability that
/// triggers on dealing damage to a player, "their" is that player.
fn their_is_the_damaged_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" from their graveyard")
        || !matches!(b.it_player, PlayerRef::TriggerPlayer)
        || !["exile ", "return ", "put "]
            .iter()
            .any(|v| l.starts_with(v))
    {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("deals combat damage to a player, ")
        && !raw.contains("deals damage to a player, ")
    {
        return None;
    }
    let t = l.replace(" from their graveyard", " from that player's graveyard");
    parse_clause(&t, b)
}

inventory::submit! { super::EffectPattern { name: "their graveyard (the damaged player's)", priority: 990, parse: their_is_the_damaged_player } }

/// "Count the colors of the sacrificed creature, then search your library for a creature
/// card that's exactly that many colors plus one." (Evolving Door): a card with exactly
/// one more color than the object (CR 105.2; the count is of the object as it last
/// existed).
fn count_colors_then_search(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("count the colors of ")?;
    let (who, search) = r.split_once(", then ")?;
    let (search, more) = search.split_once(" that's exactly that many colors")?;
    let plus = match more {
        "" => 0,
        " plus one" => 1,
        " plus two" => 2,
        _ => return None,
    };
    let (sel, tail) = super::filters_relational::object_in(who)?;
    if !tail.trim().is_empty() || matches!(sel, Sel::All(_)) {
        return None;
    }
    let mut e = parse_clause(search, b)?;
    let colors = |s: Sel| Value::DistinctAmong(Among::Colors, Box::new(s));
    let f = Filter::ValueCmp(
        Box::new(colors(Sel::Var(vars::TESTED))),
        Cmp::Eq,
        Box::new(Value::Sum(vec![colors(sel), Value::c(plus)])),
    );
    let Effect::SearchCards(spec) = &mut e else {
        return None;
    };
    let [part] = spec.parts.as_mut_slice() else {
        return None;
    };
    part.filter = Filter::and(vec![part.filter.clone(), f]);
    Some(e)
}

inventory::submit! { super::EffectPattern { name: "count the colors of X, then search for a card that's exactly that many colors plus one", priority: 140, parse: count_colors_then_search } }

thread_local! {
    static TRAILING_IF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// "Mill two cards, then destroy the chosen permanent if its mana value is less than or
/// equal to the number of cards in your graveyard." (Quag Feast): an instruction with a
/// condition after it, as the second half of a clause; the condition is checked as the
/// instruction would be performed, its pronouns as the instruction reads them.
fn instruction_if_condition(l: &str, b: &mut Builder) -> Option<Effect> {
    if TRAILING_IF.with(|c| c.get()) {
        return None;
    }
    let l = end(l);
    let (x, c) = l.rsplit_once(" if ")?;
    if x.contains(" if ") || c.contains(", ") || c.starts_with("you do") || c.starts_with("able") {
        return None;
    }
    // "[first instruction], then [instruction] if [condition]": the condition is only of
    // the second instruction, checked after the first is performed.
    let (first, x) = match x.split_once(", then ") {
        Some((f, x)) => (Some(f), x),
        None => (None, x),
    };
    // Only an instruction about an object the text named before ("the chosen permanent").
    if !(x.starts_with("destroy the ") || x.starts_with("exile the ")) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let first = match first {
        Some(f) => match parse_clause(f, b) {
            Some(e) => Some(e),
            None => {
                b.targets.truncate(saved.0);
                b.it = saved.1;
                return None;
            }
        },
        None => None,
    };
    // "Choose target creature, planeswalker, or Vehicle. ... the chosen permanent": the one
    // earlier target, when it can only be a permanent.
    let named_before = b.named.len();
    if x.contains("the chosen permanent") && !b.named.iter().any(|(p, _)| p == "the chosen permanent")
    {
        if let [t] = b.targets.as_slice() {
            let kinds = t.text.strip_prefix("target ").unwrap_or("");
            let permanent = |w: &str| {
                matches!(
                    w,
                    "creature" | "planeswalker" | "vehicle" | "artifact" | "enchantment" | "land"
                        | "battle" | "permanent" | "or" | ""
                )
            };
            if matches!(t.what, TargetKind::Object(_))
                && matches!(t.max, Value::Const(1))
                && kinds.split([' ', ',']).all(permanent)
            {
                b.named.push(("the chosen permanent".into(), Sel::Target(0)));
            }
        }
    }
    TRAILING_IF.with(|f| f.set(true));
    let e = parse_clause(x, b);
    b.named.truncate(named_before);
    let cond = e
        .as_ref()
        .and_then(|_| super::conditions_referents::parse_condition_with(c, b));
    TRAILING_IF.with(|f| f.set(false));
    let (Some(e), Some(cond)) = (e, cond) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    let e = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    Some(match first {
        Some(f) => Effect::Seq(vec![f, e]),
        None => e,
    })
}

inventory::submit! { super::EffectPattern { name: "destroy the [named object] if [condition] (in a clause)", priority: 995, parse: instruction_if_condition } }

/// "For each player, choose target permanent that player controls. Those players
/// sacrifice those permanents." (Vaevictis Asmadi, the Dire): each target is sacrificed by
/// its controller (CR 701.21a), all at once.
fn those_players_sacrifice_those_permanents(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "those players sacrifice those permanents" {
        return None;
    }
    let k = b
        .targets
        .iter()
        .rposition(|t| t.per_player.is_some() && matches!(t.what, TargetKind::Object(_)))?;
    Some(Effect::SacrificeObjects {
        what: Sel::Target(k as u8),
    })
}

inventory::submit! { super::EffectPattern { name: "those players sacrifice those permanents (per-player targets)", priority: 140, parse: those_players_sacrifice_those_permanents } }

/// "Each player who sacrificed a permanent this way reveals the top card of their library,
/// then puts it onto the battlefield if it's a permanent card." (Vaevictis Asmadi): each
/// controller of a permanent the previous instruction sacrificed (as it last existed),
/// performing the rest of the sentence in turn.
fn each_player_who_sacrificed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(rest) = l.strip_prefix("each player who sacrificed a permanent this way ") else {
        return false;
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    if !matches!(
        last,
        Some(Effect::SacrificeObjects { .. } | Effect::Sacrifice { .. })
    ) {
        return false;
    }
    let saved = b.it_player.clone();
    b.it_player = PlayerRef::Iterated;
    // Read as that player's own instructions ("reveal the top card of your library, then
    // put it onto the battlefield ...").
    let (v, r) = rest.split_once(' ').unwrap_or((rest, ""));
    let base = v.strip_suffix('s').unwrap_or(v);
    let text = format!("{base} {r}")
        .replace(" their ", " your ")
        .replace(", then puts ", ", then put ");
    let body = crate::oracle::effects::parse_clause(&text, b).map(|e| Effect::AsPlayer {
        who: PlayerRef::Iterated,
        effect: Box::new(e),
    });
    b.it_player = saved;
    let Some(body) = body else {
        return false;
    };
    let who = PlayerRef::Each(PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(
        Box::new(Sel::Var(vars::SACRIFICED)),
    ))));
    let e = Effect::ForEachPlayer {
        who,
        effect: Box::new(body),
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

inventory::submit! { FollowupPattern { name: "each player who sacrificed a permanent this way [does something]", priority: 140, apply: each_player_who_sacrificed } }

/// "Exile all other cards revealed this way, and you lose 1 life for each of the exiled
/// cards." (Spoils of the Vault): the cards the exile instruction just exiled, counted
/// after it (CR 608.2c).
fn exile_and_life_for_each(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((x, life)) = l.split_once(", and you ") else {
        return false;
    };
    let (gain, r) = if let Some(r) = life.strip_prefix("lose ") {
        (false, r)
    } else if let Some(r) = life.strip_prefix("gain ") {
        (true, r)
    } else {
        return false;
    };
    let Some((n, r)) = crate::oracle::phrases::parse_number(r) else {
        return false;
    };
    if n.as_const().is_none()
        || r.trim() != "life for each of the exiled cards"
        || !x.starts_with("exile ")
    {
        return false;
    }
    // The exile, read as the sentence it begins (it may refer to the previous one).
    let mut trial = prev.clone();
    let alone = if crate::oracle_ext::apply_followup_ext(x, &mut trial, b) {
        None
    } else {
        match crate::oracle::effects::parse_sentence(x, b) {
            Some(e) => Some(e),
            None => return false,
        }
    };
    let count = Value::Mul(
        Box::new(n),
        Box::new(Value::CountSel(Box::new(Sel::Var(vars::IT)))),
    );
    let life = if gain {
        Effect::GainLife {
            who: PlayerRef::You,
            n: count,
        }
    } else {
        Effect::LoseLife {
            who: PlayerRef::You,
            n: count,
        }
    };
    let mut v = match alone {
        None => match trial {
            Effect::Seq(v) => v,
            other => vec![other],
        },
        Some(e) => {
            let mut v = match std::mem::replace(prev, Effect::Noop) {
                Effect::Seq(v) => v,
                other => vec![other],
            };
            v.push(e);
            v
        }
    };
    // Only right after an exile instruction ("the exiled cards" are what it exiled).
    let last_exiles = v
        .last()
        .is_some_and(|e| serde_json::to_string(e).is_ok_and(|j| j.contains("\"Exile\"")));
    if !last_exiles {
        return false;
    }
    v.push(life);
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "exile ..., and you lose N life for each of the exiled cards", priority: 140, apply: exile_and_life_for_each } }

/// The cards "you" and the other player discarded, recorded after each discard.
const MY_DISCARD: Var = vars::USER + 14_014;
const THEIR_DISCARD: Var = vars::USER + 14_015;

/// "You and defending player each draw a card, then discard a card. Put two +1/+1
/// counters on ~ if you discarded the card with the greatest mana value among those cards
/// or tied for greatest." (Cait, Cage Brawler): the mana value of the card you discarded
/// against the other discarded card's.
fn if_you_discarded_the_greatest(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = l.strip_suffix(
        " if you discarded the card with the greatest mana value among those cards or tied for greatest",
    ) else {
        return false;
    };
    let Effect::Seq(v) = &*prev else {
        return false;
    };
    // [you draw, you discard, (the other player draws and discards)].
    let ok = matches!(
        v.as_slice(),
        [
            Effect::Draw { who: PlayerRef::You, .. },
            Effect::Discard { who: PlayerRef::You, .. },
            Effect::AsPlayer { effect, .. }
        ] if matches!(&**effect, Effect::Seq(w) if matches!(w.last(), Some(Effect::Discard { .. })))
    );
    if !ok {
        return false;
    }
    let Some(e) = parse_clause(x, b) else {
        return false;
    };
    let discarded = Sel::Var(crate::discard_rules::DISCARDED);
    let Effect::Seq(v) = prev else {
        return false;
    };
    v.insert(
        2,
        Effect::Store {
            var: MY_DISCARD,
            sel: discarded.clone(),
        },
    );
    v.push(Effect::Store {
        var: THEIR_DISCARD,
        sel: discarded,
    });
    let mv = |var| Value::Aggregate(AggOp::Max, Stat::ManaValue, Box::new(Sel::Var(var)));
    v.push(Effect::If {
        cond: Condition::And(vec![
            Condition::SelNonEmpty(Sel::Var(MY_DISCARD)),
            Condition::Compare(mv(MY_DISCARD), Cmp::Ge, mv(THEIR_DISCARD)),
        ]),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "if you discarded the card with the greatest mana value among those cards", priority: 140, apply: if_you_discarded_the_greatest } }
