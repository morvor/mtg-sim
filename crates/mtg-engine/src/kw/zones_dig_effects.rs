//! Card-flow instructions about the order of a graveyard and positions in a library
//! (`oracle/patterns/hands_libraries_others.rs`):
//!
//! * "reorder your graveyard as you choose" (Fossil Find), "Reorder your graveyard at
//!   random." (Search for Survivors): an instruction that lets a graveyard's order change
//!   (CR 404.2: it normally can't);
//! * "Put target nonland permanent into its owner's library just beneath the top X cards
//!   of that library." (Unexpectedly Absent, Quarry Colossus): with X 0, it's put on top;
//!   with fewer than X cards in that library, on the bottom (CR 401; Unexpectedly
//!   Absent's rulings).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// "reorder your graveyard as you choose".
pub const REORDER_GRAVEYARD_CHOOSE: &str = "zones: reorder your graveyard as you choose";
/// "reorder your graveyard at random".
pub const REORDER_GRAVEYARD_RANDOM: &str = "zones: reorder your graveyard at random";

const BENEATH_PREFIX: &str = "zones: put into its owner's library beneath the top:";

/// "Put [what] into its owner's library just beneath the top [n] cards of that library."
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Beneath {
    pub what: Sel,
    pub n: Value,
}

/// The instruction for [`Beneath`].
pub fn beneath_effect(b: &Beneath) -> Option<Effect> {
    let json = serde_json::to_string(b).ok()?;
    Some(Effect::Custom(format!("{BENEATH_PREFIX}{json}").into()))
}

/// The [`Beneath`] of an instruction made by [`beneath_effect`].
pub fn parse_beneath(name: &str) -> Option<Beneath> {
    serde_json::from_str(name.strip_prefix(BENEATH_PREFIX)?).ok()
}

const REVEAL_PREFIX: &str = "zones: reveal:";

/// "That player reveals the chosen card": the selected cards are revealed by their
/// owners (CR 701.20a).
pub fn reveal_effect(what: &Sel) -> Option<Effect> {
    let json = serde_json::to_string(what).ok()?;
    Some(Effect::Custom(format!("{REVEAL_PREFIX}{json}").into()))
}

pub struct ZonesDigEffects;

impl KeywordRules for ZonesDigEffects {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            REORDER_GRAVEYARD_CHOOSE | REORDER_GRAVEYARD_RANDOM => {
                let p = ctx.controller;
                let cards: Vec<ObjectId> = g.player(p).graveyard.clone();
                // The graveyard is listed bottom first; the order is chosen top first.
                let order: Vec<ObjectId> = if name == REORDER_GRAVEYARD_RANDOM {
                    use rand::seq::SliceRandom;
                    let mut v = cards.clone();
                    v.shuffle(&mut g.rng);
                    v
                } else {
                    let top_first: Vec<ObjectId> = cards.iter().rev().copied().collect();
                    let names = top_first.iter().map(|c| g.describe(*c)).collect();
                    let idx = g.ask_order(p, "Order your graveyard (top first)", names);
                    idx.into_iter().rev().map(|i| top_first[i]).collect()
                };
                // Cards don't change zones (CR 400.7).
                g.players[p.idx()].graveyard = order;
                g.log(|_| format!("{p} reorders their graveyard"));
                true
            }
            _ if name.starts_with(REVEAL_PREFIX) => {
                let Ok(what) = serde_json::from_str::<Sel>(&name[REVEAL_PREFIX.len()..]) else {
                    return false;
                };
                for o in g.resolve_objects(&what, ctx) {
                    let owner = g.obj(o).owner;
                    crate::reveal::reveal_in(g, owner, &[o], Some(ctx));
                }
                true
            }
            _ => {
                let Some(b) = parse_beneath(name) else {
                    return false;
                };
                let n = g.eval_value(&b.n, ctx).max(0) as u32;
                let mut to = Destination::library_top();
                to.position = LibraryPosition::FromTop(n);
                g.exec(&Effect::Move { what: b.what, to }, ctx);
                true
            }
        }
    }
}

inventory::submit! { KeywordRegistration(&ZonesDigEffects) }
