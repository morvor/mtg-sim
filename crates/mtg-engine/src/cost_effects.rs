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
        // "You may pay {1} and exile it": the object must still be there to be exiled — a
        // card a dies trigger is about can be found in the graveyard (CR 400.7, 603.10a),
        // as the exile itself finds it (`Game::resolve_sel`).
        Effect::Exile {
            what: what @ (Sel::This | Sel::TriggerObject | Sel::TriggerLki | Sel::Target(_)),
            ..
        } => {
            let follow = matches!(what, Sel::This | Sel::TriggerLki);
            let objs: Vec<_> = g
                .eval_sel(what, ctx)
                .into_iter()
                .map(|e| {
                    if follow {
                        g.follow_zone_change_trigger_object(e, ctx)
                    } else {
                        e
                    }
                })
                .filter_map(|e| e.object())
                .collect();
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

/// After a cost paid as an effect resolved ("you may pay {1} and discard a card. If you do,
/// ... the discarded card's mana value"): the cards it discarded are "the discarded
/// cards" for the instructions after it, as a discard instruction's would be.
pub fn note_paid(g: &mut Game, cost: &crate::ability::Cost, ctx: &mut Ctx) {
    let Some(paid) = g.last_paid.take() else {
        return;
    };
    let discards = cost
        .parts
        .iter()
        .any(|p| matches!(p, crate::ability::CostPart::Discard { .. }));
    if !discards {
        return;
    }
    let discarded: Vec<crate::types::Entity> = paid
        .objects
        .iter()
        .filter(|o| {
            !paid.sacrificed.contains(o) && !paid.exiled.contains(o) && !paid.tapped.contains(o)
        })
        .map(|o| crate::types::Entity::Object(g.current(*o)))
        .collect();
    ctx.set_var(crate::discard_rules::DISCARDED, discarded);
}
