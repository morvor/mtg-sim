//! CR 701.50: connive.
//!
//! * To connive, the permanent's controller draws a card, then discards a card; if a
//!   nonland card is discarded this way, they put a +1/+1 counter on the conniving
//!   permanent (CR 701.50a). Connive N: draw N, discard N, then a +1/+1 counter for each
//!   nonland card discarded (CR 701.50d).
//! * A permanent that left the battlefield still connives, using its last known
//!   information for who controlled it (CR 701.50b).
//! * Several permanents conniving at once connive one at a time, chosen in APNAP order
//!   (CR 701.50c).
//! * Conniving 0 is no connive event at all (CR 701.50e); otherwise a permanent
//!   "connives" once the process is complete, even if some or all of it was impossible
//!   (CR 701.50f): a `"connive"` event (`Event::Custom`) is reported with the permanent,
//!   its controller, and the number of nonland cards discarded.

use super::*;
use crate::types::counters;

/// `Event::Custom` name reported when a permanent connives.
pub const CONNIVED: &str = "connive";

/// Has `obj` connive `n` (CR 701.50a–b, 701.50d–f).
pub fn connive(g: &mut Game, obj: ObjectId, n: u32, source: Option<ObjectId>) {
    if n == 0 {
        return;
    }
    let p = controller_or_last(g, obj);
    g.draw_cards(p, n);
    let hand = g.player(p).hand.clone();
    let k = n.min(hand.len() as u32);
    let picks = g.ask_objects(p, source, "Connive: choose cards to discard", hand, k, k);
    let mut nonland = 0;
    for c in picks {
        let is_land = g.obj(c).chars.is_land();
        if g.discard(p, c, source).is_some() && !is_land {
            nonland += 1;
        }
    }
    if nonland > 0 && on_battlefield(g, obj) {
        g.add_counters(Entity::Object(obj), counters::PLUS1, nonland, source);
    }
    emit(g, CONNIVED, p, Some(obj), nonland as i32);
}

pub struct Connive;

impl KeywordActionRules for Connive {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Connive]
    }

    /// "[Permanents] connive [N]".
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        let mut remaining = g.resolve_objects(a.what, ctx);
        while let Some(obj) =
            next_in_apnap_order(g, &mut remaining, ctx.source, "Choose which connives next")
        {
            connive(g, obj, n, ctx.source);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Connive) }
