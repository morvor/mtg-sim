//! Adding and removing abilities (CR 113.10–113.11): "Creatures your opponents control
//! lose first strike and can't have or gain first strike" (the Archetype cycle),
//! "[objects] can't have or gain [keyword]".

use crate::ability::*;
use crate::oracle::patterns::StaticPattern;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// A single keyword ("first strike", "hexproof") → its kind.
fn one_keyword(s: &str, ctx: &CompileContext) -> Option<crate::keywords::KeywordKind> {
    let abilities = crate::oracle::keywords::parse_keyword_line(end(s), ctx)?;
    match abilities.as_slice() {
        [a] => match &a.kind {
            AbilityKind::Keyword(k) => Some(k.kind),
            _ => None,
        },
        _ => None,
    }
}

/// "[objects] lose [kw] and can't have or gain [kw]" / "[objects] can't have or gain [kw]".
fn cant_have_or_gain(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if let Some((subject, rest)) = l.split_once(" lose ") {
        let (lost, kept) = rest.split_once(" and can't have or gain ")?;
        return build(subject, Some(lost), kept, text, ctx);
    }
    let (subject, kept) = l.split_once(" can't have or gain ")?;
    build(subject, None, kept, text, ctx)
}

fn build(
    subject: &str,
    lost: Option<&str>,
    cant: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let (affected, _, tail) = parse_object_phrase(subject)?;
    if !end(tail).is_empty() {
        return None;
    }
    let k = one_keyword(cant, ctx)?;
    let mut mods = Vec::new();
    if let Some(lost) = lost {
        if one_keyword(lost, ctx)? != k {
            return None;
        }
        mods.push(Modification::RemoveKeyword(k));
    }
    mods.push(Modification::CantHaveKeyword(k));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods,
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r113 can't have or gain a keyword", priority: 100, parse: cant_have_or_gain } }

/// "~ can't have counters put on it" / "counters can't be put on ~": modeled as an effect
/// that stops counters from being put on it. It functions as the object enters the
/// battlefield as well as while it's on the battlefield (CR 113.6i).
fn no_counters(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !matches!(
        end(l),
        "~ can't have counters put on it" | "counters can't be put on ~"
    ) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::PutCounters {
                    on_objects: Some(Filter::Source),
                    on_players: None,
                    kind: None,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r113 can't have counters put on it", priority: 100, parse: no_counters } }

/// Deck-construction abilities (CR 113.6n): "A deck can have any number of cards named ~",
/// "A deck can have up to seven cards named ~". They function before the game begins
/// (see `crate::deck`).
fn deck_construction(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("a deck can have ")?;
    let name = if r.strip_prefix("any number of cards named ~").is_some_and(str::is_empty) {
        crate::deck::ANY_NUMBER.to_string()
    } else {
        let r = r.strip_prefix("up to ")?;
        let (n, rest) = parse_number(r)?;
        let Value::Const(n) = n else {
            return None;
        };
        if rest.trim() != "cards named ~" {
            return None;
        }
        format!("{}{n}", crate::deck::UP_TO)
    };
    let mut s = StaticAbility::new(StaticEffect::Custom(name.into()));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "r113 deck construction", priority: 100, parse: deck_construction } }
