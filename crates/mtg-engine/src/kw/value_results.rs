//! Amounts from this turn's events ([`Value::EventsThisTurn`]): "the number of creatures
//! that died under your control this turn", "for each card you've discarded this turn",
//! "for each opponent who lost life this turn", "for each 2 life your opponents have lost
//! this turn". Every event of the turn counts, whether or not anything triggered on it
//! (CR 603.1b), and whether or not the objects involved are still where the events put
//! them; a triggered ability with the same trigger condition, source and controller would
//! have triggered on exactly these events. Events of the current action that haven't been
//! processed yet count too (CR 608.2h: the value is determined as the instruction is
//! performed).

use crate::ability::{Filter, Tally, TriggerCond};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::Characteristics;
use crate::object::EventInfo;
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};
use std::collections::BTreeSet;
use std::sync::Arc;

/// A damage event of this turn (`TurnHistory::damage_as_dealt`): its index in
/// `Game::turn_events` and what its source was as it dealt the damage.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DamageAsDealt {
    pub index: usize,
    pub source: ObjectId,
    pub controller: PlayerId,
    pub chars: Arc<Characteristics>,
}

/// Records the damage event about to be added to this turn's events (called as it's
/// processed, before it's pushed onto `Game::turn_events`).
pub fn record_damage(g: &mut Game, source: ObjectId) {
    let o = g.obj(source);
    let rec = DamageAsDealt {
        index: g.turn_events.len(),
        source,
        controller: o.controller,
        chars: Arc::new(o.chars.clone()),
    };
    g.history.damage_as_dealt.push(rec);
}

/// `cond` matched against the `index`th event of this turn (`None`: not yet processed).
/// "[Quality] sources that dealt damage this turn" means what the sources were as they
/// dealt it, not what they are now (a creature that was a Pirate then counts even if it
/// isn't one now, CR 608.2h).
fn matches_at(
    g: &Game,
    cond: &TriggerCond,
    ctx: &Ctx,
    index: Option<usize>,
    ev: &Event,
) -> Vec<EventInfo> {
    if let (
        TriggerCond::DealsDamage {
            source,
            to,
            combat_only,
        },
        Event::Damage { source: s, .. },
    ) = (cond, ev)
    {
        let seen = index.and_then(|i| {
            g.history
                .damage_as_dealt
                .iter()
                .find(|r| r.index == i && r.source == *s)
        });
        if let Some(seen) = seen {
            let view = crate::eval::AsDeclared {
                id: *s,
                chars: &seen.chars,
                controller: seen.controller,
            };
            if !g.matches_view(&view, *s, source, ctx) {
                return vec![];
            }
            let any = TriggerCond::DealsDamage {
                source: Filter::Any,
                to: to.clone(),
                combat_only: *combat_only,
            };
            return g.trigger_matches_ctx(&any, ctx, ev);
        }
    }
    matches_of(g, cond, ctx, ev)
}

/// This turn's events (processed ones with their index), then the current action's.
fn turn_events_indexed(g: &Game) -> impl Iterator<Item = (Option<usize>, &Event)> {
    g.turn_events
        .iter()
        .enumerate()
        .map(|(i, e)| (Some(i), e))
        .chain(g.events.iter().map(|e| (None, e)))
}

/// The matches of this turn's event `ev` for `cond`. A permanent's entering the
/// battlefield is matched as the permanent entered, under the control it entered under
/// (it may have changed or come under another player's control since: "Activate only if
/// another artifact entered the battlefield under your control this turn"); other events
/// as [`Game::trigger_matches_ctx`] matches them.
fn matches_of(g: &Game, cond: &TriggerCond, ctx: &Ctx, ev: &Event) -> Vec<EventInfo> {
    if let (
        TriggerCond::EntersBattlefield(f),
        Event::ZoneChange {
            new,
            to: Zone::Battlefield,
            cause,
            by,
            ..
        },
    ) = (cond, ev)
    {
        if let Some((matched, controller)) = g.matched_as_entered(*new, f, ctx) {
            return if matched {
                vec![EventInfo {
                    object: Some(*new),
                    player: Some(controller),
                    cause: Some(*cause),
                    caused_by: *by,
                    ..Default::default()
                }]
            } else {
                vec![]
            };
        }
    }
    g.trigger_matches_ctx(cond, ctx, ev)
}

/// The permanents the latest destroy instruction destroyed, as they last existed on the
/// battlefield ("the number of creatures they controlled that were destroyed this way",
/// CR 608.2h); `vars::IT` holds the cards they became.
pub const DESTROYED: crate::ability::Var = crate::ability::vars::USER + 7520;

pub fn events_this_turn(g: &Game, cond: &TriggerCond, tally: Tally, ctx: &Ctx) -> i64 {
    let mut n = 0i64;
    let mut players = BTreeSet::new();
    // Excess damage dealt along with the damage being looked at (CR 120.10).
    let ahead = match cond {
        TriggerCond::DealtExcessDamage { .. } => g.excess_ahead.as_slice(),
        _ => &[],
    };
    let events = turn_events_indexed(g).chain(ahead.iter().map(|e| (None, e)));
    for (i, ev) in events {
        for info in matches_at(g, cond, ctx, i, ev) {
            match tally {
                Tally::Events => n += 1,
                Tally::Amount => n += info.amount.max(0) as i64,
                Tally::Players => {
                    // A player who has since left the game is still counted: the effect
                    // looks at actions taken this turn (CR 800.4i; interpretation
                    // `history-counts-players-who-left`).
                    if let Some(p) = info.player {
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
    for (i, ev) in turn_events_indexed(g) {
        for info in matches_at(g, cond, ctx, i, ev) {
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
