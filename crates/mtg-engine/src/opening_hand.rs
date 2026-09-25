//! Actions taken with cards from opening hands (CR 103.6): beginning the game with a card
//! on the battlefield, and revealing a card to create a delayed triggered ability whose
//! source is that card (CR 603.7g).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::{DelayedTrigger, Game};
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};

/// "Before you shuffle your deck to start the game, you may reveal this card from your
/// deck and exile [a card] you drafted that isn't in your deck" (CR 607.2n): the exiled
/// card is linked to the abilities of cards with that card's name.
pub fn before_shuffle_actions(g: &mut Game) {
    pregame_choices(g);
    for p in g.player_ids() {
        for card in g.player(p).library.clone() {
            let filters: Vec<Filter> = g
                .obj(card)
                .base
                .abilities
                .iter()
                .filter_map(|a| match &a.kind {
                    AbilityKind::Static(s) => match &s.effect {
                        StaticEffect::BeforeShuffleExile { what } => Some(what.clone()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            for f in filters {
                let name = g.obj(card).base.name.clone();
                let ctx = Ctx::new(Some(card), p);
                let cands: Vec<ObjectId> = g
                    .player(p)
                    .sideboard
                    .clone()
                    .into_iter()
                    .filter(|c| g.matches(*c, &f, &ctx))
                    .collect();
                if cands.is_empty()
                    || !g.ask_yes_no(
                        p,
                        Some(card),
                        &format!("Reveal {name} from your deck?"),
                        true,
                    )
                {
                    continue;
                }
                let pick = g.ask_objects(p, Some(card), "Choose a card to exile", cands, 1, 1);
                for c in pick {
                    if let Some(e) = g.move_object(c, Zone::Exile, MoveCause::Effect, Some(p)) {
                        g.named_exiles.push((p, name.clone(), e));
                    }
                }
            }
        }
    }
    g.events.clear();
}

/// Choices made before the game begins for characteristic-defining abilities (CR 607.2p),
/// e.g. "If this card is your commander, choose a color before the game begins."
pub fn pregame_choices(g: &mut Game) {
    let ids: Vec<ObjectId> = (0..g.objects.len() as u32)
        .map(ObjectId)
        .filter(|id| g.is_live(*id))
        .collect();
    for id in ids {
        let o = g.obj(id);
        let owner = o.owner;
        let is_commander = o.is_commander;
        let kinds: Vec<ChoiceKind> = o
            .base
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Static(s) => match &s.effect {
                    StaticEffect::PregameChoice {
                        kind,
                        only_if_commander,
                    } if is_commander || !*only_if_commander => Some(kind.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for kind in kinds {
            let mut ctx = Ctx::new(Some(id), owner);
            ctx.link = PREGAME_LINK;
            crate::choices::make_choice(g, owner, &kind, &mut ctx);
        }
    }
}

/// Once mulligans are complete, the starting player may take any such actions, then each
/// other player in turn order (CR 103.6); with shared team turns, each player on the
/// starting team, then each player on each other team in turn order (CR 103.6c).
pub fn opening_hand_actions(g: &mut Game) {
    for p in g.pregame_order() {
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
