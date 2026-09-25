//! Planechase (CR 901): planar decks, the planar controller, planeswalking (CR 701.31),
//! and the planar die with its Planeswalker and chaos symbols (CR 107.11, 107.12).
//!
//! Plane and phenomenon cards stay in the command zone all game (CR 311.2, 312.2). A
//! player's planar deck is the face-down plane and phenomenon cards they own in the
//! command zone, in command-zone order (the first is the top); with the single planar
//! deck option (CR 901.15) all of them form one communal deck.

use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Action, SpecialAction};
use crate::events::Event;
use crate::game::{Game, PendingTrigger, Variant};
use crate::object::{EventInfo, GameObject, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: chaos ensues (CR 311.7). "Whenever chaos ensues" triggers.
pub const CHAOS_ENSUES: &str = "chaos ensues";
/// `Event::Custom` name: a player rolled the planar die (CR 901.9).
pub const ROLLED_PLANAR_DIE: &str = "roll planar die";
/// `Event::Custom` name: a player planeswalked; the object is the plane planeswalked to
/// (CR 901.11).
pub const PLANESWALKED: &str = "planeswalk";
/// `SpecialAction::Other` name of rolling the planar die (CR 901.9, 116.2i).
pub const PLANAR_DIE_ACTION: &str = "roll the planar die";
/// `Effect::Custom` name: the planar controller planeswalks (CR 701.31).
pub const PLANESWALK_EFFECT: &str = "planeswalk";
/// `Effect::Custom` name: the controller rolls the planar die because of an effect (not
/// the special action, CR 116.2i).
pub const ROLL_PLANAR_DIE_EFFECT: &str = "roll the planar die (effect)";

/// A face of the planar die (CR 901.3a): one Planeswalker symbol {PW} (CR 107.11), one
/// chaos symbol {CHAOS} (CR 107.12), four blank faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanarFace {
    Blank,
    Chaos,
    Planeswalker,
}

/// The face shown by a roll of 1–6.
pub fn face_for(roll: u32) -> PlanarFace {
    match roll {
        1 => PlanarFace::Planeswalker,
        2 => PlanarFace::Chaos,
        _ => PlanarFace::Blank,
    }
}

pub fn is_planechase(g: &Game) -> bool {
    g.config.variant == Variant::Planechase
}

/// Whether an object is a plane or phenomenon card (by its card, even while face down).
fn is_planar_card(o: &GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.card_types.contains(CardType::Plane) || chars.card_types.contains(CardType::Phenomenon)
}

/// The planar controller (CR 901.6): normally the active player; if they've left the
/// game, the next player in turn order still in it.
pub fn planar_controller(g: &Game) -> Option<PlayerId> {
    if !is_planechase(g) {
        return None;
    }
    let a = g.turn.active;
    Some(if g.player(a).in_game() {
        a
    } else {
        g.next_player(a)
    })
}

/// A player's planar deck, top card first (the communal deck with the single planar deck
/// option, CR 901.15c).
pub fn planar_deck(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            o.face_down && is_planar_card(o) && (g.config.single_planar_deck || o.owner == p)
        })
        .collect()
}

/// The face-up plane and phenomenon cards.
pub fn face_up_planar_cards(g: &Game) -> Vec<ObjectId> {
    g.command
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            !o.face_down && is_planar_card(o)
        })
        .collect()
}

/// Called as characteristics are computed: the planar controller controls each face-up
/// plane and phenomenon card (CR 901.6, 311.5, 312.4); with the single planar deck option
/// they're also considered the owner of every card in the planar deck (CR 901.15b).
pub fn apply_planar_control(g: &mut Game) {
    let Some(pc) = planar_controller(g) else {
        return;
    };
    let single = g.config.single_planar_deck;
    for id in g.command.clone() {
        let o = &g.objects[id.0 as usize];
        if !is_planar_card(o) {
            continue;
        }
        let face_up = !o.face_down;
        let o = &mut g.objects[id.0 as usize];
        if face_up {
            o.controller = pc;
        }
        if single {
            o.owner = pc;
            o.base_controller = pc;
            if face_up {
                o.controller = pc;
            }
        }
    }
}

/// Turns the top card of `p`'s planar deck face up (it receives a new timestamp, CR
/// 613.7h). Returns it.
fn turn_top_face_up(g: &mut Game, p: PlayerId) -> Option<ObjectId> {
    let top = *planar_deck(g, p).first()?;
    crate::variants::turn_face_up_in_command(g, top);
    g.recompute();
    Some(top)
}

/// Puts a card on the bottom of its owner's planar deck face down. A face-up plane or
/// phenomenon turned face down becomes a new object (CR 311.6, 312.6, 901.7a).
fn to_bottom_face_down(g: &mut Game, id: ObjectId) {
    let new = if g.obj(id).face_down {
        id
    } else {
        let new = g.create_incarnation(id, Zone::Command);
        g.objects[new.0 as usize].face_down = true;
        new
    };
    g.command.retain(|x| *x != id);
    g.command.push(new);
    g.dirty = true;
}

/// Shuffles each planar deck (CR 103.3a): the face-down planar cards of each deck are put
/// in a random order among their places in the command zone.
pub fn shuffle_planar_decks(g: &mut Game) {
    use rand::seq::SliceRandom;
    if !is_planechase(g) {
        return;
    }
    let owners: Vec<Option<PlayerId>> = if g.config.single_planar_deck {
        vec![None]
    } else {
        g.player_ids().into_iter().map(Some).collect()
    };
    for owner in owners {
        let slots: Vec<usize> = g
            .command
            .iter()
            .enumerate()
            .filter(|(_, id)| {
                let o = g.obj(**id);
                o.face_down && is_planar_card(o) && owner.is_none_or(|p| o.owner == p)
            })
            .map(|(i, _)| i)
            .collect();
        let mut ids: Vec<ObjectId> = slots.iter().map(|i| g.command[*i]).collect();
        ids.shuffle(&mut g.rng);
        for (i, id) in slots.into_iter().zip(ids) {
            g.command[i] = id;
        }
    }
}

/// Sets the starting plane (CR 901.5): the starting player turns the top card of their
/// planar deck face up; phenomena go to the bottom until a plane is turned face up. No
/// abilities trigger during this process.
pub fn set_starting_plane(g: &mut Game) {
    if !is_planechase(g) || !face_up_planar_cards(g).is_empty() {
        return;
    }
    let p = g.turn.starting_player;
    for _ in 0..planar_deck(g, p).len() {
        let Some(top) = turn_top_face_up(g, p) else {
            return;
        };
        if g.obj(top).chars.card_types.contains(CardType::Plane) {
            return;
        }
        to_bottom_face_down(g, top);
    }
}

/// Planeswalks (CR 701.31b): each face-up plane and phenomenon card goes to the bottom of
/// its owner's planar deck face down, then `p` turns the top card of their planar deck
/// face up. Only the planar controller may planeswalk (CR 701.31a).
pub fn planeswalk(g: &mut Game, p: PlayerId) {
    if planar_controller(g) != Some(p) {
        return;
    }
    for id in face_up_planar_cards(g) {
        to_bottom_face_down(g, id);
    }
    g.recompute();
    if let Some(new) = turn_top_face_up(g, p) {
        g.emit(Event::Custom {
            name: SmolStr::new(PLANESWALKED),
            player: Some(p),
            obj: Some(new),
            amount: 0,
        });
    }
}

/// How many times `p` has rolled the planar die as a special action this turn.
fn rolls_this_turn(g: &Game, p: PlayerId) -> usize {
    g.turn_events
        .iter()
        .filter(|e| {
            matches!(e, Event::Custom { name, player: Some(q), .. }
                if name.as_str() == PLANAR_DIE_ACTION && *q == p)
        })
        .count()
}

/// Whether `p` may roll the planar die now (CR 901.9): the active player, with priority,
/// with an empty stack, in a main phase of their turn.
fn may_roll(g: &Game, p: PlayerId) -> bool {
    is_planechase(g)
        && g.turn.active == p
        && g.turn.priority == Some(p)
        && g.stack.is_empty()
        && g.turn.step.is_main()
}

/// The special action of rolling the planar die, when available (CR 901.9).
pub fn special_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    if !may_roll(g, p) {
        return vec![];
    }
    // Only if its cost can be paid.
    let n = rolls_this_turn(g, p) as u32;
    let cost = Cost::mana(crate::mana::ManaCost::generic(n));
    if n > 0 && !g.can_pay_cost(p, &cost, None, &crate::eval::Ctx::new(None, p)) {
        return vec![];
    }
    vec![Action::Special(SpecialAction::Other {
        name: PLANAR_DIE_ACTION.into(),
        obj: None,
    })]
}

/// Rolls the planar die as a special action (CR 901.9): it costs {N}, where N is the
/// number of times the player has already done so this turn.
pub fn perform_special_action(
    g: &mut Game,
    p: PlayerId,
    sa: &SpecialAction,
) -> Option<Result<(), Illegal>> {
    let SpecialAction::Other { name, .. } = sa else {
        return None;
    };
    if name.as_str() != PLANAR_DIE_ACTION {
        return None;
    }
    if !may_roll(g, p) {
        return Some(Err(Illegal("can't roll the planar die now".into())));
    }
    let n = rolls_this_turn(g, p) as u32;
    if n > 0 {
        let cost = Cost::mana(crate::mana::ManaCost::generic(n));
        let ctx = crate::eval::Ctx::new(None, p);
        if !g.pay_cost(p, &cost, None, &ctx) {
            return Some(Err(Illegal("can't pay to roll the planar die".into())));
        }
    }
    g.emit(Event::Custom {
        name: SmolStr::new(PLANAR_DIE_ACTION),
        player: Some(p),
        obj: None,
        amount: n as i32,
    });
    roll_planar_die(g, p);
    Some(Ok(()))
}

/// Rolls the planar die (CR 901.9a–c): on the chaos symbol, chaos ensues; on the
/// Planeswalker symbol, the planeswalking ability triggers; a blank does nothing.
pub fn roll_planar_die(g: &mut Game, p: PlayerId) -> PlanarFace {
    let face = face_for(g.random_range(1, 6));
    g.log(|_| format!("{p} rolls the planar die: {face:?}"));
    g.emit(Event::Custom {
        name: SmolStr::new(ROLLED_PLANAR_DIE),
        player: Some(p),
        obj: None,
        amount: 0,
    });
    match face {
        PlanarFace::Blank => {}
        PlanarFace::Chaos => {
            g.emit(Event::Custom {
                name: SmolStr::new(CHAOS_ENSUES),
                player: Some(p),
                obj: None,
                amount: 0,
            });
        }
        PlanarFace::Planeswalker => planeswalking_ability_triggers(g, p),
    }
    g.flush_events();
    face
}

/// The inherent "planeswalking ability" (CR 901.8): "Whenever you roll the Planeswalker
/// symbol on the planar die, planeswalk." It's controlled by the player whose roll caused
/// it to trigger; it's recorded as coming from the face-up plane.
fn planeswalking_ability_triggers(g: &mut Game, p: PlayerId) {
    let Some(src) = face_up_planar_cards(g)
        .first()
        .copied()
        .or_else(|| planar_deck(g, p).first().copied())
    else {
        return;
    };
    let mut t = TriggeredAbility::new(
        TriggerCond::Custom(SmolStr::new("planeswalking ability")),
        Body::effect(Effect::Custom(SmolStr::new(PLANESWALK_EFFECT))),
    );
    t.zone = FunctionZone::Command;
    let ability = AbilityDef::new(
        AbilityKind::Triggered(t),
        "Whenever you roll the Planeswalker symbol on the planar die, planeswalk.",
    );
    g.trigger_order += 1;
    g.pending_triggers.push(PendingTrigger {
        source: src,
        controller: p,
        ability,
        event: EventInfo {
            player: Some(p),
            ..Default::default()
        },
        source_lki: None,
        saved: None,
        body: None,
        order: g.trigger_order,
    });
}
