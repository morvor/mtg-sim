//! CR 702.54 Bloodthirst. "Bloodthirst N" means "If an opponent was dealt damage this
//! turn, this permanent enters with N +1/+1 counters on it." (CR 702.54a). "Bloodthirst
//! X" means "This permanent enters with X +1/+1 counters on it, where X is the total
//! damage your opponents have been dealt this turn." (CR 702.54b). Each instance applies
//! separately (CR 702.54c).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// `Value::Custom`: the total damage the opponents of the context's controller have been
/// dealt this turn, by any sources.
pub const DAMAGE_TO_OPPONENTS: &str = "bloodthirst:damage dealt to your opponents this turn";

pub struct Bloodthirst;

impl KeywordRules for Bloodthirst {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Bloodthirst]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let enter = |n: Value| Effect::EnterWithCounters {
            kind: counters::PLUS1.into(),
            n,
        };
        let effect = match kw.n {
            // CR 702.54b: "Bloodthirst X".
            Some(n) if n < 0 => enter(Value::Custom(DAMAGE_TO_OPPONENTS.into())),
            n => Effect::If {
                cond: Condition::Compare(
                    Value::CountPlayers(PlayerFilter::And(vec![
                        PlayerFilter::Opponent,
                        PlayerFilter::DealtDamageThisTurn,
                    ])),
                    Cmp::Gt,
                    Value::c(0),
                ),
                then: Box::new(enter(Value::c(n.unwrap_or(0).max(0)))),
                otherwise: Box::new(Effect::Noop),
            },
        };
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(effect)),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Bloodthirst.name(),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == DAMAGE_TO_OPPONENTS).then(|| {
            g.history
                .damage_dealt_to_players
                .iter()
                .filter(|(p, _)| g.are_opponents(ctx.controller, **p))
                .map(|(_, d)| *d as i64)
                .sum()
        })
    }
}

inventory::submit! { KeywordRegistration(&Bloodthirst) }
