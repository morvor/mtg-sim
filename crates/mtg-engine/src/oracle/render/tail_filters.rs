//! More object qualities implemented in code (`Filter::Custom`), put into words from what
//! each one checks (see [`super::custom_filters::custom_rel`]).

use super::custom_filters::CustomQuality;
use crate::ability::*;

/// The wording of a custom quality, if this module knows it.
pub(crate) fn custom_rel_more(name: &str) -> Option<CustomQuality> {
    use CustomQuality::*;
    // "a land that could produce {C}" (CR 106.7): one of its mana abilities could
    // produce mana of that type.
    if let Some(t) = name.strip_prefix(crate::kw::mana_grammar::COULD_PRODUCE) {
        let letter = t.chars().next()?;
        crate::mana::ManaType::from_letter(letter)?;
        return Some(Rel(format!(
            "that could produce {{{}}}",
            letter.to_ascii_uppercase()
        )));
    }
    Some(match name {
        // "each creature you control with a mana ability" (CR 605.1a).
        crate::kw::activated_ability_kind::HAS_MANA_ABILITY => Rel("with a mana ability".into()),
        // "protection from Ring-bearers": each player's Ring-bearer (CR 701.54a).
        crate::kw::game_rules::ANY_RING_BEARER => Kind("Ring-bearer"),
        // "When you cast an Equipment spell this way": cast with the permission the
        // ability or its source gave.
        crate::permissions::PLAYED_THIS_WAY => Rel("this way".into()),
        _ => return None,
    })
}

impl super::Renderer<'_> {
    /// Conditions implemented in code (`Condition::Custom`) this module words.
    pub(crate) fn custom_condition_tail(&mut self, name: &str) -> Option<String> {
        // "you've rolled three or more dice this turn" (the controller's rolls, CR 706).
        if let Some(n) = name.strip_prefix(crate::dice::DICE_THIS_TURN) {
            let n: i32 = n.strip_prefix(':')?.parse().ok()?;
            return Some(format!(
                "you've rolled {} or more dice this turn",
                super::number_word(n)
            ));
        }
        Some(match name {
            // CR 701.52a.
            crate::kw::visit::VISITED_ANY_THIS_TURN => {
                "you've visited an Attraction this turn".into()
            }
            // CR 500.7.
            crate::kw::game_rules::EXTRA_TURN => "it's an extra turn".into(),
            _ => return None,
        })
    }
}

impl super::Renderer<'_> {
    /// Effects implemented in code (`Effect::Custom`) this module words, as the player
    /// performing them ("you").
    pub(crate) fn custom_effect_tail(&mut self, name: &str) -> Option<String> {
        Some(match name {
            // Each player in turn may pay any amount of mana (any amount can be 0).
            crate::kw::join_forces::PAY_ANY_AMOUNT => "you may pay any amount of mana".into(),
            _ => return None,
        })
    }
}

impl super::Renderer<'_> {
    /// "the greatest mana value of a commander you own on the battlefield or in the
    /// command zone": the greater of the greatest among the same objects in two zones.
    pub(crate) fn greatest_across_zones(&mut self, a: &Value, b: &Value) -> Option<String> {
        let (Value::Aggregate(AggOp::Max, sa, xa), Value::Aggregate(AggOp::Max, sb, xb)) = (a, b)
        else {
            return None;
        };
        if format!("{sa:?}") != format!("{sb:?}") {
            return None;
        }
        let (Sel::All(fa), Sel::All(fb)) = (xa.as_ref(), xb.as_ref()) else {
            return None;
        };
        let split = |f: &Filter| -> Option<(Vec<Filter>, ZoneKind)> {
            let Filter::And(v) = f else { return None };
            let z = v.iter().find_map(|x| match x {
                Filter::InZone(z) => Some(*z),
                _ => None,
            })?;
            Some((
                v.iter()
                    .filter(|x| !matches!(x, Filter::InZone(_)))
                    .cloned()
                    .collect(),
                z,
            ))
        };
        let ((ra, za), (rb, zb)) = (split(fa)?, split(fb)?);
        if format!("{ra:?}") != format!("{rb:?}") || za == zb {
            return None;
        }
        let st = match sa {
            Stat::ManaValue => "mana value",
            Stat::Power => "power",
            Stat::Toughness => "toughness",
            _ => return None,
        };
        let zone = |z: ZoneKind| match z {
            ZoneKind::Battlefield => "on the battlefield".to_string(),
            ZoneKind::Command => "in the command zone".to_string(),
            z => format!("in {}", super::zone_word(z)),
        };
        let n = self.noun(&Filter::and(ra), super::Num::One);
        Some(format!(
            "the greatest {st} of {} {} or {}",
            super::with_article(&n),
            zone(za),
            zone(zb)
        ))
    }
}

impl super::Renderer<'_> {
    /// Alternative qualities said as predicates after the noun: "land that is snow or
    /// could produce {C}".
    pub(crate) fn or_predicates(&mut self, v: &[Filter]) -> Option<String> {
        if v.len() < 2 {
            return None;
        }
        let preds: Option<Vec<String>> = v
            .iter()
            .map(|x| match x {
                Filter::Supertype(s) => Some(format!("is {}", super::nouns::supertype_word(*s))),
                Filter::Color(c) => Some(format!("is {}", c.word())),
                Filter::Custom(n) => match custom_rel_more(n)? {
                    CustomQuality::Rel(r) => r.strip_prefix("that ").map(str::to_string),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let preds = preds?;
        // At least one quality that only a predicate can say.
        if preds.iter().all(|p| p.starts_with("is ")) {
            return None;
        }
        Some(format!("that {}", super::join_list(&preds, "or")))
    }
}

/// The kind of counters an activated ability's cost removes all of: "Remove X +1/+1
/// counters from ~" where X must be the number of them on it ("Activate only if X is the
/// number of +1/+1 counters on ~").
pub(crate) fn remove_all_counters(a: &ActivatedAbility) -> Option<String> {
    let Some(Condition::Compare(Value::X, Cmp::Eq, Value::CountersOn(on, Some(k)))) = &a.condition
    else {
        return None;
    };
    if !matches!(on.as_ref(), Sel::This) {
        return None;
    }
    a.cost
        .parts
        .iter()
        .any(|p| matches!(p, CostPart::RemoveCounters { kind, count: Value::X } if kind == k))
        .then(|| k.to_string())
}
