//! Rooms: split permanent cards with a shared type line (CR 709.5). Each half ("door",
//! CR 709.5j) is locked unless the permanent has that half's unlocked designation
//! (CR 709.5c); a locked half's name, mana cost and rules text don't exist on the
//! battlefield (CR 709.5). A half cast as a spell enters unlocked (CR 709.5d), a player
//! may pay a locked half's mana cost to unlock it as a special action (CR 116.2m,
//! 709.5e), and abilities trigger when a door is unlocked (CR 709.5h) or a Room is fully
//! unlocked (CR 709.5i).

use crate::card::{CardDef, Layout};
use crate::casting::Illegal;
use crate::decision::{Action, SpecialAction};
use crate::events::Event;
use crate::game::Game;
use crate::mana::ManaCost;
use crate::object::{Characteristics, EventInfo, FaceState, Zone};
use crate::types::*;
use smol_str::SmolStr;
use std::collections::BTreeMap;
use std::sync::Arc;

/// `Event::Custom` name: a door of a Room was unlocked. `obj` is the Room, `amount` the
/// half (0 = left, 1 = right), `player` the player who unlocked it.
pub const DOOR_UNLOCKED: &str = "door unlocked";
/// `Event::Custom` name: a player fully unlocked a Room (CR 709.5i).
pub const FULLY_UNLOCKED: &str = "fully unlock a room";
/// `SpecialAction::Other` name prefix of paying a door's unlock cost (CR 116.2m); the
/// half's index follows (`"unlock door:0"`).
pub const UNLOCK_ACTION: &str = "unlock door:";
/// `TriggerCond::Custom` name prefix of "When you unlock this door" on half N.
pub const UNLOCK_THIS_DOOR: &str = "door unlocked:";

/// The unlocked designations of Room permanents (CR 709.5c). A new object (after a zone
/// change) has none.
#[derive(Clone, Debug, Default)]
pub struct RoomState {
    pub unlocked: BTreeMap<ObjectId, [bool; 2]>,
}

/// Whether a card is a split card with a shared type line: a permanent card with two
/// halves (CR 709.5).
pub fn is_room_card(card: &CardDef) -> bool {
    card.layout == Layout::Split
        && card.faces.len() == 2
        && card
            .faces
            .iter()
            .all(|f| f.chars.card_types.has_permanent_type())
}

/// Whether `id` is a face-up permanent with a shared type line.
pub fn is_room(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.zone == Zone::Battlefield
        && !o.face_down
        && o.face == FaceState::Front
        && o.card.as_deref().is_some_and(is_room_card)
}

/// The unlocked designations of `id` (left, right).
pub fn unlocked(g: &Game, id: ObjectId) -> [bool; 2] {
    g.special
        .rooms
        .unlocked
        .get(&id)
        .copied()
        .unwrap_or_default()
}

/// The locked halves of a Room permanent.
pub fn locked_halves(g: &Game, id: ObjectId) -> Vec<usize> {
    if !is_room(g, id) {
        return vec![];
    }
    let u = unlocked(g, id);
    (0..2).filter(|i| !u[*i]).collect()
}

/// The characteristics of a Room permanent with the given unlocked halves: a locked
/// half's name, mana cost and rules text don't exist (CR 709.5); its types are the
/// shared type line (CR 709.5a).
fn room_characteristics(card: &CardDef, u: [bool; 2]) -> Characteristics {
    let mut out = card.characteristics(FaceState::Front);
    let halves: Vec<&Characteristics> = (0..2)
        .filter(|i| u[*i])
        .map(|i| &card.faces[i].chars)
        .collect();
    out.name = SmolStr::new(
        halves
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(" // "),
    );
    out.mana_cost = if halves.is_empty() {
        None
    } else {
        let mut cost = ManaCost::default();
        for c in &halves {
            if let Some(m) = &c.mana_cost {
                cost.symbols.extend(m.symbols.iter().copied());
            }
        }
        Some(cost)
    };
    // Its colors come from its (remaining) mana cost (CR 202.2).
    out.colors = match out.color_indicator {
        Some(ci) => ci,
        None => halves
            .iter()
            .fold(ColorSet::NONE, |acc, c| acc.union(c.colors)),
    };
    out.abilities = halves
        .iter()
        .flat_map(|c| c.abilities.iter().cloned())
        .collect();
    out.rules_text = Arc::from(
        halves
            .iter()
            .map(|c| &*c.rules_text)
            .collect::<Vec<_>>()
            .join("\n//\n")
            .as_str(),
    );
    out
}

/// Layer 0: the shared type line's static abilities remove the name, mana cost and rules
/// text of each locked half (CR 709.5). Called as characteristics are computed.
pub fn apply_locks(g: &mut Game, live: &[ObjectId]) {
    for id in live {
        if !is_room(g, *id) {
            continue;
        }
        let Some(card) = g.obj(*id).card.clone() else {
            continue;
        };
        let u = unlocked(g, *id);
        g.objects[id.0 as usize].chars = room_characteristics(&card, u);
    }
}

/// Gives a Room permanent the unlocked designation of `half`, as `by` unlocks it
/// (CR 709.5e, 709.5f). Emits the events for "when you unlock this door" (CR 709.5h)
/// and "whenever you fully unlock a Room" (CR 709.5i). Returns false if that half
/// wasn't locked.
pub fn unlock(g: &mut Game, id: ObjectId, half: usize, by: PlayerId) -> bool {
    if half > 1 || !locked_halves(g, id).contains(&half) {
        return false;
    }
    let entry = g.special.rooms.unlocked.entry(id).or_default();
    entry[half] = true;
    let fully = entry[0] && entry[1];
    g.dirty = true;
    g.log(|g| format!("{} unlocks a door of {}", by, g.describe(id)));
    g.emit(Event::Custom {
        name: SmolStr::new(DOOR_UNLOCKED),
        player: Some(by),
        obj: Some(id),
        amount: half as i32,
    });
    if fully {
        g.emit(Event::Custom {
            name: SmolStr::new(FULLY_UNLOCKED),
            player: Some(by),
            obj: Some(id),
            amount: 0,
        });
    }
    true
}

/// Removes the unlocked designation of `half` (CR 709.5g). Returns false if that half
/// wasn't unlocked.
pub fn lock(g: &mut Game, id: ObjectId, half: usize) -> bool {
    if half > 1 || !is_room(g, id) || !unlocked(g, id)[half] {
        return false;
    }
    if let Some(u) = g.special.rooms.unlocked.get_mut(&id) {
        u[half] = false;
    }
    g.dirty = true;
    true
}

/// CR 709.5d: a Room permanent entering the battlefield from a half cast as a spell is
/// given that half's unlocked designation. Called as the new object is created.
pub fn entering(g: &mut Game, old: ObjectId, new: ObjectId, to: Zone) {
    if to != Zone::Battlefield {
        return;
    }
    let o = g.obj(old);
    let FaceState::Half(i) = o.face else {
        return;
    };
    if o.zone != Zone::Stack || !o.card.as_deref().is_some_and(is_room_card) || i > 1 {
        return;
    }
    let by = o.controller;
    let entry = g.special.rooms.unlocked.entry(new).or_default();
    entry[i as usize] = true;
    g.emit(Event::Custom {
        name: SmolStr::new(DOOR_UNLOCKED),
        player: Some(by),
        obj: Some(new),
        amount: i as i32,
    });
}

/// The unlock costs `p` could pay now (CR 116.2m): locked halves of Rooms they control,
/// any time they have priority and the stack is empty during a main phase of their turn.
pub fn unlock_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    if !g.has_priority(p) || !g.is_sorcery_timing(p) {
        return vec![];
    }
    let mut out = Vec::new();
    for id in g.battlefield.iter().copied() {
        if g.obj(id).controller != p {
            continue;
        }
        for half in locked_halves(g, id) {
            let Some(cost) = unlock_cost(g, id, half) else {
                continue;
            };
            let ctx = crate::eval::Ctx::new(Some(id), p);
            if g.can_pay_cost(p, &cost, Some(id), &ctx) {
                out.push(Action::Special(SpecialAction::Other {
                    name: format!("{UNLOCK_ACTION}{half}"),
                    obj: Some(id),
                }));
            }
        }
    }
    out
}

/// A half's unlock cost: its mana cost (CR 116.2m).
fn unlock_cost(g: &Game, id: ObjectId, half: usize) -> Option<crate::ability::Cost> {
    let card = g.obj(id).card.clone()?;
    let m = card
        .faces
        .get(half)?
        .chars
        .mana_cost
        .clone()
        .unwrap_or_default();
    Some(crate::ability::Cost::mana(m))
}

/// Takes the unlock special action. Returns None if `sa` isn't one.
pub fn perform(g: &mut Game, p: PlayerId, sa: &SpecialAction) -> Option<Result<(), Illegal>> {
    let SpecialAction::Other { name, obj } = sa else {
        return None;
    };
    let half: usize = name.strip_prefix(UNLOCK_ACTION)?.parse().ok()?;
    let bad = |s: &str| Some(Err(Illegal(s.into())));
    let Some(id) = *obj else {
        return bad("no Room");
    };
    if !g.is_live(id) || g.obj(id).controller != p || !locked_halves(g, id).contains(&half) {
        return bad("no locked door to unlock");
    }
    if !g.is_sorcery_timing(p) {
        return bad("unlock only in a main phase of your turn with an empty stack");
    }
    let Some(cost) = unlock_cost(g, id, half) else {
        return bad("no unlock cost");
    };
    let ctx = crate::eval::Ctx::new(Some(id), p);
    if !crate::special_actions::pay(g, p, &cost, Some(id), &ctx) {
        return bad("can't pay the unlock cost");
    }
    unlock(g, id, half, p);
    Some(Ok(()))
}

/// "When you unlock this door" on half N (`door unlocked:N`): triggers when that half of
/// this Room is unlocked (CR 709.5h).
pub fn custom_trigger(name: &str, src: ObjectId, ev: &Event) -> Option<Vec<EventInfo>> {
    let half: i32 = name.strip_prefix(UNLOCK_THIS_DOOR)?.parse().ok()?;
    Some(match ev {
        Event::Custom {
            name: n,
            player,
            obj: Some(o),
            amount,
        } if n == DOOR_UNLOCKED && *o == src && *amount == half => vec![EventInfo {
            object: Some(src),
            player: *player,
            amount: half,
            ..Default::default()
        }],
        _ => vec![],
    })
}
