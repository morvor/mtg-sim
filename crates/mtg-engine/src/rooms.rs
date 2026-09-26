//! Rooms: split permanent cards with a shared type line (CR 709.5). Each half ("door",
//! CR 709.5j) is locked unless the permanent has that half's unlocked designation
//! (CR 709.5c); a locked half's name, mana cost and rules text don't exist on the
//! battlefield (CR 709.5). A half cast as a spell enters unlocked (CR 709.5d), a player
//! may pay a locked half's mana cost to unlock it as a special action (CR 116.2m,
//! 709.5e), effects may lock or unlock a door (CR 709.5f, 709.5g), and abilities trigger
//! when a door is unlocked (CR 709.5h) or a Room is fully unlocked (CR 709.5i).
//!
//! The halves and the static abilities of the shared type line are part of the copiable
//! values (CR 709.5, 709.5b): [`Characteristics::room`] carries the halves, and the locks
//! apply after copy effects according to each permanent's own designations.

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

/// The split card with a shared type line whose halves a set of characteristics
/// represents ([`Characteristics::room`]): part of the copiable values (CR 709.5,
/// 709.5b).
#[derive(Clone)]
pub struct RoomCard(pub Arc<CardDef>);

impl std::fmt::Debug for RoomCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoomCard({})", self.0.name)
    }
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

/// The split card with a shared type line whose halves the object `id` has as part of its
/// copiable values: its own card, or the Room it's a copy of (CR 709.5b).
pub fn room_card(g: &Game, id: ObjectId) -> Option<Arc<CardDef>> {
    g.obj(id).copiable.room.as_ref().map(|r| r.0.clone())
}

/// Whether `id` is a face-up permanent with a shared type line.
pub fn is_room(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    o.zone == Zone::Battlefield && !o.face_down && room_card(g, id).is_some()
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

/// Removes from the characteristics `c` of a Room permanent the name, mana cost and rules
/// text of each locked half (CR 709.5): it has the combined characteristics of its
/// unlocked halves. Its types are the shared type line (CR 709.5a). Abilities it has
/// from elsewhere (e.g. a copy effect's exceptions) are kept.
fn remove_locked_halves(c: &mut Characteristics, card: &CardDef, u: [bool; 2]) {
    if card.faces.len() != 2 {
        return;
    }
    let halves: Vec<&Characteristics> = (0..2)
        .filter(|i| u[*i])
        .map(|i| &card.faces[i].chars)
        .collect();
    c.name = SmolStr::new(
        halves
            .iter()
            .map(|h| h.name.as_str())
            .collect::<Vec<_>>()
            .join(" // "),
    );
    c.mana_cost = if halves.is_empty() {
        None
    } else {
        let mut cost = ManaCost::default();
        for h in &halves {
            if let Some(m) = &h.mana_cost {
                cost.symbols.extend(m.symbols.iter().copied());
            }
        }
        Some(cost)
    };
    // Its colors come from its (remaining) mana cost (CR 202.2).
    if c.color_indicator.is_none() {
        c.colors = halves
            .iter()
            .fold(ColorSet::NONE, |acc, h| acc.union(h.colors));
    }
    // The halves' abilities come first, the left half's then the right half's (as in the
    // combined characteristics, CR 709.4c), in the same order when copied.
    let n = [
        card.faces[0].chars.abilities.len(),
        card.faces[1].chars.abilities.len(),
    ];
    if c.abilities.len() >= n[0] + n[1] {
        let rest = c.abilities.split_off(n[0] + n[1]);
        let right = c.abilities.split_off(n[0]);
        let left = std::mem::take(&mut c.abilities);
        if u[0] {
            c.abilities.extend(left);
        }
        if u[1] {
            c.abilities.extend(right);
        }
        c.abilities.extend(rest);
    }
    c.rules_text = Arc::from(
        halves
            .iter()
            .map(|h| &*h.rules_text)
            .collect::<Vec<_>>()
            .join("\n//\n")
            .as_str(),
    );
}

/// Layer 0: the characteristics of a card with a shared type line (a Room card, a half
/// of it cast as a spell, and the permanent it becomes) include its two halves
/// (CR 709.5b).
pub fn mark_rooms(g: &mut Game, live: &[ObjectId]) {
    for id in live {
        let o = &g.objects[id.0 as usize];
        if o.face_down || !matches!(o.face, FaceState::Front | FaceState::Half(_)) {
            continue;
        }
        let Some(card) = o.card.clone().filter(|c| is_room_card(c)) else {
            continue;
        };
        g.objects[id.0 as usize].chars.room = Some(RoomCard(card));
    }
}

/// The shared type line's static abilities remove the name, mana cost and rules text of
/// each locked half of a Room permanent (CR 709.5). They're part of its copiable values,
/// so they apply to a copy of a Room according to the copy's own unlocked designations
/// (CR 709.5b). Called as characteristics are computed, after copy effects.
pub fn apply_locks(g: &mut Game, live: &[ObjectId]) {
    for id in live {
        let o = &g.objects[id.0 as usize];
        if o.zone != Zone::Battlefield || o.face_down {
            continue;
        }
        let Some(card) = o.chars.room.as_ref().map(|r| r.0.clone()) else {
            continue;
        };
        let u = unlocked(g, *id);
        let mut c = std::mem::take(&mut g.objects[id.0 as usize].chars);
        remove_locked_halves(&mut c, &card, u);
        g.objects[id.0 as usize].chars = c;
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
    let card = room_card(g, id)?;
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

/// `Effect::Custom` name: "lock or unlock a door of [the Room bound to
/// `vars::AFFECTED`]" (CR 709.5f, 709.5g).
pub const LOCK_OR_UNLOCK_EFFECT: &str = "room: lock or unlock a door";
/// `Effect::Custom` name: "unlock a locked door of [the Room]" (CR 709.5f).
pub const UNLOCK_EFFECT: &str = "room: unlock a locked door";
/// `Effect::Custom` name: "lock an unlocked door of [the Room]" (CR 709.5g).
pub const LOCK_EFFECT: &str = "room: lock an unlocked door";

/// "Lock or unlock a door of [a Room]", "unlock a locked door of ...", "lock an unlocked
/// door of ...": the player chooses a locked half to unlock (CR 709.5f) or an unlocked
/// half to lock (CR 709.5g) of each Room bound to `vars::AFFECTED`. A door that is
/// already unlocked can't be chosen to unlock, nor a locked one to lock. Returns false if
/// `name` isn't one of these effects.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &crate::eval::Ctx) -> bool {
    let (may_lock, may_unlock) = match name {
        LOCK_OR_UNLOCK_EFFECT => (true, true),
        UNLOCK_EFFECT => (false, true),
        LOCK_EFFECT => (true, false),
        _ => return false,
    };
    if g.dirty {
        g.recompute();
    }
    let p = ctx.controller;
    let rooms: Vec<ObjectId> = ctx
        .vars
        .get(&crate::ability::vars::AFFECTED)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default();
    for id in rooms {
        if !g.is_live(id) || !is_room(g, id) {
            continue;
        }
        let Some(card) = room_card(g, id) else {
            continue;
        };
        let u = unlocked(g, id);
        // (unlock?, half)
        let options: Vec<(bool, usize)> = (0..2)
            .filter_map(|h| match u[h] {
                false if may_unlock => Some((true, h)),
                true if may_lock => Some((false, h)),
                _ => None,
            })
            .collect();
        if options.is_empty() {
            continue;
        }
        let labels = options
            .iter()
            .map(|(un, h)| {
                let door = card.faces.get(*h).map_or("", |f| f.chars.name.as_str());
                format!("{} {door}", if *un { "Unlock" } else { "Lock" })
            })
            .collect();
        let k = g.ask_option(p, Some(id), "Choose a door", labels);
        let (un, h) = options[k.min(options.len() - 1)];
        if un {
            unlock(g, id, h, p);
        } else {
            lock(g, id, h);
        }
    }
    true
}
