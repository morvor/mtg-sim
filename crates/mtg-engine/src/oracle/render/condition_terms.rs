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
}
