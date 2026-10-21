//! "If you would pay life while your library has at least that many cards in it, exile
//! that many cards from the top of your library instead." (Ashiok, Wicked Manipulator): a
//! replacement effect on paying life (CR 119.4, 614.1a), compiled to a
//! `StaticEffect::Custom` (see `oracle/patterns/r119_pay_life_exile_instead.rs`).
//!
//! It isn't optional, and it doesn't let a player pay more life than their life total
//! (that's checked before it applies). Cards entering the battlefield from that library
//! (a land played from the top of it, paying life as it enters) aren't counted or exiled
//! (CR 614.13c).

use super::{KeywordRegistration, KeywordRules};
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::ability::LibraryPosition;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// `StaticEffect::Custom` name of the ability.
pub const PAY_LIFE_EXILE_INSTEAD: &str = "pay life: exile that many from library instead";

pub struct PayLifeExileInstead;

impl KeywordRules for PayLifeExileInstead {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn pay_life_instead(&self, g: &mut Game, p: PlayerId, n: u32) -> bool {
        if g.dirty {
            g.recompute();
        }
        let Some(source) = g
            .statics
            .customs
            .iter()
            .find(|(_, ctl, name)| *ctl == p && name == PAY_LIFE_EXILE_INSTEAD)
            .map(|(src, _, _)| *src)
        else {
            return false;
        };
        // The library's cards from the top, leaving out any entering the battlefield.
        let cards: Vec<ObjectId> = g
            .player(p)
            .library
            .iter()
            .rev()
            .copied()
            .filter(|c| !g.entering.contains(c))
            .collect();
        if (cards.len() as u64) < n as u64 {
            return false;
        }
        g.log(|g| format!("{} exiles {n} cards instead of paying life ({})", p, g.describe(source)));
        let moves = cards[..n as usize]
            .iter()
            .map(|c| MoveEv {
                obj: *c,
                to: Zone::Exile,
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(p),
                etb: EtbInfo::default(),
                source: Some(source),
            })
            .collect();
        g.move_objects(moves);
        true
    }
}

inventory::submit! { KeywordRegistration(&PayLifeExileInstead) }
