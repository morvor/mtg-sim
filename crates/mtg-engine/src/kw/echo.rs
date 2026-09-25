//! CR 702.30 Echo: "At the beginning of your upkeep, if this permanent came under your
//! control since the beginning of your last upkeep, sacrifice it unless you pay [cost]."
//! (CR 702.30a). Urza block cards have Oracle echo costs equal to their mana costs
//! (CR 702.30b), so the cost always comes from the keyword.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// The intervening "if" of the echo trigger (a [`Condition::Custom`]).
pub const CAME_UNDER_CONTROL: &str = "came under your control since your last upkeep";

pub struct Echo;

/// Whether the permanent `id` came under its controller's control since the beginning of
/// that player's last upkeep. During that player's upkeep, "your last upkeep" is the one
/// before it (the echo trigger checks this as the current upkeep begins).
pub fn came_under_control_since_last_upkeep(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    let p = o.controller;
    let ups = &g.player(p).upkeeps_begun;
    let in_own_upkeep =
        g.turn.step == crate::turn::Step::Upkeep && g.active_players().contains(&p);
    let last = if in_own_upkeep {
        ups.iter().rev().nth(1)
    } else {
        ups.last()
    };
    last.is_none_or(|t| o.control_since > *t)
}

impl KeywordRules for Echo {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Echo]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = kw.cost.clone().unwrap_or_default();
        let mut t = TriggeredAbility::new(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body::effect(Effect::PayOptional {
                who: PlayerRef::You,
                cost,
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
            }),
        );
        t.intervening_if = Some(Condition::Custom(CAME_UNDER_CONTROL.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Echo.name(),
        )])
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != CAME_UNDER_CONTROL {
            return None;
        }
        Some(
            ctx.source
                .is_some_and(|s| g.is_live(s) && came_under_control_since_last_upkeep(g, s)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Echo) }
