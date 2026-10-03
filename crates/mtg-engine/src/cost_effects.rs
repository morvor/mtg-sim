//! Whether an action performed as a cost ([`CostPart::Effect`]) can be performed in full
//! (CR 118.3: a player can't pay a cost without the necessary resources, and a cost that
//! includes an event that can't happen can't be paid, CR 614.17b-style):
//!
//! * "Exile the top four cards of your library" needs four cards in that library, "Exile
//!   the top card of your graveyard" a card in that graveyard;
//! * "Unattach ~" (CR 701.3d) needs ~ to be attached to something, and "Unattach an
//!   Equipment from ~" an Equipment attached to ~;
//! * "Put ~ on the bottom of its owner's library" needs ~ where the ability functions.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;

/// Whether the cost action `e` can be performed in full; `None` if this module doesn't
/// judge it.
pub fn payable(g: &Game, e: &Effect, ctx: &Ctx) -> Option<bool> {
    match e {
        Effect::Move {
            what: Sel::TopOfLibrary(who, n),
            ..
        } => {
            let n = g.eval_value(n, ctx).max(0) as usize;
            Some(
                g.eval_players(who, ctx)
                    .into_iter()
                    .all(|p| g.player(p).library.len() >= n),
            )
        }
        Effect::Move {
            what: Sel::TopOfGraveyard(who),
            ..
        } => Some(
            g.eval_players(who, ctx)
                .into_iter()
                .all(|p| !g.player(p).graveyard.is_empty()),
        ),
        Effect::Move {
            what: Sel::This, ..
        } => Some(ctx.source.is_some_and(|s| g.is_live(s))),
        Effect::Unattach { what: Sel::This } => Some(
            ctx.source
                .is_some_and(|s| g.is_live(s) && g.obj(s).attached_to.is_some()),
        ),
        Effect::Unattach {
            what: Sel::Choose { filter, count, .. },
        } => Some(g.objects_matching(filter, ctx).len() as i64 >= g.eval_value(count, ctx)),
        // "Sacrifice that artifact" (an "unless" action, CR 118.12a): a player can
        // sacrifice only permanents they control (CR 701.21a).
        Effect::SacrificeObjects { what } => {
            let objs = g.eval_sel_objects(what, ctx);
            Some(
                !objs.is_empty()
                    && objs.iter().all(|o| {
                        g.is_live(*o)
                            && g.obj(*o).zone == crate::object::Zone::Battlefield
                            && g.obj(*o).controller == ctx.controller
                    }),
            )
        }
        // "You may pay {1} and exile it": the object must still be there to be exiled.
        Effect::Exile {
            what: what @ (Sel::This | Sel::TriggerObject | Sel::TriggerLki | Sel::Target(_)),
            ..
        } => {
            let objs = g.eval_sel_objects(what, ctx);
            Some(!objs.is_empty() && objs.iter().all(|o| g.is_live(*o)))
        }
        // "Return a basic land card from your graveyard to your hand", "exile a creature
        // card from your graveyard": as many objects as it names must be there.
        Effect::Move {
            what:
                Sel::Choose {
                    filter,
                    count,
                    up_to: false,
                    ..
                },
            ..
        }
        | Effect::Exile {
            what:
                Sel::Choose {
                    filter,
                    count,
                    up_to: false,
                    ..
                },
            ..
        } => Some(g.objects_matching(filter, ctx).len() as i64 >= g.eval_value(count, ctx)),
        _ => None,
    }
}
