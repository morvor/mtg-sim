//! Dungeons (CR 309) and the venture into the dungeon keyword action (CR 701.49).
//!
//! A dungeon card's rooms are compiled into room abilities (CR 309.4c) whose trigger is
//! `TriggerCond::Custom("room:I>J,K")`: room I, whose arrows lead to rooms J and K (none
//! for the bottommost room). A player's venture marker is `Player::venture`.

use crate::ability::*;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::object::{EventInfo, GameObject, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a player moved their venture marker into a room (CR 309.4c). The
/// object is the dungeon and the amount is the room's index.
pub const ENTERED_ROOM: &str = "venture into room";
/// `Event::Custom` name: a player completed a dungeon (CR 309.7). The object is the
/// dungeon card as it last existed in the command zone.
pub const COMPLETED: &str = "complete dungeon";
/// Prefix of room abilities' trigger names.
pub const ROOM: &str = "room:";
/// The dungeons a player who owns no dungeon cards chooses from when they venture into
/// the dungeon: every player has access to these (CR 309.2a).
pub const STANDARD_DUNGEONS: [&str; 3] = [
    "Lost Mine of Phandelver",
    "Dungeon of the Mad Mage",
    "Tomb of Annihilation",
];

/// The trigger name of room `room`, whose arrows lead to `leads`.
pub fn room_trigger_name(room: usize, leads: &[usize]) -> String {
    let leads: Vec<String> = leads.iter().map(|l| l.to_string()).collect();
    format!("{ROOM}{room}>{}", leads.join(","))
}

/// Parses a room trigger name into (room, leads to).
fn parse_room(name: &str) -> Option<(usize, Vec<usize>)> {
    let (room, leads) = name.strip_prefix(ROOM)?.split_once('>')?;
    let leads = leads
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().ok())
        .collect::<Option<Vec<usize>>>()?;
    Some((room.parse().ok()?, leads))
}

/// Splits the compiler's mark off a room line: "{room:0>1,2} Cave Entrance — Scry 1." →
/// ("room:0>1,2", "Cave Entrance — Scry 1.").
pub fn split_room_mark(block: &str) -> Option<(&str, &str)> {
    let (mark, text) = block.strip_prefix('{')?.split_once("} ")?;
    parse_room(mark)?;
    Some((mark, text))
}

/// A dungeon's rooms from its room abilities: (room, rooms its arrows lead to). A room
/// whose effect the compiler doesn't support still counts as a room of the map.
pub fn rooms(o: &GameObject) -> Vec<(usize, Vec<usize>)> {
    let abilities = o
        .card
        .as_ref()
        .map(|c| &c.front().chars.abilities)
        .unwrap_or(&o.chars.abilities);
    let mut out: Vec<(usize, Vec<usize>)> = abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Triggered(t) => match &t.trigger {
                TriggerCond::Custom(n) => parse_room(n),
                _ => None,
            },
            AbilityKind::Unsupported(text) => parse_room(split_room_mark(text)?.0),
            _ => None,
        })
        .collect();
    out.sort();
    out
}

fn is_dungeon_card(o: &GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    chars.card_types.contains(CardType::Dungeon)
}

/// The dungeon card `p` owns in the command zone and their venture marker's room on it
/// (CR 309.3, 309.4).
pub fn marker(g: &Game, p: PlayerId) -> Option<(ObjectId, usize)> {
    let (d, room) = g.player(p).venture?;
    let o = g.obj(d);
    (g.is_live(d) && o.zone == Zone::Command && o.owner == p).then_some((d, room))
}

/// Whether the venture marker is on a dungeon's bottommost room (no arrows lead away).
fn on_bottommost_room(g: &Game, d: ObjectId, room: usize) -> bool {
    rooms(g.obj(d))
        .iter()
        .find(|(r, _)| *r == room)
        .is_none_or(|(_, leads)| leads.is_empty())
}

/// Moves `p`'s venture marker into a room; its room ability triggers (CR 309.4c).
fn enter_room(g: &mut Game, p: PlayerId, d: ObjectId, room: usize) {
    g.players[p.idx()].venture = Some((d, room));
    g.emit(Event::Custom {
        name: SmolStr::new(ENTERED_ROOM),
        player: Some(p),
        obj: Some(d),
        amount: room as i32,
    });
}

/// The dungeon cards `p` owns outside the game that they may venture into: those named
/// `name` when venturing into [quality] (CR 701.49d), otherwise those that don't say
/// they can be entered only that way (Undercity, see `kwa/venture.rs`). A player who owns
/// none uses the standard dungeons, or the named one (they're created outside the game the
/// first time they're needed).
fn dungeons_outside(g: &mut Game, p: PlayerId, name: Option<&str>) -> Vec<ObjectId> {
    let owned = |g: &Game| -> Vec<ObjectId> {
        g.player(p)
            .sideboard
            .iter()
            .copied()
            .filter(|id| {
                let o = g.obj(*id);
                is_dungeon_card(o)
                    && match name {
                        Some(n) => o.chars.name.eq_ignore_ascii_case(n),
                        None => !crate::kwa::venture::only_by_venturing_into_it(o),
                    }
            })
            .collect()
    };
    let have = owned(g);
    if !have.is_empty() {
        return have;
    }
    let names: Vec<&str> = match name {
        Some(n) => vec![n],
        None => STANDARD_DUNGEONS.to_vec(),
    };
    for name in names {
        if let Some(card) = crate::card::CardDb::global().get(name) {
            let id = g.create_card_object(card, p, Zone::Outside(p));
            g.players[p.idx()].sideboard.push(id);
        }
    }
    owned(g)
}

/// `p` chooses a dungeon card they own from outside the game, puts it into the command
/// zone, and puts their venture marker on its topmost room (CR 309.2a, 309.4a).
fn bring_in_dungeon(
    g: &mut Game,
    p: PlayerId,
    source: Option<ObjectId>,
    name: Option<&str>,
) -> bool {
    let options = dungeons_outside(g, p, name);
    if options.is_empty() {
        return false;
    }
    let names: Vec<String> = options
        .iter()
        .map(|id| g.obj(*id).chars.name.to_string())
        .collect();
    let pick = if options.len() == 1 {
        0
    } else {
        g.ask_option(p, source, "Choose a dungeon", names)
            .min(options.len() - 1)
    };
    let Some(d) = g.move_object(options[pick], Zone::Command, MoveCause::Venture, Some(p)) else {
        return false;
    };
    g.recompute();
    enter_room(g, p, d, 0);
    true
}

/// Removes a dungeon card from the game: its owner completes it (CR 309.7).
fn complete(g: &mut Game, p: PlayerId, d: ObjectId) {
    g.players[p.idx()].venture = None;
    g.log(|g| format!("{p} completes {}", g.obj(d).chars.name));
    g.move_object(d, Zone::Outside(p), MoveCause::StateBased, Some(p));
    g.players[p.idx()].dungeons_completed += 1;
    // "As long as you've completed a dungeon" statics change (CR 611.3a).
    g.dirty = true;
    g.emit(Event::Custom {
        name: SmolStr::new(COMPLETED),
        player: Some(p),
        obj: Some(d),
        amount: 0,
    });
}

/// Ventures into the dungeon (CR 701.49a–c): enter the first room of a new dungeon, or
/// advance to the next room (the player chooses when several arrows lead away), or — from
/// the bottommost room — complete the dungeon and enter the first room of a new one.
pub fn venture(g: &mut Game, p: PlayerId, source: Option<ObjectId>) {
    venture_into(g, p, source, None);
}

/// Ventures into the dungeon, or with `name`, "venture into [name]" (CR 701.49d): a new
/// dungeon must be the named one.
pub fn venture_into(g: &mut Game, p: PlayerId, source: Option<ObjectId>, name: Option<&str>) {
    let Some((d, room)) = marker(g, p) else {
        bring_in_dungeon(g, p, source, name);
        return;
    };
    let leads = rooms(g.obj(d))
        .into_iter()
        .find(|(r, _)| *r == room)
        .map(|(_, l)| l)
        .unwrap_or_default();
    if leads.is_empty() {
        // CR 701.49c.
        complete(g, p, d);
        bring_in_dungeon(g, p, source, name);
        return;
    }
    // CR 701.49b: choose an arrow to follow.
    let next = if leads.len() == 1 {
        leads[0]
    } else {
        let o = g.obj(d);
        let names: Vec<String> = leads
            .iter()
            .map(|l| room_label(o, *l).unwrap_or_else(|| format!("Room {l}")))
            .collect();
        leads[g
            .ask_option(p, source, "Choose the next room", names)
            .min(leads.len() - 1)]
    };
    enter_room(g, p, d, next);
}

/// A room's printed text (its name and effect), for choices.
fn room_label(o: &GameObject, room: usize) -> Option<String> {
    let card = o.card.as_ref()?;
    card.front()
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Triggered(t) => match &t.trigger {
                TriggerCond::Custom(n) if parse_room(n).is_some_and(|(r, _)| r == room) => {
                    Some(a.text.to_string())
                }
                _ => None,
            },
            AbilityKind::Unsupported(text) => {
                let (mark, text) = split_room_mark(text)?;
                (parse_room(mark)?.0 == room).then(|| text.to_string())
            }
            _ => None,
        })
}

/// CR 704.5t, 309.6: if a player's venture marker is on the bottommost room of a dungeon
/// card, and that dungeon isn't the source of a room ability that has triggered but not
/// yet left the stack, the dungeon card's owner removes it from the game.
pub fn completion_sba(g: &mut Game) -> bool {
    let mut performed = false;
    for p in g.player_ids() {
        let Some((d, room)) = marker(g, p) else {
            continue;
        };
        if on_bottommost_room(g, d, room) && !g.is_source_of_stack_trigger(d) {
            complete(g, p, d);
            performed = true;
        }
    }
    performed
}

/// Room abilities: "When you move your venture marker into this room, [effect]"
/// (CR 309.4c).
pub fn custom_trigger(g: &Game, name: &str, src: ObjectId, ev: &Event) -> Option<Vec<EventInfo>> {
    let (room, _) = parse_room(name)?;
    let _ = g;
    Some(match ev {
        Event::Custom {
            name: n,
            player,
            obj: Some(d),
            amount,
        } if n.as_str() == ENTERED_ROOM && *d == src && *amount as usize == room => {
            vec![EventInfo {
                object: Some(src),
                player: *player,
                amount: *amount,
                ..Default::default()
            }]
        }
        _ => vec![],
    })
}
