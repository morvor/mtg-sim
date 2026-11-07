//! CR 702.169 Solved, and what Cases are solved by.
//!
//! "Solved — [ability text]" is a static, triggered, or activated ability that functions
//! only while its Case is solved (CR 702.169a–d; see `cases.rs` and
//! `oracle/patterns/r719_cases.rs`). A Case becomes solved at the beginning of its
//! controller's end step if its "To solve" condition holds (CR 719.3a). Several of those
//! conditions look back over the turn; this module keeps the facts they need, which any
//! other ability may count as well:
//!
//! * [`CREATURES_ATTACKED`]: how many creatures attacked this turn ("To solve — Three or
//!   more creatures attacked this turn.", Case of the Gateway Express; "if no creatures
//!   attacked this turn");
//! * [`CREATURE_CARDS_TO_GRAVEYARDS`]: how many creature cards were put into graveyards
//!   from anywhere this turn (Case of the Gorgon's Kiss);
//! * [`SOURCES_YOU_CONTROLLED_DEALT_DAMAGE`]: how many sources you controlled as they dealt
//!   damage this turn (Case of the Burning Masks). Each object counts once however often it
//!   dealt damage; a permanent that changed zones is a new object and a new source
//!   (CR 400.7), and what happens to a source afterward doesn't matter
//!   (`TurnHistory::damage_sources`, recorded as damage is dealt).
//!
//! The spells a player cast this turn are counted by `Value::SpellsCastThisTurn` ("You've
//! cast four or more instant and sorcery spells this turn.", Case of the Ransacked Lab).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::{ObjKind, Zone};
use crate::types::*;

/// `Value::Custom`: the number of creatures that attacked this turn.
pub const CREATURES_ATTACKED: &str = "turn:creatures attacked";
/// `Value::Custom`: the number of creature cards put into graveyards from anywhere this
/// turn.
pub const CREATURE_CARDS_TO_GRAVEYARDS: &str = "turn:creature cards put into graveyards";
/// `Value::Custom`: the number of sources the ability's controller controlled as they
/// dealt damage this turn.
pub const SOURCES_YOU_CONTROLLED_DEALT_DAMAGE: &str = "turn:sources you controlled dealt damage";

/// The number of different creatures that attacked this turn.
pub fn creatures_attacked(g: &Game) -> usize {
    let mut seen: Vec<ObjectId> = Vec::new();
    for a in &g.history.attackers {
        if !seen.contains(a) {
            seen.push(*a);
        }
    }
    seen.len()
}

/// The number of creature cards put into graveyards from anywhere this turn, as they were
/// in the graveyard.
pub fn creature_cards_put_into_graveyards(g: &Game) -> usize {
    g.turn_events
        .iter()
        .filter(|e| match e {
            Event::ZoneChange {
                new,
                to: Zone::Graveyard(_),
                ..
            } => {
                let o = g.obj(*new);
                o.kind == ObjKind::Card && o.chars.is(CardType::Creature)
            }
            _ => false,
        })
        .count()
}

/// The number of different sources `p` controlled as they dealt damage this turn.
pub fn sources_controlled_that_dealt_damage(g: &Game, p: PlayerId) -> usize {
    let mut seen: Vec<ObjectId> = Vec::new();
    for (s, c) in &g.history.damage_sources {
        if *c == p && !seen.contains(s) {
            seen.push(*s);
        }
    }
    seen.len()
}

pub struct Solved;

impl KeywordRules for Solved {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// Records the source that dealt damage, with its controller at that time.
    fn after_damage(
        &self,
        g: &mut Game,
        source: ObjectId,
        _target: Entity,
        _amount: u32,
        _combat: bool,
    ) {
        let record = (source, g.obj(source).controller);
        if !g.history.damage_sources.contains(&record) {
            g.history.damage_sources.push(record);
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let n = match name {
            CREATURES_ATTACKED => creatures_attacked(g),
            CREATURE_CARDS_TO_GRAVEYARDS => creature_cards_put_into_graveyards(g),
            SOURCES_YOU_CONTROLLED_DEALT_DAMAGE => {
                sources_controlled_that_dealt_damage(g, ctx.controller)
            }
            _ => return None,
        };
        Some(n as i64)
    }
}

inventory::submit! { KeywordRegistration(&Solved) }
