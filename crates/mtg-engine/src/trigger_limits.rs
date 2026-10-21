//! Limits on how many times a triggered ability triggers ([`TriggerLimit`]): "This ability
//! triggers only once." (the Mox Collector cycle, Acrobatic Cheerleader: once ever, for as
//! long as its source remains the same object, CR 400.7) and "This ability triggers only
//! twice each turn." (Nadu, Winged Wisdom). "Only once each turn" is
//! [`TriggeredAbility::once_per_turn`].
//!
//! The count is of times the ability triggered, whatever happened to it afterwards: once it
//! has triggered, it won't trigger again even if it was countered (Acrobatic Cheerleader's
//! rulings).

use crate::ability::*;
use crate::game::Game;
use crate::types::*;

/// The key of the per-turn count in [`crate::object::GameObject::triggers_this_turn`].
const PER_TURN: u64 = 1 << 61;

/// Whether the triggered ability `uid` of `src` may trigger once more under `limit`; if
/// so, counts the triggering.
pub fn try_trigger(g: &mut Game, src: ObjectId, uid: u64, limit: TriggerLimit) -> bool {
    let o = &mut g.objects[src.0 as usize];
    let (n, max) = match limit {
        TriggerLimit::Once => (o.triggers_ever.entry(uid).or_insert(0), 1),
        TriggerLimit::PerTurn(k) => (o.triggers_this_turn.entry(uid | PER_TURN).or_insert(0), k),
    };
    if *n >= max {
        return false;
    }
    *n += 1;
    true
}

/// How many of `times` triggerings of the triggered ability `uid` of `src` its limit
/// allows (all of them without a limit); counts those.
pub fn allowed(
    g: &mut Game,
    src: ObjectId,
    uid: u64,
    limit: Option<TriggerLimit>,
    times: usize,
) -> usize {
    let Some(l) = limit else {
        return times;
    };
    (0..times)
        .take_while(|_| try_trigger(g, src, uid, l))
        .count()
}
