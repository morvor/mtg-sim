//! "Flashback {8}{G}{G}. This spell costs {X} less to cast this way, where X is the
//! greatest mana value of a commander you own on the battlefield or in the command zone."
//! (the Visions cycle): the keyword and the cost reduction that applies when it's cast
//! with flashback, compiled as two abilities.

use super::*;

impl Renderer<'_> {
    pub(crate) fn flashback_with_reduction(
        &mut self,
        a: &Ability,
        next: &Ability,
    ) -> Option<String> {
        let AbilityKind::Keyword(k) = &a.kind else {
            return None;
        };
        if k.kind != crate::keywords::KeywordKind::Flashback {
            return None;
        }
        let AbilityKind::Static(s) = &next.kind else {
            return None;
        };
        let (
            Some(Condition::CostPaid(name)),
            StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::ThisSpell,
                change: CostChange::ReduceGeneric(v),
                ..
            }),
        ) = (&s.condition, &s.effect)
        else {
            return None;
        };
        if name != "flashback" {
            return None;
        }
        let w = self.greatest_in_zones(v).unwrap_or_else(|| self.value(v));
        let line = self.ability(a);
        Some(format!(
            "{line}. This spell costs {{X}} less to cast this way, where X is {w}"
        ))
    }

    /// "the greatest mana value of a commander you own on the battlefield or in the
    /// command zone": the greater of the greatest in each zone.
    fn greatest_in_zones(&mut self, v: &Value) -> Option<String> {
        let Value::Max(a, b) = v else { return None };
        let split = |v: &Value| -> Option<(Filter, ZoneKind)> {
            let Value::Aggregate(AggOp::Max, Stat::ManaValue, s) = v else {
                return None;
            };
            let Sel::All(Filter::And(parts)) = s.as_ref() else {
                return None;
            };
            let zone = parts.iter().find_map(|p| match p {
                Filter::InZone(z) => Some(*z),
                _ => None,
            })?;
            let rest: Vec<Filter> = parts
                .iter()
                .filter(|p| !matches!(p, Filter::InZone(_)))
                .cloned()
                .collect();
            Some((Filter::and(rest), zone))
        };
        let (fa, za) = split(a)?;
        let (fb, zb) = split(b)?;
        if format!("{fa:?}") != format!("{fb:?}") || za == zb {
            return None;
        }
        let zone = |z: ZoneKind| match z {
            ZoneKind::Battlefield => "on the battlefield".to_string(),
            ZoneKind::Command => "in the command zone".to_string(),
            ZoneKind::Exile => "in exile".to_string(),
            other => format!("in {}", with_article(zone_word(other))),
        };
        let n = self.noun_det(&fa, super::nouns::Det::A);
        Some(format!(
            "the greatest mana value of {n} {} or {}",
            zone(za),
            zone(zb)
        ))
    }
}
