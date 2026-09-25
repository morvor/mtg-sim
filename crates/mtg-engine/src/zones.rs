//! Zone rules (CR 400–408) that the primitive zone moves and the layer system consult:
//!
//! * objects go to their owner's library, hand, or graveyard (CR 400.3);
//! * cards that can't leave the command zone (CR 400.4b);
//! * the owner arranges cards put into a library position or a graveyard at the same time
//!   (CR 401.4, 404.3);
//! * a face-up object in the command zone turned face down becomes a new object (CR 400.9);
//! * cards outside the game are affected only by their own characteristic-defining
//!   abilities (CR 400.11c);
//! * playing with the top card of a library revealed (CR 401.5, 401.6);
//! * who may look at face-down cards in exile (CR 406.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::replacement::{MoveEv, ReplEvent};
use crate::types::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Per-game zone bookkeeping.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ZoneState {
    /// Face-down cards in exile and the players allowed to look at them (CR 406.3). A
    /// permission ends when the card leaves exile, as it becomes a new object.
    pub may_look: Vec<(ObjectId, PlayerId)>,
    /// The top card of each library that is currently revealed because of an effect
    /// (CR 401.5).
    pub revealed_top: Vec<(PlayerId, ObjectId)>,
    /// Library cards that were revealed as the top card and then stopped being revealed
    /// (CR 401.6).
    pub unrevealed: Vec<ObjectId>,
    /// The top card of each library its owner may currently look at because of an effect
    /// ("you may look at the top card of your library any time", CR 401.5).
    pub looked_top: Vec<(PlayerId, ObjectId)>,
    /// How many special actions are being taken right now (CR 401.5).
    #[serde(default)]
    pub special_actions: u32,
}

/// `Event::Custom` name: the top card of a player's library became revealed (CR 401.5).
pub const TOP_REVEALED: &str = "top card revealed";
/// `Effect::Custom` name: the controller may look at the face-down exiled cards the
/// preceding part of the effect exiled ("it") for as long as they remain exiled (CR 406.3).
pub const MAY_LOOK_AT_EXILED: &str = "zones:may look at exiled";

// ---------------------------------------------------------------------------
// Moving objects
// ---------------------------------------------------------------------------

/// CR 400.3: an object that would go to a library, graveyard, or hand other than its
/// owner's goes to its owner's corresponding zone.
pub fn owners_zone(g: &Game, obj: ObjectId, to: Zone) -> Zone {
    let owner = g.obj(obj).owner;
    match to {
        Zone::Library(_) => Zone::Library(owner),
        Zone::Hand(_) => Zone::Hand(owner),
        Zone::Graveyard(_) => Zone::Graveyard(owner),
        z => z,
    }
}

/// CR 400.6: an object moving to a public zone where its owner will be able to look at it
/// is looked at for abilities that would affect the move: its own replacement abilities
/// for zone changes apply wherever they function, even in a hidden zone (e.g. "If ~ would
/// be put into a graveyard from anywhere, ..."). Returns (source, controller, ability,
/// replacement) entries like the collected static replacement effects.
pub fn own_move_replacements(
    g: &Game,
    m: &MoveEv,
) -> Vec<(ObjectId, PlayerId, Ability, ReplacementDef)> {
    let o = g.obj(m.obj);
    if !m.to.is_public() || m.etb.face_down.is_some() || o.face_down {
        return vec![];
    }
    // Objects without a controller are affected on behalf of their owner.
    let controller = match o.zone {
        Zone::Battlefield | Zone::Stack => o.controller,
        _ => o.owner,
    };
    let ctx = Ctx::new(Some(m.obj), controller);
    o.chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s)
                if g.ability_functions(o, s.zone, s.is_cda)
                    && s.condition.as_ref().is_none_or(|c| g.eval_cond(c, &ctx)) =>
            {
                match &s.effect {
                    StaticEffect::Replacement(d)
                        if matches!(
                            &d.event,
                            ReplacementEvent::ZoneChange {
                                filter: Filter::Source,
                                ..
                            }
                        ) =>
                    {
                        Some((m.obj, controller, a.clone(), d.clone()))
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

/// The card types of cards that can't leave the command zone (CR 400.4b).
const COMMAND_ONLY: [CardType; 5] = [
    CardType::Conspiracy,
    CardType::Phenomenon,
    CardType::Plane,
    CardType::Scheme,
    CardType::Vanguard,
];

/// Whether an object is a conspiracy, phenomenon, plane, scheme, or vanguard card (by its
/// card, even while face down).
pub fn is_command_only(o: &GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.base);
    COMMAND_ONLY.iter().any(|t| chars.card_types.contains(*t))
}

/// Zone changes that the zone rules forbid: the object stays where it is.
pub fn move_forbidden(g: &Game, mv: &MoveEv) -> bool {
    let o = g.obj(mv.obj);
    // CR 400.4b: a conspiracy, phenomenon, plane, scheme, or vanguard card that would
    // leave the command zone remains there. (Leaving the game along with its owner,
    // CR 800.4a, isn't a zone change.)
    if o.zone == Zone::Command
        && !matches!(mv.to, Zone::Command | Zone::Nowhere)
        && is_command_only(o)
    {
        return true;
    }
    crate::ante::move_forbidden(g, mv)
}

/// CR 401.4, 404.3: cards put into the same library position, or into the same
/// graveyard, at the same time: their owner arranges them in any order. `finals` are the
/// final (replaced) events of a simultaneous move, in the order they'll be performed;
/// the moves of each such group are reordered as the owner chooses.
pub fn order_simultaneous(g: &mut Game, finals: &mut [(usize, ReplEvent)]) {
    // Groups of (destination, position) with the indices of their moves.
    let mut groups: Vec<((Zone, Option<LibraryPosition>), Vec<usize>)> = Vec::new();
    for (i, (_, e)) in finals.iter().enumerate() {
        let ReplEvent::Move(m) = e else {
            continue;
        };
        if g.obj(m.obj).kind != ObjKind::Card {
            continue;
        }
        let key = match m.to {
            Zone::Graveyard(_) => (m.to, None),
            Zone::Library(_) if !matches!(m.pos, LibraryPosition::Shuffled) => (m.to, Some(m.pos)),
            _ => continue,
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(i),
            None => groups.push((key, vec![i])),
        }
    }
    for ((zone, pos), slots) in groups {
        if slots.len() < 2 {
            continue;
        }
        let owner = match zone {
            Zone::Library(p) | Zone::Graveyard(p) => p,
            _ => continue,
        };
        let events: Vec<(usize, ReplEvent)> = slots.iter().map(|i| finals[*i].clone()).collect();
        if pos == Some(LibraryPosition::BottomRandom) {
            // "In a random order": nobody chooses.
            use rand::seq::SliceRandom;
            let mut order: Vec<usize> = (0..slots.len()).collect();
            order.shuffle(&mut g.rng);
            for (k, i) in slots.iter().enumerate() {
                finals[*i] = events[order[k]].clone();
            }
            continue;
        }
        let names: Vec<String> = slots
            .iter()
            .map(|i| match &finals[*i].1 {
                ReplEvent::Move(m) => g.describe(m.obj),
                _ => String::new(),
            })
            .collect();
        let what = match zone {
            Zone::Library(_) => "your library",
            _ => "your graveyard",
        };
        let order = g.ask_order(
            owner,
            &format!("Order the cards being put into {what} (in the order they're put there)"),
            names,
        );
        for (k, i) in slots.iter().enumerate() {
            finals[*i] = events[order[k]].clone();
        }
    }
}

/// CR 400.9: a face-up object in the command zone that's turned face down becomes a new
/// object; it's put at the end of the command zone's order (the bottom of its deck).
/// Returns the object it is now.
pub fn turn_face_down_in_command(g: &mut Game, id: ObjectId) -> ObjectId {
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
    new
}

// ---------------------------------------------------------------------------
// Outside the game (CR 400.11)
// ---------------------------------------------------------------------------

/// CR 400.11c: cards outside the game can't be affected by spells or abilities, except
/// for characteristic-defining abilities printed on them. Their characteristics are
/// their printed ones as modified by those abilities only.
pub fn outside_game_characteristics(g: &mut Game) {
    let outside: Vec<ObjectId> = g
        .players
        .iter()
        .flat_map(|p| p.sideboard.iter().copied())
        .collect();
    for id in outside {
        let o = g.obj(id);
        let mut c = o.base.clone();
        let cdas: Vec<Modification> = o
            .base
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Static(s) if s.is_cda => match &s.effect {
                    StaticEffect::Continuous {
                        affected: Filter::Source,
                        mods,
                    } => Some(mods.clone()),
                    _ => None,
                },
                _ => None,
            })
            .flatten()
            .collect();
        let ctx = Ctx::new(Some(id), o.owner);
        for m in &cdas {
            crate::layers::apply_mod(&mut c, m, g, &ctx, id);
        }
        g.objects[id.0 as usize].chars = c;
    }
}

// ---------------------------------------------------------------------------
// Revealed top cards (CR 401.5, 401.6)
// ---------------------------------------------------------------------------

/// Whether an effect makes `p` play with the top card of their library revealed.
fn reveals_top(g: &Game, p: PlayerId) -> bool {
    g.statics.other.iter().any(|(src, ctl, e)| match e {
        StaticEffect::RevealTopCard(rel) => {
            g.player_rel_matches(*rel, p, &Ctx::new(Some(*src), *ctl))
        }
        _ => false,
    })
}

/// Whether an effect lets `p` look at the top card of their library any time.
fn looks_at_top(g: &Game, p: PlayerId) -> bool {
    g.statics.other.iter().any(|(src, ctl, e)| match e {
        StaticEffect::LookAtTopCard(rel) => {
            g.player_rel_matches(*rel, p, &Ctx::new(Some(*src), *ctl))
        }
        _ => false,
    })
}

/// The card on top of `p`'s library that's revealed because of an effect, if any.
pub fn revealed_top(g: &Game, p: PlayerId) -> Option<ObjectId> {
    g.zones
        .revealed_top
        .iter()
        .find(|(q, _)| *q == p)
        .map(|(_, id)| *id)
}

/// Whether `viewer` may look at the card `id` in a library: it's the revealed top card
/// (anyone), or its owner may look at the top card of their library (CR 401.5). Other
/// cards in libraries can't be looked at (CR 401.2).
pub fn can_see_in_library(g: &Game, viewer: PlayerId, id: ObjectId) -> bool {
    let Zone::Library(owner) = g.obj(id).zone else {
        return false;
    };
    if !g.is_live(id) {
        return false;
    }
    revealed_top(g, owner) == Some(id)
        || (viewer == owner && g.zones.looked_top.contains(&(owner, id)))
}

/// Runs a special action (CR 116): while it's being taken, a new top card of a library
/// isn't revealed and can't be looked at (CR 401.5).
pub fn during_special_action<T>(g: &mut Game, f: impl FnOnce(&mut Game) -> T) -> T {
    g.zones.special_actions += 1;
    let r = f(g);
    g.zones.special_actions = g.zones.special_actions.saturating_sub(1);
    if g.zones.special_actions == 0 {
        if g.dirty {
            g.recompute();
        }
        update_revealed_tops(g);
    }
    r
}

/// Brings the revealed (and looked-at) top cards of libraries up to date. While a spell
/// is being cast, an ability activated, or a special action taken, a new top card isn't
/// revealed and can't be looked at until that's finished (CR 401.5). A card that stopped
/// being revealed for any length of time and is revealed again becomes a new object
/// (CR 401.6).
pub fn update_revealed_tops(g: &mut Game) {
    if g.special.casting > 0 || g.zones.special_actions > 0 {
        return;
    }
    for p in g.player_ids() {
        let lib_top = g.library_top(p);
        let looked = if looks_at_top(g, p) { lib_top } else { None };
        g.zones.looked_top.retain(|(q, _)| *q != p);
        if let Some(t) = looked {
            g.zones.looked_top.push((p, t));
        }
        let top = if reveals_top(g, p) { lib_top } else { None };
        let prev = revealed_top(g, p);
        if prev == top {
            continue;
        }
        if let Some(prev) = prev {
            if g.is_live(prev) && matches!(g.obj(prev).zone, Zone::Library(_)) {
                g.zones.unrevealed.push(prev);
            }
        }
        g.zones.revealed_top.retain(|(q, _)| *q != p);
        let Some(mut top) = top else {
            continue;
        };
        if let Some(i) = g.zones.unrevealed.iter().position(|x| *x == top) {
            g.zones.unrevealed.remove(i);
            let new = g.create_incarnation(top, Zone::Library(p));
            if let Some(slot) = g.players[p.idx()].library.iter_mut().find(|x| **x == top) {
                *slot = new;
            }
            for (_, t) in g.zones.looked_top.iter_mut().filter(|(_, t)| *t == top) {
                *t = new;
            }
            g.dirty = true;
            top = new;
        }
        g.zones.revealed_top.push((p, top));
        g.log(|g| format!("{p} reveals {} on top of their library", g.describe(top)));
        g.emit(Event::Custom {
            name: SmolStr::new(TOP_REVEALED),
            player: Some(p),
            obj: Some(top),
            amount: 0,
        });
    }
    // Cards no longer in a library don't need tracking.
    let live: Vec<ObjectId> = g
        .zones
        .unrevealed
        .iter()
        .copied()
        .filter(|id| g.is_live(*id) && matches!(g.obj(*id).zone, Zone::Library(_)))
        .collect();
    g.zones.unrevealed = live;
}

/// A library was shuffled: its revealed top card stops being revealed (CR 401.6).
pub fn library_shuffled(g: &mut Game, p: PlayerId) {
    if let Some(top) = revealed_top(g, p) {
        g.zones.revealed_top.retain(|(q, _)| *q != p);
        if g.is_live(top) {
            g.zones.unrevealed.push(top);
        }
    }
}

// ---------------------------------------------------------------------------
// Face-down cards in exile (CR 406.3)
// ---------------------------------------------------------------------------

/// Whether `p` may look at the face-down card `id` in exile (CR 406.3).
pub fn may_look(g: &Game, p: PlayerId, id: ObjectId) -> bool {
    g.is_live(id)
        && g.obj(id).zone == Zone::Exile
        && g.zones.may_look.iter().any(|(o, q)| *o == id && *q == p)
}

/// Lets `p` look at a face-down card in exile until it leaves exile (CR 406.3).
pub fn allow_look(g: &mut Game, p: PlayerId, id: ObjectId) {
    if !g.zones.may_look.contains(&(id, p)) {
        g.zones.may_look.push((id, p));
    }
    // Permissions for cards that have left exile are gone for good.
    let live: Vec<(ObjectId, PlayerId)> = g
        .zones
        .may_look
        .iter()
        .copied()
        .filter(|(o, _)| g.is_live(*o) && g.obj(*o).zone == Zone::Exile)
        .collect();
    g.zones.may_look = live;
}

/// Performs a zone-related `Effect::Custom`, if `name` is one.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    if crate::ante::custom_effect(g, name, ctx) {
        return true;
    }
    if name != MAY_LOOK_AT_EXILED {
        return false;
    }
    let objs: Vec<ObjectId> = ctx
        .vars
        .get(&vars::IT)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default();
    for o in objs {
        if g.obj(o).zone == Zone::Exile && g.obj(o).face_down {
            allow_look(g, ctx.controller, o);
        }
    }
    true
}
