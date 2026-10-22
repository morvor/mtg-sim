//! CR 701.43: exert.
//!
//! * To exert a permanent, its controller chooses to have it not untap during their next
//!   untap step (CR 701.43a): the permanent's `exerted` flag, cleared in its controller's
//!   untap step, plus who exerted it. If another player controls it by then (control was
//!   gained until end of turn and has reverted), it untaps as usual during that player's
//!   untap step: it's not the next untap step of the player who exerted it.
//! * A permanent can be exerted even if it's untapped or already exerted; exerting it again
//!   doesn't extend the effect (CR 701.43b).
//! * An object that isn't on the battlefield can't be exerted (CR 701.43c).
//! * "You may exert [this creature] as it attacks" is an optional cost to attack
//!   (CR 701.43d), in `kw/exert.rs`; "{T}, Exert this creature: ..." is a cost
//!   (`CostPart::ExertSelf`).
//!
//! Each exertion reports an event for "whenever you exert a creature" triggers.

use super::*;

/// `Event::Custom` name reported when a player exerts a permanent (the same name as
/// [`crate::kw::exert::EXERTED`], the condition of the linked "When you do" abilities,
/// which are triggered separately, CR 607.2h).
pub const EXERTED_EVENT: &str = crate::kw::exert::EXERTED;

/// `p` exerts `obj` (CR 701.43a–c). Returns false if it isn't on the battlefield.
pub fn exert(g: &mut Game, obj: ObjectId, p: PlayerId) -> bool {
    if !on_battlefield(g, obj) {
        return false;
    }
    g.objects[obj.0 as usize].exerted = true;
    g.kwa.exerted_by.retain(|(o, _)| *o != obj);
    g.kwa.exerted_by.push((obj, p));
    g.log(|g| format!("{p} exerts {}", g.describe(obj)));
    emit(g, EXERTED_EVENT, p, Some(obj), 0);
    true
}

/// Whether an exerted permanent's "doesn't untap" applies in its controller's current
/// untap step: only if the controller exerted it (CR 701.43a: "your next untap step").
pub fn keeps_tapped(g: &Game, obj: ObjectId) -> bool {
    let o = g.obj(obj);
    o.exerted
        && g
            .kwa
            .exerted_by
            .iter()
            .find(|(x, _)| *x == obj)
            .is_none_or(|(_, p)| *p == o.controller)
}

pub struct Exert;

impl KeywordActionRules for Exert {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Exert]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let p = g.eval_player(a.who, ctx).unwrap_or(ctx.controller);
        for o in g.resolve_objects(a.what, ctx) {
            exert(g, o, p);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Exert) }
