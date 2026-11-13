//! CR 701.32: set in motion (and CR 701.33: abandon, in `variants.rs`).
//!
//! * Only a scheme card may be set in motion, only during an Archenemy game, and only by
//!   the archenemy (CR 701.32a).
//! * To set a scheme in motion, move it off the top of your scheme deck if it's there and
//!   turn it face up if it isn't; it's considered set in motion even if neither happened
//!   (CR 701.32b): "set that scheme in motion again" on a scheme that's already face up
//!   still makes its "When you set this scheme in motion" abilities trigger.
//! * Schemes are set in motion one at a time: "set N schemes in motion" does it N times
//!   (CR 701.32c).

use super::*;
use crate::variants::{scheme_deck, turn_face_up_in_command, SET_IN_MOTION};

fn is_scheme_card(o: &crate::object::GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.card_types.contains(CardType::Scheme)
}

/// `p` sets the scheme card `scheme` in motion (CR 701.32a–b). Returns false if it can't
/// be.
pub fn set_scheme_in_motion(g: &mut Game, p: PlayerId, scheme: ObjectId) -> bool {
    let scheme = g.current(scheme);
    let o = g.obj(scheme);
    if !crate::life_totals::is_archenemy(g, p)
        || o.zone != Zone::Command
        || o.owner != p
        || !is_scheme_card(o)
    {
        return false;
    }
    if scheme_deck(g, p).first() == Some(&scheme) {
        // Off the top of the scheme deck: to the end of the command-zone order.
        g.command.retain(|x| *x != scheme);
        g.command.push(scheme);
    }
    turn_face_up_in_command(g, scheme);
    g.recompute();
    g.log(|g| format!("{p} sets {} in motion", g.obj(scheme).chars.name));
    emit(g, SET_IN_MOTION, p, Some(scheme), 0);
    true
}

pub struct SetInMotion;

impl KeywordActionRules for SetInMotion {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::SetInMotion]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let players = g.eval_players(a.who, ctx);
        if matches!(a.what, Sel::None) {
            // The top scheme of the scheme deck, N times, one at a time (CR 701.32c).
            let n = number(g, a.n, ctx).max(1);
            for p in players {
                for _ in 0..n {
                    crate::variants::set_in_motion(g, p);
                }
            }
            return;
        }
        let schemes = g.resolve_objects(a.what, ctx);
        for p in players {
            for s in &schemes {
                set_scheme_in_motion(g, p, *s);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&SetInMotion) }
