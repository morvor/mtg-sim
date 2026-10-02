//! "Repeat this process" (CR 608.2c): open-ended repetition of a spell's or ability's
//! instructions.
//!
//! The text before "repeat this process" is the process ([`Effect::RepeatProcess`]'s
//! body). Whether it's performed again is decided after each pass, from that pass's
//! results: "If it's a permanent card, you may put it onto the battlefield. If you do,
//! repeat this process" repeats only after a pass in which the card was put onto the
//! battlefield, and "You may repeat this process any number of times" asks again after
//! every pass. The instruction to repeat is part of the process, so each repetition can
//! repeat it again.
//!
//! Everything happens while the spell or ability resolves: state-based actions aren't
//! checked and triggered abilities wait until it has finished resolving (CR 608.2,
//! 117.5) — a player whose life total drops to 0 may still choose to continue.
//!
//! A loop the rules leave to the players ("any number of times") ends when its player
//! stops. As a safeguard against agents that never stop and processes that change
//! nothing, the process ends after [`MAX_PASSES`] passes (a stand-in for the shortcut
//! the players would agree on, CR 732.2a).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;

/// Set (to 1) in [`Ctx::nums`] while the current pass of a process has asked to repeat.
pub const REPEAT_REQUESTED: Var = u16::MAX - 608;

/// The most passes a process is performed in one resolution.
pub const MAX_PASSES: u32 = 1000;

/// Performs `body` (the process), again after each pass that performed
/// [`Effect::RepeatThisProcess`].
pub fn run(g: &mut Game, body: &Effect, ctx: &mut Ctx) {
    // A process nested in another keeps the outer pass's request.
    let outer = ctx.nums.remove(&REPEAT_REQUESTED);
    let mut passes = 0;
    loop {
        passes += 1;
        if passes > 1 {
            // "If you do" in a pass is about that pass's instructions, not an earlier
            // pass's.
            ctx.prev_happened = false;
        }
        g.exec(body, ctx);
        let again = ctx.nums.remove(&REPEAT_REQUESTED).is_some();
        if !again || passes >= MAX_PASSES || g.is_over() {
            break;
        }
    }
    if let Some(v) = outer {
        ctx.nums.insert(REPEAT_REQUESTED, v);
    }
}

/// "repeat this process": the process is performed again once the current pass ends.
pub fn request(ctx: &mut Ctx) {
    ctx.nums.insert(REPEAT_REQUESTED, 1);
}

/// Whether `e` contains an instruction to repeat its process that isn't already inside a
/// process of its own.
pub fn has_open_repeat(e: &Effect) -> bool {
    match e {
        Effect::RepeatThisProcess => true,
        Effect::RepeatProcess { .. } => false,
        Effect::Seq(v) => v.iter().any(has_open_repeat),
        Effect::If {
            then, otherwise, ..
        }
        | Effect::PayOptional {
            then, otherwise, ..
        } => has_open_repeat(then) || has_open_repeat(otherwise),
        Effect::May { effect, .. }
        | Effect::AsPlayer { effect, .. }
        | Effect::ForEachPlayer { effect, .. }
        | Effect::ForEach { effect, .. } => has_open_repeat(effect),
        _ => false,
    }
}
