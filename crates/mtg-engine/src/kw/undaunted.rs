//! CR 702.125 Undaunted: a static ability that functions while the spell with undaunted is
//! on the stack. "Undaunted" means "This spell costs {1} less to cast for each opponent
//! you have." (CR 702.125a). Players who have left the game aren't counted
//! (CR 702.125b), and each instance applies (CR 702.125c): the reduction is made once per
//! keyword instance. It reduces only generic mana (CR 601.2f).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

pub struct Undaunted;

/// The number of opponents `p` has: players still in the game on other teams.
pub fn opponent_count(g: &Game, p: PlayerId) -> u32 {
    g.opponents(p)
        .into_iter()
        .filter(|q| g.player(*q).in_game())
        .count() as u32
}

impl KeywordRules for Undaunted {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Undaunted]
    }

    fn cost_reduction(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        _kw: &Keyword,
        cost: &mut Cost,
        _x: u32,
    ) {
        let n = opponent_count(g, p);
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(n);
        }
    }
}

inventory::submit! { KeywordRegistration(&Undaunted) }
