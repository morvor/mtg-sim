//! "Target opponent mills a card, then repeats this process until a creature card or X
//! cards have been put into their graveyard this way, whichever comes first." (Helm of
//! Obedience, CR 701.17a): one card at a time, counting only the cards that were put into
//! that graveyard. If a replacement effect keeps milled cards out of the graveyard
//! (Leyline of the Void), the process goes on until the library is empty (its ruling); it
//! also stops when no card can be milled.
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
            loop {
                let before = g.player(p).library.len();
                if before == 0 {
                    break;
                }
                let milled = g.mill(p, 1);
                let mut found = false;
                for c in milled {
                    if g.is_live(c) && g.obj(c).zone == Zone::Graveyard(p) {
                        found |= g.matches(c, &stop_filter, ctx);
                        put.push(Entity::Object(c));
                    }
                }
                // Stopped by the card, the count, or a library that didn't change.
                if found || put.len() >= limit || g.player(p).library.len() >= before {
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
