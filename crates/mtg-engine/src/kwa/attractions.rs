//! CR 701.51: open an Attraction (rolling to visit Attractions, CR 701.52, is in
//! `variants.rs`).
//!
//! * A player's Attraction deck is the face-down Attraction cards they own in the command
//!   zone, in command-zone order (the first is the top) (CR 717.2). A player can open an
//!   Attraction only while playing with an Attraction deck (CR 701.51a): otherwise, or with
//!   an empty deck, nothing happens.
//! * To open an Attraction, move the top card of your Attraction deck off it, turn it face
//!   up, and put it onto the battlefield under your control (CR 701.51b).
//! * "Whenever you open an Attraction" triggers only if the Attraction card is put onto
//!   the battlefield this way (CR 701.51c): an [`OPENED`] event reports it.

use super::*;
use crate::events::MoveCause;
use crate::replacement::{EtbInfo, MoveEv};

/// `Event::Custom` name: a player opened an Attraction (the object is the Attraction on
/// the battlefield).
pub const OPENED: &str = "open an attraction";

fn is_attraction_card(o: &crate::object::GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.subtypes.iter().any(|s| s.as_str() == "Attraction")
}

/// `p`'s Attraction deck, top card first (CR 717.2).
pub fn attraction_deck(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            o.face_down && o.owner == p && is_attraction_card(o)
        })
        .collect()
}

/// `p` opens an Attraction (CR 701.51). Returns the Attraction on the battlefield.
pub fn open_attraction(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> Option<ObjectId> {
    let top = *attraction_deck(g, p).first()?;
    // Off the deck and face up, then onto the battlefield.
    g.objects[top.0 as usize].face_down = false;
    g.dirty = true;
    let moved = g.move_object_ev(MoveEv {
        obj: top,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source,
    });
    let entered = moved.filter(|id| on_battlefield(g, *id) && is_attraction_card(g.obj(*id)));
    match entered {
        Some(a) => {
            g.log(|g| format!("{p} opens {}", g.describe(a)));
            emit(g, OPENED, p, Some(a), 0);
        }
        None => {
            // It didn't enter: if it's still in the command zone, it stays off the deck,
            // face up.
            if g.is_live(top) && g.obj(top).zone == Zone::Command {
                g.command.retain(|x| *x != top);
                g.command.push(top);
            }
        }
    }
    entered
}

pub struct OpenAttraction;

impl KeywordActionRules for OpenAttraction {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::OpenAttraction]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx).max(1);
        for p in g.eval_players(a.who, ctx) {
            for _ in 0..n {
                open_attraction(g, p, ctx.source);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&OpenAttraction) }
