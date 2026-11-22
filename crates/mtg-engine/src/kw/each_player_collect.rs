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
use crate::types::{Entity, ObjectId, PlayerId};

/// One player's choice.
pub const PICK: Var = vars::USER + 2746;
/// All the players' choices so far.
pub const COLLECTED: Var = vars::USER + 2747;
/// The player who chose each object of [`COLLECTED`] (same order): the player who puts it
/// onto the battlefield (CR 110.2a; Nature's Wrath's ruling: "who puts the permanent onto
/// the battlefield").
pub const COLLECTED_BY: Var = vars::USER + 2748;
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
        let mut all = ctx.vars.remove(&COLLECTED).unwrap_or_default();
        let mut by = ctx.vars.remove(&COLLECTED_BY).unwrap_or_default();
        // A fresh collection (the move's `Store` reset it): forget earlier choosers.
        by.truncate(all.len());
        for e in pick {
            if !all.contains(&e) {
                all.push(e);
                by.push(Entity::Player(ctx.controller));
            }
        }
        ctx.vars.insert(COLLECTED, all);
        ctx.vars.insert(COLLECTED_BY, by);
        true
    }
}

/// The player who chose `obj` in the current collection, if it is one of [`COLLECTED`].
pub fn putter(ctx: &Ctx, obj: ObjectId) -> Option<PlayerId> {
    let i = ctx
        .vars
        .get(&COLLECTED)?
        .iter()
        .position(|e| *e == Entity::Object(obj))?;
    match ctx.vars.get(&COLLECTED_BY)?.get(i)? {
        Entity::Player(p) => Some(*p),
        _ => None,
    }
}

inventory::submit! { KeywordRegistration(&EachPlayerCollect) }
