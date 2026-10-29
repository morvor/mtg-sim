//! CR 701.31: planeswalk (the process itself is `planechase::planeswalk`).
//!
//! * Only the planar controller of a Planechase game may planeswalk (CR 701.31a).
//! * Each face-up plane and phenomenon card goes to the bottom of its owner's planar deck
//!   face down, then the player turns the top card of their planar deck face up
//!   (CR 701.31b); it happens through the planeswalking ability, a phenomenon's triggered
//!   ability leaving the stack, or an instruction to planeswalk (CR 701.31c).
//! * The card turned face up is the plane the player planeswalks to; each card turned face
//!   down is one they planeswalk away from (CR 701.31d): "When you planeswalk to [this
//!   plane]" triggers on the `planechase::PLANESWALKED` event about it, and "When you
//!   planeswalk away from [this plane]" ([`PLANESWALKED_AWAY`]) looks back in time
//!   (CR 603.10g): it's triggered by [`planeswalking_away`] before the card is turned face
//!   down, when its abilities still exist.

use super::*;
use crate::game::PendingTrigger;
use crate::object::EventInfo;

/// `TriggerCond::Custom` name: "When you planeswalk away from [this plane]".
pub const PLANESWALKED_AWAY: &str = "planeswalk away from this";

/// Called as `p` planeswalks, before the face-up planar cards `away` are turned face down:
/// their "When you planeswalk away from [this]" abilities trigger (CR 603.10g, 701.31d).
pub fn planeswalking_away(g: &mut Game, p: PlayerId, away: &[ObjectId]) {
    for &card in away {
        let abilities: Vec<Ability> = g
            .obj(card)
            .chars
            .abilities
            .iter()
            .filter(|a| {
                matches!(&a.kind, AbilityKind::Triggered(t)
                    if matches!(&t.trigger, TriggerCond::Custom(n) if n.as_str() == PLANESWALKED_AWAY))
            })
            .cloned()
            .collect();
        for ability in abilities {
            g.trigger_order += 1;
            let order = g.trigger_order;
            g.pending_triggers.push(PendingTrigger {
                source: card,
                controller: p,
                ability,
                event: EventInfo {
                    object: Some(card),
                    player: Some(p),
                    ..Default::default()
                },
                source_lki: Some(Box::new(g.obj(card).chars.clone())),
                saved: None,
                body: None,
                order,
            });
        }
    }
}
