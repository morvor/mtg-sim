//! CR 702.159 Visit: "Visit — [Effect]" means "Whenever you roll to visit your
//! Attractions, if the result is equal to a number that is lit up on this Attraction,
//! [effect]." (CR 702.159a; the triggered ability is compiled from the card's text, see
//! `oracle/patterns/r703_704_variants.rs`, and triggers from `variants.rs`).
//!
//! "Claim the prize" (CR 702.159b): the paragraph starting with "Prize —" is part of the
//! visit ability. It's compiled as an ability of the Attraction with the
//! [`PRIZE`] trigger condition, which never triggers on its own: claiming the prize
//! ([`CLAIM_THE_PRIZE`]) performs its effect as part of the resolving visit ability, and
//! "whenever you claim the prize of an Attraction" abilities trigger.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::EventInfo;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `TriggerCond::Custom` of an Attraction's prize paragraph (performed by claiming the
/// prize, never triggered by events).
pub const PRIZE: &str = "visit:prize";
/// `Effect::Custom`: "claim the prize" (CR 702.159b).
pub const CLAIM_THE_PRIZE: &str = "visit:claim the prize";
/// `Event::Custom`: `player` claimed the prize of the Attraction `obj`.
pub const CLAIMED: &str = "claimed the prize";
/// `TriggerCond::Custom`: "whenever you claim the prize of an Attraction".
pub const YOU_CLAIM: &str = "visit:you claim the prize of an attraction";

pub struct Visit;

impl KeywordRules for Visit {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Visit]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CLAIM_THE_PRIZE {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        // The prize of the Attraction whose visit ability this is (as it last existed on
        // the battlefield if it has left).
        let prize = g.obj(src).chars.abilities.iter().find_map(|a| match &a.kind {
            AbilityKind::Triggered(t)
                if matches!(&t.trigger, TriggerCond::Custom(n) if n == PRIZE) =>
            {
                Some(t.body.effect.clone())
            }
            _ => None,
        });
        g.log(|g| format!("{} claims the prize of {}", ctx.controller, g.obj(src).chars.name));
        g.emit(Event::Custom {
            name: CLAIMED.into(),
            player: Some(ctx.controller),
            obj: Some(src),
            amount: 0,
        });
        if let Some(e) = prize {
            g.exec(&e, ctx);
        }
        true
    }

    fn custom_trigger(
        &self,
        _g: &Game,
        name: &str,
        _src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        match name {
            PRIZE => Some(vec![]),
            YOU_CLAIM => Some(match ev {
                Event::Custom {
                    name: n,
                    player: Some(p),
                    obj,
                    ..
                } if n == CLAIMED && *p == ctl => vec![EventInfo {
                    object: *obj,
                    player: Some(*p),
                    ..Default::default()
                }],
                _ => vec![],
            }),
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&Visit) }
