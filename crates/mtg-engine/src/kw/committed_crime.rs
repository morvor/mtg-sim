//! "if/as long as you've committed a crime this turn" (CR 700.13): whether the
//! controller of the ability has committed a crime this turn. The condition is
//! `Condition::Custom` named [`COMMITTED_CRIME_THIS_TURN`], parsed in
//! `oracle/patterns/conditions_committed_crime.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// `Condition::Custom` name of "you've committed a crime this turn".
pub const COMMITTED_CRIME_THIS_TURN: &str = "crime:you've committed a crime this turn";

pub struct CommittedCrime;

impl KeywordRules for CommittedCrime {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == COMMITTED_CRIME_THIS_TURN).then(|| {
            g.turn_events.iter().any(
                |e| matches!(e, Event::CrimeCommitted { player } if *player == ctx.controller),
            )
        })
    }
}

inventory::submit! { KeywordRegistration(&CommittedCrime) }
