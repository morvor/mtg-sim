//! CR 702.149 Training: "Whenever this creature and at least one other creature with power
//! greater than this creature's power attack, put a +1/+1 counter on this creature."
//! (CR 702.149a). Each instance triggers separately (CR 702.149b).
//!
//! Whether it triggers is determined as attackers are declared: raising a creature's power
//! afterward doesn't make it trigger, and once it has triggered, the other creature
//! leaving combat or getting smaller doesn't stop the counter (Cloaked Cadet rulings).
//!
//! "When this creature trains" means "When a resolving training ability puts one or more
//! +1/+1 counters on this creature" (CR 702.149c): the training ability reports it with
//! the [`TRAINED`] event.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{EventInfo, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: the training ability's effect, "put a +1/+1 counter on this creature".
pub const TRAIN: &str = "training:put a +1/+1 counter on this creature";
/// `Event::Custom` name: a resolving training ability put one or more +1/+1 counters on
/// the creature (`obj`).
pub const TRAINED: &str = "trained";
/// `TriggerCond::Custom`: "When this creature trains" (CR 702.149c).
pub const TRAINS_SELF: &str = "training:this creature trains";

pub struct Training;

impl KeywordRules for Training {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Training]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        // "... and at least one other creature with power greater than this creature's
        // power attack": both are declared as attackers together.
        let stronger_attacker = Filter::And(vec![
            Filter::Type(CardType::Creature),
            Filter::Attacking,
            Filter::Other,
            Filter::Power(Cmp::Gt, Box::new(Value::PowerOf(Box::new(Sel::This)))),
        ]);
        let t = TriggeredAbility::new(
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::Attacks(Filter::Source)),
                cond: Condition::Exists(stronger_attacker),
            },
            Body::effect(Effect::Custom(SmolStr::new(TRAIN))),
        );
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Training.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != TRAIN {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        // The same object, if it's still on the battlefield (CR 400.7).
        if !g.is_live(src) || g.obj(src).zone != Zone::Battlefield {
            return true;
        }
        let before = g.obj(src).counter(counters::PLUS1);
        g.add_counters(Entity::Object(src), counters::PLUS1, 1, Some(src));
        let put = g.obj(src).counter(counters::PLUS1).saturating_sub(before);
        if put > 0 {
            g.emit(Event::Custom {
                name: SmolStr::new(TRAINED),
                player: Some(ctx.controller),
                obj: Some(src),
                amount: put as i32,
            });
        }
        true
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != TRAINS_SELF {
            return None;
        }
        Some(match ev {
            Event::Custom {
                name: n,
                player,
                obj: Some(o),
                amount,
            } if n == TRAINED && *o == src => vec![EventInfo {
                object: Some(src),
                player: *player,
                amount: *amount,
                ..Default::default()
            }],
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&Training) }
