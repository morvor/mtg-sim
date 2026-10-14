//! "Whenever ~ blocks or becomes blocked by a creature, ...": the other creature of the
//! block (the triggering object, see `combat.rs`), and instructions done to both.

use super::players::Case;
use super::*;

/// Whether a trigger condition is "blocks or becomes blocked by a creature" (CR 509.3a,
/// 509.3c): the triggering object is the other creature either way.
pub(crate) fn is_block_pair(t: &TriggerCond) -> bool {
    let TriggerCond::AnyOf(v) = t else {
        return false;
    };
    v.len() == 2
        && v.iter()
            .any(|c| matches!(c, TriggerCond::BlocksCreature { .. }))
        && v.iter()
            .any(|c| matches!(c, TriggerCond::BlockedByCreature { .. }))
}

impl Renderer<'_> {
    /// The other creature of the block.
    pub(crate) fn block_pair_other(&self, case: Case) -> Option<String> {
        if !self.block_pair_trigger {
            return None;
        }
        Some(match case {
            Case::Poss => "{alt:that creature's|the other creature's}".into(),
            _ => "{alt:that creature|the other creature}".into(),
        })
    }

    /// "~ and that creature" / "both creatures": the creature with the ability (or the one
    /// it's attached to) and the other creature of the block.
    pub(crate) fn block_pair_both(&mut self, parts: &[Sel], case: Case) -> Option<String> {
        if !self.block_pair_trigger {
            return None;
        }
        let [first, Sel::TriggerObject] = parts else {
            return None;
        };
        if !matches!(first, Sel::This | Sel::AttachedTo) {
            return None;
        }
        let a = self.sel(first, Case::Obj);
        let b = self.block_pair_other(Case::Obj)?;
        Some(match case {
            Case::Poss => format!("{{alt:both creatures'|{a}'s and {b}'s}}"),
            _ => format!("{{alt:both creatures|{a} and {b}}}"),
        })
    }

    /// "~ and that creature phase out": the same instruction for both, in a row.
    pub(crate) fn block_pair_phase_part(
        &mut self,
        v: &[Effect],
        i: usize,
    ) -> Option<(usize, String)> {
        if !self.block_pair_trigger {
            return None;
        }
        let (Effect::PhaseOut { what: a }, Some(Effect::PhaseOut { what: b })) =
            (&v[i], v.get(i + 1))
        else {
            return None;
        };
        let both = self.block_pair_both(&[a.clone(), b.clone()], Case::Subj)?;
        Some((2, format!("{both} phase out")))
    }
}
