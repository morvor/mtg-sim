//! "Creature cards put into your graveyard this way": the cards an earlier instruction
//! (a mill, or a dig's selections and "the rest") moved that are now in a graveyard (see
//! `patterns::value_results::this_way_sel`).

use super::nouns::Det;
use super::players::{decline, Case};
use super::*;

impl Renderer<'_> {
    /// The cards an earlier instruction put into a graveyard, with their qualities, if `s`
    /// is that group.
    pub(crate) fn graveyard_this_way(&mut self, s: &Sel, case: Case) -> Option<String> {
        let Sel::Matching(inner, f) = s else {
            return None;
        };
        let Sel::Before(moved) = inner.as_ref() else {
            return None;
        };
        let Sel::Matching(group, Filter::InZone(ZoneKind::Graveyard)) = moved.as_ref() else {
            return None;
        };
        let Sel::Union(parts) = group.as_ref() else {
            return None;
        };
        if !matches!(parts.as_slice(), [Sel::Var(a), Sel::Var(b)]
            if *a == vars::IT && *b == crate::dig_steps::DUG_PLACED)
        {
            return None;
        }
        // The owner of the graveyard ("your graveyard", "their graveyard"): cards go to
        // their owner's graveyard (CR 404.2).
        let mut owner = None;
        let rest: Vec<Filter> = match f {
            Filter::And(v) => v.clone(),
            f => vec![f.clone()],
        }
        .into_iter()
        .filter(|x| match x {
            Filter::OwnedBy(r) if owner.is_none() => {
                owner = Some(r.clone());
                false
            }
            _ => true,
        })
        .collect();
        let zone = match owner {
            None => "{alt:a graveyard|graveyards}".to_string(),
            Some(PlayerRel::You) => "your graveyard".to_string(),
            Some(_) => "{alt:their|that player's} graveyard".to_string(),
        };
        let n = self.noun(&Filter::and(rest), Num::Many);
        Some(decline(format!("{n} put into {zone} this way"), case))
    }
}

impl Renderer<'_> {
    /// "Exile a creature card from each graveyard", "choose three cards in each graveyard":
    /// cards chosen from each player's zone in turn, gathered into one group (see
    /// `patterns::hand_graveyard_grammar`, `Cards::per_owner`), then exiled or kept chosen.
    pub(crate) fn seq_per_owner(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::Store {
            var,
            sel: Sel::Union(empty),
        }, Effect::ForEachPlayer { who, effect }, rest @ ..] = v
        else {
            return None;
        };
        if !empty.is_empty() || !matches!(who, PlayerRef::EachPlayer) {
            return None;
        }
        let Effect::Store {
            var: v2,
            sel: Sel::Union(parts),
        } = effect.as_ref()
        else {
            return None;
        };
        let [Sel::Var(v3), Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count,
            up_to,
            ..
        }] = parts.as_slice()
        else {
            return None;
        };
        if v2 != var || v3 != var {
            return None;
        }
        // The cards' qualities without the zone and its owner (the iterated player).
        let parts: Vec<Filter> = match filter {
            Filter::And(v) => v.clone(),
            f => vec![f.clone()],
        };
        let mut zone = None;
        let mut owned = false;
        let quality: Vec<Filter> = parts
            .into_iter()
            .filter(|x| match x {
                Filter::InZone(z) => {
                    zone = Some(*z);
                    false
                }
                Filter::OwnedBy(PlayerRel::Iterated) => {
                    owned = true;
                    false
                }
                _ => true,
            })
            .collect();
        let (Some(z), true) = (zone, owned) else {
            return None;
        };
        let det = if *up_to {
            Det::UpTo(match count {
                Value::Const(n) => number_word(*n),
                other => self.value(other),
            })
        } else {
            self.det_for(count)
        };
        let what = self.noun_det(&Filter::and(quality), det);
        let zone = zone_word(z);
        match rest.first() {
            Some(Effect::Exile {
                what: Sel::Var(w),
                face_down: false,
                ..
            }) if w == var => Some((format!("exile {what} from each {zone}"), 3)),
            _ => Some((format!("choose {what} in each {zone}"), 2)),
        }
    }
}
