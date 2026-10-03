//! CR 701.20 Reveal: showing cards to all players.
//!
//! * A revealed card stays in its zone (CR 701.20b); it stays revealed only as long as
//!   necessary (CR 701.20a): a card revealed while a spell or ability is being cast,
//!   activated, or resolving (to pay a cost, or by its effect) is revealed until that
//!   spell or ability leaves the stack; a card whose revealing caused a triggered ability
//!   to trigger stays revealed until that ability leaves the stack (or isn't put on the
//!   stack). A card that changes zones is a new object and isn't revealed.
//! * A card that's already revealed may be revealed again (CR 701.20c).
//! * Revealed cards in a library that's shuffled or otherwise reordered stop being
//!   revealed and become new objects (CR 701.20d).
//! * "[Players] play with their hands revealed" (Telepathy) reveals the cards in those
//!   hands for as long as the effect lasts.

use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a player revealed a card (`obj`).
pub const REVEALED: &str = "revealed";

/// `StaticEffect::Custom` prefix: "[players] play with their hands revealed" — followed by
/// "you", "opponents", or "each".
pub const HANDS_REVEALED: &str = "hands revealed:";

/// A card that was revealed and how long it stays revealed.
#[derive(Clone, Debug)]
pub struct Revealed {
    pub card: ObjectId,
    pub by: PlayerId,
    /// The spell or ability on the stack for which it was revealed: it stays revealed
    /// until that object leaves the stack (CR 701.20a).
    pub stack_obj: Option<ObjectId>,
}

#[derive(Clone, Debug, Default)]
pub struct RevealState {
    pub revealed: Vec<Revealed>,
}

/// The spell or ability being cast, activated, or resolved in `ctx`, if any.
fn stack_object(g: &Game, ctx: Option<&Ctx>) -> Option<ObjectId> {
    let ctx = ctx?;
    ctx.stack_obj
        .or(ctx.source)
        .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Stack)
}

/// `p` reveals `cards` (CR 701.20a). Revealing doesn't move them (CR 701.20b), and a card
/// that's already revealed can be revealed again (CR 701.20c).
pub fn reveal(g: &mut Game, p: PlayerId, cards: &[ObjectId], source: Option<ObjectId>) {
    let ctx = source.map(|s| Ctx::new(Some(s), p));
    reveal_in(g, p, cards, ctx.as_ref());
}

/// Like [`reveal`], for a spell or ability being cast, activated, or resolving in `ctx`.
pub fn reveal_in(g: &mut Game, p: PlayerId, cards: &[ObjectId], ctx: Option<&Ctx>) {
    // Forget reveals that have ended (the new ones may be kept by a triggered ability
    // that's about to trigger on them).
    prune(g);
    let stack_obj = stack_object(g, ctx);
    for c in cards {
        if !g.is_live(*c) {
            continue;
        }
        g.log(|g| format!("{p} reveals {}", g.describe(*c)));
        g.reveals.revealed.push(Revealed {
            card: *c,
            by: p,
            stack_obj,
        });
        g.emit(Event::Custom {
            name: SmolStr::new(REVEALED),
            player: Some(p),
            obj: Some(*c),
            amount: 0,
        });
    }
}

/// Whether a triggered ability that triggered on `card` being revealed is still waiting to
/// be put on the stack or is on the stack.
fn trigger_pending_for(g: &Game, card: ObjectId) -> bool {
    let about = |e: &EventInfo| e.object == Some(card) || e.lki == Some(card);
    g.pending_triggers.iter().any(|t| about(&t.event))
        || g.stack.iter().any(|s| {
            g.obj(*s).stack.as_deref().is_some_and(|si| {
                matches!(si.kind, StackKind::Triggered { .. })
                    && si.event.as_ref().is_some_and(about)
            })
        })
}

fn still_revealed(g: &Game, r: &Revealed) -> bool {
    if !g.is_live(r.card) {
        return false;
    }
    let on_stack = r
        .stack_obj
        .is_some_and(|s| g.is_live(s) && g.obj(s).zone == Zone::Stack);
    on_stack || trigger_pending_for(g, r.card)
}

/// Forgets reveals that have ended.
pub fn prune(g: &mut Game) {
    let keep: Vec<Revealed> = g
        .reveals
        .revealed
        .iter()
        .filter(|r| still_revealed(g, r))
        .cloned()
        .collect();
    g.reveals.revealed = keep;
}

/// Whether an effect makes `p` play with their hand revealed (Telepathy).
pub fn hand_revealed(g: &Game, p: PlayerId) -> bool {
    g.statics.customs.iter().any(|(src, ctl, name)| {
        let Some(who) = name.strip_prefix(HANDS_REVEALED) else {
            return false;
        };
        let _ = src;
        match who {
            "you" => p == *ctl,
            "opponents" => g.are_opponents(*ctl, p),
            "each" => true,
            _ => false,
        }
    })
}

/// Whether `card` is currently revealed to all players.
pub fn is_revealed(g: &Game, card: ObjectId) -> bool {
    if !g.is_live(card) {
        return false;
    }
    let o = g.obj(card);
    match o.zone {
        Zone::Hand(p) if hand_revealed(g, p) => return true,
        Zone::Library(p) if crate::zones::revealed_top(g, p) == Some(card) => return true,
        _ => {}
    }
    g.reveals
        .revealed
        .iter()
        .any(|r| r.card == card && still_revealed(g, r))
}

/// `p`'s library was shuffled or otherwise reordered: the revealed cards in it stop being
/// revealed and become new objects (CR 701.20d).
pub fn library_reordered(g: &mut Game, p: PlayerId) {
    let in_library: Vec<ObjectId> = g
        .reveals
        .revealed
        .iter()
        .map(|r| r.card)
        .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Library(p))
        .collect();
    if in_library.is_empty() {
        return;
    }
    g.reveals.revealed.retain(|r| !in_library.contains(&r.card));
    for c in in_library {
        let new = g.create_incarnation(c, Zone::Library(p));
        if let Some(slot) = g.players[p.idx()].library.iter_mut().find(|x| **x == c) {
            *slot = new;
        }
    }
    g.dirty = true;
}
