//! Game-rules behaviors used by the oracle patterns `oracle/patterns/game_rules_*.rs`:
//! energy payments (CR 107.14), and other named custom behaviors.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

const PAY_ENERGY: &str = "game rules:pay energy:";

/// `Value::Custom` name: how many times the spell being cast (or considered) has been cast
/// from the command zone this game (CR 903.8; Myth Unbound).
pub const TIMES_IT_WAS_CAST: &str = "game rules:times it's been cast from the command zone";

/// `Condition::Custom` name: the current turn is an extra turn (CR 500.7).
pub const EXTRA_TURN: &str = "game rules:it's an extra turn";

/// `Filter::Custom` name: a Ring-bearer, of any player ("protection from Ring-bearers",
/// CR 701.54e).
pub const ANY_RING_BEARER: &str = "game rules:a ring-bearer";

/// `Filter::Custom` name: a loyalty ability on the stack ("target loyalty ability of a
/// planeswalker", CR 606).
pub const LOYALTY_ABILITY: &str = "game rules:loyalty ability on the stack";

/// `Restriction::Custom` names: players (relative to the effect's controller) can't
/// activate loyalty abilities (CR 606): "Players can't activate planeswalkers' loyalty
/// abilities." / "Your opponents can't ... activate planeswalkers' loyalty abilities this
/// turn.".
pub const NO_LOYALTY_EACH: &str = "game rules:can't activate loyalty abilities:each player";
pub const NO_LOYALTY_OPPONENTS: &str = "game rules:can't activate loyalty abilities:opponents";
/// `Restriction::Custom` name: "Enchanted player can't activate abilities that aren't mana
/// abilities or loyalty abilities." (CR 605, 606).
pub const ENCHANTED_ONLY_MANA_OR_LOYALTY: &str =
    "game rules:enchanted player can only activate mana and loyalty abilities";

/// Whether the restriction `name` from an effect controlled by `ctl` with source `src`
/// keeps `p` from activating `act`.
fn activation_forbidden(
    g: &Game,
    name: &str,
    src: Option<ObjectId>,
    ctl: PlayerId,
    p: PlayerId,
    act: &ActivatedAbility,
) -> bool {
    match name {
        NO_LOYALTY_EACH => act.is_loyalty,
        NO_LOYALTY_OPPONENTS => act.is_loyalty && g.are_opponents(ctl, p),
        ENCHANTED_ONLY_MANA_OR_LOYALTY => {
            let enchanted = src
                .filter(|s| g.is_live(*s))
                .and_then(|s| g.obj(s).attached_to)
                == Some(Entity::Player(p));
            enchanted && !act.is_loyalty && !act.is_mana_ability
        }
        _ => false,
    }
}

/// `Effect::Custom` name: "Pay {E}{E}." as an instruction: the controller pays `n` energy
/// if they can (CR 107.14: they can't pay more than they have); whether they did is
/// recorded for "If you can't, ...".
pub fn pay_energy(n: i32) -> String {
    format!("{PAY_ENERGY}{n}")
}

pub fn is_pay_energy(name: &str) -> bool {
    name.starts_with(PAY_ENERGY)
}

/// The amount of a mandatory energy payment.
pub fn pay_energy_amount(name: &str) -> Option<i32> {
    name.strip_prefix(PAY_ENERGY)?.parse().ok()
}

pub struct GameRules;

impl KeywordRules for GameRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != TIMES_IT_WAS_CAST {
            return None;
        }
        let Some(spell) = ctx.event.as_ref().and_then(|e| e.spell) else {
            return Some(0);
        };
        let owner = g.obj(spell).owner;
        let key = crate::kw::partner::commander_key(g, spell);
        Some(
            g.player(owner)
                .commander_casts
                .get(&key)
                .copied()
                .unwrap_or(0) as i64,
        )
    }

    fn custom_condition(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
        (name == EXTRA_TURN).then_some(g.turn.extra)
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name == ANY_RING_BEARER {
            return Some(crate::kwa::ring::is_ring_bearer(g, g.obj(id).controller, id));
        }
        if name != LOYALTY_ABILITY {
            return None;
        }
        let si = g.obj(id).stack.as_deref();
        Some(matches!(
            si.map(|s| &s.kind),
            Some(crate::object::StackKind::Activated { ability, .. })
                if matches!(&ability.kind, AbilityKind::Activated(a) if a.is_loyalty)
        ))
    }

    fn activation_allowed(&self, g: &Game, p: PlayerId, _src: ObjectId, a: &Ability) -> bool {
        let AbilityKind::Activated(act) = &a.kind else {
            return true;
        };
        let statics = g.statics.restrictions.iter().filter_map(|(s, c, r)| match r {
            Restriction::Custom(n) => Some((n.as_str(), Some(*s), *c)),
            _ => None,
        });
        let effects = g.rule_effects.iter().filter_map(|e| match &e.restriction {
            Restriction::Custom(n) => Some((n.as_str(), e.source, e.controller)),
            _ => None,
        });
        !statics
            .chain(effects)
            .any(|(n, s, c)| activation_forbidden(g, n, s, c, p, act))
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(n) = pay_energy_amount(name) {
            let p = ctx.controller;
            let cost = Cost::free().with(CostPart::PayEnergy(Value::c(n)));
            let paid = g.player(p).counter(counters::ENERGY) >= n.max(0) as u32
                && g.pay_cost(p, &cost, ctx.source, ctx);
            ctx.prev_happened = paid;
            return true;
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&GameRules) }
