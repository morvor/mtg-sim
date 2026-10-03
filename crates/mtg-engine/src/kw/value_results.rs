//! Amounts from this turn's events ([`Value::EventsThisTurn`]): "the number of creatures
//! that died under your control this turn", "for each card you've discarded this turn",
//! "for each opponent who lost life this turn", "for each 2 life your opponents have lost
//! this turn". Every event of the turn counts, whether or not anything triggered on it
//! (CR 603.1b), and whether or not the objects involved are still where the events put
//! them; a triggered ability with the same trigger condition, source and controller would
//! have triggered on exactly these events. Events of the current action that haven't been
//! processed yet count too (CR 608.2h: the value is determined as the instruction is
//! performed).

use crate::ability::{Tally, TriggerCond};
use crate::eval::Ctx;
use crate::game::Game;
use std::collections::BTreeSet;

/// The permanents the latest destroy instruction destroyed, as they last existed on the
/// battlefield ("the number of creatures they controlled that were destroyed this way",
/// CR 608.2h); `vars::IT` holds the cards they became.
pub const DESTROYED: crate::ability::Var = crate::ability::vars::USER + 7520;

pub fn events_this_turn(g: &Game, cond: &TriggerCond, tally: Tally, ctx: &Ctx) -> i64 {
    let mut n = 0i64;
    let mut players = BTreeSet::new();
    for ev in g.turn_events.iter().chain(g.events.iter()) {
        for info in g.trigger_matches_ctx(cond, ctx, ev) {
            match tally {
                Tally::Events => n += 1,
                Tally::Amount => n += info.amount.max(0) as i64,
                Tally::Players => {
                    // A player who has left the game isn't counted (Gnoll War Band's
                    // ruling; CR 800.4a).
                    if let Some(p) = info.player.filter(|p| g.players_in_game().contains(p)) {
                        players.insert(p);
                    }
                }
            }
        }
    }
    if tally == Tally::Players {
        n = players.len() as i64;
    }
    n
}

/// The objects of this turn's events matching `cond` ([`crate::ability::Sel::ThisTurn`]):
/// for a spell cast, the spell; for damage dealt, its source; otherwise the object the
/// event was about. Each object once, in the order of the events.
pub fn event_objects_this_turn(
    g: &Game,
    cond: &TriggerCond,
    ctx: &Ctx,
) -> Vec<crate::types::ObjectId> {
    let mut out = Vec::new();
    for ev in g.turn_events.iter().chain(g.events.iter()) {
        for info in g.trigger_matches_ctx(cond, ctx, ev) {
            let o = match cond {
                TriggerCond::DealsDamage { .. } => info.other.or(info.object),
                _ => info.spell.or(info.object),
            };
            if let Some(o) = o.filter(|o| !out.contains(o)) {
                out.push(o);
            }
        }
    }
    out
}

/// `Value::Custom` "noncombat damage dealt this turn to:[you|opponents]": the total
/// noncombat damage dealt this turn to you or to your opponents ("the total amount of
/// noncombat damage dealt to your opponents this turn", CR 120.2b).
pub struct NoncombatDamageValue;

impl super::KeywordRules for NoncombatDamageValue {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let who = name.strip_prefix(crate::oracle::patterns::value_results::NONCOMBAT_DAMAGE_TO)?;
        let me = ctx.controller;
        let total = g
            .turn_events
            .iter()
            .chain(g.events.iter())
            .filter_map(|e| match e {
                crate::events::Event::Damage {
                    target: crate::types::Entity::Player(p),
                    amount,
                    combat: false,
                    ..
                } => {
                    let hit = match who {
                        "you" => *p == me,
                        "opponents" => g.are_opponents(me, *p),
                        _ => false,
                    };
                    hit.then_some(*amount as i64)
                }
                _ => None,
            })
            .sum();
        Some(total)
    }
}

inventory::submit! { super::KeywordRegistration(&NoncombatDamageValue) }

/// `Value::Custom`: the number of colors of mana spent to cast the spell that caused the
/// ability to trigger ("the number of colors of mana spent to cast that spell", CR
/// 601.2h, 105.1), from its last known information if it has left the stack.
pub const COLORS_SPENT_ON_THAT_SPELL: &str = "trigger spell:colors of mana spent";
/// `Value::Custom`: how many times the spell that caused the ability to trigger was kicked
/// (CR 702.33).
pub const TIMES_THAT_SPELL_WAS_KICKED: &str = "trigger spell:times kicked";

fn trigger_spell_cast(g: &Game, ctx: &Ctx) -> Option<crate::object::CastInfo> {
    let spell = ctx.event.as_ref().and_then(|e| e.spell)?;
    let o = g.obj(spell);
    o.stack
        .as_deref()
        .map(|si| &si.cast)
        .or(o.cast.as_deref())
        .filter(|c| c.was_cast)
        .cloned()
}

pub struct TriggerSpellValues;

impl super::KeywordRules for TriggerSpellValues {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        match name {
            COLORS_SPENT_ON_THAT_SPELL => Some(trigger_spell_cast(g, ctx).map_or(0, |c| {
                let mut s = crate::types::ColorSet::NONE;
                for m in &c.mana_spent {
                    if let Some(col) = m.color() {
                        s.insert(col);
                    }
                }
                s.count() as i64
            })),
            TIMES_THAT_SPELL_WAS_KICKED => {
                Some(trigger_spell_cast(g, ctx).map_or(0, |c| c.times_kicked as i64))
            }
            _ => None,
        }
    }
}

inventory::submit! { super::KeywordRegistration(&TriggerSpellValues) }
