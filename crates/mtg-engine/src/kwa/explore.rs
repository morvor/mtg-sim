//! CR 701.44: explore.
//!
//! * To explore, the permanent's controller reveals the top card of their library; a land
//!   card goes to their hand, otherwise (a nonland card, or no card at all) the permanent
//!   gets a +1/+1 counter and they may put the revealed card into their graveyard
//!   (CR 701.44a).
//! * A permanent "explores" once the process is complete, even if some or all of it was
//!   impossible (CR 701.44b): an `"explore"` event (`Event::Custom`) is reported with the
//!   exploring permanent and its controller, with amount 1 if a land card was revealed, 0
//!   if a nonland card was, and -1 if no card was.
//! * A permanent that left the battlefield still explores, using its last known
//!   information for who controlled it (CR 701.44c).
//! * Several permanents exploring at once explore one at a time, chosen in APNAP order
//!   (CR 701.44d).

use super::*;
use crate::events::MoveCause;
use crate::types::counters;

/// `Event::Custom` name reported when a permanent explores.
pub const EXPLORED: &str = "explore";
/// Amounts of the [`EXPLORED`] event.
pub const REVEALED_LAND: i32 = 1;
pub const REVEALED_NONLAND: i32 = 0;
pub const REVEALED_NOTHING: i32 = -1;

/// Has `obj` explore once (CR 701.44a–c).
pub fn explore(g: &mut Game, obj: ObjectId, source: Option<ObjectId>) {
    let p = controller_or_last(g, obj);
    let result = match g.library_top(p) {
        // "Otherwise" includes revealing no card at all (an empty library).
        None => {
            if on_battlefield(g, obj) {
                g.add_counters(Entity::Object(obj), counters::PLUS1, 1, source);
            }
            REVEALED_NOTHING
        }
        Some(card) => {
            g.log(|g| format!("{p} reveals {} (explore)", g.describe(card)));
            if g.obj(card).chars.is_land() {
                g.move_object(card, Zone::Hand(p), MoveCause::Effect, Some(p));
                REVEALED_LAND
            } else {
                if on_battlefield(g, obj) {
                    g.add_counters(Entity::Object(obj), counters::PLUS1, 1, source);
                }
                if g.ask_yes_no(
                    p,
                    source,
                    "Explore: put the revealed card into your graveyard?",
                    false,
                ) && g.is_live(card)
                {
                    let owner = g.obj(card).owner;
                    g.move_object(card, Zone::Graveyard(owner), MoveCause::Effect, Some(p));
                }
                REVEALED_NONLAND
            }
        }
    };
    emit(g, EXPLORED, p, Some(obj), result);
}

pub struct Explore;

impl KeywordActionRules for Explore {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Explore]
    }

    /// "[Permanents] explore [N times]".
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let times = number(g, a.n, ctx);
        let mut remaining = g.resolve_objects(a.what, ctx);
        while let Some(obj) =
            next_in_apnap_order(g, &mut remaining, ctx.source, "Choose which explores next")
        {
            for _ in 0..times {
                explore(g, obj, ctx.source);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Explore) }
