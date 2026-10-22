//! CR 702.69 Gravestorm: "When you cast this spell, copy it for each permanent that was
//! put into a graveyard from the battlefield this turn. If the spell has any targets, you
//! may choose new targets for any of the copies." A triggered ability that functions on
//! the stack (CR 702.69a); each instance triggers separately (CR 702.69b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;

/// The number of permanents put into a graveyard from the battlefield this turn, counted
/// as the gravestorm ability resolves (tokens included).
pub const GRAVESTORM_COUNT: &str = "gravestorm:permanents put into a graveyard this turn";

pub struct Gravestorm;

impl KeywordRules for Gravestorm {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Gravestorm]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Body::effect(Effect::CopySpell {
                what: Sel::This,
                count: Value::Custom(GRAVESTORM_COUNT.into()),
                new_targets: true,
            }),
        );
        t.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Gravestorm.name(),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<i64> {
        if name != GRAVESTORM_COUNT {
            return None;
        }
        let n = g
            .turn_events
            .iter()
            .chain(g.events.iter())
            .filter(|e| {
                matches!(
                    e,
                    Event::ZoneChange {
                        from: Zone::Battlefield,
                        to: Zone::Graveyard(_),
                        ..
                    }
                )
            })
            .count();
        Some(n as i64)
    }
}

inventory::submit! { KeywordRegistration(&Gravestorm) }
