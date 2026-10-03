//! CR 702.100 Evolve: "Whenever a creature you control enters, if that creature's power is
//! greater than this creature's power and/or that creature's toughness is greater than
//! this creature's toughness, put a +1/+1 counter on this creature." (CR 702.100a).
//!
//! The comparison is an intervening "if" clause (CR 603.4): it's made when the creature
//! enters and again as the ability resolves, power to power and toughness to toughness,
//! using the entered creature's last known information if it has left the battlefield. A
//! creature can't have a greater power or toughness than a noncreature permanent
//! (CR 702.100c). Each instance triggers separately (CR 702.100d).
//!
//! A creature "evolves" when one or more +1/+1 counters are put on it as its evolve
//! ability resolves (CR 702.100b): reported as an [`Event::Custom`] named
//! [`EVOLVED_EVENT`], which "Whenever ~ evolves" ([`TriggerCond::Custom`] [`EVOLVES`])
//! triggers on.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::EventInfo;
use crate::types::*;

/// `Event::Custom` name: the object `obj` evolved.
pub const EVOLVED_EVENT: &str = "evolved";
/// `TriggerCond::Custom`: "Whenever ~ evolves".
pub const EVOLVES: &str = "evolve:this evolves";
/// `Effect::Custom`: "put a +1/+1 counter on this creature" of an evolve ability.
const EVOLVE: &str = "evolve:put a +1/+1 counter on this creature";

pub struct Evolve;

/// The intervening "if" clause of evolve: the entered creature (the trigger object) has
/// a greater power and/or toughness than this creature, both being creatures.
fn greater_condition() -> Condition {
    let entered = || Box::new(Sel::TriggerObject);
    let this = || Box::new(Sel::This);
    Condition::And(vec![
        // CR 702.100c.
        Condition::SelMatches(Sel::TriggerObject, Filter::creature()),
        Condition::SelMatches(Sel::This, Filter::creature()),
        Condition::Or(vec![
            Condition::Compare(
                Value::PowerOf(entered()),
                Cmp::Gt,
                Value::PowerOf(this()),
            ),
            Condition::Compare(
                Value::ToughnessOf(entered()),
                Cmp::Gt,
                Value::ToughnessOf(this()),
            ),
        ]),
    ])
}

impl KeywordRules for Evolve {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Evolve]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::and(vec![
                Filter::creature(),
                Filter::ControlledBy(PlayerRel::You),
            ])),
            Body::effect(Effect::Custom(EVOLVE.into())),
        );
        t.intervening_if = Some(greater_condition());
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Evolve.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EVOLVE {
            return false;
        }
        let Some(this) = ctx.source.filter(|s| g.is_live(*s)) else {
            return true;
        };
        let placed = g.add_counters(Entity::Object(this), counters::PLUS1, 1, Some(this));
        if placed > 0 && g.is_live(this) {
            g.emit(Event::Custom {
                name: EVOLVED_EVENT.into(),
                player: Some(ctx.controller),
                obj: Some(this),
                amount: placed as i32,
            });
        }
        true
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != EVOLVES {
            return None;
        }
        Some(match ev {
            Event::Custom {
                name: n,
                obj: Some(o),
                ..
            } if n == EVOLVED_EVENT && *o == src => vec![EventInfo {
                object: Some(src),
                player: Some(ctl),
                ..Default::default()
            }],
            _ => vec![],
        })
    }
}

inventory::submit! { KeywordRegistration(&Evolve) }
