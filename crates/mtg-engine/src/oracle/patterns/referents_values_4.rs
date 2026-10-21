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
    if !lower.contains(" this way") {
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
        || about_exiled_this_way(l, &mut whole, b)
        || super::referents_instead_4::this_way_instead_pub(l, &mut whole, b);
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
