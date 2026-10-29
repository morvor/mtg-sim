//! Conditions about the order of cards in a graveyard (`zones.rs`,
//! [`crate::zones::ABOVE_SOURCE_IN_GRAVEYARD`]): "if ~ is in your graveyard with a creature
//! card directly above it", "only if three or more creature cards are above ~". A
//! graveyard's order matters for these old cards (CR 404.1, 404.2).

use super::nouns::Det;
use super::*;

/// The cards above the source a filter asks for: (the filter without the order quality,
/// "above" or "directly above").
fn above(f: &Filter) -> Option<(Filter, &'static str)> {
    let Filter::And(v) = f else {
        return None;
    };
    let which = v.iter().find_map(|x| match x {
        Filter::Custom(n) if n == crate::zones::ABOVE_SOURCE_IN_GRAVEYARD => Some("above"),
        Filter::Custom(n) if n == crate::zones::DIRECTLY_ABOVE_SOURCE_IN_GRAVEYARD => {
            Some("directly above")
        }
        _ => None,
    })?;
    let rest: Vec<Filter> = v
        .iter()
        .filter(|x| match x {
            Filter::Custom(n) => {
                n != crate::zones::ABOVE_SOURCE_IN_GRAVEYARD
                    && n != crate::zones::DIRECTLY_ABOVE_SOURCE_IN_GRAVEYARD
            }
            x => !matches!(x, Filter::InZone(ZoneKind::Graveyard)),
        })
        .cloned()
        .collect();
    Some((Filter::and(rest), which))
}

impl Renderer<'_> {
    /// "a creature card", "three or more creature cards": the cards above ~ the condition
    /// asks for, and whether they're more than one.
    fn cards_above(&mut self, c: &Condition) -> Option<(String, bool, &'static str)> {
        match c {
            Condition::Exists(f) => {
                let (f, which) = above(f)?;
                Some((self.noun_det(&f, Det::A), false, which))
            }
            Condition::Compare(Value::Count(f), Cmp::Ge, Value::Const(n)) => {
                let (f, which) = above(f)?;
                if *n == 1 {
                    return Some((self.noun_det(&f, Det::A), false, which));
                }
                let many = self.noun(&f, Num::Many);
                Some((format!("{} or more {many}", number_word(*n)), true, which))
            }
            _ => None,
        }
    }

    /// The conditions of this module, if `c` is one.
    pub(crate) fn graveyard_order_condition(&mut self, c: &Condition) -> Option<String> {
        // "~ is in your graveyard with a creature card directly above it".
        if let Condition::And(v) = c {
            if let [Condition::SelMatches(Sel::This, Filter::And(here)), other] = v.as_slice() {
                let in_your_graveyard = here
                    .iter()
                    .any(|x| matches!(x, Filter::InZone(ZoneKind::Graveyard)))
                    && here
                        .iter()
                        .any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You)));
                if in_your_graveyard {
                    let (n, _, which) = self.cards_above(other)?;
                    let m = self.me();
                    return Some(format!("{m} is in your graveyard with {n} {which} it"));
                }
            }
            return None;
        }
        // "three or more creature cards are above ~".
        let (n, many, which) = self.cards_above(c)?;
        let be = if many { "are" } else { "is" };
        let m = self.me();
        Some(format!("{n} {be} {which} {m}"))
    }
}
