//! CR 701.12 Exchange. Exchanges of control (`Effect::ExchangeControl`) and of life totals
//! (`Effect::ExchangeLifeTotals`) have their own effects; this module handles the others
//! (`Effect::Exchange`):
//!
//! * a life total and a creature's power or toughness: the player gains or loses the life
//!   needed to equal the value, and the value is set to the player's former life total
//!   (CR 701.12g);
//! * two creatures' powers (or toughnesses): each is set to the other's previous value
//!   (CR 701.12g);
//! * the cards in two of a player's zones (CR 701.12d), even if one is empty (CR 701.12f).
//!
//! If the entire exchange can't be completed, no part of it occurs (CR 701.12a).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::*;
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExchangeSpec {
    /// "Exchange [player]'s life total with [creature]'s toughness" (or power).
    LifeAndStat {
        player: PlayerRef,
        what: Sel,
        power: bool,
    },
    /// "Exchange [a]'s power and [b]'s power" (or toughness) for a duration.
    Stats {
        a: Sel,
        b: Sel,
        power: bool,
        duration: Duration,
    },
    /// "Exchange your [zone] and [zone]".
    Zones {
        player: PlayerRef,
        a: ZoneKind,
        b: ZoneKind,
    },
}

/// A creature on the battlefield from a selection, if exactly one is there.
fn creature(g: &mut Game, s: &Sel, ctx: &mut Ctx) -> Option<ObjectId> {
    g.resolve_objects(s, ctx)
        .into_iter()
        .find(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Battlefield && g.obj(*o).is_creature())
}

/// Sets a creature's power or toughness to `v` (a layer 7b effect, CR 613.4b).
fn set_stat(g: &mut Game, ctx: &Ctx, obj: ObjectId, power: bool, v: i32, duration: Duration) {
    let (p, t) = if power {
        (Some(Value::c(v)), None)
    } else {
        (None, Some(Value::c(v)))
    };
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: ctx.source,
        controller: ctx.controller,
        timestamp: ts,
        duration,
        affected: Affected::Objects(vec![obj]),
        mods: vec![Modification::SetPT(p, t)],
        layer1: None,
        created_turn: g.turn.number,
    });
    g.dirty = true;
}

fn stat(g: &Game, obj: ObjectId, power: bool) -> i32 {
    let o = g.obj(obj);
    if power {
        o.power()
    } else {
        o.toughness()
    }
}

pub fn perform(g: &mut Game, spec: &ExchangeSpec, ctx: &mut Ctx) {
    ctx.prev_happened = false;
    match spec {
        ExchangeSpec::LifeAndStat {
            player,
            what,
            power,
        } => {
            let Some(p) = g.eval_player(player, ctx) else {
                return;
            };
            let Some(c) = creature(g, what, ctx) else {
                return;
            };
            if g.dirty {
                g.recompute();
            }
            let life = g.player(p).life;
            let v = stat(g, c, *power);
            // CR 701.12g, 119.7–8: a player who can't gain (lose) life can't be given a
            // higher (lower) life total this way; then no part of the exchange occurs.
            if (v > life && g.cant_gain_life(p)) || (v < life && g.cant_lose_life(p)) {
                return;
            }
            set_stat(g, ctx, c, *power, life, Duration::Permanent);
            if v > life {
                g.gain_life(p, (v - life) as u32);
            } else if v < life {
                g.lose_life(p, (life - v) as u32);
            }
            ctx.prev_happened = true;
        }
        ExchangeSpec::Stats {
            a,
            b,
            power,
            duration,
        } => {
            let (Some(a), Some(b)) = (creature(g, a, ctx), creature(g, b, ctx)) else {
                return;
            };
            if a == b {
                return;
            }
            if g.dirty {
                g.recompute();
            }
            let (va, vb) = (stat(g, a, *power), stat(g, b, *power));
            set_stat(g, ctx, a, *power, vb, duration.clone());
            set_stat(g, ctx, b, *power, va, duration.clone());
            ctx.prev_happened = true;
        }
        ExchangeSpec::Zones { player, a, b } => {
            let Some(p) = g.eval_player(player, ctx) else {
                return;
            };
            let zone = |k: ZoneKind| match k {
                ZoneKind::Hand => Some(Zone::Hand(p)),
                ZoneKind::Library => Some(Zone::Library(p)),
                ZoneKind::Graveyard => Some(Zone::Graveyard(p)),
                _ => None,
            };
            let (Some(za), Some(zb)) = (zone(*a), zone(*b)) else {
                return;
            };
            let cards = |g: &Game, z: Zone| -> Vec<ObjectId> {
                match z {
                    Zone::Hand(p) => g.player(p).hand.clone(),
                    Zone::Library(p) => g.player(p).library.clone(),
                    Zone::Graveyard(p) => g.player(p).graveyard.clone(),
                    _ => vec![],
                }
            };
            let (in_a, in_b) = (cards(g, za), cards(g, zb));
            // CR 701.12d: only cards owned by the same player can be exchanged.
            if in_a.iter().chain(in_b.iter()).any(|c| g.obj(*c).owner != p) {
                return;
            }
            // The cards move at the same time; a zone may be empty (CR 701.12f).
            let mv = |obj: ObjectId, to: Zone| MoveEv {
                obj,
                to,
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(p),
                etb: EtbInfo::default(),
                source: ctx.source,
            };
            let moves: Vec<MoveEv> = in_a
                .iter()
                .map(|c| mv(*c, zb))
                .chain(in_b.iter().map(|c| mv(*c, za)))
                .collect();
            g.move_objects(moves);
            ctx.prev_happened = true;
        }
    }
}
