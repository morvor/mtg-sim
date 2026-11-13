//! Mana grammar VI (cards5-mana-replacements): counts of the mana spent on one payment
//! (CR 601.2h, 602.2b, 702.24a).
//!
//! - "for each {B} or {R} spent this way" in "Whenever ~'s cumulative upkeep is paid"
//!   (Balduvian Fallen): the mana of those colors spent on the payment the trigger
//!   reports (`kw/payment_mana.rs`).
//! - "for each {B} spent to cast it", "for each {R} spent to activate this ability": the
//!   mana of those types spent on the spell or the activation (`kw/cast_history.rs`).
//! - "for each mana from Treasures spent to activate this ability": the mana from that
//!   kind of source spent on the activation (the Value Mr. House's dice count uses).
//!
//! Mana production (CR 106, 605):
//!
//! - "Whenever an Island is tapped for mana, its controller may add an additional {U}. If
//!   that Island is snow, its controller may add an additional {U}{U} instead." (Snowfall):
//!   a triggered mana ability (CR 605.1b) whose optional addition is chosen by the tapped
//!   land's controller (CR 106.12a), with a condition on the tapped land choosing the
//!   amount.
//! - "add three mana in any combination of its colors", "add one mana of any of that
//!   card's colors" (Omnath, Locus of All; CR 106.1, 106.5), after "You may reveal that
//!   card if it has three or more colored mana symbols in its mana cost" (a count of the
//!   symbols as a condition, CR 107.4e) and with "If you don't reveal it, ..." after "If
//!   you do, ...".

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// One mana symbol of a single type ("{B}", "{C}") at the start of `s`.
fn one_symbol(s: &str) -> Option<(ManaType, &str)> {
    let r = s.strip_prefix('{')?;
    let (sym, rest) = r.split_once('}')?;
    if sym.len() != 1 {
        return None;
    }
    let t = ManaType::from_letter(sym.chars().next()?.to_ascii_uppercase())?;
    Some((t, rest))
}

/// "{B}", "{B} or {R}", "{W}, {U}, or {B}": single symbols naming the counted types.
fn symbol_list(s: &str) -> Option<(Vec<ManaType>, &str)> {
    let (t, mut rest) = one_symbol(s)?;
    let mut types = vec![t];
    loop {
        let next = [", or ", " or ", " and/or ", ", "]
            .iter()
            .find_map(|sep| rest.strip_prefix(sep));
        let Some(n) = next else { break };
        let Some((t, r)) = one_symbol(n) else { break };
        if types.contains(&t) {
            return None;
        }
        types.push(t);
        rest = r;
    }
    // "{U}{U}" (two symbols together: Bladecoil Serpent's pairs) isn't a list of types.
    if rest.starts_with('{') {
        return None;
    }
    Some((types, rest))
}

/// The `what` of `cast history:spent:` for mana of `types`.
fn types_what(types: &[ManaType]) -> String {
    match types {
        [t] => format!("type:{t:?}"),
        _ => {
            let t: Vec<String> = types.iter().map(|t| format!("{t:?}")).collect();
            format!("types:{}", t.join(","))
        }
    }
}

/// Whether the ability being parsed is a trigger on paying a cumulative upkeep (the
/// payment "this way" refers to).
fn upkeep_payment_trigger(b: &Builder) -> bool {
    if !b.in_trigger {
        return false;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let name = crate::oracle::card_name().to_lowercase();
    let raw = if name.is_empty() {
        raw
    } else {
        raw.replace(&name, "~")
    };
    let raw = raw.replace("this creature", "~").replace("this permanent", "~");
    ["whenever ~'s cumulative upkeep is paid", "whenever you pay ~'s cumulative upkeep"]
        .iter()
        .any(|t| raw.contains(t))
}

fn word_end(s: &str) -> bool {
    s.is_empty() || s.starts_with([' ', ',', '.'])
}

/// After "the number of" / "for each": a count of mana spent on one payment.
pub(crate) fn count_phrase(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    // "mana from Treasures spent to activate this ability".
    if let Some(x) = r.strip_prefix("mana from ") {
        let (kind, rest) = x
            .split_once(" that was spent to activate this ability")
            .or_else(|| x.split_once(" spent to activate this ability"))?;
        if !word_end(rest) {
            return None;
        }
        let what = super::mana_4::mana_from_kind(kind)?;
        return Some((
            super::history_conditions::spent_value("this ability", &what),
            rest.to_string(),
        ));
    }
    let (types, rest) = symbol_list(r)?;
    let rest = rest.strip_prefix(" spent ")?;
    if let Some(tail) = rest.strip_prefix("this way") {
        if !word_end(tail) || !upkeep_payment_trigger(b) {
            return None;
        }
        return Some((
            super::history_conditions::spent_value(crate::kw::cast_history::THIS_WAY, &types_what(&types)),
            tail.to_string(),
        ));
    }
    let what = types_what(&types);
    if let Some(tail) = rest.strip_prefix("to activate this ability") {
        if !word_end(tail) {
            return None;
        }
        return Some((
            super::history_conditions::spent_value("this ability", &what),
            tail.to_string(),
        ));
    }
    // "to cast ~" / "to cast it" / "to cast this spell".
    let x = rest.strip_prefix("to cast ")?;
    for word in ["this spell", "this creature", "~", "it", "them", "that spell"] {
        let Some(tail) = x.strip_prefix(word) else {
            continue;
        };
        if !word_end(tail) {
            continue;
        }
        let key = super::history_conditions::payment_key(&format!("cast {word}"), Some(b))?;
        return Some((
            super::history_conditions::spent_value(key, &what),
            tail.to_string(),
        ));
    }
    None
}

/// "its controller may add an additional {U}" in a tapped-for-mana trigger (Snowfall):
/// the optional addition is chosen by the player who adds the mana, the tapped land's
/// controller, as the triggered mana ability resolves (CR 106.12a, 605.1b).
fn player_may_add_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger {
        return None;
    }
    let l = end(l);
    let (subject, rest) = ["its controller may ", "that player may "]
        .iter()
        .find_map(|p| l.strip_prefix(p).map(|r| (&p[..p.len() - "may ".len()], r)))?;
    let rest = rest.strip_prefix("add an additional ")?;
    let saved = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&format!("{subject}adds an additional {rest}"), b);
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    match e? {
        e @ Effect::AddMana { .. } => {
            let Effect::AddMana { who, .. } = &e else {
                return None;
            };
            Some(Effect::May {
                who: who.clone(),
                effect: Box::new(e.clone()),
            })
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "mana 6: its controller may add an additional [mana]", priority: 100, parse: player_may_add_additional } }

/// The mana a sentence adds, optionally (`May`) or not.
fn adds_mana(e: &Effect) -> bool {
    match e {
        Effect::AddMana { .. } => true,
        Effect::May { effect, .. } => matches!(**effect, Effect::AddMana { .. }),
        _ => false,
    }
}

/// "If that Island is snow, its controller may add an additional {U}{U} instead." after
/// a tapped-for-mana trigger's "its controller may add an additional {U}" (Snowfall): a
/// condition on the tapped permanent, checked as the mana ability resolves, replacing the
/// amount added (and leaving the rest of the sentence, the optional choice, as it was).
fn if_tapped_is_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !b.in_trigger || !adds_mana(prev) {
        return false;
    }
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((cond, body)) = r.split_once(", ") else {
        return false;
    };
    let Some(body) = body.strip_suffix(" instead") else {
        return false;
    };
    // The condition is about the tapped permanent ("that Island", "it").
    if !matches!(b.it, Sel::TriggerObject) {
        return false;
    }
    let saved = b.targets.len();
    // "that Island": the tapped permanent, named by a word of the trigger's subject.
    let renamed;
    let cond = match cond.strip_prefix("that ").and_then(|r| r.split_once(' ')) {
        Some((w, rest))
            if crate::oracle::phrases::subtype_word(w).is_some()
                || matches!(w, "land" | "permanent" | "creature" | "artifact") =>
        {
            renamed = format!("it {rest}");
            renamed.as_str()
        }
        _ => cond,
    };
    let Some(c) = super::conditions_referents::parse_condition_with(cond, b) else {
        return false;
    };
    let Ok(cj) = serde_json::to_string(&c) else {
        return false;
    };
    if !cj.contains("TriggerObject") {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(body, b) else {
        b.targets.truncate(saved);
        return false;
    };
    if b.targets.len() != saved || !adds_mana(&e) {
        b.targets.truncate(saved);
        return false;
    }
    // Both say who adds the mana and whether it's optional the same way.
    let (who_a, may_a) = adder(prev);
    let (who_b, may_b) = adder(&e);
    if format!("{who_a:?}") != format!("{who_b:?}") || may_a != may_b {
        return false;
    }
    *prev = Effect::If {
        cond: c,
        then: Box::new(e),
        otherwise: Box::new(prev.clone()),
    };
    true
}

/// Who adds the mana and whether they may choose not to.
fn adder(e: &Effect) -> (Option<PlayerRef>, bool) {
    match e {
        Effect::AddMana { who, .. } => (Some(who.clone()), false),
        Effect::May { effect, .. } => (adder(effect).0, true),
        _ => (None, false),
    }
}

inventory::submit! { FollowupPattern { name: "mana 6: if that land is [quality], [player] adds [mana] instead", priority: 100, apply: if_tapped_is_instead } }

/// "[N] colored mana symbols in its mana cost", "[N] red mana symbols in that card's mana
/// cost" after "has" in a condition about an object ("You may reveal that card if it has
/// three or more colored mana symbols in its mana cost", Omnath, Locus of All): a count of
/// the symbols of its mana cost (CR 107.4e, 202.1: a hybrid symbol is of each of its
/// colors, so it counts once as colored).
pub(crate) fn has_mana_symbols(sel: &Sel, h: &str) -> Option<Condition> {
    let (cmp, n, rest) = super::statics_conditions::amount_cmp(h)?;
    let (w, x) = rest.trim().split_once(' ')?;
    let stat = match w {
        "colored" => Stat::ColoredManaSymbols,
        _ => Stat::ManaSymbols(crate::types::Color::from_word(w)?),
    };
    let x = x
        .strip_prefix("mana symbols in ")
        .or_else(|| x.strip_prefix("mana symbol in "))?;
    if !matches!(x, "its mana cost" | "that card's mana cost") {
        return None;
    }
    Some(Condition::Compare(
        Value::Aggregate(AggOp::Sum, stat, Box::new(sel.clone())),
        cmp,
        n,
    ))
}

/// "[its / that card's] colors" at the end of a mana phrase: the object.
fn colors_owner(s: &str, b: &mut Builder) -> Option<Sel> {
    let who = s.strip_suffix(" colors")?;
    let who = match who {
        "its" => "it",
        w => w.strip_suffix("'s")?,
    };
    let saved = b.targets.len();
    let (sel, rest) = crate::oracle::effects::object_ref(who, b)?;
    if b.targets.len() != saved || !rest.trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    let j = serde_json::to_string(&sel).unwrap_or_default();
    if j.contains(&format!(
        "\"Var\":{}",
        super::oracle_hardening_referents::NO_REFERENT
    )) {
        return None;
    }
    Some(sel)
}

/// "add three mana in any combination of its colors" (Omnath, Locus of All), "add one
/// mana of any of that card's colors": mana of the colors of the object the text refers
/// to, each mana's color chosen separately (CR 106.1, 106.5: none if it's colorless).
fn add_mana_of_its_colors(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("add ")?;
    if let Some(x) = r.strip_prefix("one mana of any of ") {
        let sel = colors_owner(x, b)?;
        return Some(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::AnyColorAmong(Filter::In(Box::new(sel))),
            restriction: None,
        });
    }
    let (n, tail) = crate::oracle::phrases::parse_number(r)?;
    if !matches!(n, Value::Const(k) if k > 0) {
        return None;
    }
    let x = tail.trim().strip_prefix("mana in any combination of ")?;
    let sel = colors_owner(x, b)?;
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::CombinationOfColorsOf(sel, n),
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "mana 6: add N mana in any combination of its colors", priority: 100, parse: add_mana_of_its_colors } }

/// Whether an effect includes an optional reveal ("You may reveal that card").
fn has_optional_reveal(e: &Effect) -> bool {
    match e {
        Effect::May { effect, .. } => {
            let j = serde_json::to_string(effect).unwrap_or_default();
            j.contains("\"reveal\":true") || j.starts_with("{\"Reveal")
        }
        Effect::Seq(v) => v.iter().any(has_optional_reveal),
        Effect::If { then, otherwise, .. } => {
            has_optional_reveal(then) || has_optional_reveal(otherwise)
        }
        _ => false,
    }
}

/// "If you don't reveal it, put it into your hand." after "You may reveal that card [if
/// ...]. If you do, ..." (Omnath, Locus of All): what happens when the card wasn't
/// revealed, whether its controller chose not to or couldn't (the condition didn't
/// hold). The same as "If you don't, ...", with the instruction named.
fn if_you_dont_reveal_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(body) = ["if you don't reveal it, ", "if you don't reveal that card, "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    if !has_optional_reveal(prev) {
        return false;
    }
    let saved = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(body, b) else {
        b.targets.truncate(saved);
        return false;
    };
    let neg = Effect::If {
        cond: Condition::Not(Box::new(Condition::PrevHappened)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, neg]);
    true
}

inventory::submit! { FollowupPattern { name: "mana 6: if you don't reveal it, [effect]", priority: 100, apply: if_you_dont_reveal_it } }
