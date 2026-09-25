//! Flipping coins (CR 705) and rolling dice (CR 706).
//!
//! * [`CoinFlip`] ([`Effect::FlipCoins`]): the flipper calls heads or tails unless the
//!   effect cares only about which side comes up (CR 705.2); effects can fix a flip's
//!   result (CR 705.3).
//! * [`DieRoll`] ([`Effect::RollDice`]): N-sided dice (CR 706.1a), natural results and
//!   modifiers (CR 706.2), results tables (CR 706.3), "roll again" (CR 706.3c), doubles
//!   (CR 706.5), ignored rolls (CR 706.6), and stored results (CR 706.8).
//! * [`DiceStatic`] ([`StaticEffect::Dice`]): static abilities that change how coins are
//!   flipped or dice are rolled, and optional die-roll modifiers (CR 706.2a, 706.2b).
//!
//! Rolling the planar die (CR 901.9) is also a die roll for triggers, but it has no
//! numerical result (CR 706.7); see [`planar_die_rolled`].

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::object::EventInfo;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// "The result" of the most recent die roll of the resolving ability (inside a results
/// table row: that row's result). Read with `Value::Var(RESULT)`.
pub const RESULT: Var = vars::USER + 120;
/// The natural result of the most recent die roll (CR 706.2).
pub const NATURAL: Var = vars::USER + 121;
/// 1 if the most recent roll of two or more dice rolled doubles (CR 706.5).
pub const DOUBLES: Var = vars::USER + 122;
/// Set by "roll again" in a results table row (CR 706.3c).
pub const AGAIN: Var = vars::USER + 123;
/// Coin flips won, lost, and coins that came up heads/tails in the most recent flip.
pub const WINS: Var = vars::USER + 124;
pub const LOSSES: Var = vars::USER + 125;
pub const HEADS: Var = vars::USER + 126;
pub const TAILS: Var = vars::USER + 127;

/// `Value::Custom`: "the result" — of the resolving ability's own roll, or else of the
/// roll that triggered the ability ("whenever you roll one or more dice, ... equal to
/// the result"). A planar die roll has no numerical result (CR 706.7): 0.
pub const THE_RESULT: &str = "die:the result";
/// `Condition::Custom`: "if you rolled doubles" (CR 706.5).
pub const ROLLED_DOUBLES: &str = "die:rolled doubles";
/// `Effect::Custom`: "roll again" (CR 706.3c).
pub const ROLL_AGAIN: &str = "die:roll again";
/// `Effect::Custom`: "reroll any number of this permanent's stored results" (CR 706.8b).
pub const REROLL_STORED: &str = "die:reroll stored results";
/// `Value::Custom`: "the greatest number of stored results on it of the same value"
/// (CR 706.8a).
pub const STORED_SAME: &str = "die:stored results of the same value";
/// `TriggerCond::Custom`: "whenever you roll a die's highest natural result".
pub const NATURAL_MAX: &str = "die:highest natural result";

/// Per-game dice and coin bookkeeping.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiceState {
    /// Natural results the next die rolls will show, in order (a loaded die, for tests and
    /// replays of physical rolls). Values outside 1..=N are clamped into that range.
    pub loaded: VecDeque<u32>,
    /// What the next coin flips will show (true = heads).
    pub loaded_coins: VecDeque<bool>,
    /// Results stored on permanents (CR 706.8a): (kind of die, value) per object.
    pub stored: BTreeMap<ObjectId, Vec<(u32, i64)>>,
    /// Once-each-turn die-roll modifiers used: (source, modifier index, turn).
    pub used: Vec<(ObjectId, usize, u32)>,
}

/// "Roll [count] d[sides] [and add bonus] [and ignore ...]" plus its results table.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DieRoll {
    pub who: PlayerRef,
    pub count: Value,
    /// The kind of die: an N-sided die (CR 706.1a).
    pub sides: u32,
    /// A modifier printed in the instruction ("and add the number of cards in your hand",
    /// CR 706.2).
    pub bonus: Option<Value>,
    /// Rolls the instruction says to ignore (CR 706.6).
    pub ignore: DiceIgnore,
    /// The results table (CR 706.3a).
    pub table: Vec<ResultRow>,
    /// "store those results on it" (CR 706.8a).
    pub store_on: Option<Sel>,
}

impl DieRoll {
    pub fn new(sides: u32) -> DieRoll {
        DieRoll {
            who: PlayerRef::You,
            count: Value::c(1),
            sides,
            bonus: None,
            ignore: DiceIgnore::None,
            table: vec![],
            store_on: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceIgnore {
    None,
    /// "and ignore the lower/lowest roll(s)".
    Lowest(u32),
    /// "and ignore all but the highest roll".
    AllButHighest,
}

/// A striation of a results table: "lo—hi | effect", "N | effect", or "N+ | effect".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultRow {
    pub lo: i64,
    /// `None`: "N+" (no upper bound).
    pub hi: Option<i64>,
    pub effect: Effect,
}

impl ResultRow {
    pub fn contains(&self, r: i64) -> bool {
        r >= self.lo && self.hi.is_none_or(|h| r <= h)
    }
}

/// "Flip [count] coin(s)" and what happens depending on the result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinFlip {
    pub who: PlayerRef,
    pub count: Value,
    /// The flipper calls heads or tails and wins or loses (CR 705.2). False when the
    /// effect cares only about whether the coin comes up heads or tails.
    pub call: bool,
    /// "flip a coin until you lose a flip".
    pub until_lose: bool,
    pub on_win: Effect,
    pub on_lose: Effect,
    pub on_heads: Effect,
    pub on_tails: Effect,
}

impl CoinFlip {
    pub fn new() -> CoinFlip {
        CoinFlip {
            who: PlayerRef::You,
            count: Value::c(1),
            call: true,
            until_lose: false,
            on_win: Effect::Noop,
            on_lose: Effect::Noop,
            on_heads: Effect::Noop,
            on_tails: Effect::Noop,
        }
    }
}

impl Default for CoinFlip {
    fn default() -> Self {
        CoinFlip::new()
    }
}

/// Static abilities about coins and dice.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiceStatic {
    /// "If you would roll one or more dice, instead roll that many dice plus one and
    /// ignore the lowest roll." (`who` relative to the controller).
    ExtraDie { who: PlayerRel },
    /// "If you would flip a coin, instead flip two coins and ignore one."
    ExtraCoin { who: PlayerRel },
    /// "The first time you flip one or more coins each turn, those coins come up heads and
    /// you win those flips." (CR 705.3)
    FirstFlipsWin { who: PlayerRel },
    /// An optional modifier of die rolls (CR 706.2a).
    Modifier(DieModifier),
}

/// An optional modifier of die rolls, possibly with a cost (CR 706.2a).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DieModifier {
    /// Whose rolls it can modify, relative to its controller.
    pub whose: PlayerRel,
    pub kind: ModifierKind,
    pub cost: Cost,
    pub once_per_turn: bool,
    /// Only a roll with this natural result ("if you roll a 3").
    pub natural: Option<i64>,
    /// Only rolls of this kind of die.
    pub sides: Option<u32>,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierKind {
    Reroll,
    Add(i64),
}

/// One die as it's being rolled: its natural result and changes from modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RolledDie {
    pub sides: u32,
    pub natural: i64,
    pub adjust: i64,
}

impl RolledDie {
    pub fn result(&self) -> i64 {
        self.natural + self.adjust
    }
}

/// A natural result of an N-sided die: equally likely outcomes 1..=N (CR 706.1a, 706.1b).
fn natural(g: &mut Game, sides: u32) -> i64 {
    let sides = sides.max(1);
    match g.dice.loaded.pop_front() {
        Some(v) => v.clamp(1, sides) as i64,
        None => g.random_range(1, sides) as i64,
    }
}

/// A coin flip: heads or tails with equal likelihood (CR 705.1).
fn coin(g: &mut Game) -> bool {
    match g.dice.loaded_coins.pop_front() {
        Some(h) => h,
        None => g.random_range(0, 1) == 1,
    }
}

/// Static abilities about coins and dice, with their sources and controllers.
fn statics(g: &Game) -> Vec<(ObjectId, PlayerId, usize, DiceStatic)> {
    let mut out = Vec::new();
    let mut idx: BTreeMap<ObjectId, usize> = BTreeMap::new();
    for (src, ctl, e) in &g.statics.other {
        if let StaticEffect::Dice(d) = e {
            let i = idx.entry(*src).or_insert(0);
            out.push((*src, *ctl, *i, d.clone()));
            *i += 1;
        }
    }
    out
}

fn applies_to(g: &Game, rel: PlayerRel, controller: PlayerId, p: PlayerId) -> bool {
    g.player_rel_matches(rel, p, &Ctx::new(None, controller))
}

/// Performs a die roll instruction (CR 706).
pub fn roll(g: &mut Game, spec: &DieRoll, ctx: &mut Ctx) {
    let players = g.eval_players(&spec.who, ctx);
    let mut rolled = false;
    for p in players {
        let n = g.eval_value(&spec.count, ctx).max(0) as u32;
        let mut guard = 0;
        loop {
            guard += 1;
            ctx.nums.remove(&AGAIN);
            // CR 706.3c: rolling again uses the same kind and number of dice, including
            // the instruction's modifiers.
            let bonus = spec.bonus.as_ref().map_or(0, |b| g.eval_value(b, ctx));
            let dice = roll_dice(g, p, n, spec.sides, spec.ignore, bonus, ctx.source);
            rolled |= !dice.is_empty();
            let results: Vec<i64> = dice.iter().map(|d| d.result() + bonus).collect();
            let total: i64 = results.iter().sum();
            ctx.nums
                .insert(NATURAL, dice.first().map_or(0, |d| d.natural));
            // CR 706.5: doubles.
            let doubles = results.len() >= 2 && results.iter().all(|r| *r == results[0]);
            ctx.nums.insert(DOUBLES, doubles as i64);
            if let Some(sel) = &spec.store_on {
                for o in g.resolve_objects(sel, ctx) {
                    let v = g.dice.stored.entry(o).or_default();
                    v.extend(results.iter().map(|r| (spec.sides, *r)));
                }
            }
            // CR 706.3a: use each result to determine which effect of the table happens.
            for r in &results {
                if let Some(row) = spec.table.iter().find(|row| row.contains(*r)) {
                    ctx.nums.insert(RESULT, *r);
                    let e = row.effect.clone();
                    g.exec(&e, ctx);
                }
            }
            ctx.nums.insert(RESULT, total);
            if ctx.nums.get(&AGAIN).copied() != Some(1) || guard >= 100 {
                break;
            }
        }
    }
    ctx.nums.remove(&AGAIN);
    // "Roll a six-sided die. When you do, ..." (a reflexive trigger, CR 603.12).
    ctx.prev_happened = rolled;
}

/// Rolls `n` N-sided dice for player `p`, applying effects that change the roll (extra
/// dice and ignored rolls, CR 706.6) and modifiers (CR 706.2). Returns the dice that
/// count (their results don't include `bonus`, the instruction's own modifier), after
/// emitting a [`Event::DieRolled`] for each.
pub fn roll_dice(
    g: &mut Game,
    p: PlayerId,
    n: u32,
    sides: u32,
    ignore: DiceIgnore,
    bonus: i64,
    source: Option<ObjectId>,
) -> Vec<RolledDie> {
    if n == 0 {
        return vec![];
    }
    // "Instead roll that many dice plus one and ignore the lowest roll", once for each
    // such effect.
    let extra = statics(g)
        .iter()
        .filter(|(_, ctl, _, d)| {
            matches!(d, DiceStatic::ExtraDie { who } if applies_to(g, *who, *ctl, p))
        })
        .count() as u32;
    let mut dice: Vec<RolledDie> = (0..n + extra)
        .map(|_| RolledDie {
            sides,
            natural: natural(g, sides),
            adjust: 0,
        })
        .collect();
    let drop = match ignore {
        DiceIgnore::None => extra,
        DiceIgnore::Lowest(k) => extra + k,
        DiceIgnore::AllButHighest => (n + extra).saturating_sub(1),
    };
    // CR 706.6: ignored rolls never happened; ties for the lowest: the player chooses.
    for _ in 0..drop.min(dice.len() as u32 - 1) {
        let low = dice.iter().map(|d| d.natural).min().unwrap_or(0);
        let tied: Vec<usize> = (0..dice.len())
            .filter(|i| dice[*i].natural == low)
            .collect();
        let pick = if tied.len() > 1 {
            let opts = tied
                .iter()
                .map(|i| format!("Ignore die {} (showing {})", i + 1, dice[*i].natural))
                .collect();
            tied[g.ask_option(p, source, "Choose the roll to ignore", opts)]
        } else {
            tied[0]
        };
        dice.remove(pick);
    }
    apply_modifiers(g, p, &mut dice, source);
    for d in &dice {
        g.emit(Event::DieRolled {
            player: p,
            sides,
            result: (d.result() + bonus).max(0) as u32,
            natural: d.natural as u32,
            planar: false,
        });
    }
    g.end_event_batch();
    g.log(|_| {
        format!(
            "{p} rolls {}d{sides}: {:?}",
            dice.len(),
            dice.iter().map(|d| d.result() + bonus).collect::<Vec<_>>()
        )
    });
    dice
}

/// CR 706.2a, 706.2b: optional modifiers. Effects that reroll dice are considered first,
/// then effects that increase or decrease results; when several could apply, the player
/// who rolled chooses which one is considered next. Each modifier's controller decides
/// whether to use it (and pays its cost; mana abilities can be activated while paying).
fn apply_modifiers(
    g: &mut Game,
    roller: PlayerId,
    dice: &mut [RolledDie],
    source: Option<ObjectId>,
) {
    for reroll_phase in [true, false] {
        let mut considered: Vec<(ObjectId, usize)> = Vec::new();
        for _ in 0..64 {
            let turn = g.turn.number;
            let cands: Vec<(ObjectId, PlayerId, usize, DieModifier)> = statics(g)
                .into_iter()
                .filter_map(|(src, ctl, i, d)| match d {
                    DiceStatic::Modifier(m) => Some((src, ctl, i, m)),
                    _ => None,
                })
                .filter(|(_, _, _, m)| (m.kind == ModifierKind::Reroll) == reroll_phase)
                .filter(|(src, _, i, _)| !considered.contains(&(*src, *i)))
                .filter(|(src, ctl, i, m)| {
                    applies_to(g, m.whose, *ctl, roller)
                        && !(m.once_per_turn && g.dice.used.iter().any(|u| *u == (*src, *i, turn)))
                        && dice.iter().any(|d| modifies(m, d))
                        && g.can_pay_cost(*ctl, &m.cost, Some(*src), &Ctx::new(Some(*src), *ctl))
                })
                .collect();
            if cands.is_empty() {
                break;
            }
            let k = if cands.len() == 1 {
                0
            } else {
                let opts = cands.iter().map(|c| c.3.text.clone()).collect();
                g.ask_option(
                    roller,
                    source,
                    "Choose a die roll modifier to consider",
                    opts,
                )
            };
            let (src, ctl, i, m) = cands[k].clone();
            considered.push((src, i));
            let mut paid = false;
            for d in dice.iter_mut() {
                if !modifies(&m, d) {
                    continue;
                }
                let prompt = format!("{}: use it on the die showing {}?", m.text, d.result());
                if !g.ask_yes_no(ctl, Some(src), &prompt, false) {
                    continue;
                }
                if !paid {
                    if !g.pay_cost(ctl, &m.cost, Some(src), &Ctx::new(Some(src), ctl)) {
                        break;
                    }
                    paid = true;
                    if m.once_per_turn {
                        g.dice.used.push((src, i, turn));
                    }
                }
                match m.kind {
                    // CR 706.2b: a rerolled die's first roll never happened.
                    ModifierKind::Reroll => {
                        d.natural = natural(g, d.sides);
                        d.adjust = 0;
                    }
                    ModifierKind::Add(x) => {
                        d.adjust += x;
                        break;
                    }
                }
            }
        }
    }
}

fn modifies(m: &DieModifier, d: &RolledDie) -> bool {
    m.natural.is_none_or(|v| d.natural == v) && m.sides.is_none_or(|s| d.sides == s)
}

/// CR 706.7: the planar die was rolled. Abilities that trigger on rolling dice trigger,
/// but it has no numerical result.
pub fn planar_die_rolled(g: &mut Game, p: PlayerId) {
    g.emit(Event::DieRolled {
        player: p,
        sides: 6,
        result: 0,
        natural: 0,
        planar: true,
    });
    g.end_event_batch();
}

/// Performs a coin flip instruction (CR 705).
pub fn flip(g: &mut Game, spec: &CoinFlip, ctx: &mut Ctx) {
    let players = g.eval_players(&spec.who, ctx);
    for p in players {
        let n = g.eval_value(&spec.count, ctx).max(0) as u32;
        let st = statics(g);
        // CR 705.3: "the first time you flip one or more coins each turn, those coins
        // come up heads and you win those flips."
        let first_this_turn = !g
            .turn_events
            .iter()
            .chain(g.events.iter())
            .any(|e| matches!(e, Event::CoinFlipped { player, .. } if *player == p));
        let fixed = first_this_turn
            && st.iter().any(|(_, ctl, _, d)| {
                matches!(d, DiceStatic::FirstFlipsWin { who } if applies_to(g, *who, *ctl, p))
            });
        let extra = st
            .iter()
            .filter(|(_, ctl, _, d)| {
                matches!(d, DiceStatic::ExtraCoin { who } if applies_to(g, *who, *ctl, p))
            })
            .count();
        let (mut wins, mut losses, mut heads, mut tails) = (0i64, 0i64, 0i64, 0i64);
        let mut i = 0u32;
        loop {
            if spec.until_lose {
                if losses > 0 || i >= 1000 {
                    break;
                }
            } else if i >= n {
                break;
            }
            i += 1;
            // CR 705.2: the player who flips calls heads or tails.
            let call = spec.call.then(|| {
                g.ask_option(
                    p,
                    ctx.source,
                    "Call the coin flip",
                    vec!["Heads".into(), "Tails".into()],
                ) == 0
            });
            // "Flip two coins and ignore one": the ignored flip never happened.
            let flips: Vec<bool> = (0..=extra).map(|_| coin(g)).collect();
            let up = if flips.len() > 1 {
                // By default, keep a flip that wins (or comes up heads).
                let prefer = call.unwrap_or(true);
                let default = flips.iter().position(|h| *h == prefer).unwrap_or(0);
                let options = flips
                    .iter()
                    .map(|h| format!("Keep {}", if *h { "heads" } else { "tails" }))
                    .collect();
                let k = match g.ask(
                    p,
                    Decision::ChooseOption {
                        source: ctx.source,
                        prompt: "Choose the flip to keep".into(),
                        options,
                    },
                ) {
                    Answer::Index(i) if i < flips.len() => i,
                    _ => default,
                };
                flips[k]
            } else {
                flips[0]
            };
            // CR 705.3: ignore the actual result. Only the first of a sequence of flips
            // made one at a time is "the first time".
            let fixed_now = fixed && (!spec.until_lose || i == 1);
            let (up, won, lost) = if fixed_now {
                (true, true, false)
            } else {
                match call {
                    Some(c) => (up, up == c, up != c),
                    None => (up, false, false),
                }
            };
            wins += won as i64;
            losses += lost as i64;
            heads += up as i64;
            tails += !up as i64;
            g.emit(Event::CoinFlipped {
                player: p,
                won,
                lost,
                heads: up,
            });
            g.log(|_| {
                format!(
                    "{p} flips a coin: {}{}",
                    if up { "heads" } else { "tails" },
                    if won {
                        " (wins)"
                    } else if lost {
                        " (loses)"
                    } else {
                        ""
                    }
                )
            });
        }
        g.end_event_batch();
        ctx.nums.insert(WINS, wins);
        ctx.nums.insert(LOSSES, losses);
        ctx.nums.insert(HEADS, heads);
        ctx.nums.insert(TAILS, tails);
        if wins > 0 && losses == 0 {
            g.exec(&spec.on_win, ctx);
        }
        if losses > 0 {
            g.exec(&spec.on_lose, ctx);
        }
        if heads > 0 && tails == 0 {
            g.exec(&spec.on_heads, ctx);
        }
        if tails > 0 && heads == 0 {
            g.exec(&spec.on_tails, ctx);
        }
    }
}

/// `Value::Custom` values of this module.
pub fn custom_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    match name {
        THE_RESULT => Some(
            ctx.nums
                .get(&RESULT)
                .copied()
                .or_else(|| ctx.event.as_ref().map(|e| e.amount as i64))
                .unwrap_or(0),
        ),
        STORED_SAME => {
            let obj = ctx
                .vars
                .get(&vars::AFFECTED)
                .and_then(|v| v.first().copied())
                .and_then(|e| e.object())
                .or(ctx.source)?;
            let stored = g.dice.stored.get(&obj)?;
            let mut counts: BTreeMap<i64, i64> = BTreeMap::new();
            for (_, v) in stored {
                *counts.entry(*v).or_insert(0) += 1;
            }
            Some(counts.values().copied().max().unwrap_or(0))
        }
        _ => None,
    }
}

/// `Condition::Custom` conditions of this module.
pub fn custom_condition(name: &str, ctx: &Ctx) -> Option<bool> {
    match name {
        ROLLED_DOUBLES => Some(ctx.nums.get(&DOUBLES).copied() == Some(1)),
        _ => None,
    }
}

/// `Effect::Custom` effects of this module. Returns true if handled.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    match name {
        ROLL_AGAIN => {
            ctx.nums.insert(AGAIN, 1);
            true
        }
        REROLL_STORED => {
            reroll_stored(g, ctx);
            true
        }
        _ => false,
    }
}

/// CR 706.8b: the controller chooses any number of the source's stored results; one die
/// of the noted kind is rolled for each, and the new results replace them.
fn reroll_stored(g: &mut Game, ctx: &mut Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    let p = ctx.controller;
    let stored = g.dice.stored.get(&src).cloned().unwrap_or_default();
    let mut keep: Vec<(u32, i64)> = Vec::new();
    let mut reroll: BTreeMap<u32, u32> = BTreeMap::new();
    for (i, (sides, v)) in stored.iter().enumerate() {
        let prompt = format!("Reroll stored result {} (d{sides} showing {v})?", i + 1);
        if g.ask_yes_no(p, Some(src), &prompt, false) {
            *reroll.entry(*sides).or_insert(0) += 1;
        } else {
            keep.push((*sides, *v));
        }
    }
    for (sides, n) in reroll {
        for d in roll_dice(g, p, n, sides, DiceIgnore::None, 0, Some(src)) {
            keep.push((sides, d.result()));
        }
    }
    g.dice.stored.insert(src, keep);
    g.dirty = true;
}

/// `TriggerCond::Custom` triggers of this module: "whenever you roll a die's highest
/// natural result" (a numerical result: never the planar die, CR 706.7).
pub fn custom_trigger(g: &Game, name: &str, ctl: PlayerId, ev: &Event) -> Option<Vec<EventInfo>> {
    let rel = name.strip_prefix(NATURAL_MAX)?.strip_prefix(':');
    let Event::DieRolled {
        player,
        sides,
        natural,
        planar: false,
        ..
    } = ev
    else {
        return Some(vec![]);
    };
    let rel = match rel {
        Some("any") => PlayerRel::Any,
        Some("opponent") => PlayerRel::Opponent,
        _ => PlayerRel::You,
    };
    if natural != sides || !applies_to(g, rel, ctl, *player) {
        return Some(vec![]);
    }
    Some(vec![EventInfo {
        player: Some(*player),
        amount: *natural as i32,
        ..Default::default()
    }])
}
