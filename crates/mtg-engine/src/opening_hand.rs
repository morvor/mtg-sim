//! Actions taken with cards from opening hands (CR 103.6): beginning the game with a card
//! on the battlefield, and revealing a card to create a delayed triggered ability whose
//! source is that card (CR 603.7g).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::{DelayedTrigger, Game};
use crate::object::Zone;
use crate::types::PlayerId;

/// Once mulligans are complete, the starting player may take any such actions, then each
/// other player in turn order (CR 103.6).
pub fn opening_hand_actions(g: &mut Game) {
    for p in g.apnap() {
        opening_hand_actions_for(g, p);
    }
    g.flush_events();
}

/// The opening-hand actions available to one player.
pub fn opening_hand_actions_for(g: &mut Game, p: PlayerId) {
    for card in g.zone_objects(Zone::Hand(p)) {
        if g.obj(card).zone != Zone::Hand(p) {
            continue;
        }
        let actions: Vec<Option<Box<(TriggerCond, Body)>>> = g
            .obj(card)
            .chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Static(s) => match &s.effect {
                    StaticEffect::OpeningHand { delayed } => Some(delayed.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for delayed in actions {
            let name = g.obj(card).chars.name.clone();
            match delayed {
                None => {
                    let prompt = format!("Begin the game with {name} on the battlefield?");
                    if g.ask_yes_no(p, Some(card), &prompt, true) {
                        g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(p));
                        break;
                    }
                }
                Some(d) => {
                    // CR 103.6b: each card may be revealed this way only once.
                    let prompt = format!("Reveal {name} from your opening hand?");
                    if !g.ask_yes_no(p, Some(card), &prompt, true) {
                        continue;
                    }
                    g.emit(Event::Custom {
                        name: "revealed".into(),
                        player: Some(p),
                        obj: Some(card),
                        amount: 0,
                    });
                    let (trigger, body) = *d;
                    // CR 603.7g: the source is the card with the static ability; its
                    // controller is the player who took the action.
                    let id = g.new_effect_id();
                    g.delayed_triggers.push(DelayedTrigger {
                        id,
                        source: Some(card),
                        controller: p,
                        trigger,
                        body,
                        once: true,
                        ctx: Ctx::new(Some(card), p),
                        created_turn: g.turn.number,
                        created_step: None,
                    });
                }
            }
        }
    }
}
