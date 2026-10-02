//! Aggregate values (CR 107, 607.3, 608.2h): characteristics of several objects combined
//! ("the greatest power among creatures you control", "the total mana value of ..."),
//! counts of different values among objects ("the number of card types among cards in
//! your graveyard", "the number of colors that spell is"), and values over players ("the
//! highest life total among all players"). See [`Value::Aggregate`],
//! [`Value::DistinctAmong`] and [`Value::OverPlayers`].

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use smol_str::SmolStr;
use std::collections::BTreeSet;

fn combine(op: AggOp, vals: impl Iterator<Item = i64>) -> i64 {
    match op {
        AggOp::Sum => vals.sum(),
        AggOp::Max => vals.max().unwrap_or(0),
        AggOp::Min => vals.min().unwrap_or(0),
    }
}

/// [`Value::Aggregate`].
pub fn aggregate(g: &Game, op: AggOp, stat: &Stat, sel: &Sel, ctx: &Ctx) -> i64 {
    let objs = g.eval_sel_objects(sel, ctx);
    let of = |id: ObjectId| -> i64 {
        let o = g.obj(id);
        match stat {
            Stat::Power => o.power() as i64,
            Stat::Toughness => o.toughness() as i64,
            Stat::PowerOrToughness => o.power().max(o.toughness()) as i64,
            Stat::ManaValue => g.mana_value_of(id) as i64,
            Stat::Counters(Some(k)) => o.counter(k) as i64,
            Stat::Counters(None) => o.counters.values().map(|n| *n as i64).sum(),
        }
    };
    combine(op, objs.into_iter().map(of))
}

/// [`Value::DistinctAmong`].
pub fn distinct_among(g: &Game, what: Among, sel: &Sel, ctx: &Ctx) -> i64 {
    let objs = g.eval_sel_objects(sel, ctx);
    let chars = || objs.iter().map(|o| &g.obj(*o).chars);
    match what {
        Among::CardTypes | Among::PermanentTypes => {
            let mut set = CardTypeSet::NONE;
            for c in chars() {
                set = set.union(c.card_types);
            }
            set.iter()
                .filter(|t| {
                    what == Among::CardTypes
                        || matches!(
                            t,
                            CardType::Artifact
                                | CardType::Battle
                                | CardType::Creature
                                | CardType::Enchantment
                                | CardType::Land
                                | CardType::Planeswalker
                        )
                })
                .count() as i64
        }
        Among::CreatureTypes => {
            // An object that is every creature type has all of them (CR 702.73a).
            if chars().any(crate::kw::changeling::every_creature_type) {
                return subtype_lists().creature.len() as i64;
            }
            let mut set = BTreeSet::new();
            for c in chars() {
                if c.is(CardType::Creature) || c.is(CardType::Kindred) {
                    for s in &c.subtypes {
                        if is_creature_type(s) {
                            set.insert(s.clone());
                        }
                    }
                }
            }
            set.len() as i64
        }
        Among::BasicLandTypes => {
            let mut set = BTreeSet::new();
            for c in chars() {
                for s in &c.subtypes {
                    if is_basic_land_type(s) {
                        set.insert(s.clone());
                    }
                }
            }
            set.len() as i64
        }
        Among::Colors => {
            let mut set = ColorSet::NONE;
            for c in chars() {
                set = set.union(c.colors);
            }
            set.count() as i64
        }
        Among::ManaValues => objs
            .iter()
            .map(|o| g.mana_value_of(*o))
            .collect::<BTreeSet<_>>()
            .len() as i64,
        Among::ManaCosts => chars()
            .filter_map(|c| c.mana_cost.as_ref())
            .filter(|m| !m.symbols.is_empty())
            .map(|m| {
                let mut v: Vec<String> = m.symbols.iter().map(|s| format!("{s:?}")).collect();
                v.sort();
                v
            })
            .collect::<BTreeSet<_>>()
            .len() as i64,
        Among::Powers => objs
            .iter()
            .map(|o| g.obj(*o).power())
            .collect::<BTreeSet<_>>()
            .len() as i64,
        Among::Names => crate::names::distinct_name_count(chars()) as i64,
        Among::CounterKinds => objs
            .iter()
            .flat_map(|o| {
                g.obj(*o)
                    .counters
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .map(|(k, _)| k.clone())
            })
            .collect::<BTreeSet<_>>()
            .len() as i64,
        Among::LargestCreatureTypeGroup => {
            let all = chars()
                .filter(|c| crate::kw::changeling::every_creature_type(c))
                .count() as i64;
            let mut counts: std::collections::BTreeMap<SmolStr, i64> = Default::default();
            for c in chars() {
                if crate::kw::changeling::every_creature_type(c)
                    || !(c.is(CardType::Creature) || c.is(CardType::Kindred))
                {
                    continue;
                }
                let mut seen = BTreeSet::new();
                for s in &c.subtypes {
                    if is_creature_type(s) && seen.insert(s.clone()) {
                        *counts.entry(s.clone()).or_default() += 1;
                    }
                }
            }
            all + counts.values().copied().max().unwrap_or(0)
        }
    }
}

/// [`Value::OverPlayers`].
pub fn over_players(g: &Game, op: AggOp, f: &PlayerFilter, v: &Value, ctx: &Ctx) -> i64 {
    let players: Vec<PlayerId> = g
        .players_in_game()
        .into_iter()
        .filter(|p| g.player_filter_matches(f, *p, ctx))
        .collect();
    let mut c = ctx.clone();
    let vals: Vec<i64> = players
        .into_iter()
        .map(|p| {
            c.iter_player = Some(p);
            g.eval_value(v, &c)
        })
        .collect();
    combine(op, vals.into_iter())
}
