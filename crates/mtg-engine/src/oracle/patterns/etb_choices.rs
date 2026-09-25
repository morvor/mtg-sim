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
    if let Some(r) = s.strip_prefix("tapped with ") {
        let c = counters(r, ctx)?;
        return Some(Effect::seq(vec![Effect::EnterTapped, c]));
    }
    if let Some(r) = s.strip_prefix("with ") {
        return counters(r, ctx);
    }
    None
}

/// "a +1/+1 counter on it", "X +1/+1 counters on it, where X is ...", "a number of
/// +1/+1 counters on it equal to ...", "your choice of a reach counter or a vigilance
/// counter on it".
fn counters(s: &str, ctx: &CompileContext) -> Option<Effect> {
    let s = end(s);
    if let Some(r) = s.strip_prefix("your choice of ") {
        let (a, b) = r.split_once(" or ")?;
        let (_, a) = parse_number(a)?;
        let (ka, a) = crate::oracle::costs::counter_kind(a)?;
        if end(a) != "counter" {
            return None;
        }
        let (_, b) = parse_number(b)?;
        let (kb, b) = crate::oracle::costs::counter_kind(b)?;
        let b = strip(b, "counter")?;
        on_self(b)?;
        return Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options: vec![
                (
                    format!("{ka} counter"),
                    Effect::EnterWithCounters {
                        kind: ka,
                        n: Value::c(1),
                    },
                ),
                (
                    format!("{kb} counter"),
                    Effect::EnterWithCounters {
                        kind: kb,
                        n: Value::c(1),
                    },
                ),
            ],
        });
    }
    if let Some(r) = s.strip_prefix("a number of ") {
        let (kind, r) = crate::oracle::costs::counter_kind(r)?;
        let r = strip(r, "counters")?;
        let r = on_self(r)?;
        let v = strip(r, "equal to ").and_then(|v| etb_value(v, ctx))?;
        return Some(Effect::EnterWithCounters { kind, n: v });
    }
    let (n, r) = if let Some(r) = s.strip_prefix("twice x ") {
        (Value::Mul(Box::new(Value::c(2)), Box::new(Value::X)), r)
    } else {
        parse_number(s)?
    };
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    let tail = on_self(r)?;
    let tail = tail.trim();
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
    let fixed = [
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
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Value::Count(f))
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
    if let Some(r) = l.strip_prefix("if you don't, ") {
        let e = self_entry(r, ctx)?;
        return Some(Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
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
fn choose_effect(l: &str, _b: &mut Builder) -> Option<Effect> {
    choose_clause(l)
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

/// "~ is the chosen type in addition to its other types."
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
        _ => return None,
    };
    Some(vec![static_ability(
        StaticEffect::Continuous { affected, mods },
        block,
    )])
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
