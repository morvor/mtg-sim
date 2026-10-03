//! The monarch (CR 725) and the initiative (CR 726): the designations' inherent
//! triggered abilities, and who gets a designation when the player who has it leaves the
//! game.
//!
//! The inherent triggered abilities have no source and are controlled by the player who
//! had the designation when they triggered (an exception to CR 113.8). A placeholder
//! object that is in no zone stands in for the missing source.

use crate::ability::*;
use crate::events::Event;
use crate::game::{Game, PendingTrigger};
use crate::object::*;
use crate::turn::Step;
use crate::types::*;
use smol_str::SmolStr;
use std::collections::BTreeMap;

/// Name of the monarch's "At the beginning of the monarch's end step, that player draws a
/// card" (CR 725.2).
pub const MONARCH_DRAW: &str = "The monarch (end step)";
/// Name of the monarch's "Whenever a creature deals combat damage to the monarch, its
/// controller becomes the monarch" (CR 725.2).
pub const MONARCH_STEAL: &str = "The monarch (combat damage)";
/// Name of the initiative's "At the beginning of the upkeep of the player who has the
/// initiative, that player ventures into Undercity" (CR 726.2).
pub const INITIATIVE_UPKEEP: &str = "The initiative (upkeep)";
/// Name of the initiative's "Whenever one or more creatures a player controls deal combat
/// damage to the player who has the initiative, the controller of those creatures takes
/// the initiative" (CR 726.2).
pub const INITIATIVE_STEAL: &str = "The initiative (combat damage)";
/// Name of the initiative's "Whenever a player takes the initiative, that player ventures
/// into Undercity" (CR 726.2).
pub const INITIATIVE_TAKEN: &str = "The initiative (taken)";

/// The dungeon the initiative ventures into (CR 726.2).
const UNDERCITY: &str = "Undercity";

/// Puts a triggered ability that has no source into the pending list (CR 725.2, 726.2):
/// `controller` controls it. A placeholder object in no zone stands in for its source.
pub fn sourceless_trigger(
    g: &mut Game,
    name: &str,
    controller: PlayerId,
    text: &str,
    body: Body,
    event: EventInfo,
) {
    let tr = TriggeredAbility::new(TriggerCond::Custom(SmolStr::new(name)), body);
    let ability = AbilityDef::new(AbilityKind::Triggered(tr), text);
    let chars = Characteristics {
        name: SmolStr::new(name),
        rules_text: std::sync::Arc::from(""),
        ..Default::default()
    };
    let mut obj = GameObject::new(
        ObjectId(0),
        ObjKind::Emblem,
        controller,
        Zone::Nowhere,
        chars,
    );
    obj.controller = controller;
    let src = ObjectId(g.objects.len() as u32);
    obj.id = src;
    obj.timestamp = g.new_timestamp();
    g.objects.push(obj);
    g.trigger_order += 1;
    let order = g.trigger_order;
    g.pending_triggers.push(PendingTrigger {
        source: src,
        controller,
        ability,
        event,
        source_lki: None,
        saved: None,
        body: None,
        order,
    });
}

/// Whether the triggered ability with this source is one of the designations' inherent
/// abilities named `name`.
pub fn is_inherent(g: &Game, src: ObjectId, name: &str) -> bool {
    let o = g.obj(src);
    o.zone == Zone::Nowhere && o.kind == ObjKind::Emblem && o.chars.name == name
}

fn venture_into_undercity() -> Body {
    Body::effect(crate::kwa::venture::venture_into(UNDERCITY))
}

/// Whether `p` is taking the current turn (with shared team turns, any player of the
/// active team, CR 805.4).
fn is_their_turn(g: &Game, p: PlayerId) -> bool {
    g.active_players().contains(&p)
}

/// The creature that dealt combat damage to a player in `ev`: (creature, damaged player,
/// the creature's controller).
fn combat_damage_to_player(g: &Game, ev: &Event) -> Option<(ObjectId, PlayerId, PlayerId)> {
    let Event::Damage {
        source,
        target: Entity::Player(p),
        amount,
        combat: true,
    } = ev
    else {
        return None;
    };
    let o = g.obj(*source);
    (*amount > 0 && o.is_creature()).then_some((*source, *p, o.controller))
}

/// Detects the inherent triggered abilities of the monarch (CR 725.2) and the initiative
/// (CR 726.2) that trigger on one event.
pub fn detect(g: &mut Game, ev: &Event) {
    match ev {
        Event::StepBegan {
            step: Step::End, ..
        } => {
            // "At the beginning of the monarch's end step, that player draws a card."
            if let Some(m) = g.monarch.filter(|m| is_their_turn(g, *m)) {
                sourceless_trigger(
                    g,
                    MONARCH_DRAW,
                    m,
                    "At the beginning of the monarch's end step, that player draws a card.",
                    Body::effect(Effect::Draw {
                        who: PlayerRef::You,
                        n: Value::c(1),
                    }),
                    EventInfo {
                        player: Some(m),
                        ..Default::default()
                    },
                );
            }
        }
        Event::StepBegan {
            step: Step::Upkeep, ..
        } => {
            // "At the beginning of the upkeep of the player who has the initiative, that
            // player ventures into Undercity."
            if let Some(i) = g.initiative.filter(|i| is_their_turn(g, *i)) {
                sourceless_trigger(
                    g,
                    INITIATIVE_UPKEEP,
                    i,
                    "At the beginning of the upkeep of the player who has the initiative, that player ventures into Undercity.",
                    venture_into_undercity(),
                    EventInfo {
                        player: Some(i),
                        ..Default::default()
                    },
                );
            }
        }
        Event::TookInitiative { player } => {
            // "Whenever a player takes the initiative, that player ventures into
            // Undercity" — also when they already had it (CR 726.5).
            sourceless_trigger(
                g,
                INITIATIVE_TAKEN,
                *player,
                "Whenever a player takes the initiative, that player ventures into Undercity.",
                venture_into_undercity(),
                EventInfo {
                    player: Some(*player),
                    ..Default::default()
                },
            );
        }
        _ => {
            // "Whenever a creature deals combat damage to the monarch, its controller
            // becomes the monarch."
            if let Some((creature, damaged, controller)) = combat_damage_to_player(g, ev) {
                if g.monarch == Some(damaged) {
                    sourceless_trigger(
                        g,
                        MONARCH_STEAL,
                        damaged,
                        "Whenever a creature deals combat damage to the monarch, its controller becomes the monarch.",
                        Body::effect(Effect::BecomeMonarch {
                            who: PlayerRef::TriggerPlayer,
                        }),
                        EventInfo {
                            object: Some(creature),
                            player: Some(controller),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }
}

/// Detects the initiative's "whenever one or more creatures a player controls deal combat
/// damage to the player who has the initiative" for a batch of simultaneous events: it
/// triggers once for each player whose creatures dealt such damage (CR 726.2, 603.2c).
pub fn detect_batch(g: &mut Game, batch: &[Event]) {
    let Some(holder) = g.initiative else {
        return;
    };
    let mut by_controller: BTreeMap<PlayerId, Vec<ObjectId>> = BTreeMap::new();
    for ev in batch {
        if let Some((creature, damaged, controller)) = combat_damage_to_player(g, ev) {
            if damaged == holder {
                let v = by_controller.entry(controller).or_default();
                if !v.contains(&creature) {
                    v.push(creature);
                }
            }
        }
    }
    for (controller, creatures) in by_controller {
        sourceless_trigger(
            g,
            INITIATIVE_STEAL,
            holder,
            "Whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the controller of those creatures takes the initiative.",
            Body::effect(Effect::TakeInitiative {
                who: PlayerRef::TriggerPlayer,
            }),
            EventInfo {
                object: creatures.first().copied(),
                objects: creatures,
                player: Some(controller),
                ..Default::default()
            },
        );
    }
}

/// The player who gets a designation when `leaving`, who has it, leaves the game: the
/// active player, or — if the active player is the one leaving or there is no active
/// player — the next player in turn order still in the game (CR 725.4, 726.4). `None` if
/// no player is left.
fn successor(g: &Game, leaving: PlayerId) -> Option<PlayerId> {
    let active = g.turn.active;
    if active != leaving && g.player(active).in_game() {
        return Some(active);
    }
    let n = g.players.len();
    (1..=n)
        .map(|i| PlayerId(((active.idx() + i) % n) as u8))
        .find(|q| *q != leaving && g.player(*q).in_game())
}

/// A player left the game (CR 800.4a): if they were the monarch, the active player (or
/// the next player in turn order) becomes the monarch at the same time, and likewise for
/// the initiative (CR 725.4, 726.4). If no player is left who can get it, the game
/// continues without it.
pub fn player_left(g: &mut Game, p: PlayerId) {
    if g.monarch == Some(p) {
        g.monarch = None;
        if let Some(next) = successor(g, p) {
            crate::designations::become_monarch(g, next);
        }
    }
    if g.initiative == Some(p) {
        g.initiative = None;
        if let Some(next) = successor(g, p) {
            crate::designations::take_initiative(g, next);
        }
    }
}
