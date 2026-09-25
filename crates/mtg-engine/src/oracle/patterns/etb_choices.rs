//! Enters-the-battlefield replacement effects (CR 614.1c–d, 614.12) and "as this enters,
//! choose ..." abilities whose choices other abilities refer to as "the chosen [value]"
//! (CR 607.2d).
//!
//! Grammar handled here (text is normalized: the card's own name is `~`):
//!
//! ```text
//! etb      := "as ~ enters, " as-body
//!           | "~ enters " entry [" unless " cond | " if " cond]
//!           | "if " cond ", " ("~" | "it") " enters " entry
//!           | etb ". as it enters, " as-body
//! entry    := "tapped" | "tapped with " counters | "with " counters
//!           | "tapped and doesn't untap during your untap step"
//! counters := count kind "counter(s) on it" [" for each " thing | ", where x is " value]
//!           | "a number of " kind " counters on it equal to " value
//!           | "your choice of a " kind " counter or a " kind " counter on it"
//! as-body  := sentence ("." sentence)*   (choose ..., you may pay N life,
//!             you may reveal a ... card from your hand, if you don't, it enters tapped)
//! ```
//!
//! "As this enters" bodies are executed while the replacement effect applies, before
//! the permanent enters (CR 614.12a); see [`ReplacementAction::AsEnters`].

use super::{AbilityPattern, ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::{split_sentences, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::{parse_condition, parse_value_phrase};
use crate::oracle::CompileContext;
use crate::types::*;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

fn etb_replacement(action: ReplacementAction) -> StaticEffect {
    StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action,
        self_replacement: false,
        optional: false,
    })
}

/// Wraps an entry effect as a replacement action, using the plain actions for the
/// unconditional common cases.
fn entry_action(e: Effect) -> ReplacementAction {
    match e {
        Effect::EnterTapped => ReplacementAction::EnterTapped,
        Effect::EnterWithCounters { kind, n } => ReplacementAction::EnterWithCounters(kind, n),
        other => ReplacementAction::AsEnters(Box::new(other)),
    }
}

// ---------------------------------------------------------------------------
// Whole-ability pattern
// ---------------------------------------------------------------------------

fn etb_ability(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = block
        .to_lowercase()
        .replace("enters the battlefield", "enters");
    let l = end(&lower);
    let effects = parse_etb(l, ctx)?;
    Some(
        effects
            .into_iter()
            .map(|e| static_ability(e, block))
            .collect(),
    )
}

/// Pronouns for the permanent itself in "it enters", "on it".
const SELF_PRONOUNS: [&str; 5] = ["~", "it", "he", "she", "they"];

fn parse_etb(l: &str, ctx: &CompileContext) -> Option<Vec<StaticEffect>> {
    let l = end(l);
    // "~ enters tapped. As it enters, choose a color."
    for sep in [". as it enters, ", ". as ~ enters, "] {
        if let Some((a, b)) = l.split_once(sep) {
            let mut v = parse_etb(a, ctx)?;
            let body = parse_as_enters_body(b, ctx)?;
            v.push(etb_replacement(ReplacementAction::AsEnters(Box::new(body))));
            return Some(v);
        }
    }
    if let Some(r) = l.strip_prefix("as ~ enters, ") {
        let body = parse_as_enters_body(r, ctx)?;
        return Some(vec![etb_replacement(ReplacementAction::AsEnters(
            Box::new(body),
        ))]);
    }
    // "~ enters with X +1/+1 counters on it. If X is 5 or more, it enters with an
    // additional X +1/+1 counters on it.": one replacement effect per sentence.
    let sentences = split_sentences(l);
    if sentences.len() > 1 {
        let mut v = Vec::new();
        for s in sentences {
            v.extend(parse_etb(&s.to_lowercase(), ctx)?);
        }
        return Some(v);
    }
    // "If ~ would enter, instead sacrifice each other permanent named ~ you control, then
    // put ~ onto the battlefield." (a replacement effect that still puts it onto the
    // battlefield: the sacrifice happens as it enters)
    if l == "if ~ would enter, instead sacrifice each other permanent named ~ you control, then put ~ onto the battlefield"
    {
        return Some(vec![etb_replacement(ReplacementAction::AsEnters(
            Box::new(Effect::SacrificeObjects {
                what: Sel::All(Filter::and(vec![
                    Filter::Other,
                    Filter::Permanent,
                    Filter::SameNameAs(Box::new(Sel::This)),
                    Filter::ControlledBy(PlayerRel::You),
                ])),
            }),
        ))]);
    }
    // "If [condition], ~ enters [tapped / with counters]."
    if let Some(r) = l.strip_prefix("if ") {
        for p in SELF_PRONOUNS {
            let sep = format!(", {p} enters ");
            if let Some((c, rest)) = r.split_once(sep.as_str()) {
                let cond = etb_condition(c, ctx)?;
                let e = entry(rest, ctx)?;
                return Some(vec![etb_replacement(ReplacementAction::AsEnters(
                    Box::new(Effect::If {
                        cond,
                        then: Box::new(e),
                        otherwise: Box::new(Effect::Noop),
                    }),
                ))]);
            }
        }
        return None;
    }
    let r = l.strip_prefix("~ enters ")?;
    // "~ enters tapped and doesn't untap during your untap step."
    if r == "tapped and doesn't untap during your untap step" {
        return Some(vec![
            etb_replacement(ReplacementAction::EnterTapped),
            StaticEffect::Restriction(Restriction::DoesntUntap(Filter::Source)),
        ]);
    }
    // "... unless [condition]"
    if let Some((m, c)) = r.split_once(" unless ") {
        let e = entry(m, ctx)?;
        let cond = etb_condition(c, ctx)?;
        return Some(vec![etb_replacement(ReplacementAction::AsEnters(
            Box::new(Effect::If {
                cond,
                then: Box::new(Effect::Noop),
                otherwise: Box::new(e),
            }),
        ))]);
    }
    // "... if [condition]"
    if let Some((m, c)) = r.split_once(" if ") {
        let e = entry(m, ctx)?;
        let cond = etb_condition(c, ctx)?;
        return Some(vec![etb_replacement(ReplacementAction::AsEnters(
            Box::new(Effect::If {
                cond,
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            }),
        ))]);
    }
    let e = entry(r, ctx)?;
    // Separate replacement effects for "enters tapped with N counters" (each modifies
    // how it enters, CR 614.1c).
    match e {
        Effect::Seq(v) => Some(
            v.into_iter()
                .map(|x| etb_replacement(entry_action(x)))
                .collect(),
        ),
        other => Some(vec![etb_replacement(entry_action(other))]),
    }
}

/// How the permanent enters: "tapped", "tapped with two charge counters on it",
/// "with a +1/+1 counter on it for each creature card in your graveyard".
fn entry(s: &str, ctx: &CompileContext) -> Option<Effect> {
    let s = end(s);
    if s == "tapped" {
        return Some(Effect::EnterTapped);
    }
    // CR 722.3a: "enters prepared".
    if s == "prepared" {
        return Some(Effect::EnterPrepared);
    }
    if let Some(r) = s
        .strip_prefix("tapped with ")
        .or_else(|| s.strip_prefix("tapped and with "))
    {
        let c = with_parts(r, ctx)?;
        return Some(Effect::seq(vec![Effect::EnterTapped, c]));
    }
    if let Some(r) = s.strip_prefix("with ") {
        return with_parts(r, ctx);
    }
    None
}

/// What a permanent enters with: counters and/or abilities joined by "and" ("two +1/+1
/// counters on it and with flying", "two +1/+1 counters and a lifelink counter on it",
/// "a +1/+1 counter on it for each red creature you control and a +1/+1 counter on it
/// for each green creature you control").
fn with_parts(s: &str, ctx: &CompileContext) -> Option<Effect> {
    let s = end(s);
    if let Some(e) = counters(s, ctx).or_else(|| entry_abilities(s, ctx)) {
        return Some(e);
    }
    // "a +1/+1 counter, a flying counter, a deathtouch counter, and a shield counter on it"
    if let Some(body) = on_self_suffix(s).filter(|b| b.contains(", ")) {
        let items: Option<Vec<Effect>> = body
            .split(", and ")
            .flat_map(|p| p.split(", "))
            .map(|item| counters(&format!("{} on it", item.trim()), ctx))
            .collect();
        if let Some(items) = items {
            return Some(Effect::seq(items));
        }
    }
    for (i, _) in s.match_indices(" and ") {
        let (a, b) = (&s[..i], &s[i + " and ".len()..]);
        let b = b.strip_prefix("with ").unwrap_or(b);
        // "two +1/+1 counters and a lifelink counter on it": both share "on it".
        let ea = counters(a, ctx).or_else(|| {
            if a.contains(" on ") {
                return None;
            }
            let at = b.find(" on it")?;
            counters(&format!("{a}{}", &b[at..at + " on it".len()]), ctx)
        });
        let Some(ea) = ea else { continue };
        if let Some(eb) = with_parts(b, ctx) {
            return Some(Effect::seq(vec![ea, eb]));
        }
    }
    None
}

/// "flying", "haste", "first strike": abilities a permanent enters with, which it has
/// for as long as it remains on the battlefield (CR 614.1c).
fn entry_abilities(s: &str, ctx: &CompileContext) -> Option<Effect> {
    Some(Effect::OnEntry(Box::new(Effect::Modify {
        what: Sel::This,
        mods: keyword_list(s, ctx)?,
        duration: Duration::Permanent,
    })))
}

/// "flying", "flying and haste", "first strike, vigilance, and lifelink".
fn keyword_list(s: &str, ctx: &CompileContext) -> Option<Vec<Modification>> {
    let mut mods = Vec::new();
    for p in crate::oracle::keywords::split_keyword_phrases(s) {
        let p = p.trim();
        if p.is_empty() || p.contains("counter") || p.contains('"') {
            return None;
        }
        for a in crate::oracle::keywords::parse_keyword_line(p, ctx)? {
            match &a.kind {
                AbilityKind::Keyword(k) => mods.push(Modification::AddKeyword(k.clone())),
                _ => return None,
            }
        }
    }
    (!mods.is_empty()).then_some(mods)
}

/// "a 3/3 creature", "a 2/2 creature with flying": the characteristics an "as enters"
/// ability gives the permanent (CR 707.2).
fn creature_form(s: &str, in_addition: bool, ctx: &CompileContext) -> Option<Vec<Modification>> {
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (pt, mut r) = r.split_once(' ')?;
    let (p, t) = pt.split_once('/')?;
    let p: i32 = p.parse().ok()?;
    let t: i32 = t.parse().ok()?;
    let mut mods = vec![Modification::SetPT(Some(Value::c(p)), Some(Value::c(t)))];
    // "a 1/6 Wall artifact creature with defender in addition to its other types": types
    // it already has, plus added creature types.
    let mut added = Vec::new();
    loop {
        let (w, rest) = r.split_once(' ').unwrap_or((r, ""));
        if w == "creature" || w.is_empty() {
            break;
        }
        if let Some(ct) = CardType::from_word(w) {
            if !ctx.type_line.card_types.contains(ct) {
                return None;
            }
        } else {
            let st = subtype_word(w)?;
            if !is_creature_type(&st) {
                return None;
            }
            added.push(st);
        }
        r = rest;
    }
    if !added.is_empty() {
        if !in_addition {
            return None;
        }
        mods.push(Modification::AddSubtypes(added));
    }
    let r = r.strip_prefix("creature")?.trim();
    if !r.is_empty() {
        mods.extend(keyword_list(r.strip_prefix("with ")?, ctx)?);
    }
    Some(mods)
}

/// "a +1/+1 counter on it", "X +1/+1 counters on it, where X is ...", "a number of
/// +1/+1 counters on it equal to ...", "your choice of a reach counter or a vigilance
/// counter on it".
fn counters(s: &str, ctx: &CompileContext) -> Option<Effect> {
    let s = end(s);
    if let Some(r) = s.strip_prefix("your choice of ") {
        return counter_choice(r);
    }
    if let Some(r) = s.strip_prefix("a number of ") {
        let (kind, r) = crate::oracle::costs::counter_kind(r)?;
        let r = strip(r, "counters")?;
        let r = on_self(r)?;
        let v = strip(r, "equal to ").and_then(|v| etb_value(v, ctx))?;
        return Some(Effect::EnterWithCounters { kind, n: v });
    }
    // "an additional X +1/+1 counters on it", "an additional +1/+1 counter on it"
    let (s, additional) = match s.strip_prefix("an additional ") {
        Some(r) => (r, true),
        None => (s, false),
    };
    let (n, r) = if let Some(r) = s.strip_prefix("twice x ") {
        (Value::Mul(Box::new(Value::c(2)), Box::new(Value::X)), r)
    } else if additional && (s.starts_with('+') || s.starts_with('-')) {
        (Value::c(1), s)
    } else {
        parse_number(s)?
    };
    // "three additional time counters"
    let r = strip(r, "additional").unwrap_or(r);
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    let tail = on_self(r)?;
    let tail = tail.trim();
    // "a +1/+1 counter on it plus an additional +1/+1 counter on it for each other
    // creature you control"
    if let Some(more) = tail.strip_prefix("plus ") {
        let Effect::EnterWithCounters { kind: k2, n: n2 } = counters(more, ctx)? else {
            return None;
        };
        if k2 != kind || !more.starts_with("an additional ") {
            return None;
        }
        return Some(Effect::EnterWithCounters {
            kind,
            n: Value::Sum(vec![n, n2]),
        });
    }
    let n = if tail.is_empty() {
        n
    } else if let Some(t) = tail.strip_prefix("for each ") {
        let each = for_each_value(t, ctx)?;
        match n {
            Value::Const(1) => each,
            other => Value::Mul(Box::new(other), Box::new(each)),
        }
    } else if let Some(t) = tail.strip_prefix(", where x is ") {
        if !matches!(n, Value::X) {
            return None;
        }
        etb_value(t, ctx)?
    } else {
        return None;
    };
    Some(Effect::EnterWithCounters { kind, n })
}

/// "a reach counter or a vigilance counter on it", "a +1/+1, first strike, or vigilance
/// counter on it", "two different counters on it from among menace, deathtouch, and
/// lifelink": the choice is made as the permanent enters (CR 614.12a).
fn counter_choice(r: &str) -> Option<Effect> {
    let r = end(r);
    let one = |k: &CounterKind| Effect::EnterWithCounters {
        kind: k.clone(),
        n: Value::c(1),
    };
    let options = if let Some(x) = r.strip_prefix("two different counters on it from among ") {
        let kinds = counter_list(x)?;
        let mut options = Vec::new();
        for (i, a) in kinds.iter().enumerate() {
            for b in &kinds[i + 1..] {
                options.push((
                    format!("{a} and {b} counters"),
                    Effect::seq(vec![one(a), one(b)]),
                ));
            }
        }
        options
    } else {
        let body = on_self_suffix(r)?;
        let kinds = if let Some((a, b)) = body.split_once(" counter or ") {
            let a = a.strip_prefix("a ")?;
            let b = b.strip_prefix("a ")?.strip_suffix(" counter")?;
            vec![counter_name(a)?, counter_name(b)?]
        } else {
            let list = body.strip_prefix("a ")?.strip_suffix(" counter")?;
            counter_list(list)?
        };
        kinds
            .iter()
            .map(|k| (format!("{k} counter"), one(k)))
            .collect()
    };
    (options.len() >= 2).then_some(Effect::ChooseOne {
        who: PlayerRef::You,
        options,
    })
}

/// "X on it" -> "X".
fn on_self_suffix(s: &str) -> Option<&str> {
    ["on it", "on him", "on her", "on them"]
        .iter()
        .find_map(|p| s.strip_suffix(&format!(" {p}")))
}

/// A counter kind: "+1/+1", "flying", "first strike" (CR 122.1b).
fn counter_name(s: &str) -> Option<CounterKind> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if crate::layers::keyword_counter(s).is_some() {
        return Some(s.into());
    }
    let (k, rest) = crate::oracle::costs::counter_kind(s)?;
    rest.trim().is_empty().then_some(k)
}

/// "menace, deathtouch, and lifelink", "+1/+1, first strike, or vigilance"
fn counter_list(s: &str) -> Option<Vec<CounterKind>> {
    let mut out = Vec::new();
    for p in s
        .split(", and ")
        .flat_map(|p| p.split(", or "))
        .flat_map(|p| p.split(", "))
    {
        out.push(counter_name(p)?);
    }
    (out.len() >= 2).then_some(out)
}

/// Strips "on it" (or "on him"/"on her"/"on them") and returns the rest.
fn on_self(s: &str) -> Option<&str> {
    let s = s.trim_start();
    for p in ["on it", "on him", "on her", "on them"] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',']) {
                return Some(r);
            }
        }
    }
    None
}

/// "for each [X]" amounts.
fn for_each_value(s: &str, ctx: &CompileContext) -> Option<Value> {
    let s = end(s);
    // "{G}{G} spent to cast it": each two green mana spent
    if let Some(v) = s
        .strip_suffix(" spent to cast it")
        .or_else(|| s.strip_suffix(" spent to cast ~"))
        .and_then(mana_symbols_spent)
    {
        return Some(v);
    }
    // "+1/+1 counter among other creatures you control"
    if let Some((k, r)) = s.split_once(" counter among ") {
        if !k.contains(' ') {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            return Some(Value::CountersOn(Box::new(Sel::All(f)), Some(k.into())));
        }
    }
    // "loyalty counter on planeswalkers you control"
    if let Some((k, r)) = s.split_once(" counter on ") {
        if !k.contains(' ') {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            return Some(Value::CountersOn(Box::new(Sel::All(f)), Some(k.into())));
        }
    }
    // "each other Zombie you control and each Zombie card in your graveyard"
    if let Some((a, b)) = s.split_once(" and each ") {
        let va = for_each_value(a, ctx)?;
        let vb = for_each_value(b, ctx)?;
        return Some(Value::Sum(vec![va, vb]));
    }
    let fixed = [
        ("mana spent to cast it", Value::ManaSpent),
        ("mana spent to cast ~", Value::ManaSpent),
        ("other spell cast this turn", Value::StormCount),
        (
            "creature that died under your control this turn",
            Value::Custom("creatures_you_controlled_died_this_turn".into()),
        ),
        ("color of mana spent to cast it", Value::ColorsSpent),
        ("color of mana spent to cast ~", Value::ColorsSpent),
        ("time it was kicked", Value::TimesKicked),
        ("time ~ was kicked", Value::TimesKicked),
        ("creature that died this turn", Value::CreaturesDiedThisTurn),
        (
            "opponent you have",
            Value::CountPlayers(PlayerFilter::Opponent),
        ),
    ];
    for (p, v) in fixed {
        if s == p {
            return Some(v);
        }
    }
    let _ = ctx;
    // "creature that player controls" after "choose an opponent" (CR 607.2d)
    for sfx in [" that player controls", " the chosen player controls"] {
        if let Some(r) = s.strip_suffix(sfx) {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            return Some(Value::Count(Filter::and(vec![
                f,
                Filter::ControlledBy(PlayerRel::Chosen),
            ])));
        }
    }
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Value::Count(f))
}

/// "{G}{G}" (as in "for each {G}{G} spent to cast it"): the number of complete groups of
/// that many mana of one color spent to cast it.
fn mana_symbols_spent(sym: &str) -> Option<Value> {
    let letters: Vec<&str> = sym
        .strip_prefix('{')?
        .strip_suffix('}')?
        .split("}{")
        .collect();
    let l0 = letters[0].to_uppercase();
    if l0.len() != 1
        || !"WUBRG".contains(l0.as_str())
        || letters.iter().any(|x| x.to_uppercase() != l0)
    {
        return None;
    }
    let spent = Value::Custom(format!("mana_spent_of:{l0}").into());
    Some(match letters.len() {
        1 => spent,
        n => Value::Div(Box::new(spent), n as i32, false),
    })
}

/// Values in "equal to [value]" / "where X is [value]".
fn etb_value(s: &str, ctx: &CompileContext) -> Option<Value> {
    let s = end(s);
    let fixed = [
        ("the amount of mana spent to cast it", Value::ManaSpent),
        ("the amount of mana spent to cast ~", Value::ManaSpent),
        (
            "the amount of life you gained this turn",
            Value::LifeGainedThisTurn(PlayerRef::You),
        ),
        (
            "the amount of life you've gained this turn",
            Value::LifeGainedThisTurn(PlayerRef::You),
        ),
        (
            "the number of creatures that died this turn",
            Value::CreaturesDiedThisTurn,
        ),
    ];
    for (p, v) in fixed {
        if s == p {
            return Some(v);
        }
    }
    // "the greatest power among other creatures you control", "... among creatures you
    // control and creature cards in your graveyard"
    if let Some(r) = s.strip_prefix("the greatest power among ") {
        let (f, _, tail) = parse_object_phrase(r)?;
        let tail = end(tail);
        if tail.is_empty() {
            return Some(Value::GreatestPower(f));
        }
        let (g, _, tail2) = parse_object_phrase(tail.strip_prefix("and ")?)?;
        if !end(tail2).is_empty() {
            return None;
        }
        return Some(Value::Max(
            Box::new(Value::GreatestPower(f)),
            Box::new(Value::GreatestPower(g)),
        ));
    }
    if s == "the total number of cards in all players' hands" {
        return Some(Value::Custom("cards_in_all_hands".into()));
    }
    // "one plus the number of other creatures you control"
    if let Some((a, b)) = s.split_once(" plus ") {
        let (va, ta) = parse_number(a)?;
        if !ta.trim().is_empty() {
            return None;
        }
        return Some(Value::Sum(vec![va, etb_value(b, ctx)?]));
    }
    // "the number of creature cards in all graveyards", "the number of instant and
    // sorcery cards in all graveyards" (cards of either type)
    if let Some(r) = s
        .strip_prefix("the number of ")
        .and_then(|r| r.strip_suffix(" in all graveyards"))
    {
        let r = r.replacen(" and ", " or ", 1);
        let (f, _, tail) = parse_object_phrase(&r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Value::Count(f.in_zone(ZoneKind::Graveyard)));
    }
    // "three minus X"
    if let Some((a, b)) = s.split_once(" minus ") {
        let (va, ta) = parse_number(a)?;
        if !ta.trim().is_empty() || b != "x" {
            return None;
        }
        return Some(Value::Diff(Box::new(va), Box::new(Value::X)));
    }
    if s == "the total life lost by your opponents this turn" {
        return Some(Value::Custom("life_lost_by_opponents_this_turn".into()));
    }
    // "the number of other creatures on the battlefield"
    let s2 = s
        .strip_suffix(" on the battlefield")
        .unwrap_or(s)
        .to_string();
    let mut b = Builder::new(ctx);
    let (v, tail) = parse_value_phrase(&s2, &mut b)?;
    if !end(&tail).is_empty() || !b.targets.is_empty() {
        return None;
    }
    // Pronoun-based values ("its power") would refer to the entering object itself.
    if matches!(
        v,
        Value::PowerOf(_) | Value::ToughnessOf(_) | Value::ManaValueOf(_) | Value::EventAmount
    ) {
        return None;
    }
    Some(v)
}

// ---------------------------------------------------------------------------
// "As ~ enters, ..." bodies
// ---------------------------------------------------------------------------

/// Parses the sentences of an "as this enters" ability.
fn parse_as_enters_body(s: &str, ctx: &CompileContext) -> Option<Effect> {
    let mut out = Vec::new();
    for sent in split_sentences(s) {
        let lower = sent.to_lowercase();
        let l = end(&lower);
        let l = l.strip_prefix("then ").unwrap_or(l);
        out.push(as_enters_sentence(l, ctx)?);
    }
    (!out.is_empty()).then(|| Effect::seq(out))
}

fn as_enters_sentence(l: &str, ctx: &CompileContext) -> Option<Effect> {
    let l = end(l);
    // "choose a basic land type. then you may pay 2 life" arrives as separate sentences;
    // "choose X, then Y" within one sentence:
    if let Some((a, b)) = l.split_once(", then ") {
        let ea = as_enters_sentence(a, ctx)?;
        let eb = as_enters_sentence(b, ctx)?;
        return Some(Effect::seq(vec![ea, eb]));
    }
    if let Some(e) = choose_clause(l) {
        return Some(e);
    }
    // "look at an opponent's hand" (hidden information for that player only)
    if l == "look at an opponent's hand" {
        return Some(Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Opponent,
            },
            Effect::RevealHand {
                who: PlayerRef::ChosenOpponent,
            },
        ]));
    }
    // "you may pay N life" — an optional cost; "if you don't" refers to it.
    if let Some(r) = l.strip_prefix("you may pay ") {
        let (n, r) = parse_number(r)?;
        if end(r) != "life" {
            return None;
        }
        return Some(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::free().with(CostPart::PayLife(n)),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        });
    }
    // "you may reveal a Faerie card from your hand"
    if let Some(r) = l.strip_prefix("you may reveal ") {
        let (n, r) = parse_number(r)?;
        let r = r.trim_start();
        let r = r.strip_suffix(" from your hand")?;
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::free().with(CostPart::RevealFromHand {
                filter: f,
                count: n,
            }),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        });
    }
    // "you may return a nonland permanent you control to its owner's hand", "you may
    // sacrifice a creature": an optional cost paid as it enters; "if you do" refers to it.
    if let Some(r) = l.strip_prefix("you may ") {
        let (cost, false) = crate::oracle::costs::parse_cost(r)? else {
            return None;
        };
        let simple = cost.mana.is_none()
            && cost.parts.len() == 1
            && matches!(
                cost.parts[0],
                CostPart::Sacrifice { .. }
                    | CostPart::Discard { .. }
                    | CostPart::Exile { .. }
                    | CostPart::ReturnToHand { .. }
                    | CostPart::TapUntapped { .. }
                    | CostPart::PayEnergy(_)
            );
        if !simple {
            return None;
        }
        return Some(Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        });
    }
    if let Some(r) = l.strip_prefix("if you don't, ") {
        let e = self_entry(r, ctx)?;
        return Some(Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    if let Some(r) = l.strip_prefix("if you do, ") {
        let e = self_entry(r, ctx)?;
        return Some(Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    // "if you do or if you control a Dragon, ~ enters with a +1/+1 counter on it"
    if let Some(r) = l.strip_prefix("if you do or if ") {
        let (c, rest) = r.split_once(", ")?;
        let cond = etb_condition(c, ctx)?;
        let e = self_entry(rest, ctx)?;
        return Some(Effect::If {
            cond: Condition::Or(vec![Condition::PrevHappened, cond]),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    // "it becomes your choice of a 3/3 creature, a 2/2 creature with flying, or a 1/6
    // creature with defender" (CR 707.2: part of its copiable values)
    if let Some(r) = SELF_PRONOUNS
        .iter()
        .find_map(|p| l.strip_prefix(&format!("{p} becomes your choice of ")))
    {
        if !ctx.type_line.card_types.contains(CardType::Creature) {
            return None;
        }
        let (r, in_addition) = match r.strip_suffix(" in addition to its other types") {
            Some(x) => (x, true),
            None => (r, false),
        };
        let mut options = Vec::new();
        for part in r
            .split(", or ")
            .flat_map(|p| p.split(", "))
            .flat_map(|p| p.split(" or "))
        {
            let part = part.trim();
            options.push((
                part.to_string(),
                Effect::EnterAs(creature_form(part, in_addition, ctx)?),
            ));
        }
        return (options.len() >= 2).then_some(Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        });
    }
    // "~ enters tapped unless you revealed a Dragon card this way or you control a Dragon"
    if let Some(r) = SELF_PRONOUNS
        .iter()
        .find_map(|p| l.strip_prefix(&format!("{p} enters ")))
    {
        if let Some((m, c)) = r.split_once(" unless ") {
            let e = entry(m, ctx)?;
            let cond = etb_condition(c, ctx)?;
            return Some(Effect::If {
                cond,
                then: Box::new(Effect::Noop),
                otherwise: Box::new(e),
            });
        }
        return entry(r, ctx);
    }
    None
}

/// "it enters tapped", "~ enters tapped".
fn self_entry(s: &str, ctx: &CompileContext) -> Option<Effect> {
    let s = end(s);
    let r = SELF_PRONOUNS
        .iter()
        .find_map(|p| s.strip_prefix(&format!("{p} enters ")))?;
    entry(r, ctx)
}

/// "choose a color", "choose a creature type", "choose a color and an opponent", ...
fn choose_clause(l: &str) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose ")?;
    if r.starts_with("a number between ") {
        return Some(Effect::Choose {
            who: PlayerRef::You,
            kind: choice_kind(r)?,
        });
    }
    let mut out = Vec::new();
    for part in r.split(" and ") {
        out.push(Effect::Choose {
            who: PlayerRef::You,
            kind: choice_kind(part.trim())?,
        });
    }
    Some(Effect::seq(out))
}

fn choice_kind(s: &str) -> Option<ChoiceKind> {
    Some(match s {
        "a color" => ChoiceKind::Color,
        "a creature type" => ChoiceKind::CreatureType,
        "a card name" | "any card name" => ChoiceKind::CardName,
        "a nonland card name" => ChoiceKind::CardNameFiltered("nonland".into()),
        "a creature card name" => ChoiceKind::CardNameFiltered("creature".into()),
        "an opponent" => ChoiceKind::Opponent,
        "a player" => ChoiceKind::Player,
        "a basic land type" => ChoiceKind::BasicLandType,
        "a card type" => ChoiceKind::CardType,
        "odd or even" => ChoiceKind::OddOrEven,
        "a number" => ChoiceKind::Number { min: 0, max: 1000 },
        _ => {
            // "a noncreature, nonland card name", "a nonbasic land card name"
            if let Some(words) = s
                .strip_prefix("a ")
                .or_else(|| s.strip_prefix("an "))
                .and_then(|r| r.strip_suffix(" card name"))
            {
                let ok = words.split([',', ' ']).filter(|w| !w.is_empty()).all(|w| {
                    let t = w.strip_prefix("non").unwrap_or(w);
                    CardType::from_word(t).is_some() || matches!(t, "basic" | "legendary")
                });
                return ok.then(|| ChoiceKind::CardNameFiltered(words.into()));
            }
            // "a card type other than creature", "... other than creature or land"
            if let Some(r) = s.strip_prefix("a card type other than ") {
                let mut excluded = Vec::new();
                for w in r.split(" or ") {
                    excluded.push(CardType::from_word(w.trim())?);
                }
                let words = [
                    CardType::Artifact,
                    CardType::Battle,
                    CardType::Creature,
                    CardType::Enchantment,
                    CardType::Instant,
                    CardType::Kindred,
                    CardType::Land,
                    CardType::Planeswalker,
                    CardType::Sorcery,
                ]
                .into_iter()
                .filter(|t| !excluded.contains(t))
                .map(|t| t.word().to_string())
                .collect();
                return Some(ChoiceKind::OneOf(words));
            }
            if let Some(c) = s.strip_prefix("a color other than ") {
                return Some(ChoiceKind::ColorOtherThan(Color::from_word(c)?));
            }
            if let Some(r) = s.strip_prefix("a number between ") {
                let (a, b) = r.split_once(" and ")?;
                let (Value::Const(a), ra) = parse_number(a)? else {
                    return None;
                };
                let (Value::Const(b), rb) = parse_number(b)? else {
                    return None;
                };
                if !ra.trim().is_empty() || !rb.trim().is_empty() {
                    return None;
                }
                return Some(ChoiceKind::Number { min: a, max: b });
            }
            return word_list_choice(s);
        }
    })
}

/// "Elemental, Elf, or Faerie", "Island or Swamp", "artifact, creature, or enchantment":
/// a choice among listed types or colors (each a known word, so that e.g. "left or
/// right" isn't mistaken for one).
fn word_list_choice(s: &str) -> Option<ChoiceKind> {
    if !s.contains(" or ") {
        return None;
    }
    let mut words = Vec::new();
    for w in s
        .split(", or ")
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" or "))
    {
        let w = w.trim();
        if w.is_empty() || w.contains(' ') {
            return None;
        }
        let canon = if let Some(c) = Color::from_word(w) {
            c.word().to_string()
        } else if let Some(t) = CardType::from_word(w) {
            t.word().to_string()
        } else if let Some(st) = subtype_word(w) {
            if !is_creature_type(&st) && !is_basic_land_type(&st) {
                return None;
            }
            st.to_string()
        } else {
            return None;
        };
        words.push(canon);
    }
    (words.len() >= 2).then_some(ChoiceKind::OneOf(words))
}

// ---------------------------------------------------------------------------
// Conditions used by ETB replacements
// ---------------------------------------------------------------------------

fn etb_condition(c: &str, ctx: &CompileContext) -> Option<Condition> {
    let c = end(c);
    // "A or B" where both halves are conditions ("you revealed a Dragon card this way or
    // you control a Dragon").
    if let Some(cond) = etb_condition_atom(c, ctx) {
        return Some(cond);
    }
    if let Some((a, b)) = c.split_once(" or ") {
        let ca = etb_condition_atom(a, ctx)?;
        let cb = etb_condition(b, ctx)?;
        return Some(Condition::Or(vec![ca, cb]));
    }
    None
}

fn etb_condition_atom(c: &str, ctx: &CompileContext) -> Option<Condition> {
    let c = end(c);
    if c.starts_with("you revealed ") && c.ends_with(" this way") {
        return Some(Condition::PrevHappened);
    }
    // "you've cast another red spell this turn" (other than the entering spell)
    if let Some(col) = c
        .strip_prefix("you've cast another ")
        .and_then(|r| r.strip_suffix(" spell this turn"))
        .and_then(Color::from_word)
    {
        return Some(Condition::Custom(
            format!("you_cast_another_spell_this_turn:{}", col.letter()).into(),
        ));
    }
    // "X is 5 or more": the value of X chosen as the permanent spell was cast (CR 107.3m)
    if let Some(r) = c.strip_prefix("x is ") {
        let (n, rest) = parse_number(r)?;
        let cmp = match end(rest) {
            "or more" | "or greater" => Cmp::Ge,
            "or less" => Cmp::Le,
            _ => return None,
        };
        return Some(Condition::Compare(Value::X, cmp, n));
    }
    parse_condition(c, ctx)
}

/// Conditions the core condition parser doesn't know (registered for all abilities).
fn more_conditions(c: &str) -> Option<Condition> {
    let c = end(c);
    // "you control two or fewer other lands"
    if let Some(r) = c.strip_prefix("you control ") {
        let (n, rest) = parse_number(r)?;
        let rest = strip(rest, "or fewer")?;
        let (f, _, tail) = parse_object_phrase(rest)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Compare(
            Value::Count(f.you_control()),
            Cmp::Le,
            n,
        ));
    }
    // "your opponents control eight or more lands"
    if let Some(r) = c.strip_prefix("your opponents control ") {
        let (n, rest) = parse_number(r)?;
        let rest = strip(rest, "or more")?;
        let (f, _, tail) = parse_object_phrase(rest)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Compare(
            Value::Count(f.opp_controls()),
            Cmp::Ge,
            n,
        ));
    }
    // "a player has 13 or less life"
    if let Some(r) = c
        .strip_prefix("a player has ")
        .or_else(|| c.strip_prefix("an opponent has "))
    {
        let who = if c.starts_with("a player") {
            PlayerRef::EachPlayer
        } else {
            PlayerRef::EachOpponent
        };
        let (n, rest) = parse_number(r)?;
        let cmp = match end(rest) {
            "or less life" => Cmp::Le,
            "or more life" => Cmp::Ge,
            _ => return None,
        };
        return Some(Condition::PlayerMatches(
            who,
            PlayerFilter::Life(cmp, Box::new(n)),
        ));
    }
    // Adamant: "at least three blue mana was spent to cast ~", "at least three mana of
    // the same color was spent to cast it".
    if let Some(r) = c.strip_prefix("at least ") {
        let (n, rest) = parse_number(r)?;
        let rest = rest.trim();
        let body = rest
            .strip_suffix(" was spent to cast ~")
            .or_else(|| rest.strip_suffix(" was spent to cast it"))?;
        let v = if body == "mana of the same color" {
            Value::Custom("max_mana_spent_of_one_color".into())
        } else {
            let letter = match body.strip_suffix(" mana")? {
                "white" => 'W',
                "blue" => 'U',
                "black" => 'B',
                "red" => 'R',
                "green" => 'G',
                "colorless" => 'C',
                _ => return None,
            };
            Value::Custom(format!("mana_spent_of:{letter}").into())
        };
        return Some(Condition::Compare(v, Cmp::Ge, n));
    }
    // "you gained life this turn", "you gained 2 or more life this turn"
    if c == "you gained life this turn" {
        return Some(Condition::Compare(
            Value::LifeGainedThisTurn(PlayerRef::You),
            Cmp::Ge,
            Value::c(1),
        ));
    }
    if let Some(r) = c.strip_prefix("you gained ") {
        let (n, rest) = parse_number(r)?;
        if end(rest) == "or more life this turn" {
            return Some(Condition::Compare(
                Value::LifeGainedThisTurn(PlayerRef::You),
                Cmp::Ge,
                n,
            ));
        }
        return None;
    }
    // "two or more creatures died this turn"
    if let Some(rest) = c.strip_suffix(" or more creatures died this turn") {
        let (n, tail) = parse_number(rest)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some(Condition::Compare(Value::CreaturesDiedThisTurn, Cmp::Ge, n));
    }
    // "there are three or more creature cards in your graveyard"
    if let Some(r) = c.strip_prefix("there are ") {
        let (n, rest) = parse_number(r)?;
        let rest = strip(rest, "or more")?;
        let what = rest.strip_suffix(" in your graveyard")?;
        let (f, _, tail) = parse_object_phrase(what)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Compare(
            Value::CardsInGraveyard(PlayerRef::You, f),
            Cmp::Ge,
            n,
        ));
    }
    // "an opponent has more cards in hand than you"
    if c == "an opponent has more cards in hand than you" {
        return Some(Condition::PlayerMatches(
            PlayerRef::EachOpponent,
            PlayerFilter::HandSize(Cmp::Gt, Box::new(Value::HandSize(PlayerRef::You))),
        ));
    }
    // "a player has one or fewer cards in hand"
    if let Some(r) = c.strip_prefix("a player has ") {
        let (n, rest) = parse_number(r)?;
        let cmp = match end(rest) {
            "or fewer cards in hand" => Cmp::Le,
            "or more cards in hand" => Cmp::Ge,
            _ => return None,
        };
        return Some(Condition::PlayerMatches(
            PlayerRef::EachPlayer,
            PlayerFilter::HandSize(cmp, Box::new(n)),
        ));
    }
    // "two or more colors of mana were spent to cast it"
    if let Some(rest) = c
        .strip_suffix(" or more colors of mana were spent to cast it")
        .or_else(|| c.strip_suffix(" or more colors of mana were spent to cast ~"))
    {
        let (n, tail) = parse_number(rest)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some(Condition::Compare(Value::ColorsSpent, Cmp::Ge, n));
    }
    // "you've cast two or more spells this turn"
    if let Some(r) = c.strip_prefix("you've cast ") {
        let (n, rest) = parse_number(r)?;
        if end(rest) == "or more spells this turn" {
            return Some(Condition::Compare(
                Value::Custom("spells_you_cast_this_turn".into()),
                Cmp::Ge,
                n,
            ));
        }
        return None;
    }
    // "you have two or more opponents"
    if let Some(r) = c.strip_prefix("you have ") {
        let (n, rest) = parse_number(r)?;
        if end(rest) == "or more opponents" {
            return Some(Condition::Compare(
                Value::CountPlayers(PlayerFilter::Opponent),
                Cmp::Ge,
                n,
            ));
        }
        return None;
    }
    match c {
        "you attacked this turn" | "you attacked with a creature this turn" => {
            return Some(Condition::Custom("you_attacked_this_turn".into()))
        }
        "a permanent left the battlefield under your control this turn" => {
            return Some(Condition::Custom(
                "permanent_left_under_your_control_this_turn".into(),
            ))
        }
        "an opponent lost life this turn" => {
            return Some(Condition::Custom("opponent_lost_life_this_turn".into()))
        }
        "you were the starting player" => {
            return Some(Condition::Custom("you_were_the_starting_player".into()))
        }
        "you weren't the starting player" => {
            return Some(Condition::Not(Box::new(Condition::Custom(
                "you_were_the_starting_player".into(),
            ))))
        }
        "it wasn't cast or no mana was spent to cast it" => {
            return Some(Condition::Or(vec![
                Condition::Not(Box::new(Condition::WasCast)),
                Condition::Compare(Value::ManaSpent, Cmp::Eq, Value::c(0)),
            ]))
        }
        "a creature died this turn" => {
            return Some(Condition::Compare(
                Value::CreaturesDiedThisTurn,
                Cmp::Ge,
                Value::c(1),
            ))
        }
        "you cast it from your hand"
        | "you cast ~ from your hand"
        | "~ was cast from your hand" => {
            return Some(Condition::And(vec![
                Condition::WasCast,
                Condition::CastFrom(ZoneKind::Hand),
            ]))
        }
        "you didn't cast it from your hand" | "you didn't cast ~ from your hand" => {
            return Some(Condition::Not(Box::new(Condition::And(vec![
                Condition::WasCast,
                Condition::CastFrom(ZoneKind::Hand),
            ]))))
        }
        _ => {}
    }
    None
}

/// "you control a Forest or a Plains", "you control an Island or a Swamp".
fn control_either(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    let parts: Vec<&str> = r.split(" or ").collect();
    if parts.len() < 2 {
        return None;
    }
    let mut v = Vec::new();
    for p in parts {
        let p = p.trim();
        let p = p.strip_prefix("a ").or_else(|| p.strip_prefix("an "))?;
        // The article marks each alternative as singular ("a Plains").
        let (f, _, tail) = parse_object_phrase(p)?;
        if !end(tail).is_empty() {
            return None;
        }
        v.push(Condition::Exists(f.you_control()));
    }
    Some(Condition::Or(v))
}

// ---------------------------------------------------------------------------
// Effects referring to choices
// ---------------------------------------------------------------------------

/// "choose a creature type" etc. as an effect of a spell or ability; the choice is
/// stored on the source (CR 607.2d).
fn choose_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let e = choose_clause(l)?;
    // "Choose an opponent. You and that player each ...": "that player" is the chosen one.
    let chose_player = |e: &Effect| {
        matches!(
            e,
            Effect::Choose {
                kind: ChoiceKind::Opponent | ChoiceKind::Player,
                ..
            }
        )
    };
    let any = match &e {
        Effect::Seq(v) => v.iter().any(chose_player),
        other => chose_player(other),
    };
    if any {
        b.it_player = PlayerRef::ChosenOpponent;
    }
    Some(e)
}

/// "add {R} or one mana of the chosen color", "add two mana of the chosen color".
fn chosen_color_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("add ")?;
    let r = end(r);
    if let Some(syms) = r.strip_suffix(" or one mana of the chosen color") {
        let mut opts = Vec::new();
        for w in syms.split(", ") {
            let w = w.trim().trim_start_matches("or ").trim();
            let inner = w.strip_prefix('{')?.strip_suffix('}')?;
            let t = ManaType::from_letter(inner.to_uppercase().chars().next()?)?;
            if inner.len() != 1 {
                return None;
            }
            opts.push(t);
        }
        return Some(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::OneOfOrChosenColor(opts),
            restriction: None,
        });
    }
    let (n, rest) = parse_number(r)?;
    if end(rest) == "mana of the chosen color" && !matches!(n, Value::Const(1)) {
        return Some(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::ChosenColor(n),
            restriction: None,
        });
    }
    None
}

/// "~ becomes prepared", "target creature becomes unprepared", "each creature you
/// control becomes prepared" (CR 722.3a–b).
fn prepared_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let (subj, prepared) = if let Some(x) = l
        .strip_suffix(" becomes prepared")
        .or_else(|| l.strip_suffix(" become prepared"))
    {
        (x, true)
    } else if let Some(x) = l
        .strip_suffix(" becomes unprepared")
        .or_else(|| l.strip_suffix(" become unprepared"))
    {
        (x, false)
    } else {
        return None;
    };
    let what = match subj {
        "~" => Sel::This,
        "it" | "that creature" | "that permanent" => b.it.clone(),
        _ => {
            if let Some(r) = subj.strip_prefix("each ") {
                let (f, _, tail) = parse_object_phrase(r)?;
                if !end(tail).is_empty() {
                    return None;
                }
                Sel::All(f)
            } else {
                let (spec, tail) = parse_target(subj)?;
                if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
                    return None;
                }
                Sel::Target(b.add_target(spec, subj))
            }
        }
    };
    Some(Effect::SetPrepared { what, prepared })
}

/// "~ is prepared", "~ isn't prepared".
fn prepared_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let (subj, yes) = if let Some(x) = c.strip_suffix(" isn't prepared") {
        (x, false)
    } else if let Some(x) = c.strip_suffix(" is prepared") {
        (x, true)
    } else {
        return None;
    };
    if subj != "~" && subj != "it" {
        return None;
    }
    let f = if yes {
        Filter::Prepared
    } else {
        Filter::not(Filter::Prepared)
    };
    Some(Condition::SelMatches(Sel::This, f))
}

/// Modal permanents with anchor words (CR 614.12c): "As ~ enters, choose Abzan or
/// Mardu." followed by "• Abzan — [ability]" lines. Each anchored ability is an ability
/// the permanent has as long as that word was chosen as it entered (CR 607.2m).
fn anchor_words(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || !block.contains('\n') {
        return None;
    }
    let mut lines = block.lines();
    let head = lines.next()?.trim();
    let hl = head.to_lowercase();
    let prefix = "as ~ enters, choose ";
    if !hl.starts_with(prefix) {
        return None;
    }
    let (a, b) = end(&head[prefix.len()..]).split_once(" or ")?;
    let words: Vec<String> = vec![a.trim().to_string(), b.trim().to_string()];
    if words
        .iter()
        .any(|w| w.is_empty() || !w.chars().all(|c| c.is_alphabetic()))
    {
        return None;
    }
    let mut out = vec![static_ability(
        etb_replacement(ReplacementAction::AsEnters(Box::new(Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::OneOf(words.clone()),
        }))),
        head,
    )];
    let mut seen = 0;
    for line in lines {
        let line = line.trim().trim_start_matches('•').trim();
        let (w, ability_text) = line.split_once(" — ")?;
        let word = words.iter().find(|x| x.eq_ignore_ascii_case(w.trim()))?;
        seen += 1;
        for ab in crate::oracle::parse_ability(ability_text, ctx)? {
            if matches!(ab.kind, AbilityKind::Unsupported(_)) {
                return None;
            }
            let mut st = StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::Source,
                mods: vec![Modification::AddAbility(ab)],
            });
            st.condition = Some(Condition::Chose(word.as_str().into()));
            out.push(AbilityDef::new(AbilityKind::Static(st), line));
        }
    }
    (seen == words.len()).then_some(out)
}

/// "~ is the chosen type in addition to its other types.", "~ is the chosen color.",
/// "All nonland permanents are the chosen color.", "Lands you control are the chosen
/// type in addition to their other types.": characteristic-changing statics using the
/// choice made for the source (CR 607.2d; layers 4 and 5, CR 613.1d–e).
fn chosen_type_static(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let (affected, mods) = match l {
        "~ is the chosen type in addition to its other types" | "~ is the chosen type" => {
            (Filter::Source, vec![Modification::AddChosenType])
        }
        "enchanted land is the chosen type" => (
            Filter::AttachedToSource,
            vec![Modification::SetChosenBasicLandType],
        ),
        _ => {
            let (subj, pred) = l
                .split_once(" is the chosen ")
                .or_else(|| l.split_once(" are the chosen "))?;
            let mods = match pred {
                "color" => vec![Modification::SetChosenColor],
                "type in addition to its other types"
                | "type in addition to their other types"
                | "creature type in addition to its other creature types"
                | "creature type in addition to their other creature types"
                | "creature type in addition to its other types"
                | "creature type in addition to their other types" => {
                    vec![Modification::AddChosenType]
                }
                _ => return None,
            };
            (chosen_static_subject(subj)?, mods)
        }
    };
    Some(vec![static_ability(
        StaticEffect::Continuous { affected, mods },
        block,
    )])
}

/// Subjects of [`chosen_type_static`]: "~", "enchanted land", "each creature you
/// control", "all nonland permanents", "lands you control" (permanents only).
fn chosen_static_subject(s: &str) -> Option<Filter> {
    match s {
        "~" => return Some(Filter::Source),
        "enchanted creature" | "enchanted land" | "enchanted permanent" | "equipped creature" => {
            return Some(Filter::AttachedToSource)
        }
        _ => {}
    }
    let (r, want_plural) = if let Some(r) = s.strip_prefix("each ") {
        (r, false)
    } else if let Some(r) = s.strip_prefix("all ") {
        (r, true)
    } else {
        (s, true)
    };
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural != want_plural || !end(tail).is_empty() {
        return None;
    }
    // Objects not on the battlefield ("cards", "spells") aren't handled here.
    if filter_mentions_other_zones(&f) {
        return None;
    }
    Some(f)
}

fn filter_mentions_other_zones(f: &Filter) -> bool {
    match f {
        Filter::Card | Filter::Spell | Filter::InZone(_) | Filter::PermanentCard => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_mentions_other_zones),
        Filter::Not(x) => filter_mentions_other_zones(x),
        _ => false,
    }
}

inventory::submit! {
    AbilityPattern { name: "etb replacements and as-enters choices", priority: 50, parse: etb_ability }
}
inventory::submit! {
    AbilityPattern { name: "chosen type statics", priority: 50, parse: chosen_type_static }
}
inventory::submit! {
    AbilityPattern { name: "anchor words", priority: 50, parse: anchor_words }
}
inventory::submit! {
    EffectPattern { name: "choose a color/type/name/player", priority: 100, parse: choose_effect }
}
inventory::submit! {
    EffectPattern { name: "mana of the chosen color", priority: 100, parse: chosen_color_mana }
}
inventory::submit! {
    ConditionPattern { name: "etb conditions", priority: 100, parse: more_conditions }
}
inventory::submit! {
    EffectPattern { name: "become prepared", priority: 100, parse: prepared_effect }
}
inventory::submit! {
    ConditionPattern { name: "is prepared", priority: 100, parse: prepared_condition }
}
inventory::submit! {
    ConditionPattern { name: "you control an X or a Y", priority: 100, parse: control_either }
}
