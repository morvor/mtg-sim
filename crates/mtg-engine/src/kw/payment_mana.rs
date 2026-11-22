//! The mana spent on a cost paid while a spell or ability resolves (CR 118.3, 601.2h):
//! "Whenever ~'s cumulative upkeep is paid, it gets +1/+0 until end of turn for each {B}
//! or {R} spent this way" (Balduvian Fallen, CR 702.24a).
//!
//! When a resolving "you may pay [cost]" is paid, the types of the mana spent are logged
//! in [`crate::kw::keyword_state::KeywordState::payments`] and the payment's number (its
//! index plus one) is kept in the resolution's [`PAID_MANA`] number. An event reporting
//! that payment ("cumulative upkeep paid", `kw/cumulative_upkeep.rs`) carries the number
//! as its amount, so a trigger on it counts the mana of that payment, not of a later one
//! (`cast history:spent:this way:<what>`, `kw/cast_history.rs`).

use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::ManaType;

/// `Ctx::nums` entry: the number of the payment the resolving effect just made.
pub const PAID_MANA: Var = vars::USER + 47391;

/// The mana types a `B,R` list names.
pub fn parse_types(s: &str) -> Option<Vec<ManaType>> {
    s.split(',')
        .map(|t| ManaType::ALL.iter().copied().find(|m| format!("{m:?}") == t))
        .collect()
}

/// Logs the mana the cost just paid by the resolving effect spent (from
/// `Game::last_paid`), and keeps the payment's number in the resolution's context.
pub fn note(g: &mut Game, ctx: &mut Ctx) {
    let Some(paid) = g.last_paid.as_ref() else {
        return;
    };
    let mana = paid.mana_spent.clone();
    g.kw_state.payments.push(mana);
    ctx.nums
        .insert(PAID_MANA, g.kw_state.payments.len() as i64);
}

/// The number of the payment the resolving effect last made (0 if none).
pub fn last_payment(ctx: &Ctx) -> i32 {
    ctx.nums.get(&PAID_MANA).copied().unwrap_or(0) as i32
}

/// The mana spent on the payment the triggering event reports (none if it reports none).
pub fn paid_this_way(g: &Game, ctx: &Ctx) -> Vec<ManaType> {
    let n = ctx.event.as_ref().map_or(0, |e| e.amount);
    usize::try_from(n - 1)
        .ok()
        .and_then(|i| g.kw_state.payments.get(i))
        .cloned()
        .unwrap_or_default()
}
