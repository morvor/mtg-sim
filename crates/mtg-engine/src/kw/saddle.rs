//! CR 702.171 Saddle.
//!
//! * "Saddle N" means "Tap any number of other untapped creatures you control with total
//!   power N or greater: This permanent becomes saddled until end of turn. Activate only
//!   as a sorcery." (CR 702.171a). The cost is a [`CostPart::TapTotalPower`] for saddle,
//!   paid like crew's (`kw/crew.rs`), so effects that change how creatures crew can speak
//!   of saddling too ("~ saddles Mounts and crews Vehicles as though its power were 2
//!   greater").
//! * Saddled is a designation (`GameObject::saddled`, [`SADDLED`]) with no rules meaning
//!   of its own. Only permanents can become saddled; a permanent stays saddled until the
//!   end of the turn (the cleanup step, CR 514.2) or until it leaves the battlefield (it
//!   becomes a new object, CR 400.7). It isn't a copiable value (CR 702.171b).
//!   Becoming saddled reports a [`BECAME_SADDLED`] event, for "whenever ~ becomes
//!   saddled [for the first time each turn]".
//! * A creature "saddles" a permanent as it's tapped to pay the cost of that permanent's
//!   saddle ability (CR 702.171c): those creatures are recorded for the turn
//!   (`TurnHistory::crewed`, with the saddle keyword) — "creature that saddled it this
//!   turn" ([`SADDLED_IT_THIS_TURN`]) — and "whenever ~ saddles a Mount" triggers look at
//!   the objects the saddle ability's cost tapped ([`SADDLES_A_MOUNT`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{EventInfo, StackKind, Zone};
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom`: the saddled designation (CR 702.171b), evaluated in `custom.rs`.
pub const SADDLED: &str = "saddled";
/// `Event::Custom` name: a permanent (`obj`) became saddled. The amount is the number of
/// creatures that saddled it, if a saddle ability made it saddled.
pub const BECAME_SADDLED: &str = "became saddled";
/// `Effect::Custom`: each permanent in [`SADDLE_VAR`] becomes saddled until end of turn.
const BECOME_SADDLED: &str = "saddle:becomes saddled";
/// The variable holding the permanents that become saddled (see [`becomes_saddled`]).
const SADDLE_VAR: Var = vars::USER + 1717;
/// `TriggerCond::Custom`: "whenever ~ saddles a Mount" (CR 702.171c). The trigger object
/// is the Mount ("that Mount").
pub const SADDLES_A_MOUNT: &str = "saddle:~ saddles a mount";
/// `Filter::Custom`: "creature that saddled it this turn" — a creature tapped this turn to
/// pay for a saddle ability of the source (CR 702.171c).
pub const SADDLED_IT_THIS_TURN: &str = "saddle:saddled it this turn";

/// The variable of an activated ability's saved context holding the objects its cost
/// tapped (see `Game::activate_ability`).
const COST_OBJECTS: Var = vars::USER + 91;

/// "[Permanents] become saddled until end of turn".
pub fn becomes_saddled(what: Sel) -> Effect {
    Effect::ForEach {
        sel: what,
        var: SADDLE_VAR,
        effect: Box::new(Effect::Custom(BECOME_SADDLED.into())),
    }
}

/// Whether `id` is a saddled permanent (CR 702.171b).
pub fn is_saddled(g: &Game, id: ObjectId) -> bool {
    let o = g.obj(id);
    g.is_live(id) && o.zone == Zone::Battlefield && o.saddled
}

/// The permanent `id` becomes saddled until end of turn (CR 702.171a–b); `saddlers` is the
/// number of creatures that saddled it, if its saddle ability did. Only permanents can
/// become saddled.
pub fn become_saddled(g: &mut Game, id: ObjectId, saddlers: usize) -> bool {
    if !g.is_live(id) || g.obj(id).zone != Zone::Battlefield {
        return false;
    }
    g.obj_mut(id).saddled = true;
    let controller = g.obj(id).controller;
    g.log(|g| format!("{} becomes saddled", g.describe(id)));
    g.emit(Event::Custom {
        name: SmolStr::new(BECAME_SADDLED),
        player: Some(controller),
        obj: Some(id),
        amount: saddlers as i32,
    });
    true
}

/// Whether `creature` saddled `mount` this turn (CR 702.171c).
pub fn saddled_this_turn(g: &Game, mount: ObjectId, creature: ObjectId) -> bool {
    g.history
        .crewed
        .iter()
        .any(|r| r.keyword == KeywordKind::Saddle && r.vehicle == mount && r.creature == creature)
}

/// The saddle ability "Saddle N" stands for (CR 702.171a).
fn saddle_ability(n: i32) -> Ability {
    let cost = Cost::free().with(CostPart::TapTotalPower {
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::Other,
            Filter::ControlledBy(PlayerRel::You),
        ]),
        power: Value::c(n),
        keyword: KeywordKind::Saddle,
    });
    let mut act = ActivatedAbility::new(cost, Body::effect(becomes_saddled(Sel::This)));
    act.timing = ActivationTiming::Sorcery;
    AbilityDef::new(AbilityKind::Activated(act), KeywordKind::Saddle.name())
}

/// Whether the stack object `id` is a saddle ability.
fn is_saddle_ability(g: &Game, id: ObjectId) -> bool {
    g.obj(id).stack.as_deref().is_some_and(|si| {
        matches!(&si.kind, StackKind::Activated { ability, .. } if ability.text == KeywordKind::Saddle.name())
    })
}

pub struct Saddle;

impl KeywordRules for Saddle {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Saddle]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![saddle_ability(kw.n.unwrap_or(0))])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != BECOME_SADDLED {
            return false;
        }
        let what: Vec<ObjectId> = ctx
            .vars
            .get(&SADDLE_VAR)
            .into_iter()
            .flatten()
            .filter_map(|e| e.object())
            .collect();
        // The creatures tapped to pay for the resolving saddle ability, if it's one.
        let saddlers = ctx.vars.get(&COST_OBJECTS).map_or(0, Vec::len);
        let by_ability = ctx.stack_obj.is_some_and(|s| is_saddle_ability(g, s));
        for id in what {
            let id = g.current(id);
            let n = if by_ability && Some(id) == ctx.source {
                saddlers
            } else {
                0
            };
            become_saddled(g, id, n);
        }
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != SADDLED_IT_THIS_TURN {
            return None;
        }
        let src = ctx.source?;
        Some(saddled_this_turn(g, src, id))
    }

    /// CR 702.171c: "whenever ~ saddles a Mount": the source was tapped to pay for a
    /// saddle ability that was just activated.
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != SADDLES_A_MOUNT {
            return None;
        }
        let Event::AbilityActivated {
            ability: Some(a),
            source: mount,
            player,
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        let saddled = is_saddle_ability(g, *a)
            && g.saved_ctx
                .get(a)
                .and_then(|c| c.vars.get(&COST_OBJECTS))
                .is_some_and(|v| v.contains(&Entity::Object(src)));
        Some(if saddled {
            vec![EventInfo {
                object: Some(*mount),
                other: Some(src),
                player: Some(*player),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Saddle) }
