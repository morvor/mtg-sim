//! CR 702.134 Mentor: "Whenever this creature attacks, put a +1/+1 counter on target
//! attacking creature with power less than this creature's power." (CR 702.134a). Each
//! instance triggers separately (CR 702.134b). The target's power is compared with the
//! mentoring creature's power as targets are chosen and again on resolution (CR 608.2b),
//! using that creature's last known information if it has left the battlefield.
//!
//! "Whenever [a creature] mentors a creature" triggers whenever a mentor ability whose
//! source is the first creature and whose target is the second resolves (CR 702.134c):
//! the resolving ability reports a [`MENTORED`] event, which the [`MENTORS`] and
//! [`EQUIPPED_MENTORS`] trigger conditions match.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::EventInfo;
use crate::types::*;
use smol_str::SmolStr;

/// `Event::Custom` name: a mentor ability resolved. `obj` is the creature it targeted (the
/// mentored creature), `amount` the id of the ability's source (the mentoring creature).
pub const MENTORED: &str = "mentor:mentored";
/// `Effect::Custom`: reports [`MENTORED`] for the resolving mentor ability.
const REPORT: &str = "mentor:report";
/// `TriggerCond::Custom`: "whenever ~ mentors a creature". The trigger object is the
/// mentored creature ("that creature").
pub const MENTORS: &str = "mentor:~ mentors a creature";
/// `TriggerCond::Custom`: "whenever equipped creature mentors a creature".
pub const EQUIPPED_MENTORS: &str = "mentor:equipped creature mentors a creature";

pub struct Mentor;

impl KeywordRules for Mentor {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Mentor]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let target = TargetSpec::object(
            Filter::and(vec![
                Filter::creature(),
                Filter::Attacking,
                Filter::Power(Cmp::Lt, Box::new(Value::PowerOf(Box::new(Sel::This)))),
            ]),
            "target attacking creature with lesser power",
        );
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::simple(
                    vec![target],
                    Effect::Seq(vec![
                        Effect::AddCounters {
                            what: Sel::Target(0),
                            kind: counters::PLUS1.into(),
                            n: Value::c(1),
                        },
                        Effect::Custom(SmolStr::new(REPORT)),
                    ]),
                ),
            )),
            KeywordKind::Mentor.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REPORT {
            return false;
        }
        let (Some(src), Some(Entity::Object(t))) = (
            ctx.source,
            ctx.targets.first().and_then(|v| v.first()).copied(),
        ) else {
            return true;
        };
        g.emit(Event::Custom {
            name: SmolStr::new(MENTORED),
            player: Some(ctx.controller),
            obj: Some(g.current(t)),
            amount: src.0 as i32,
        });
        true
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != MENTORS && name != EQUIPPED_MENTORS {
            return None;
        }
        let Event::Custom {
            name: n,
            player,
            obj: Some(mentored),
            amount,
        } = ev
        else {
            return Some(vec![]);
        };
        if n != MENTORED {
            return Some(vec![]);
        }
        let mentor = ObjectId(*amount as u32);
        let who = if name == MENTORS {
            Some(src)
        } else {
            match g.obj(src).attached_to {
                Some(Entity::Object(o)) => Some(o),
                _ => None,
            }
        };
        let matches = who.is_some_and(|w| g.current(w) == g.current(mentor));
        Some(if matches {
            vec![EventInfo {
                object: Some(*mentored),
                other: Some(mentor),
                player: *player,
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Mentor) }
