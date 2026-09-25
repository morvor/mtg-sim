//! Oracle patterns for flipping coins (CR 705) and rolling dice (CR 706):
//!
//! - "Flip a coin." / "Flip two coins." / "Flip a coin until you lose a flip.", followed by
//!   "If you win/lose the flip, ..." or "If the coin comes up heads/tails, ..." (a flip
//!   that only cares about heads or tails has no winner, CR 705.2).
//! - "Roll a d20." / "Roll two six-sided dice and ignore the lower roll." / "Roll a d20
//!   and add [value]." / "roll five six-sided dice and store those results on it", with a
//!   results table printed on the following lines ("1—9 | ...", "20 | ...", "15+ | ...")
//!   that is part of the same ability (CR 706.3a, 706.3b), and "(You may) roll again"
//!   (CR 706.3c).
//! - "... equal to the result" (CR 706.4), "if you rolled doubles" (CR 706.5).
//! - Statics: extra dice and ignored rolls (CR 706.6), "flip two coins and ignore one",
//!   fixed flip results (CR 705.3), optional die-roll modifiers (CR 706.2a), and stored
//!   results (CR 706.8).
//! - Triggers: "whenever you roll a die's highest natural result", "whenever you lose a
//!   coin flip".

use super::{
    BlockGroupPattern, ConditionPattern, EffectPattern, FollowupPattern, StaticPattern,
    TriggerPattern,
};
use crate::ability::*;
use crate::dice::*;
use crate::oracle::effects::{parse_clause, parse_sentence, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Results tables
// ---------------------------------------------------------------------------

/// "1—9 | text" → (1, Some(9), text); "20 | text" → (20, Some(20), text); "15+ | text" →
/// (15, None, text).
fn table_row(line: &str) -> Option<(i64, Option<i64>, &str)> {
    let (range, text) = line.trim().split_once(" | ")?;
    let range = range.trim();
    if let Some(n) = range.strip_suffix('+') {
        return Some((n.trim().parse().ok()?, None, text));
    }
    for sep in ['—', '–', '-'] {
        if let Some((a, b)) = range.split_once(sep) {
            return Some((a.trim().parse().ok()?, Some(b.trim().parse().ok()?), text));
        }
    }
    let n: i64 = range.parse().ok()?;
    Some((n, Some(n), text))
}

/// The lines of a results table join the ability that rolls (CR 706.3b), as further
/// sentences of its effect text.
fn group_table_rows(blocks: Vec<String>, _ctx: &CompileContext) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for b in blocks {
        match out.last_mut() {
            Some(last) if table_row(&b).is_some() => {
                last.push(' ');
                last.push_str(b.trim());
            }
            _ => out.push(b),
        }
    }
    out
}

inventory::submit! { BlockGroupPattern { name: "r706 results table", priority: 80, group: group_table_rows } }

/// The last die roll in an effect (the one a table row or "roll again" belongs to).
fn last_roll(e: &mut Effect) -> Option<&mut DieRoll> {
    match e {
        Effect::RollDice(r) => Some(r),
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_roll),
        Effect::May { effect, .. } => last_roll(effect),
        _ => None,
    }
}

/// A results table row after the sentence that rolls, or a further sentence of the last
/// row's effect.
fn f_table_row(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(roll) = last_roll(prev) else {
        return false;
    };
    if let Some((lo, hi, text)) = table_row(l) {
        let Some(e) = parse_row_text(text, b) else {
            return false;
        };
        roll.table.push(ResultRow { lo, hi, effect: e });
        return true;
    }
    // A sentence after a row continues that row's effect.
    let Some(row) = roll.table.last_mut() else {
        return false;
    };
    if crate::oracle_ext::apply_followup_ext(l, &mut row.effect, b) {
        return true;
    }
    let Some(e) = parse_sentence(l, b) else {
        return false;
    };
    let old = std::mem::take(&mut row.effect);
    row.effect = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "r706 results table row", priority: 10, apply: f_table_row } }

/// A row's effect: one sentence (a row ends at its period; further sentences are added
/// by [`f_table_row`]).
fn parse_row_text(text: &str, b: &mut Builder) -> Option<Effect> {
    let t = end(text.trim());
    parse_sentence(t, b)
}

// ---------------------------------------------------------------------------
// Rolling dice
// ---------------------------------------------------------------------------

/// "a d20", "two d4", "a six-sided die", "five six-sided dice", "x six-sided dice".
fn dice_phrase(s: &str) -> Option<(Value, u32, &str)> {
    let (n, r) = parse_number(s)?;
    let r = r.trim_start();
    let (w, rest) = split_word(r);
    if let Some(d) = w.strip_prefix('d') {
        if let Ok(sides) = d.trim_end_matches([',', '.']).parse::<u32>() {
            return Some((n, sides, rest));
        }
    }
    let (sides, r) = sided(r)?;
    let r = strip(r, "dice").or_else(|| strip(r, "die"))?;
    Some((n, sides, r))
}

/// "six-sided " → 6.
fn sided(r: &str) -> Option<(u32, &str)> {
    let (w, rest) = split_word(r);
    let n = w.strip_suffix("-sided")?;
    let (v, _) = parse_number(n)?;
    let Value::Const(v) = v else {
        return None;
    };
    Some((v as u32, rest))
}

fn p_roll(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("roll ") {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix("you roll ") {
        (PlayerRef::You, r)
    } else {
        return None;
    };
    let (count, sides, rest) = dice_phrase(r)?;
    let mut spec = DieRoll::new(sides);
    spec.who = who;
    spec.count = count;
    let mut r = rest.trim().to_string();
    while !r.is_empty() {
        let next = if let Some(x) = r.strip_prefix("and ignore the lower roll") {
            spec.ignore = DiceIgnore::Lowest(1);
            x.to_string()
        } else if let Some(x) = r.strip_prefix("and ignore the lowest roll") {
            spec.ignore = DiceIgnore::Lowest(1);
            x.to_string()
        } else if let Some(x) = r.strip_prefix("and ignore all but the highest roll") {
            spec.ignore = DiceIgnore::AllButHighest;
            x.to_string()
        } else if let Some(x) = r.strip_prefix("and store those results on it") {
            spec.store_on = Some(Sel::This);
            x.to_string()
        } else if let Some(x) = r.strip_prefix("and store those results on ~") {
            spec.store_on = Some(Sel::This);
            x.to_string()
        } else if let Some(x) = r.strip_prefix("and add ") {
            let (v, rest) = crate::oracle::statics::parse_value_phrase(x, b)?;
            spec.bonus = Some(v);
            rest
        } else {
            return None;
        };
        r = next.trim().to_string();
    }
    Some(Effect::RollDice(Box::new(spec)))
}

inventory::submit! { EffectPattern { name: "r706 roll dice", priority: 0, parse: p_roll } }

fn p_roll_again(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "roll again" => Some(Effect::Custom(SmolStr::new(ROLL_AGAIN))),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r706 roll again", priority: 0, parse: p_roll_again } }

fn p_reroll_stored(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "reroll any number of ~'s stored results" | "reroll any number of its stored results" => {
            Some(Effect::Custom(SmolStr::new(REROLL_STORED)))
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r706 reroll stored results", priority: 0, parse: p_reroll_stored } }

/// "... equal to the result" (CR 706.4): the amount is the roll's result.
fn p_equal_to_result(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let rewritten = if let Some(x) = l.strip_suffix(" equal to the result") {
        if x.contains(" a number of ") {
            x.replacen(" a number of ", " x ", 1)
        } else {
            // "gain life", "draw cards", "deals damage": "gain x life".
            let mut done = None;
            for noun in ["life", "cards", "damage"] {
                if let Some(head) = x.strip_suffix(&format!(" {noun}")) {
                    done = Some(format!("{head} x {noun}"));
                    break;
                }
            }
            done?
        }
    } else if let Some(x) = l.strip_suffix(", where x is the result") {
        x.to_string()
    } else {
        return None;
    };
    let e = parse_clause(&rewritten, b)?;
    Some(Effect::seq(vec![
        Effect::SetX {
            value: Value::Custom(SmolStr::new(THE_RESULT)),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "r706 equal to the result", priority: 0, parse: p_equal_to_result } }

fn c_doubles(c: &str) -> Option<Condition> {
    if c == "you rolled doubles" {
        return Some(Condition::Custom(SmolStr::new(ROLLED_DOUBLES)));
    }
    // "the result is 1", "the result is 15 or greater".
    let r = c.strip_prefix("the result is ")?;
    let (n, cmp) = if let Some(n) = r
        .strip_suffix(" or greater")
        .or_else(|| r.strip_suffix(" or higher"))
    {
        (n, Cmp::Ge)
    } else if let Some(n) = r
        .strip_suffix(" or less")
        .or_else(|| r.strip_suffix(" or lower"))
    {
        (n, Cmp::Le)
    } else {
        (r, Cmp::Eq)
    };
    let n: i32 = n.trim().parse().ok()?;
    Some(Condition::Compare(
        Value::Custom(SmolStr::new(THE_RESULT)),
        cmp,
        Value::c(n),
    ))
}

inventory::submit! { ConditionPattern { name: "r706 rolled doubles", priority: 0, parse: c_doubles } }

// ---------------------------------------------------------------------------
// Flipping coins
// ---------------------------------------------------------------------------

fn p_flip(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("flip ")
        .or_else(|| l.strip_prefix("you flip "))?;
    let mut spec = CoinFlip::new();
    if r == "a coin until you lose a flip" {
        spec.until_lose = true;
        return Some(Effect::FlipCoins(Box::new(spec)));
    }
    let (n, rest) = parse_number(r)?;
    if !matches!(rest.trim(), "coin" | "coins") {
        return None;
    }
    spec.count = n;
    Some(Effect::FlipCoins(Box::new(spec)))
}

inventory::submit! { EffectPattern { name: "r705 flip coins", priority: 0, parse: p_flip } }

/// "[effect] for each flip you won", "... for each coin that comes up heads".
fn p_for_each_flip(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, var) = [
        (" for each flip you won", WINS),
        (" for each flip you win", WINS),
        (" for each flip you lost", LOSSES),
        (" for each flip you lose", LOSSES),
        (" for each coin that comes up heads", HEADS),
        (" for each coin that comes up tails", TAILS),
    ]
    .into_iter()
    .find_map(|(suffix, var)| l.strip_suffix(suffix).map(|h| (h, var)))?;
    let e = parse_clause(head, b)?;
    Some(Effect::Repeat {
        times: Value::Var(var),
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "r705 for each flip", priority: 0, parse: p_for_each_flip } }

fn last_flip(e: &mut Effect) -> Option<&mut CoinFlip> {
    match e {
        Effect::FlipCoins(f) => Some(f),
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_flip),
        _ => None,
    }
}

/// "If you win the flip, X." / "If you lose the flip, X." / "If the coin comes up heads,
/// X." / "If it comes up tails, X." after a coin flip.
fn f_flip_result(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((cond, rest)) = r.split_once(", ") else {
        return false;
    };
    #[derive(PartialEq)]
    enum Which {
        Win,
        Lose,
        Heads,
        Tails,
    }
    let which = match cond {
        "you win the flip" => Which::Win,
        "you lose the flip" => Which::Lose,
        "the coin comes up heads" | "it comes up heads" => Which::Heads,
        "the coin comes up tails" | "it comes up tails" => Which::Tails,
        _ => return false,
    };
    if last_flip(prev).is_none() {
        return false;
    }
    let Some(e) = parse_clause(rest, b) else {
        return false;
    };
    let Some(f) = last_flip(prev) else {
        return false;
    };
    match which {
        Which::Win => f.on_win = e,
        Which::Lose => f.on_lose = e,
        Which::Heads => f.on_heads = e,
        Which::Tails => f.on_tails = e,
    }
    // CR 705.2: an effect that cares only about heads or tails has no call.
    let wl = !matches!(f.on_win, Effect::Noop) || !matches!(f.on_lose, Effect::Noop);
    let ht = !matches!(f.on_heads, Effect::Noop) || !matches!(f.on_tails, Effect::Noop);
    f.call = wl || !ht;
    true
}

inventory::submit! { FollowupPattern { name: "r705 coin flip results", priority: 10, apply: f_flip_result } }

// ---------------------------------------------------------------------------
// Statics
// ---------------------------------------------------------------------------

fn dice_static(d: DiceStatic, text: &str) -> Vec<Ability> {
    vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Dice(d))),
        text,
    )]
}

fn s_dice(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let d = match l {
        "if you would roll one or more dice, instead roll that many dice plus one and ignore the lowest roll" => {
            DiceStatic::ExtraDie { who: PlayerRel::You }
        }
        "if you would flip a coin, instead flip two coins and ignore one" => {
            DiceStatic::ExtraCoin { who: PlayerRel::You }
        }
        "the first time you flip one or more coins each turn, those coins come up heads and you win those flips" => {
            DiceStatic::FirstFlipsWin { who: PlayerRel::You }
        }
        "you may tap ~ to increase the result of a die any player rolled by 1" => {
            DiceStatic::Modifier(DieModifier {
                whose: PlayerRel::Any,
                kind: ModifierKind::Add(1),
                cost: Cost {
                    mana: None,
                    parts: vec![CostPart::Tap],
                },
                once_per_turn: false,
                natural: None,
                sides: None,
                text: text.to_string(),
            })
        }
        "once each turn, you may pay {1} to reroll one or more dice you rolled" => {
            DiceStatic::Modifier(DieModifier {
                whose: PlayerRel::You,
                kind: ModifierKind::Reroll,
                cost: Cost::mana(crate::mana::ManaCost::parse("{1}")?),
                once_per_turn: true,
                natural: None,
                sides: None,
                text: text.to_string(),
            })
        }
        _ => {
            // "If you roll a 3 on a six-sided die, you may reroll that die."
            let r = l.strip_prefix("if you roll a ")?;
            let (n, r) = r.split_once(" on a ")?;
            let (sides, r) = sided(r)?;
            if r.trim() != "die, you may reroll that die" {
                return None;
            }
            DiceStatic::Modifier(DieModifier {
                whose: PlayerRel::You,
                kind: ModifierKind::Reroll,
                cost: Cost::free(),
                once_per_turn: false,
                natural: Some(n.parse().ok()?),
                sides: Some(sides),
                text: text.to_string(),
            })
        }
    };
    Some(dice_static(d, text))
}

inventory::submit! { StaticPattern { name: "r705-706 dice statics", priority: 0, parse: s_dice } }

/// "~ gets +X/+X, where X is the greatest number of stored results on it of the same
/// value." (CR 706.8a)
fn s_stored_results(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l)
        != "~ gets +x/+x, where x is the greatest number of stored results on it of the same value"
    {
        return None;
    }
    let v = Value::Custom(SmolStr::new(STORED_SAME));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::ModifyPT(v.clone(), v)],
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r706 stored results", priority: 0, parse: s_stored_results } }

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

fn t_dice_coins(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (rel, rest) = if let Some(x) = r.strip_prefix("you ") {
        (PlayerRel::You, x)
    } else if let Some(x) = r.strip_prefix("a player ") {
        (PlayerRel::Any, x)
    } else if let Some(x) = r.strip_prefix("an opponent ") {
        (PlayerRel::Opponent, x)
    } else {
        return None;
    };
    let cond = match rest {
        "roll a die's highest natural result" | "rolls a die's highest natural result" => {
            let who = match rel {
                PlayerRel::Any => "any",
                PlayerRel::Opponent => "opponent",
                _ => "you",
            };
            TriggerCond::Custom(SmolStr::new(format!("{NATURAL_MAX}:{who}")))
        }
        "lose a coin flip" | "loses a coin flip" => TriggerCond::Where {
            trigger: Box::new(TriggerCond::FlipCoin(rel)),
            cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(0)),
        },
        _ => return None,
    };
    Some((cond, Sel::None, PlayerRef::TriggerPlayer))
}

inventory::submit! { TriggerPattern { name: "r705-706 dice and coin triggers", priority: 0, parse: t_dice_coins } }
