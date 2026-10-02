//! Evaluates the turn-history conditions compiled by
//! `oracle/patterns/grant_conditions.rs`: "you've committed a crime this turn" (CR
//! 700.13), "you've surveilled this turn" (CR 701.25) and "you sacrificed a [permanent type] this turn" (CR 701.21), each about the
//! controller of the ability.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::grant_conditions::{
    ACTION_THIS_TURN, COMMITTED_CRIME_THIS_TURN, SACRIFICED_THIS_TURN,
};
use crate::types::CardType;

pub struct GrantConditions;

impl KeywordRules for GrantConditions {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// Records the named actions players perform ("surveil", "scry", CR 701.25, 701.22).
    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::Custom {
            name,
            player: Some(p),
            ..
        } = ev
        {
            g.history.custom_actions.push((*p, name.clone()));
        }
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if let Some(action) = name.strip_prefix(ACTION_THIS_TURN) {
            return Some(
                g.history
                    .custom_actions
                    .iter()
                    .any(|(p, a)| *p == ctx.controller && a == action),
            );
        }
        if name == COMMITTED_CRIME_THIS_TURN {
            return Some(
                g.history
                    .crimes
                    .get(&ctx.controller)
                    .is_some_and(|n| *n > 0),
            );
        }
        let ty = name.strip_prefix(SACRIFICED_THIS_TURN)?;
        let want = CardType::from_word(ty);
        // The sacrificed objects' last known information (CR 608.2h).
        Some(g.history.sacrificed.iter().any(|(p, o)| {
            *p == ctx.controller
                && match want {
                    Some(t) => g.obj(*o).chars.card_types.contains(t),
                    None => true,
                }
        }))
    }
}

inventory::submit! { KeywordRegistration(&GrantConditions) }
