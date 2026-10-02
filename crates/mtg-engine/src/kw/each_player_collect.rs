//! "Each player may put a creature card from their hand onto the battlefield" (Show and
//! Tell, Hunted Wumpus): each player chooses in APNAP order, then the chosen cards are put
//! onto the battlefield at the same time (CR 101.4). Each player's choice is stored in
//! [`PICK`]; this `Effect::Custom` ([`COLLECT`]) adds it to [`COLLECTED`], which the
//! single move afterwards moves (compiled in `oracle/patterns/player_subjects.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// One player's choice.
pub const PICK: Var = vars::USER + 2746;
/// All the players' choices so far.
pub const COLLECTED: Var = vars::USER + 2747;
/// The `Effect::Custom` name: add [`PICK`] to [`COLLECTED`].
pub const COLLECT: &str = "each player: collect the choice";

pub struct EachPlayerCollect;

impl KeywordRules for EachPlayerCollect {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, _g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != COLLECT {
            return false;
        }
        let pick = ctx.vars.get(&PICK).cloned().unwrap_or_default();
        let all = ctx.vars.entry(COLLECTED).or_default();
        for e in pick {
            if !all.contains(&e) {
                all.push(e);
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&EachPlayerCollect) }
