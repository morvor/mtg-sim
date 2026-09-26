//! CR 702.105 Dethrone: "Whenever this creature attacks the player with the most life or
//! tied for most life, put a +1/+1 counter on this creature." (CR 702.105a). Each
//! instance triggers separately (CR 702.105b).
//!
//! Whether the attacked player has the most life is part of the trigger event, not an
//! intervening "if" clause: once it has triggered, changes to life totals don't matter.
//! Attacking a planeswalker or a battle isn't attacking a player. In Two-Headed Giant
//! each teammate has the team's life total (CR 810.9), so attacking either player of the
//! team with the most life counts.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// `Condition::Custom`: the player of the trigger event has the most life or is tied for
/// most life.
pub const EVENT_PLAYER_HAS_MOST_LIFE: &str = "dethrone:that player has the most life";
/// `Condition::Custom`: the ability's source is attacking a player who has the most life
/// or is tied for most life ("if it's attacking the player with the most life or tied for
/// most life").
pub const ATTACKING_PLAYER_WITH_MOST_LIFE: &str =
    "dethrone:it's attacking the player with the most life";

/// Whether `p` has the most life or is tied for most life among the players in the game.
pub fn has_most_life(g: &Game, p: PlayerId) -> bool {
    let most = g
        .players_in_game()
        .into_iter()
        .map(|q| g.player(q).life)
        .max();
    most.is_some_and(|m| g.player(p).life >= m)
}

pub struct Dethrone;

impl KeywordRules for Dethrone {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Dethrone]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let t = TriggeredAbility::new(
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::AttacksRecipient {
                    attacker: Filter::Source,
                    recipient: DamageRecipient::Player(PlayerRel::Any),
                }),
                cond: Condition::Custom(EVENT_PLAYER_HAS_MOST_LIFE.into()),
            },
            Body::effect(Effect::AddCounters {
                what: Sel::This,
                kind: counters::PLUS1.into(),
                n: Value::c(1),
            }),
        );
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Dethrone.name(),
        )])
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        match name {
            EVENT_PLAYER_HAS_MOST_LIFE => Some(
                ctx.event
                    .as_ref()
                    .and_then(|e| e.player)
                    .is_some_and(|p| has_most_life(g, p)),
            ),
            ATTACKING_PLAYER_WITH_MOST_LIFE => Some(ctx.source.is_some_and(|s| {
                let s = g.current(s);
                g.combat.as_ref().is_some_and(|c| {
                    matches!(c.attack_target(s), Some(Entity::Player(p)) if has_most_life(g, p))
                })
            })),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&Dethrone) }
