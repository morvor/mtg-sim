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
/// "The other result" of "roll two d4 and choose one result": the result not chosen.
pub const OTHER: Var = vars::USER + 8500;
/// The number of even and odd results of the most recent roll ("for each even result").
pub const EVENS: Var = vars::USER + 8501;
pub const ODDS: Var = vars::USER + 8502;
/// `RESULT_COUNT + n`: how many results of the most recent roll were `n` (1..=20, "if you
/// rolled 6 exactly seven times").
pub const RESULT_COUNT: Var = vars::USER + 8530;
/// The number of different results of the most recent roll ("for each different result").
pub const DIFFERENT: Var = vars::USER + 8503;
/// The greatest result of the most recent roll ("if any of those results was 10 or
/// higher").
pub const HIGHEST: Var = vars::USER + 8504;
/// After a flip by several players at once: how many flipped, and how many of them had
/// every coin come up heads ("until both players' coins come up heads on the same flip").
pub const PLAYERS_FLIPPED: Var = vars::USER + 8505;
pub const PLAYERS_ALL_HEADS: Var = vars::USER + 8506;

/// `Condition::Custom` prefix: "any of those results was N or higher"
/// (`die:any result at least:N`): of the resolving ability's own roll, or else of the dice
/// rolled together that triggered it.
pub const ANY_RESULT_AT_LEAST: &str = "die:any result at least";

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

/// `TriggerCond::Custom` prefix: "when you roll a natural 20" (`die:natural:20:you`).
pub const NATURAL_N: &str = "die:natural";
/// `TriggerCond::Custom` prefix: "whenever you roll your third die each turn"
/// (`die:nth die:3:you`).
pub const NTH_DIE: &str = "die:nth die";
/// `Condition::Custom` prefix: "you've rolled three or more dice this turn"
/// (`die:rolled dice this turn:3`).
pub const DICE_THIS_TURN: &str = "die:rolled dice this turn";
/// `Condition::Custom` prefix: "you've rolled a 20 this turn" (`die:rolled result this
/// turn:20`).
pub const RESULT_THIS_TURN: &str = "die:rolled result this turn";

/// `PLAYER_RESULT + p`: the result of player `p`'s roll in the resolving ability's most
/// recent roll instruction; `PLAYER_ROLLED + p` is 1 if `p` rolled in it.
pub const PLAYER_RESULT: Var = vars::USER + 8600;
pub const PLAYER_ROLLED: Var = vars::USER + 8640;
/// `Condition::Custom`, about the iterated player: "had the highest result" — no player who
/// rolled in the resolving ability's roll instruction had a higher result (ties included:
/// "if one or more opponents had the highest result").
pub const HAD_HIGHEST: &str = "die:had the highest result";

/// `PlayerModification::Custom` (until end of turn): "The next time [this player] would
/// roll one or more dice this turn, instead they roll that many dice plus one and you
/// choose one of those rolls to ignore." (Bamboozling Beeble). The effect's controller
/// chooses; it's used up by the next roll instruction (or planar die roll) of the player.
pub const NEXT_ROLL_EXTRA: &str = "die:next roll one more, controller ignores one";

/// The pending [`NEXT_ROLL_EXTRA`] effects on `p`, used up now: their controllers, in
/// timestamp order.
fn take_next_roll_extras(g: &mut Game, p: PlayerId) -> Vec<PlayerId> {
    let is = |e: &crate::game::PlayerEffect| {
        e.players.contains(&p)
            && matches!(&e.effect, PlayerModification::Custom(n) if n == NEXT_ROLL_EXTRA)
    };
    let mut used: Vec<(crate::types::Timestamp, PlayerId)> = g
        .player_effects
        .iter()
        .filter(|e| is(e))
        .map(|e| (e.timestamp, e.controller))
        .collect();
    if used.is_empty() {
        return vec![];
    }
    used.sort_by_key(|(t, _)| *t);
    g.player_effects.retain(|e| !is(e));
    g.dirty = true;
    used.into_iter().map(|(_, c)| c).collect()
}

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
    /// (source, permanent): permanents an effect of the source phased out until an event
    /// (`until::exec_phase_out_until`), for "target creature that hasn't been phased out
    /// with this Attraction" (Ferris Wheel; a new object has no such history, CR 400.7).
    #[serde(default)]
    pub phased_out_by: Vec<(ObjectId, ObjectId)>,
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
    /// "roll two d4 and choose one result": the roller chooses which result is "that
    /// result"; the other is "the other result".
    #[serde(default)]
    pub choose_one: bool,
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
            choose_one: false,
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
    /// With `until_lose`: "flip a coin X times or until you lose a flip, whichever comes
    /// first".
    #[serde(default)]
    pub max_flips: Option<Value>,
    /// With `until_lose`: "... or choose to stop flipping" (Fiery Gambit): after each flip
    /// the flipper won, they choose whether to flip again.
    #[serde(default)]
    pub may_stop: bool,
    /// "If you lose a flip, ~ has no effect." (Fiery Gambit): the instructions after the
    /// flip are all about flips won (`dice_coins_5`), which count only if no flip was
    /// lost; read by the renderer.
    #[serde(default)]
    pub loss_voids: bool,
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
            max_flips: None,
            may_stop: false,
            loss_voids: false,
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
    /// "If you would roll one or more planar dice, instead roll that many planar dice plus
    /// one and ignore one." (Ichor Elixir): the roller chooses the roll to ignore (CR 706.6).
    ExtraPlanarDie { who: PlayerRel },
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
    /// "increase or decrease the result by N": the modifier's controller chooses.
    AddOrSubtract(i64),
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
            ctx.nums
                .insert(EVENS, results.iter().filter(|r| *r % 2 == 0).count() as i64);
            ctx.nums
                .insert(ODDS, results.iter().filter(|r| *r % 2 != 0).count() as i64);
            for n in 1..=20i64 {
                let k = results.iter().filter(|r| **r == n).count() as i64;
                ctx.nums.insert(RESULT_COUNT + n as Var, k);
            }
            let mut different = results.clone();
            different.sort_unstable();
            different.dedup();
            ctx.nums.insert(DIFFERENT, different.len() as i64);
            if let Some(m) = results.iter().max() {
                ctx.nums.insert(HIGHEST, *m);
            }
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
            // "Each player rolls a d20. If one or more opponents had the highest result,
            // ...": each player's result.
            ctx.nums.insert(PLAYER_RESULT + p.0 as Var, total);
            ctx.nums.insert(PLAYER_ROLLED + p.0 as Var, 1);
            if spec.choose_one && results.len() >= 2 {
                let opts = results.iter().map(|r| format!("Choose {r}")).collect();
                let k = g
                    .ask_option(p, ctx.source, "Choose one result", opts)
                    .min(results.len() - 1);
                ctx.nums.insert(RESULT, results[k]);
                ctx.nums.insert(OTHER, total - results[k]);
            }
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
    // "The next time target player would roll one or more dice this turn, instead they
    // roll that many dice plus one and you choose one of those rolls to ignore."
    let choosers = take_next_roll_extras(g, p);
    let mut dice: Vec<RolledDie> = (0..n + extra + choosers.len() as u32)
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
    // CR 706.6: then each of those players chooses a roll to ignore, after seeing all the
    // results (Bamboozling Beeble's ruling).
    for c in choosers {
        if dice.len() <= 1 {
            break;
        }
        let opts = dice
            .iter()
            .enumerate()
            .map(|(i, d)| format!("Ignore die {} (showing {})", i + 1, d.natural))
            .collect();
        let k = g.ask_option(c, source, "Choose a roll to ignore", opts);
        dice.remove(k.min(dice.len() - 1));
    }
    apply_modifiers(g, p, &mut dice, source, false);
    for d in &dice {
        g.emit(Event::DieRolled {
            player: p,
            sides,
            result: (d.result() + bonus).max(0) as u32,
            natural: d.natural as u32,
            planar: false,
        });
    }
    // "... or until any player rolls a 1, whichever comes first" (Drop Tower's rulings:
    // the effect ends as soon as the result is known).
    let results: Vec<i64> = dice.iter().map(|d| d.result() + bonus).collect();
    g.expire_effects(|d| {
        matches!(d, Duration::UntilEndOfTurnOrRoll(n) if results.contains(&(*n as i64)))
    });
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
/// For the planar die (`planar`), only modifiers that don't refer to a number of sides or
/// a natural result are considered, and those that increase or decrease the result can be
/// used (and their costs paid, using up a "once each turn" modifier) but change nothing:
/// the planar die has no numerical result (CR 706.7).
fn apply_modifiers(
    g: &mut Game,
    roller: PlayerId,
    dice: &mut [RolledDie],
    source: Option<ObjectId>,
    planar: bool,
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
                .filter(|(_, _, _, m)| !planar || (m.natural.is_none() && m.sides.is_none()))
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
                let prompt = if planar {
                    let face = crate::planechase::face_for(d.natural as u32);
                    format!("{}: use it on the planar die showing {face:?}?", m.text)
                } else {
                    format!("{}: use it on the die showing {}?", m.text, d.result())
                };
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
                    // No numerical result to change (CR 706.7).
                    ModifierKind::Add(_) | ModifierKind::AddOrSubtract(_) if planar => break,
                    // CR 706.2b: a rerolled die's first roll never happened.
                    ModifierKind::Reroll => {
                        d.natural = natural(g, d.sides);
                        d.adjust = 0;
                    }
                    ModifierKind::Add(x) => {
                        d.adjust += x;
                        break;
                    }
                    ModifierKind::AddOrSubtract(x) => {
                        let opts = vec![
                            format!("Increase the result by {x}"),
                            format!("Decrease the result by {x}"),
                        ];
                        let k = g.ask_option(ctl, Some(src), "Change the result", opts);
                        d.adjust += if k == 1 { -x } else { x };
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

/// Rolls the planar die for `p` (CR 901.9): its natural result 1–6 (see
/// [`crate::planechase::face_for`]) after any rerolls `p` and others choose to apply;
/// modifiers that increase or decrease the result may be used but change nothing
/// (Monitor Monitor's reroll applies to any die roll, CR 706.2). Modifiers that refer to
/// a numerical result or a number of sides ignore it (CR 706.7).
pub fn roll_planar_natural(g: &mut Game, p: PlayerId) -> u32 {
    // "Instead roll that many planar dice plus one and ignore one", once for each such
    // effect: the player keeps one of the rolls (the others never happened, CR 706.6).
    let extra = statics(g)
        .iter()
        .filter(|(_, ctl, _, d)| {
            matches!(d, DiceStatic::ExtraPlanarDie { who } if applies_to(g, *who, *ctl, p))
        })
        .count();
    let choosers = take_next_roll_extras(g, p);
    let mut rolls: Vec<i64> = (0..=extra + choosers.len()).map(|_| natural(g, 6)).collect();
    // "... and you choose one of those rolls to ignore" (CR 706.6).
    for c in choosers {
        let opts = rolls
            .iter()
            .map(|r| format!("Ignore {:?}", crate::planechase::face_for(*r as u32)))
            .collect();
        let k = g.ask_option(c, None, "Choose a planar die roll to ignore", opts);
        rolls.remove(k.min(rolls.len() - 1));
    }
    let kept = if rolls.len() > 1 {
        let opts = rolls
            .iter()
            .map(|r| format!("Keep {:?}", crate::planechase::face_for(*r as u32)))
            .collect();
        let k = g.ask_option(p, None, "Choose the planar die roll to keep", opts);
        rolls[k.min(rolls.len() - 1)]
    } else {
        rolls[0]
    };
    let mut dice = [RolledDie {
        sides: 6,
        natural: kept,
        adjust: 0,
    }];
    apply_modifiers(g, p, &mut dice, None, true);
    dice[0].natural as u32
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
/// "Flip two coins and ignore one": the number of extra coins `p` flips for each flip.
fn extra_coins(g: &Game, p: PlayerId) -> usize {
    statics(g)
        .iter()
        .filter(|(_, ctl, _, d)| {
            matches!(d, DiceStatic::ExtraCoin { who } if applies_to(g, *who, *ctl, p))
        })
        .count()
}

/// CR 705.2: the player who flips calls heads or tails, then flips the coin — or, with
/// "flip two coins and ignore one", the coins replacing it.
fn call_and_flip(
    g: &mut Game,
    spec: &CoinFlip,
    p: PlayerId,
    source: Option<ObjectId>,
) -> (Option<bool>, Vec<bool>) {
    let call = spec.call.then(|| {
        g.ask_option(
            p,
            source,
            "Call the coin flip",
            vec!["Heads".into(), "Tails".into()],
        ) == 0
    });
    let extra = extra_coins(g, p);
    let flips: Vec<bool> = (0..=extra).map(|_| coin(g)).collect();
    (call, flips)
}

pub fn flip(g: &mut Game, spec: &CoinFlip, ctx: &mut Ctx) {
    let players = g.eval_players(&spec.who, ctx);
    ctx.nums.insert(PLAYERS_FLIPPED, players.len() as i64);
    ctx.nums.insert(PLAYERS_ALL_HEADS, 0);
    // Coins flipped at once — by one player or by several ("each player flips a coin") —
    // are all called and flipped before any flip is ignored, so the results of all of
    // them are known then.
    let mut at_once: Vec<std::collections::VecDeque<(Option<bool>, Vec<bool>)>> = players
        .iter()
        .map(|p| {
            if spec.until_lose {
                Default::default()
            } else {
                let n = g.eval_value(&spec.count, ctx).max(0) as u32;
                (0..n)
                    .map(|_| call_and_flip(g, spec, *p, ctx.source))
                    .collect()
            }
        })
        .collect();
    for (pi, p) in players.into_iter().enumerate() {
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
        let (mut wins, mut losses, mut heads, mut tails) = (0i64, 0i64, 0i64, 0i64);
        let at_once = &mut at_once[pi];
        let mut i = 0u32;
        let max = spec
            .max_flips
            .as_ref()
            .map_or(1000, |v| g.eval_value(v, ctx).clamp(0, 1000) as u32);
        loop {
            if spec.until_lose {
                if losses > 0 || i >= max {
                    break;
                }
                // "... or choose to stop flipping": after each flip (Fiery Gambit's
                // rulings).
                if spec.may_stop
                    && i > 0
                    && !g.ask_yes_no(
                        p,
                        ctx.source,
                        &format!("You've won {wins} flips. Flip again?"),
                        false,
                    )
                {
                    break;
                }
            } else if i >= n {
                break;
            }
            i += 1;
            let (call, flips) = match at_once.pop_front() {
                Some(x) => x,
                None => call_and_flip(g, spec, p, ctx.source),
            };
            // "Flip two coins and ignore one": the ignored flip never happened.
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
        if heads > 0 && tails == 0 {
            *ctx.nums.entry(PLAYERS_ALL_HEADS).or_insert(0) += 1;
        }
        // "Each player flips a coin. Each player whose coin comes up tails ...", "that
        // player flips a coin. If they lose the flip, ...": the results are the flipper's.
        let saved = ctx.iter_player;
        if !matches!(spec.who, PlayerRef::You) {
            ctx.iter_player = Some(p);
        }
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
        ctx.iter_player = saved;
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
        HAD_HIGHEST => {
            let results: Vec<(Var, i64)> = (0..PLAYER_ROLLED - PLAYER_RESULT)
                .filter(|k| ctx.nums.get(&(PLAYER_ROLLED + k)) == Some(&1))
                .map(|k| (k, ctx.nums.get(&(PLAYER_RESULT + k)).copied().unwrap_or(0)))
                .collect();
            let Some(p) = ctx.iter_player else {
                return Some(false);
            };
            let max = results.iter().map(|(_, r)| *r).max();
            Some(
                results
                    .iter()
                    .any(|(k, r)| *k == p.0 as Var && Some(*r) == max),
            )
        }
        _ => {
            let n: i64 = name
                .strip_prefix(ANY_RESULT_AT_LEAST)?
                .strip_prefix(':')?
                .parse()
                .ok()?;
            let highest = ctx.nums.get(&HIGHEST).copied().or_else(|| {
                let e = ctx.event.as_ref()?;
                // A planar die roll has no numerical result (CR 706.7).
                Some(e.amounts.iter().copied().max().unwrap_or(e.amount) as i64)
            });
            Some(highest.is_some_and(|h| h >= n))
        }
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
    if let Some(r) = name.strip_prefix(NATURAL_N) {
        return Some(natural_n_trigger(g, r, ctl, ev));
    }
    if let Some(r) = name.strip_prefix(NTH_DIE) {
        return Some(nth_die_trigger(g, r, ctl, ev));
    }
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

/// ":20:you" → (20, You).
fn number_and_rel(r: &str) -> Option<(i64, PlayerRel)> {
    let mut it = r.strip_prefix(':')?.split(':');
    let n: i64 = it.next()?.parse().ok()?;
    let rel = match it.next() {
        Some("any") => PlayerRel::Any,
        Some("opponent") => PlayerRel::Opponent,
        _ => PlayerRel::You,
    };
    Some((n, rel))
}

/// The custom-name suffix for a player relation (see [`number_and_rel`]).
pub fn rel_word(rel: PlayerRel) -> &'static str {
    match rel {
        PlayerRel::Any => "any",
        PlayerRel::Opponent => "opponent",
        _ => "you",
    }
}

/// "When you roll a natural 20" (CR 706.2): the die's natural result, before modifiers.
fn natural_n_trigger(g: &Game, r: &str, ctl: PlayerId, ev: &Event) -> Vec<EventInfo> {
    let Some((n, rel)) = number_and_rel(r) else {
        return vec![];
    };
    match ev {
        Event::DieRolled {
            player,
            natural,
            planar: false,
            result,
            ..
        } if *natural as i64 == n && applies_to(g, rel, ctl, *player) => vec![EventInfo {
            player: Some(*player),
            amount: *result as i32,
            ..Default::default()
        }],
        _ => vec![],
    }
}

/// The dice `p` rolled this turn, through the event being handled (the last recorded).
fn dice_rolled_this_turn(g: &Game, p: PlayerId) -> usize {
    g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::DieRolled { player, .. } if *player == p))
        .count()
}

/// "Whenever you roll your third die each turn": the roll that is that player's Nth die
/// roll this turn (the planar die is a die too, CR 706.7).
fn nth_die_trigger(g: &Game, r: &str, ctl: PlayerId, ev: &Event) -> Vec<EventInfo> {
    let Some((n, rel)) = number_and_rel(r) else {
        return vec![];
    };
    match ev {
        Event::DieRolled { player, result, .. }
            if applies_to(g, rel, ctl, *player)
                && dice_rolled_this_turn(g, *player) as i64 == n =>
        {
            vec![EventInfo {
                player: Some(*player),
                amount: *result as i32,
                ..Default::default()
            }]
        }
        _ => vec![],
    }
}

/// Conditions about this turn's dice: "you've rolled three or more dice this turn",
/// "you've rolled a 20 this turn" (a numerical result: not the planar die, CR 706.7).
fn history_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    let p = ctx.controller;
    if let Some(n) = name.strip_prefix(DICE_THIS_TURN) {
        let n: usize = n.strip_prefix(':')?.parse().ok()?;
        return Some(dice_rolled_this_turn(g, p) >= n);
    }
    if let Some(n) = name.strip_prefix(RESULT_THIS_TURN) {
        let n: u32 = n.strip_prefix(':')?.parse().ok()?;
        return Some(g.turn_events.iter().any(|e| {
            matches!(e, Event::DieRolled { player, result, planar: false, .. }
                if *player == p && *result == n)
        }));
    }
    None
}

/// The dice-history conditions, through the registry-wide hooks.
struct DiceHistory;

impl crate::kw::KeywordRules for DiceHistory {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        history_condition(g, name, ctx)
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != crate::oracle::patterns::dice_coins_5::NOT_PHASED_OUT_WITH_THIS {
            return None;
        }
        let src = ctx.source.map(|s| g.ability_source_of(s));
        Some(!g
            .dice
            .phased_out_by
            .iter()
            .any(|(s, o)| *o == id && Some(*s) == src))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&DiceHistory) }

/// After a cost's action ("{6}, Roll a d8: Monstrosity X, where X is the result."): a die
/// rolled to pay the cost keeps its result for the ability's effect (CR 118.1, 602.2b,
/// 706.2).
pub fn note_cost_roll(e: &Effect, ctx: &Ctx, paid: &mut crate::casting::PaidCost) {
    if !matches!(e, Effect::RollDice(_)) {
        return;
    }
    for k in [RESULT, NATURAL, HIGHEST] {
        if let Some(v) = ctx.nums.get(&k) {
            paid.nums.push((k, *v));
        }
    }
}
