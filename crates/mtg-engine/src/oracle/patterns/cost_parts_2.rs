//! More cost parts (CR 118, 602.1a), read on top of `oracle/costs.rs` and `cost_parts.rs`:
//!
//! * a relationship the objects of a cost must have together (`Filter::Together`, chosen
//!   as in `target_groups.rs`): "Discard two nonland cards with the same name", "Tap two
//!   untapped creatures you control that share a creature type", "Reveal two cards from
//!   your hand that share a color", "Exile any number of historic cards from your
//!   graveyard with total mana value 8 or greater";
//! * graveyard order (CR 404.2): "Exile the top creature card of your graveyard";
//! * revealing: "Reveal your hand", "Reveal ~ from your hand" (an ability that functions
//!   in its owner's hand, CR 113.6);
//! * counters from one permanent: "Remove X counters from an artifact or creature you
//!   control" (all from the same permanent, of any kinds);
//! * a face-down permanent: "Exile a face-down permanent you control face up" (a card in
//!   exile is face up, CR 406.3; a face-down permanent leaving the battlefield is revealed,
//!   CR 708.9).

use super::CostPattern;
use crate::ability::*;
use crate::oracle::costs::{counter_kind, parse_cost_part};
use crate::oracle::phrases::*;

fn with_group(f: &Filter, grp: &TargetGroup) -> Filter {
    Filter::and(vec![f.clone(), Filter::Together(grp.clone())])
}

/// "[cost part] with the same name", "[cost part] that share a creature type": the part,
/// with the objects it chooses required to have the relationship.
fn grouped(p: &str) -> Option<CostPart> {
    let (i, j, grp) = find_group_phrase(p)?;
    if j != p.len() {
        return None;
    }
    let mut part = parse_cost_part(&p[..i])?;
    match &mut part {
        CostPart::Sacrifice { filter, count }
        | CostPart::Discard { filter, count, .. }
        | CostPart::Exile { filter, count, .. }
        | CostPart::TapUntapped { filter, count }
        | CostPart::RevealFromHand { filter, count } => {
            // A relationship among several objects.
            if count.as_const() == Some(1) || !crate::relational::groups_of(filter).is_empty() {
                return None;
            }
            *filter = with_group(filter, &grp);
        }
        _ => return None,
    }
    Some(part)
}

inventory::submit! { CostPattern { name: "cost parts: objects chosen together (same name, share a type or color, total)", priority: 40, parse: grouped } }

/// "Exile the top creature card of your graveyard" (CR 404.2).
fn exile_top_of_graveyard(p: &str) -> Option<CostPart> {
    let r = strip(p, "exile")?;
    let (filter, rest) = super::r404_graveyard_order::top_card_phrase(r)?;
    if !end(rest).is_empty() || format!("{filter:?}").contains("Defending") {
        return None;
    }
    Some(CostPart::Exile {
        filter,
        zone: ZoneKind::Graveyard,
        count: Value::c(1),
    })
}

inventory::submit! { CostPattern { name: "cost parts: exile the top [type] card of your graveyard", priority: 50, parse: exile_top_of_graveyard } }

/// "Reveal your hand", "Reveal ~ from your hand".
fn reveal(p: &str) -> Option<CostPart> {
    match end(p) {
        "reveal your hand" => Some(CostPart::Effect(Box::new(Effect::RevealHand {
            who: PlayerRef::You,
        }))),
        "reveal ~ from your hand" | "reveal ~" => Some(CostPart::RevealSelf),
        _ => None,
    }
}

inventory::submit! { CostPattern { name: "cost parts: reveal your hand, reveal ~ from your hand", priority: 50, parse: reveal } }

/// "Remove X counters from an artifact or creature you control", "Remove a +1/+1 counter
/// from a creature you control" with a count above one: from one permanent.
fn remove_from_one(p: &str) -> Option<CostPart> {
    let r = strip(p, "remove")?;
    let (n, r) = parse_number(r)?;
    if n.as_const() == Some(1) {
        return None;
    }
    let (kind, r) = match strip(r, "counters") {
        Some(r) => (None, r),
        None => {
            let (k, r) = counter_kind(r)?;
            (Some(k), strip(r, "counters")?)
        }
    };
    let r = strip(r, "from")?;
    // One permanent: "an artifact or creature you control", "a creature you control".
    let r = strip(r, "an").or_else(|| strip(r, "a"))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() || !format!("{f:?}").contains("ControlledBy(You)") {
        return None;
    }
    Some(CostPart::RemoveCountersFromOne {
        kind,
        filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
        count: n,
    })
}

inventory::submit! { CostPattern { name: "cost parts: remove N counters from one permanent you control", priority: 50, parse: remove_from_one } }

/// "Pay {1} for each +1/+1 counter on ~": the mana repeated once for each (CR 118, 702.24a
/// for the same repetition in cumulative upkeep), counted as the cost is paid.
fn pay_for_each(p: &str) -> Option<CostPart> {
    let r = strip(p, "pay")?;
    let (sym, each) = r.split_once(" for each ")?;
    if !sym.starts_with('{') || !sym.ends_with('}') || sym.contains("{e}") {
        return None;
    }
    let m = crate::mana::ManaCost::parse(&sym.to_uppercase())?;
    let times = super::statics::parse_for_each(each, Some(&Sel::This))?;
    Some(CostPart::Repeated {
        cost: Box::new(Cost::mana(m)),
        times,
    })
}

inventory::submit! { CostPattern { name: "cost parts: pay {N} for each [thing]", priority: 50, parse: pay_for_each } }

/// "Exile a face-down permanent you control face up".
fn exile_face_down(p: &str) -> Option<CostPart> {
    let r = strip(p, "exile")?.strip_suffix(" face up")?;
    let (n, r) = parse_number(r)?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty()
        || !format!("{f:?}").contains("FaceDown")
        || !format!("{f:?}").contains("ControlledBy(You)")
    {
        return None;
    }
    Some(CostPart::Exile {
        filter: f,
        zone: ZoneKind::Battlefield,
        count: n,
    })
}

inventory::submit! { CostPattern { name: "cost parts: exile a face-down permanent face up", priority: 50, parse: exile_face_down } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::costs::parse_cost;

    fn parts(s: &str) -> Vec<CostPart> {
        parse_cost(s).unwrap_or_else(|| panic!("no cost: {s}")).0.parts
    }

    fn group(p: &CostPart) -> Vec<TargetGroup> {
        match p {
            CostPart::Sacrifice { filter, .. }
            | CostPart::Discard { filter, .. }
            | CostPart::Exile { filter, .. }
            | CostPart::TapUntapped { filter, .. }
            | CostPart::RevealFromHand { filter, .. } => crate::relational::groups_of(filter),
            _ => vec![],
        }
    }

    #[test]
    fn grouped_cost_parts() {
        let p = parts("Discard two nonland cards with the same name");
        assert!(matches!(group(&p[0]).as_slice(), [TargetGroup::SameName]));
        let p = parts("Tap two untapped creatures you control that share a creature type");
        assert!(matches!(p[0], CostPart::TapUntapped { .. }));
        assert!(matches!(group(&p[0]).as_slice(), [TargetGroup::ShareCreatureType]));
        let p = parts("{1}, {T}, Reveal two cards from your hand that share a color");
        assert!(matches!(group(&p[1]).as_slice(), [TargetGroup::ShareColor]));
        let p = parts("Exile x historic cards from your graveyard with total mana value 8 or greater");
        assert!(matches!(group(&p[0]).as_slice(), [TargetGroup::TotalAtLeast(..)]));
    }

    #[test]
    fn other_parts() {
        assert!(matches!(
            parts("Exile the top creature card of your graveyard").as_slice(),
            [CostPart::Exile { zone: ZoneKind::Graveyard, .. }]
        ));
        assert!(matches!(parts("Reveal your hand").as_slice(), [CostPart::Effect(_)]));
        assert!(matches!(
            parts("{B}, Reveal ~ from your hand").as_slice(),
            [CostPart::RevealSelf]
        ));
        assert!(matches!(
            parts("{2}, {T}, Remove X counters from an artifact or creature you control").as_slice(),
            [CostPart::Tap, CostPart::RemoveCountersFromOne { kind: None, .. }]
        ));
        assert!(matches!(
            parts("Exile a face-down permanent you control face up").as_slice(),
            [CostPart::Exile { zone: ZoneKind::Battlefield, .. }]
        ));
    }
}
