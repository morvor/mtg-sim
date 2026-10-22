//! Conditions implemented in code (`Condition::Custom`) about the object itself or the
//! creatures that crewed it, each with its negation: "~ has hexproof as long as it hasn't
//! dealt damage yet", "as long as they haven't dealt combat damage yet", "if an Assassin
//! crewed it this turn" (CR 702.122).

use super::*;
use crate::oracle::patterns::grant_conditions as gc;

/// "crew:crewed it this turn, a creature of subtype X" (see `kw/crew.rs`).
const CREWED_BY_SUBTYPE: &str = "crew:crewed it this turn, a creature of subtype ";

impl Renderer<'_> {
    /// The condition (or its negation), if this module words it.
    pub(crate) fn custom_condition_terms(&mut self, name: &str, negated: bool) -> Option<String> {
        // What was paid to activate the ability whose activation triggered.
        {
            use crate::kw::trigger_modifiers as tm;
            let paid = if name == tm::SACRIFICED_TO_ACTIVATE {
                Some("one or more permanents were sacrificed".to_string())
            } else if name == tm::LIFE_PAID_TO_ACTIVATE {
                Some("life was paid".to_string())
            } else {
                name.strip_prefix(tm::LOYALTY_REMOVED_TO_ACTIVATE)
                    .and_then(|n| n.parse::<i32>().ok())
                    .map(|n| format!("you removed {} or more loyalty counters", number_word(n)))
            };
            if let (Some(p), false) = (paid, negated) {
                return Some(format!("{p} to activate it"));
            }
        }
        let known = [
            gc::SOURCE_DEALT_DAMAGE,
            gc::SOURCE_DEALT_COMBAT_DAMAGE,
            gc::SOURCE_CAST_FROM_EXILE,
        ]
        .contains(&name)
            || name.starts_with(CREWED_BY_SUBTYPE);
        if !known {
            return None;
        }
        let me = self.me();
        let it = format!("{{alt:{me}|it|they}}");
        Some(match name {
            // "as long as it hasn't dealt damage yet": the object, since it last entered
            // (CR 400.7).
            gc::SOURCE_DEALT_DAMAGE if negated => {
                format!("{it} {{alt:hasn't|haven't}} dealt damage yet")
            }
            gc::SOURCE_DEALT_DAMAGE => format!("{it} {{alt:has|have}} dealt damage"),
            gc::SOURCE_DEALT_COMBAT_DAMAGE if negated => {
                format!("{it} {{alt:hasn't|haven't}} dealt combat damage yet")
            }
            gc::SOURCE_DEALT_COMBAT_DAMAGE => {
                format!("{it} {{alt:has|have}} dealt combat damage")
            }
            gc::SOURCE_CAST_FROM_EXILE if negated => format!("{me} wasn't cast from exile"),
            gc::SOURCE_CAST_FROM_EXILE => format!("{me} was cast from exile"),
            n if n.starts_with(CREWED_BY_SUBTYPE) => {
                let t = &n[CREWED_BY_SUBTYPE.len()..];
                if negated {
                    format!("no {t} crewed it this turn")
                } else {
                    format!("{} crewed it this turn", with_article(t))
                }
            }
            _ => return None,
        })
    }

    /// Comparisons worded as a whole: "two or more of those creatures are attacking you
    /// and/or planeswalkers you control", "you have the most life or are tied for most
    /// life" (CR 119).
    pub(crate) fn compare_terms(&mut self, c: &Condition) -> Option<String> {
        match c {
            Condition::Compare(Value::CountSel(sel), Cmp::Ge, Value::Const(n))
                if matches!(sel.as_ref(), Sel::Matching(s, Filter::Custom(q))
                    if matches!(s.as_ref(), Sel::TriggerObjects)
                        && q == crate::kw::activated_ability_kind::ATTACKING_YOU_OR_YOUR_PLANESWALKER) =>
            {
                Some(format!(
                    "{} or more of {{alt:those creatures|them}} are attacking you and/or planeswalkers you control",
                    number_word(*n)
                ))
            }
            Condition::Compare(Value::LifeTotal(PlayerRef::You), Cmp::Ge, Value::OverPlayers(AggOp::Max, PlayerFilter::Any, v))
                if matches!(v.as_ref(), Value::LifeTotal(PlayerRef::Iterated)) =>
            {
                Some("you have the most life or are tied for most life".into())
            }
            // "If a creature spell you cast this turn was countered by a spell or ability
            // an opponent controlled".
            Condition::AllTriggerConditionsThisTurn(v) => {
                let [TriggerCond::CounteredBy { filter, by }] = v.as_slice() else {
                    return None;
                };
                // "a creature spell you cast this turn".
                let cast_by = |f: &Filter| match f {
                    Filter::And(v) if v.iter().any(|x| matches!(x, Filter::Custom(n) if n == crate::custom::WAS_CAST)) => {
                        v.iter().find_map(|x| match x {
                            Filter::ControlledBy(r) => Some(*r),
                            _ => None,
                        })
                    }
                    _ => None,
                };
                let n = match (cast_by(filter), filter) {
                    (Some(r), Filter::And(v)) => {
                        let rest: Vec<Filter> = v
                            .iter()
                            .filter(|x| {
                                !matches!(x, Filter::ControlledBy(_))
                                    && !matches!(x, Filter::Custom(n) if n == crate::custom::WAS_CAST)
                            })
                            .cloned()
                            .collect();
                        let n = self.noun_det(&Filter::and(rest), super::nouns::Det::A);
                        let w = self.rel_subject(r);
                        format!("{n} {w} cast")
                    }
                    _ => self.noun_det(filter, super::nouns::Det::A),
                };
                let by = match by {
                    PlayerRel::Opponent => " an opponent controlled",
                    PlayerRel::You => " you controlled",
                    PlayerRel::Any => "",
                    _ => return None,
                };
                Some(format!(
                    "{n} this turn was countered by a spell or ability{by}"
                ))
            }
            _ => None,
        }
    }
}
