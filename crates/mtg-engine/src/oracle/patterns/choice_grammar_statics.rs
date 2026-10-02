//! Statics that give objects the type or color chosen as the permanent entered (CR
//! 607.2d), including objects outside the battlefield (CR 611.3, a static ability's
//! effect applies to the objects it describes wherever they are):
//!
//! - "Creatures you control are the chosen type. The same is true for creature spells you
//!   control and creature cards you own that aren't on the battlefield." (Conspiracy: the
//!   chosen creature type replaces their other creature types, CR 205.1a);
//! - "Slivers you control and nontoken creatures you control are the chosen type in
//!   addition to their other creature types. The same is true for ..." (Rukarumel);
//! - "Each creature card in your graveyard has the chosen creature type in addition to
//!   its other types." (Ashes of the Fallen);
//! - "All cards that aren't on the battlefield, spells, and permanents are the chosen color
//!   in addition to their other colors." (Painter's Servant).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The zones of cards that aren't on the battlefield (or the stack).
const OFF_BATTLEFIELD: [ZoneKind; 5] = [
    ZoneKind::Hand,
    ZoneKind::Library,
    ZoneKind::Graveyard,
    ZoneKind::Exile,
    ZoneKind::Command,
];

/// One alternative per zone: `f` in each of `zones`.
fn in_zones(f: &Filter, zones: &[ZoneKind]) -> Vec<Filter> {
    zones
        .iter()
        .map(|z| Filter::and(vec![f.clone(), Filter::InZone(*z)]))
        .collect()
}

/// The objects a subject names, each alternative with its zone: "creatures you control",
/// "slivers you control and nontoken creatures you control", "each creature card in your
/// graveyard", "all cards that aren't on the battlefield, spells, and permanents".
fn subject(s: &str) -> Option<Vec<Filter>> {
    let s = s.trim();
    if s == "all cards that aren't on the battlefield, spells, and permanents" {
        let mut v = in_zones(&Filter::Card, &OFF_BATTLEFIELD);
        v.push(Filter::InZone(ZoneKind::Stack));
        v.push(Filter::InZone(ZoneKind::Battlefield));
        return Some(v);
    }
    let s = s.strip_prefix("each ").unwrap_or(s);
    let mut out = Vec::new();
    for part in s.split(" and ") {
        let (f, _, tail) = parse_object_phrase(part.trim())?;
        if !end(tail).is_empty() {
            return None;
        }
        match f.zone() {
            Some(_) => out.push(f),
            None => {
                if !mentions(&f, &|x| matches!(x, Filter::ControlledBy(PlayerRel::You))) {
                    return None;
                }
                out.push(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]));
            }
        }
    }
    Some(out)
}

fn mentions(f: &Filter, p: &dyn Fn(&Filter) -> bool) -> bool {
    p(f) || match f {
        Filter::And(v) | Filter::Or(v) => v.iter().any(|x| mentions(x, p)),
        Filter::Not(x) => mentions(x, p),
        _ => false,
    }
}

fn chosen_quality_static(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let raw = crate::oracle::raw_text().to_lowercase();
    // "The same is true for creature spells you control and creature cards you own that
    // aren't on the battlefield."
    let (first, same) = match l.split_once(". the same is true for ") {
        Some((a, b)) => (a, Some(b)),
        None => (l, None),
    };
    let (subj, pred) = first
        .split_once(" are the chosen ")
        .or_else(|| first.split_once(" has the chosen "))?;
    let (mods, choice) = match pred {
        "type" => (
            vec![Modification::RemoveAllCreatureTypes, Modification::AddChosenType],
            "choose a creature type",
        ),
        "type in addition to their other creature types"
        | "creature type in addition to its other types"
        | "creature type in addition to their other types" => {
            (vec![Modification::AddChosenType], "choose a creature type")
        }
        "color in addition to their other colors" => {
            (vec![Modification::AddChosenColor], "choose a color")
        }
        _ => return None,
    };
    // The choice is made as the permanent enters (CR 607.2d).
    if !raw
        .lines()
        .any(|x| x.starts_with("as ") && x.contains(" enters, ") && x.contains(choice))
    {
        return None;
    }
    let mut alts = subject(subj)?;
    if let Some(same) = same {
        if same != "creature spells you control and creature cards you own that aren't on the battlefield" {
            return None;
        }
        alts.push(Filter::and(vec![
            Filter::creature(),
            Filter::InZone(ZoneKind::Stack),
            Filter::ControlledBy(PlayerRel::You),
        ]));
        let cards = Filter::and(vec![
            Filter::creature(),
            Filter::Card,
            Filter::OwnedBy(PlayerRel::You),
        ]);
        alts.extend(in_zones(&cards, &OFF_BATTLEFIELD));
    }
    let affected = if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods,
        })),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "choice grammar: [objects, in any zone] are the chosen type/color", priority: 49, parse: chosen_quality_static } }
