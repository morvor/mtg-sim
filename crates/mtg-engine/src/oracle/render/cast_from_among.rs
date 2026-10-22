//! "Look at the top six cards of your library. You may cast a spell from among them
//! without paying its mana cost. Put the rest on the bottom of your library in a random
//! order.": the cards looked at that are still in the library after the cast.

use super::*;

/// The group a cast instruction chooses its card from (`In(Var(x))`).
fn cast_group(e: &Effect) -> Option<Var> {
    let Effect::CastCard {
        what: Sel::Choose { filter, .. },
        ..
    } = e
    else {
        return None;
    };
    let Filter::And(v) = filter else {
        return None;
    };
    v.iter().find_map(|f| match f {
        Filter::In(s) => match s.as_ref() {
            Sel::Var(x) => Some(*x),
            _ => None,
        },
        _ => None,
    })
}

impl Renderer<'_> {
    /// "Put the rest on the bottom of your library in a random order", after a cast from
    /// among the cards an earlier instruction looked at (`v[i]` is the move).
    pub(crate) fn rest_after_cast(&mut self, v: &[Effect], i: usize) -> Option<(String, usize)> {
        let Effect::Move {
            what: Sel::All(Filter::And(parts)),
            to,
        } = v.get(i)?
        else {
            return None;
        };
        let [Filter::In(g), Filter::InZone(ZoneKind::Library)] = parts.as_slice() else {
            return None;
        };
        let Sel::Var(x) = g.as_ref() else {
            return None;
        };
        // The cast just before chose among that group.
        if i == 0 || cast_group(&v[i - 1]) != Some(*x) {
            return None;
        }
        // The group is the cards a dig in your library looked at.
        let yours = v[..i].iter().any(|e| {
            matches!(
                e,
                Effect::Dig {
                    who: PlayerRef::You,
                    ..
                }
            )
        });
        if !yours || to.zone != ZoneKind::Library || to.controller.is_some() {
            return None;
        }
        let order = match to.position {
            LibraryPosition::BottomRandom => "in a random order",
            LibraryPosition::Bottom => "{opt:in any order}",
            _ => return None,
        };
        Some((
            format!("put the rest on the bottom of your library {order}"),
            1,
        ))
    }
}
