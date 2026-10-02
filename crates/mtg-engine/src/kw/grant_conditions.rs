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
    ACTION_THIS_TURN, COMMITTED_CRIME_THIS_TURN, SACRIFICED_THIS_TURN, SOURCE_CAST_FROM_EXILE,
    SOURCE_DEALT_COMBAT_DAMAGE, SOURCE_DEALT_DAMAGE,
};
use crate::types::CardType;

pub struct GrantConditions;

impl KeywordRules for GrantConditions {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// Records that the source dealt (combat) damage ("as long as it hasn't dealt damage
    /// yet").
    fn after_damage(
        &self,
        g: &mut Game,
        source: crate::types::ObjectId,
        _target: crate::types::Entity,
        _amount: u32,
        combat: bool,
    ) {
        let o = g.obj_mut(source);
        o.dealt_damage.0 = true;
        if combat {
            o.dealt_damage.1 = true;
        }
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
        if name == SOURCE_DEALT_DAMAGE || name == SOURCE_DEALT_COMBAT_DAMAGE {
            let combat = name == SOURCE_DEALT_COMBAT_DAMAGE;
            return Some(ctx.source.is_some_and(|s| {
                let d = g.obj(s).dealt_damage;
                if combat {
                    d.1
                } else {
                    d.0
                }
            }));
        }
        if name == SOURCE_CAST_FROM_EXILE {
            // The card now: a spell, or the permanent that spell became, cast from exile.
            let Some(src) = ctx.source else {
                return Some(false);
            };
            let o = g.obj(g.current(src));
            let from_exile = |c: &crate::object::CastInfo| {
                c.was_cast && c.from == Some(crate::ability::ZoneKind::Exile)
            };
            return Some(
                o.stack.as_deref().is_some_and(|si| from_exile(&si.cast))
                    || o.cast.as_deref().is_some_and(from_exile),
            );
        }
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
