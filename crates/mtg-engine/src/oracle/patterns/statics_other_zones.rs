//! Static abilities whose effect also covers spells and cards outside the battlefield
//! (a static ability's effect applies to whatever its text indicates, wherever it is,
//! CR 611.3a):
//!
//! - "[static ability]. The same is true for creature spells you control and creature
//!   cards you own that aren't on the battlefield." (Biotransference, Maskwood Nexus,
//!   Roshan, Arcane Adaptation, Encroaching Mycosynth, Celestial Dawn): the first
//!   sentence's modifications apply to the listed objects too.
//! - "Lands you control and land cards in your library are basic." (Rootpath Purifier):
//!   a supertype added to objects in two zones.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// The zones of "cards ... that aren't on the battlefield": every zone a card can be in
/// but the battlefield, the stack included (a spell is a card on the stack, CR 112.1; a
/// spell you own that another player controls is still a card you own that isn't on the
/// battlefield).
pub(crate) const NOT_ON_BATTLEFIELD: [ZoneKind; 6] = [
    ZoneKind::Hand,
    ZoneKind::Library,
    ZoneKind::Graveyard,
    ZoneKind::Exile,
    ZoneKind::Command,
    ZoneKind::Stack,
];

/// The alternatives of "creature spells you control and creature cards you own that
/// aren't on the battlefield", each with its zone.
pub(crate) fn other_zone_objects(s: &str) -> Option<Vec<Filter>> {
    let mut out = Vec::new();
    for part in s.split(" and ") {
        let part = part.trim();
        let (base, off) = match part.strip_suffix(" that aren't on the battlefield") {
            Some(r) => (r, true),
            None => (part, false),
        };
        // "land cards in your library", "cards in your hand".
        let own_zone = [
            (" in your library", ZoneKind::Library),
            (" in your hand", ZoneKind::Hand),
            (" in your graveyard", ZoneKind::Graveyard),
        ]
        .into_iter()
        .find_map(|(s, z)| base.strip_suffix(s).map(|r| (r, z)));
        if let Some((r, z)) = own_zone {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).trim().is_empty() {
                return None;
            }
            out.push(Filter::and(vec![
                f,
                Filter::InZone(z),
                Filter::OwnedBy(PlayerRel::You),
            ]));
            continue;
        }
        let (f, _, tail) = parse_object_phrase(base)?;
        if !end(tail).trim().is_empty() {
            return None;
        }
        if off {
            for z in NOT_ON_BATTLEFIELD {
                out.push(Filter::and(vec![f.clone(), Filter::InZone(z)]));
            }
        } else {
            // Spells (the stack) or cards in a named zone ("land cards in your library").
            match f.zone() {
                Some(ZoneKind::Battlefield) | None => return None,
                Some(_) => out.push(f),
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// The modifications that can apply to objects in any zone: type, supertype, subtype and
/// color changes (the objects' other characteristics stay as printed).
fn zone_neutral(m: &Modification) -> bool {
    matches!(
        m,
        Modification::AddTypes(_)
            | Modification::AddSubtypes(_)
            | Modification::AddSupertypes(_)
            | Modification::SetColors(_)
            | Modification::AddColors(_)
            | Modification::AllCreatureTypes
            | Modification::AddChosenType
            | Modification::AddChosenColor
            | Modification::RemoveAllCreatureTypes
    )
}

/// The affected objects on the battlefield, with that zone made explicit (an alternative
/// of a filter over several zones needs its own zone, see `Game::static_affected`).
fn on_battlefield(f: Filter) -> Filter {
    match f.zone() {
        Some(_) => f,
        None => Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
    }
}

/// "[static ability]. The same is true for [spells and cards elsewhere]."
fn same_is_true_elsewhere(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let i = l.find(". the same is true for ")?;
    let first = &block[..i + 1];
    let same = &l[i + ". the same is true for ".len()..];
    let alts = other_zone_objects(same)?;
    let abilities = crate::oracle::statics::parse_static(first, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let mut s = s.clone();
    if s.condition.is_some() || s.zone != FunctionZone::Battlefield {
        return None;
    }
    let StaticEffect::Continuous { affected, mods } = &mut s.effect else {
        return None;
    };
    if !mods.iter().all(zone_neutral) {
        return None;
    }
    let mut all = match std::mem::replace(affected, Filter::Any) {
        Filter::Or(v) if v.iter().all(|f| f.zone().is_some()) => v,
        f => vec![on_battlefield(f)],
    };
    all.extend(alts);
    *affected = Filter::Or(all);
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block)])
}

inventory::submit! { AbilityPattern { name: "statics other zones: [static]. the same is true for [spells and cards]", priority: 48, parse: same_is_true_elsewhere } }

/// "Lands you control and land cards in your library are basic." (Rootpath Purifier):
/// the objects of each listed kind, in their zones, have the supertype.
fn supertype_in_zones(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let (subj, pred) = end(l).split_once(" are ")?;
    let sup = Supertype::from_word(pred)?;
    let (first, rest) = subj.split_once(" and ")?;
    let (f, _, tail) = parse_object_phrase(first)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    let mut alts = vec![on_battlefield(f)];
    alts.extend(other_zone_objects(rest)?);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Or(alts),
            mods: vec![Modification::AddSupertypes(vec![sup])],
        })),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "statics other zones: [objects] and [cards in a zone] are [supertype]", priority: 45, parse: supertype_in_zones } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rootpath_subject() {
        let (f, _, tail) = parse_object_phrase("lands you control").unwrap();
        assert_eq!(tail.trim(), "", "{f:?}");
        let v = other_zone_objects("land cards in your library");
        assert!(v.is_some());
    }
}
