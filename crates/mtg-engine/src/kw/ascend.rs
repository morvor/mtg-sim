//! CR 702.131 Ascend and the city's blessing.
//!
//! * On an instant or sorcery spell, ascend represents a spell ability: "If you control
//!   ten or more permanents and you don't have the city's blessing, you get the city's
//!   blessing for the rest of the game." (CR 702.131a). It's compiled as the spell's first
//!   spell ability ([`ASCEND`], see `oracle/patterns/k702_125_139.rs`), so it happens
//!   before the rest of the spell's effect.
//! * On a permanent, it represents a static ability: "Any time you control ten or more
//!   permanents and you don't have the city's blessing, you get the city's blessing for
//!   the rest of the game." (CR 702.131b). [`has_citys_blessing`] counts a player who
//!   qualifies right now as having it (so continuous effects that depend on it apply at
//!   once, before triggers are checked, CR 702.131d), and the designation is recorded
//!   (`Player::has_citys_blessing`) whenever state-based actions are checked, before they
//!   are performed (so a tenth permanent that leaves because of one still gave it), after
//!   which it stays for the rest of the game.
//!
//! The city's blessing is a designation with no rules meaning of its own; any number of
//! players may have it (CR 702.131c).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Effect::Custom`: ascend's spell ability (CR 702.131a).
pub const ASCEND: &str = "ascend:spell";

pub struct Ascend;

/// The number of permanents `p` controls.
fn permanents_controlled(g: &Game, p: PlayerId) -> usize {
    g.permanents().filter(|o| o.controller == p).count()
}

/// Whether `p` controls ten or more permanents.
pub fn controls_ten_permanents(g: &Game, p: PlayerId) -> bool {
    permanents_controlled(g, p) >= 10
}

/// Whether a permanent's ascend ability gives `p` the city's blessing now: they control a
/// permanent with ascend and ten or more permanents (CR 702.131b).
fn ascends_now(g: &Game, p: PlayerId) -> bool {
    g.player(p).in_game()
        && g.permanents()
            .any(|o| o.controller == p && o.chars.has_keyword(KeywordKind::Ascend))
        && controls_ten_permanents(g, p)
}

/// Whether `p` has the city's blessing (CR 702.131c).
pub fn has_citys_blessing(g: &Game, p: PlayerId) -> bool {
    g.player(p).has_citys_blessing || ascends_now(g, p)
}

/// `p` gets the city's blessing for the rest of the game.
fn get_blessing(g: &mut Game, p: PlayerId) {
    if g.player(p).has_citys_blessing {
        return;
    }
    g.players[p.idx()].has_citys_blessing = true;
    // CR 702.131d: continuous effects are reapplied.
    g.dirty = true;
    g.log(|_| format!("{p} gets the city's blessing"));
}

impl KeywordRules for Ascend {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Ascend]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != ASCEND {
            return false;
        }
        let p = ctx.controller;
        if controls_ten_permanents(g, p) {
            get_blessing(g, p);
        }
        true
    }

    /// Records the city's blessing of each player a permanent's ascend ability gives it.
    fn static_state_checks(&self, g: &mut Game) {
        for p in g.player_ids() {
            if !g.player(p).has_citys_blessing && ascends_now(g, p) {
                get_blessing(g, p);
            }
        }
    }
}

inventory::submit! { KeywordRegistration(&Ascend) }
