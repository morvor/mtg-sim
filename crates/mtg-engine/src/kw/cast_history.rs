//! How a spell was cast and paid for, as conditions and amounts about it (CR 601.2,
//! 601.2h): "if it was cast", "if you cast it from your graveyard", "if he was kicked",
//! "if at least four mana was spent to cast it", "if no colored mana was spent to cast
//! it", "if mana from a Treasure was spent to cast it", "if three or more mana from
//! creatures was spent to cast it", and the same for the mana spent to activate an
//! ability ("if mana from a Treasure was spent to activate this ability").
//!
//! The spell is one of:
//! - the source's own spell (`self`: a resolving spell, or the spell a permanent was);
//! - the spell that caused the ability to trigger (`that spell`), as it last existed;
//! - the ability being activated or resolving (`this ability`).
//!
//! A copy of a spell, or a permanent that wasn't cast, wasn't cast (CR 707.10): it had no
//! mana spent on it. Mana sources are recorded as the mana is spent (`record_payment`) and
//! looked at as they last existed (a sacrificed Treasure is still known to have been a
//! Treasure, CR 608.2h).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::{ManaType, SpendContext};
use crate::object::{CastInfo, Zone};
use crate::types::*;

/// `Filter::Custom`: the object (a spell, or a permanent that was one) was cast.
pub const WAS_CAST: &str = "cast history:was cast";
/// `Filter::Custom` prefix: the object was cast from this zone (`cast from:Hand`).
pub const CAST_FROM: &str = "cast history:cast from:";
/// `Filter::Custom`: the object was cast this turn.
pub const CAST_THIS_TURN: &str = "cast history:cast this turn";
/// `Filter::Custom` prefix: the object was cast with this keyword's alternative cost or
/// permission (`cast for:Warp`: "if that creature was cast for its warp cost").
pub const CAST_FOR: &str = "cast history:cast for:";
/// `Filter::Custom`: the object was kicked (CR 702.33).
pub const KICKED: &str = "cast history:kicked";
/// `Value::Custom`: how many times the ability being resolved has been activated this
/// turn, this activation included ("if this ability has been activated four or more times
/// this turn").
pub const TIMES_ACTIVATED: &str = "cast history:times this ability was activated this turn";
/// `Filter::Custom` prefix: a spell cast this turn no later than the spell `whose` names
/// (`self` or `that spell`): "if it's the second creature spell you cast this turn" counts
/// the creature spells cast up to it.
pub const CAST_BY_THEN: &str = "cast history:cast no later than:";
/// `Value::Custom` prefix: an amount of the mana spent on a spell or ability:
/// `cast history:spent:<whose>:<what>`, `whose` being `self`, `that spell` or `this
/// ability`, `what` being `total`, `colored`, `colors` (how many colors), `type:<C>` (one
/// type of mana), `source subtype:<Treasure>` or `source type:<Creature>`.
pub const SPENT: &str = "cast history:spent:";

/// The `Value::Custom` name for an amount of mana spent.
pub fn spent(whose: &str, what: &str) -> String {
    format!("{SPENT}{whose}:{what}")
}

/// Records where the mana that paid a cost came from: on the spell being cast, or on the
/// ability being activated (with the types of that mana, which a spell records as its
/// total cost is paid).
pub fn record_payment(g: &mut Game, spend: &SpendContext, types: &[ManaType], sources: Vec<ObjectId>) {
    let (target, ability) = match (spend.is_spell, spend.source, spend.ability_on_stack) {
        (true, Some(spell), _) => (spell, false),
        (false, _, Some(ability)) => (ability, true),
        _ => return,
    };
    let Some(o) = g.objects.get_mut(target.0 as usize) else {
        return;
    };
    if o.zone != Zone::Stack {
        return;
    }
    if let Some(si) = o.stack.as_mut() {
        si.cast.mana_sources.extend(sources);
        if ability {
            si.cast.mana_spent.extend(types.iter().copied());
        }
    }
}

/// The cast information of an object: a spell's, or the spell's a permanent was.
fn object_cast(g: &Game, id: ObjectId) -> Option<&CastInfo> {
    let o = g.try_obj(id)?;
    o.stack
        .as_deref()
        .map(|si| &si.cast)
        .or(o.cast.as_deref())
}

/// The payment `whose` refers to.
fn payment<'a>(g: &'a Game, whose: &str, ctx: &'a Ctx) -> Option<&'a CastInfo> {
    match whose {
        "self" => g.cast_info(ctx).filter(|c| c.was_cast),
        "that spell" => {
            let spell = ctx.event.as_ref().and_then(|e| e.spell)?;
            object_cast(g, spell).filter(|c| c.was_cast)
        }
        w if w.starts_with("target ") => {
            let slot: usize = w["target ".len()..].parse().ok()?;
            let id = ctx.targets.get(slot)?.iter().find_map(|e| match e {
                Entity::Object(o) => Some(*o),
                _ => None,
            })?;
            object_cast(g, id).filter(|c| c.was_cast)
        }
        "this ability" => {
            let a = ctx.stack_obj?;
            g.try_obj(a)?.stack.as_deref().map(|si| &si.cast)
        }
        _ => None,
    }
}

fn amount(g: &Game, c: &CastInfo, what: &str) -> Option<i64> {
    let n = |f: &dyn Fn(&ManaType) -> bool| c.mana_spent.iter().filter(|m| f(m)).count() as i64;
    if what == "total" {
        return Some(c.mana_spent.len() as i64);
    }
    if what == "colored" {
        return Some(n(&|m| m.color().is_some()));
    }
    if what == "colors" {
        let mut s = ColorSet::NONE;
        for m in &c.mana_spent {
            if let Some(col) = m.color() {
                s.insert(col);
            }
        }
        return Some(s.count() as i64);
    }
    if let Some(t) = what.strip_prefix("type:") {
        let t = ManaType::ALL.iter().find(|m| format!("{m:?}") == t)?;
        return Some(n(&|m| m == t));
    }
    let from = |f: &dyn Fn(&crate::object::GameObject) -> bool| {
        c.mana_sources
            .iter()
            .filter(|s| g.try_obj(**s).is_some_and(f))
            .count() as i64
    };
    if let Some(st) = what.strip_prefix("source subtype:") {
        return Some(from(&|o| o.chars.subtypes.iter().any(|x| x.as_str() == st)));
    }
    if let Some(t) = what.strip_prefix("source type:") {
        let t = CardType::from_word(t)?;
        return Some(from(&|o| o.chars.card_types.contains(t)));
    }
    None
}

struct CastHistory;

impl KeywordRules for CastHistory {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == TIMES_ACTIVATED {
            let n = ctx.source.and_then(|s| {
                g.obj(s)
                    .activations_this_turn
                    .get(&ctx.ability_uid)
                    .copied()
            });
            return Some(n.unwrap_or(0) as i64);
        }
        let r = name.strip_prefix(SPENT)?;
        let (whose, what) = r.split_once(':')?;
        Some(payment(g, whose, ctx).map_or(0, |c| amount(g, c, what).unwrap_or(0)))
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if let Some(whose) = name.strip_prefix(CAST_BY_THEN) {
            let order = &g.history.spells_cast;
            let pos = |o: ObjectId| order.iter().position(|(_, s)| *s == o);
            // The source's own spell: the source, or the spell it was (CR 400.7).
            let then = match whose {
                "self" => ctx
                    .source
                    .and_then(|s| pos(s).or_else(|| g.obj(s).prev.and_then(pos))),
                "that spell" => ctx.event.as_ref().and_then(|e| e.spell).and_then(pos),
                _ => None,
            };
            return Some(match (then, pos(id)) {
                (Some(then), Some(i)) => i <= then,
                _ => false,
            });
        }
        let cast = object_cast(g, id).filter(|c| c.was_cast);
        if name == WAS_CAST {
            return Some(cast.is_some());
        }
        if name == CAST_THIS_TURN {
            return Some(cast.is_some_and(|c| c.turn == g.turn.number));
        }
        if name == KICKED {
            return Some(cast.is_some_and(|c| c.times_kicked > 0 || c.paid.iter().any(|p| p == "kicker")));
        }
        if let Some(k) = name.strip_prefix(CAST_FOR) {
            return Some(cast.is_some_and(|c| {
                matches!(&c.method, crate::object::CastMethod::Keyword(kind) if format!("{kind:?}") == k)
            }));
        }
        let z = name.strip_prefix(CAST_FROM)?;
        Some(cast.is_some_and(|c| c.from.is_some_and(|f| format!("{f:?}") == z)))
    }
}

inventory::submit! { KeywordRegistration(&CastHistory) }
