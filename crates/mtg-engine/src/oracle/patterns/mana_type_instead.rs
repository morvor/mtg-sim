//! Replacement effects that change the type of the mana a land produces, keeping the
//! amount (CR 106.12b):
//!
//! - "If a land is tapped for mana, it produces {B} instead of any other type." (Infernal
//!   Darkness), "... it produces colorless mana instead of any other type." (Ritual of
//!   Subdual)
//! - "If tapped for mana, Plains produce {R}, Islands produce {G}, Swamps produce {W},
//!   Mountains produce {U}, and Forests produce {B} instead of any other type." (Naked
//!   Singularity): one replacement effect for each land type. A land with two of those
//!   types (or a land type listed twice after a text change) is affected by two of them,
//!   and the player tapping it chooses the order they apply in (CR 616.1), so the last
//!   one decides the type of all the mana it produces.

use super::StaticPattern;
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// "{b}", "{c}", "colorless mana".
fn mana_type(s: &str) -> Option<ManaType> {
    let s = s.trim();
    if s == "colorless mana" {
        return Some(ManaType::C);
    }
    let inner = s.strip_prefix('{')?.strip_suffix('}')?;
    let mut chars = inner.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    ManaType::from_letter(c)
}

fn type_instead(filter: Filter, t: ManaType, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::ProduceMana(filter),
                action: ReplacementAction::ManaTypeInstead(t),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )
}

/// "If a land is tapped for mana, it produces {B} instead of any other type."
fn it_produces_type_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cond, action) = end(l).strip_prefix("if ")?.split_once(", it produces ")?;
    let t = mana_type(action.strip_suffix(" instead of any other type")?)?;
    let (subject, tail) = cond.split_once(" is tapped ")?;
    if tail != "for mana" {
        return None;
    }
    let s = subject
        .strip_prefix("a ")
        .or_else(|| subject.strip_prefix("an "))
        .unwrap_or(subject);
    let (f, _, rest) = parse_object_phrase(s)?;
    if !end(rest).is_empty() {
        return None;
    }
    Some(vec![type_instead(f, t, text)])
}

inventory::submit! { StaticPattern { name: "r106 it produces [type] instead of any other type", priority: 60, parse: it_produces_type_instead } }

/// "Plains", "Islands", ... (lowercase) as land subtypes.
fn basic_land_type(plural: &str) -> Option<&'static str> {
    Some(match plural {
        "plains" => "Plains",
        "islands" => "Island",
        "swamps" => "Swamp",
        "mountains" => "Mountain",
        "forests" => "Forest",
        _ => return None,
    })
}

/// "If tapped for mana, Plains produce {R}, Islands produce {G}, ..., and Forests produce
/// {B} instead of any other type."
fn land_types_produce_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let list = end(l)
        .strip_prefix("if tapped for mana, ")?
        .strip_suffix(" instead of any other type")?;
    let mut out = Vec::new();
    for item in list.split(", ") {
        let item = item.strip_prefix("and ").unwrap_or(item);
        let (lands, t) = item.split_once(" produce ")?;
        let filter = Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Subtype(Subtype::from(basic_land_type(lands)?)),
        ]);
        out.push(type_instead(filter, mana_type(t)?, text));
    }
    (!out.is_empty()).then_some(out)
}

inventory::submit! { StaticPattern { name: "r106 land types produce [type] instead of any other type", priority: 60, parse: land_types_produce_instead } }
