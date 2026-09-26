//! CR 701.68: blight.
//!
//! * To blight N means to put N -1/-1 counters on a creature you control (CR 701.68a).
//!   The creature is chosen as the instruction is performed.
//! * A player who can't put N -1/-1 counters on a creature they control (usually because
//!   they control no creatures) can't choose to blight, whether it's optional or a cost
//!   (CR 701.68b).
//! * "The blighted creature" is the creature the player chose (CR 701.68c), stored in
//!   [`kvars::BLIGHTED`].
//! * A player "blights" once the process is complete, whatever events actually occurred
//!   (CR 701.68d): a `"blight"` event (`Event::Custom`) is reported.

use super::*;
use crate::types::counters;

/// `Event::Custom` name reported when a player blights.
pub const BLIGHTED_EVENT: &str = "blight";

fn creatures(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| o.controller == p && o.is_creature())
        .map(|o| o.id)
        .collect()
}

/// Whether `p` could put -1/-1 counters on a creature they control (CR 701.68b).
pub fn can_blight(g: &Game, p: PlayerId) -> bool {
    !creatures(g, p).is_empty()
}

/// `p` blights N (CR 701.68a–d). Returns the blighted creature.
pub fn blight(g: &mut Game, p: PlayerId, n: u32, ctx: &mut Ctx) -> Option<ObjectId> {
    if g.dirty {
        g.recompute();
    }
    let cands = creatures(g, p);
    let chosen = match cands.as_slice() {
        [] => None,
        [one] => Some(*one),
        _ => g
            .ask_objects(
                p,
                ctx.source,
                &format!("Blight {n}: choose a creature you control"),
                cands.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .or(Some(cands[0])),
    };
    if let Some(c) = chosen {
        g.add_counters(Entity::Object(c), counters::MINUS1, n, ctx.source);
    }
    ctx.set_var(
        kvars::BLIGHTED,
        chosen.map(Entity::Object).into_iter().collect(),
    );
    emit(g, BLIGHTED_EVENT, p, chosen, n as i32);
    chosen
}

pub struct Blight;

impl KeywordActionRules for Blight {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Blight]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            blight(g, p, n, ctx);
        }
    }

    fn can_choose(&self, g: &Game, a: &Args, ctx: &Ctx) -> bool {
        g.eval_players(a.who, ctx)
            .into_iter()
            .all(|p| can_blight(g, p))
    }
}

inventory::submit! { KeywordActionRegistration(&Blight) }
