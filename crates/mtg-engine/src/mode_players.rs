//! "Choose one or more. Each mode must target a different player." (Vindictive Lich,
//! Balor, Shadrix Silverquill): `Modal::different_players`.
//!
//! The modes are chosen as the spell or ability is put on the stack (CR 601.2b, 700.2),
//! and only a set of modes whose targets can all be legal can be chosen (CR 700.2a,
//! 601.2c): with this instruction, that's a set of modes for which there are different
//! players to target, one per mode — "You can't choose more modes ... than you have
//! opponents" with "target opponent" modes. The player targets are then chosen mode by
//! mode, each different from those of the other modes, and a target can't be changed to
//! a player another mode targets (CR 115.7).

use crate::ability::{Modal, TargetKind, TargetSpec};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::ChosenMode;
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

/// The players a mode could target, or `None` if it targets no player (the instruction
/// doesn't restrict it).
pub fn mode_players(
    g: &Game,
    specs: &[TargetSpec],
    ctx: &Ctx,
    stack_obj: ObjectId,
) -> Option<Vec<PlayerId>> {
    let player_slots: Vec<&TargetSpec> = specs
        .iter()
        .filter(|s| matches!(s.what, TargetKind::Player(_)))
        .collect();
    if player_slots.is_empty() {
        return None;
    }
    let mut out: Vec<PlayerId> = Vec::new();
    for s in player_slots {
        for e in g.legal_target_candidates(s, ctx, stack_obj) {
            if let Some(p) = e.player() {
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        }
    }
    Some(out)
}

/// Whether each mode can be given a different player from its set, none of `used`
/// (bipartite matching by augmenting paths; there are only a few modes).
pub fn matchable(sets: &[Option<Vec<PlayerId>>], used: &[PlayerId]) -> bool {
    fn assign(
        k: usize,
        sets: &[Option<Vec<PlayerId>>],
        used: &[PlayerId],
        owner: &mut BTreeMap<PlayerId, usize>,
        seen: &mut BTreeSet<PlayerId>,
    ) -> bool {
        let Some(set) = &sets[k] else {
            return true;
        };
        for p in set {
            if used.contains(p) || !seen.insert(*p) {
                continue;
            }
            let free = match owner.get(p).copied() {
                None => true,
                Some(m) => assign(m, sets, used, owner, seen),
            };
            if free {
                owner.insert(*p, k);
                return true;
            }
        }
        false
    }
    let mut owner = BTreeMap::new();
    (0..sets.len()).all(|k| assign(k, sets, used, &mut owner, &mut BTreeSet::new()))
}

/// The modes' player sets, for the chosen modes `picks` (in order).
pub fn sets_for(
    g: &Game,
    modal: &Modal,
    picks: &[usize],
    ctx: &Ctx,
    stack_obj: ObjectId,
) -> Vec<Option<Vec<PlayerId>>> {
    picks
        .iter()
        .map(|m| mode_players(g, &modal.modes[*m].targets, ctx, stack_obj))
        .collect()
}

/// The players the `k`th chosen mode may target: ones not targeted by earlier modes
/// (`used`) that leave a different player for each later mode.
pub fn allowed_for(sets: &[Option<Vec<PlayerId>>], k: usize, used: &[PlayerId]) -> Vec<PlayerId> {
    let Some(set) = &sets[k] else {
        return vec![];
    };
    set.iter()
        .copied()
        .filter(|p| !used.contains(p))
        .filter(|p| {
            let mut u = used.to_vec();
            u.push(*p);
            matchable(&sets[k + 1..], &u)
        })
        .collect()
}

/// The players the chosen modes target, in order.
pub fn targeted_players(chosen: &[ChosenMode]) -> Vec<PlayerId> {
    chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().filter_map(|e| e.player()))
        .collect()
}

/// Whether no player is the target of two different chosen modes.
pub fn players_distinct(chosen: &[ChosenMode]) -> bool {
    let per_mode: Vec<Vec<PlayerId>> = chosen
        .iter()
        .map(|cm| {
            cm.targets
                .iter()
                .flatten()
                .filter_map(|e| e.player())
                .collect()
        })
        .collect();
    per_mode.iter().enumerate().all(|(i, a)| {
        per_mode
            .iter()
            .skip(i + 1)
            .all(|b| a.iter().all(|p| !b.contains(p)))
    })
}
