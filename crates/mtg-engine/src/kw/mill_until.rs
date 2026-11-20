//! "Target opponent mills a card, then repeats this process until a creature card or X
//! cards have been put into their graveyard this way, whichever comes first." (Helm of
//! Obedience, CR 701.17a): one card at a time, counting only the cards that were put into
//! that graveyard. If a replacement effect keeps milled cards out of the graveyard
//! (Leyline of the Void), the process goes on until the library is empty (its ruling). A
//! milled card that comes back into the library (Darksteel Colossus shuffled in) doesn't
//! end it: the process goes on with the next card, and may mill that card again.
//!
//! It also stops when a mill moves nothing, and when every card left in the library is one
//! this process already milled (Wheel of Sun and Moon puts each milled card on the bottom):
//! CR 104.4b makes such an endless loop of mandatory actions a draw, which the engine
//! doesn't do here (register `mill-until-stops-when-nothing-moves`, a known deviation
//! pending the loops work). [`MILL_CAP_PER_CARD`] bounds the number of mills as a safety
//! net against a returned card that keeps coming back on top.
//!
//! The cards put into the graveyard this way are "them" for the rest of the effect
//! (`vars::IT`). An `Effect::Custom` named by [`effect_name`] (compiled in
//! `oracle/patterns/paper_loops.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Filter, PlayerRef, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;
use std::collections::HashSet;

/// Mills per card of the library at the start (plus one) after which the process ends: with
/// shuffled-back cards it ends far earlier in practice (a returned card would have to come
/// back on top dozens of times in a row).
const MILL_CAP_PER_CARD: usize = 64;

const PREFIX: &str = "mill until a card of a type or x cards:";

/// The `Effect::Custom` name: the card type that stops the process and the target slot of
/// the player who mills.
pub fn effect_name(stop: CardType, slot: u8) -> String {
    format!("{PREFIX}{}:target {slot}", format!("{stop:?}").to_lowercase())
}

fn parse(name: &str) -> Option<(CardType, u8)> {
    let (t, who) = name.strip_prefix(PREFIX)?.split_once(':')?;
    let slot = who.strip_prefix("target ")?.parse().ok()?;
    Some((CardType::from_word(t)?, slot))
}

/// The card type and target slot of an instruction (for the renderer).
pub fn parts(name: &str) -> Option<(CardType, u8)> {
    parse(name)
}

pub struct MillUntil;

impl KeywordRules for MillUntil {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((stop, slot)) = parse(name) else {
            return false;
        };
        let limit = g.eval_value(&Value::X, ctx).max(0) as usize;
        let stop_filter = Filter::and(vec![Filter::Type(stop), Filter::Card]);
        let mut put = Vec::new();
        for p in g.eval_players(&PlayerRef::Target(slot), ctx) {
            // The cards this process milled that are back in the library (shuffled in,
            // put on the bottom): the process goes on while any other card is left.
            let mut returned: HashSet<ObjectId> = HashSet::new();
            let cap = MILL_CAP_PER_CARD * (g.player(p).library.len() + 1);
            for _ in 0..cap {
                let lib = &g.player(p).library;
                if lib.iter().all(|c| returned.contains(c)) {
                    // Empty, or only cards it already milled are left (a loop of
                    // mandatory actions the engine ends here; see the module docs).
                    break;
                }
                let Some(&top) = lib.last() else { break };
                let milled = g.mill(p, 1);
                let now = g.current(top);
                if now == top && g.is_in_zone(top, Zone::Library(p)) {
                    // Nothing moved (the mill was replaced by something else).
                    break;
                }
                if g.obj(now).zone == Zone::Library(p) {
                    returned.insert(now);
                }
                let mut found = false;
                for c in milled {
                    if g.is_live(c) && g.obj(c).zone == Zone::Graveyard(p) {
                        found |= g.matches(c, &stop_filter, ctx);
                        put.push(Entity::Object(c));
                    }
                }
                if found || put.len() >= limit {
                    break;
                }
            }
        }
        ctx.prev_value = put.len() as i64;
        ctx.set_var(vars::IT, put);
        true
    }
}

inventory::submit! { KeywordRegistration(&MillUntil) }
