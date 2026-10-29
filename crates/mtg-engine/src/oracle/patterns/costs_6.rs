//! Cost grammar VI (cards5-costs-sets): costs over sets of objects and what their
//! effects say about those sets.
//!
//! - "with fifteen or more black mana symbols among their mana costs" (Baron Helmut
//!   Zemo's boast cost): a count of one color's mana symbols over the objects chosen
//!   together (`TargetGroup::ManaSymbolsAmong`; hybrid and Phyrexian symbols of the color
//!   count, CR 107.4a, 107.4e).
//! - "Exile one or more other artifacts you control with total mana value X" (Fabrication
//!   Foundry): as many objects as the player chooses as the cost is paid, together having
//!   the relationship (`CostPart::AnyNumberOf`, see `cost_sets.rs`); with "total mana
//!   value X" the total is exactly the X announced for the activation (CR 107.3a), which
//!   the target filter reads too ("mana value X or less").

use super::{AbilityPattern, CostPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, find_group_phrase, parse_number, split_word};
use crate::types::Color;

/// "exile one or more [objects] [relationship]", "sacrifice any number of [objects]
/// [relationship]": the count chosen as the cost is paid. Only with a relationship the
/// objects must have together (without one, `cost_parts::amount_as_x` reads the count as
/// the ability's X).
fn any_number_with_relationship(p: &str) -> Option<CostPart> {
    let p = end(p);
    let (verb, rest) = split_word(p);
    if !matches!(verb, "exile" | "sacrifice") {
        return None;
    }
    let (min, rest) = if let Some(r) = rest.strip_prefix("one or more ") {
        (1, r)
    } else {
        (0, rest.strip_prefix("any number of ")?)
    };
    let (i, j, grp) = find_group_phrase(rest)?;
    if j != rest.len() {
        return None;
    }
    let inner = crate::oracle::costs::parse_cost_part(&format!("{verb} x {}", &rest[..i]))?;
    let with = |f: Filter| Filter::and(vec![f, Filter::Together(grp.clone())]);
    let part = match inner {
        CostPart::Exile {
            filter,
            zone,
            count: Value::X,
        } if !crate::casting::filter_mentions_x(&filter) => CostPart::Exile {
            filter: with(filter),
            zone,
            count: Value::c(1),
        },
        CostPart::Sacrifice {
            filter,
            count: Value::X,
        } if !crate::casting::filter_mentions_x(&filter) => CostPart::Sacrifice {
            filter: with(filter),
            count: Value::c(1),
        },
        _ => return None,
    };
    Some(CostPart::AnyNumberOf {
        part: Box::new(part),
        min,
    })
}

inventory::submit! { CostPattern { name: "costs 6: exile one or more [objects] with total mana value X", priority: 45, parse: any_number_with_relationship } }

/// "exile a creature card from your graveyard and pay its mana cost": one card exiled,
/// then its mana cost paid (`CostPart::AndPayItsManaCost`).
fn exile_and_pay_its_mana_cost(p: &str) -> Option<CostPart> {
    let head = end(p).strip_suffix(" and pay its mana cost")?;
    if !head.starts_with("exile ") {
        return None;
    }
    match crate::oracle::costs::parse_cost_part(head)? {
        part @ CostPart::Exile {
            count: Value::Const(1),
            zone: ZoneKind::Graveyard | ZoneKind::Hand | ZoneKind::Exile,
            ..
        } => Some(CostPart::AndPayItsManaCost(Box::new(part))),
        _ => None,
    }
}

inventory::submit! { CostPattern { name: "costs 6: exile a [card] and pay its mana cost", priority: 45, parse: exile_and_pay_its_mana_cost } }

/// "Add one mana of any type that land could produce" after a cost "Untap a tapped land
/// [an opponent controls]" (Benthic Explorers): the land the cost untapped (CR 106.7; its
/// rulings: replacement effects on that land's mana abilities count, restrictions and
/// riders don't, and neither do the costs of those abilities).
fn mana_untapped_land_could_produce(
    l: &str,
    _b: &mut crate::oracle::effects::Builder,
) -> Option<Effect> {
    if end(l) != "add one mana of any type that land could produce" {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let cost = raw.split(": add one mana of any type that land could produce").next()?;
    let cost = cost.rsplit('\n').next()?;
    let untaps_a_land = cost.split(", ").any(|p| {
        p.strip_prefix("untap a tapped land ")
            .is_some_and(|r| matches!(r, "you control" | "an opponent controls"))
    });
    if !untaps_a_land || cost.contains("sacrifice") {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::CouldProduce(Filter::In(Box::new(Sel::Var(
            crate::cost_sets::COST_UNTAPPED,
        )))),
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "costs 6: add one mana of any type that land could produce (the untapped land)", priority: 10, parse: mana_untapped_land_could_produce } }

/// "{T}, Sacrifice ~ and any number of creatures you control: ~ deals X damage to any
/// target, where X is the total power of the creatures sacrificed this way, then exile ~
/// and those creature cards." (Sword of the Ages): the other creatures are the ones besides
/// ~ (it's sacrificed by the first part of the cost, so "other" is implied); "those
/// creature cards" are the sacrificed creatures as the new objects they became in the
/// graveyard (CR 400.7, 400.7j), and "~" the source's (the power counted is each
/// creature's last known power, the ruling).
fn sacrifice_self_and_any_number_then_exile(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let (cost, effect) = crate::oracle::split_cost(block.trim())?;
    let lower = cost.to_lowercase();
    const SAC: &str = "sacrifice ~ and any number of ";
    let i = lower.find(SAC)?;
    let after = &cost[i + SAC.len()..];
    if after.to_lowercase().starts_with("other ") {
        return None;
    }
    let cost = format!("{}{SAC}other {after}", &cost[..i]);
    // "then exile ~ and those [type] cards."
    let (effect, exile_those) = match effect.trim_end().strip_suffix('.').and_then(|e| {
        let (head, t) = e.rsplit_once(", then exile ~ and those ")?;
        let t = t.strip_suffix(" cards")?;
        Some((head, crate::types::CardType::from_word(t)?))
    }) {
        Some((head, t)) => (format!("{head}, then exile ~."), Some(t)),
        None => (effect.to_string(), None),
    };
    // The sacrificed objects must be of that type ("any number of creatures ...: ... those
    // creature cards").
    if let Some(t) = exile_those {
        let noun = after.split(' ').next()?;
        if crate::types::CardType::from_word(noun) != Some(t) {
            return None;
        }
    }
    let mut abilities = crate::oracle::parse_ability(&format!("{cost}: {effect}"), ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Activated(act) = &mut a.kind else {
        return None;
    };
    if let Some(t) = exile_those {
        let Effect::Seq(v) = &mut act.body.effect else {
            return None;
        };
        let Some(Effect::Exile { what, .. }) = v.last_mut() else {
            return None;
        };
        if !matches!(what, Sel::This) {
            return None;
        }
        // What the cost sacrificed is ~ and the creatures: the new objects they became in
        // the graveyard (tokens cease to exist, CR 111.7).
        let _ = t;
        *what = Sel::Matching(
            Box::new(Sel::Var(crate::zones::COST_MOVED)),
            Filter::InZone(ZoneKind::Graveyard),
        );
    }
    a.text = block.trim().to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "costs 6: sacrifice ~ and any number of [objects] (other implied), then exile ~ and those cards", priority: 59, parse: sacrifice_self_and_any_number_then_exile } }

/// "with N or more [color] mana symbols among their mana costs" at the start of `t`: the
/// rest and the relationship.
pub fn mana_symbols_among(t: &str) -> Option<(&str, TargetGroup)> {
    let r = t.strip_prefix("with ")?;
    let (n, r) = parse_number(r)?;
    let r = r.trim_start().strip_prefix("or more ")?;
    let (color, r) = r.split_once(' ')?;
    let c = Color::from_word(color)?;
    let r = r.strip_prefix("mana symbols among their mana costs")?;
    Some((r, TargetGroup::ManaSymbolsAmong(c, Box::new(n))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::costs::parse_cost;

    #[test]
    fn mana_symbols_among_a_cost_set() {
        let (c, _) = parse_cost(
            "Exile x black cards from your graveyard with fifteen or more black mana symbols among their mana costs",
        )
        .expect("cost");
        let [CostPart::Exile { filter, .. }] = c.parts.as_slice() else {
            panic!("{c:?}")
        };
        assert!(matches!(
            crate::relational::groups_of(filter).as_slice(),
            [TargetGroup::ManaSymbolsAmong(Color::Black, _)]
        ));
        assert!(mana_symbols_among("with two or more red mana symbols among their mana costs").is_some());
        assert!(mana_symbols_among("with two or more mana symbols among their mana costs").is_none());
    }

    #[test]
    fn one_or_more_with_a_total() {
        for (text, min) in [
            ("{2}{W}, {T}, Exile one or more other artifacts you control with total mana value X", 1),
            ("Sacrifice any number of creatures with total power X", 0),
        ] {
            let (c, _) = parse_cost(text).unwrap_or_else(|| panic!("{text}"));
            let Some(CostPart::AnyNumberOf { part, min: m }) = c.parts.last() else {
                panic!("{c:?}")
            };
            assert_eq!(*m, min);
            let (CostPart::Exile { filter, .. } | CostPart::Sacrifice { filter, .. }) = &**part
            else {
                panic!("{part:?}")
            };
            assert!(matches!(
                crate::relational::groups_of(filter).as_slice(),
                [TargetGroup::TotalExactly(_, v)] if matches!(**v, Value::X)
            ));
        }
        let (c, _) = parse_cost("Exile a creature card from your graveyard and pay its mana cost").expect("cost");
        assert!(matches!(c.parts.as_slice(), [CostPart::AndPayItsManaCost(p)] if matches!(**p, CostPart::Exile { zone: ZoneKind::Graveyard, .. })));
        assert!(parse_cost("Sacrifice a creature and pay its mana cost").is_none());
        // No relationship: not this pattern.
        assert!(any_number_with_relationship("exile one or more cards from your graveyard").is_none());
    }
}

#[cfg(test)]
mod compile_tests {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    pub(crate) fn unsupported(text: &str, types: &str) -> Vec<String> {
        let tl = TypeLine::parse(types);
        let ctx = CompileContext {
            card_name: "Probe",
            full_name: "Probe",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: Some("1"),
            toughness: Some("1"),
        };
        compile(text, &ctx).unsupported
    }

    #[test]
    fn cost_set_texts_compile() {
        for (t, ty) in [
            ("{2}{W}, {T}, Exile one or more other artifacts you control with total mana value X: Return target artifact card with mana value X or less from your graveyard to the battlefield. Activate only as a sorcery.", "Artifact"),
            ("{T}, Untap a tapped land an opponent controls: Add one mana of any type that land could produce.", "Creature — Merfolk"),
            ("Exile a creature card from your graveyard and pay its mana cost: Create a token that's a copy of that card. Activate only as a sorcery.", "Enchantment"),
            ("{T}, Sacrifice Probe and any number of creatures you control: Probe deals X damage to any target, where X is the total power of the creatures sacrificed this way, then exile Probe and those creature cards.", "Artifact"),
            ("As an additional cost to cast this spell, choose a creature you control or a warped creature card you own in exile.\nProbe deals damage equal to the power of the chosen creature or card to target creature.", "Instant"),
            ("As an additional cost to cast this spell, you may choose a creature type and behold two creatures of that type.\nSearch your library for a creature card with mana value X or less, reveal it, put it into your hand, then shuffle. If this spell's additional cost was paid and the revealed card is the chosen type, put that card onto the battlefield instead of putting it into your hand.", "Sorcery"),
            ("As an additional cost to cast this spell, you may pay {1}{R} and/or {1}{G} any number of times.\nDestroy target artifact. For each additional {1}{R} you paid, destroy another target artifact. For each additional {1}{G} you paid, destroy another target artifact, and you gain 1 life.", "Sorcery"),
        ] {
            assert_eq!(unsupported(t, ty), Vec::<String>::new(), "{t}");
        }
    }
}
