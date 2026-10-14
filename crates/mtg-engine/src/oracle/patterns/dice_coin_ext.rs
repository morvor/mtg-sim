//! More grammar for flipping coins (CR 705) and rolling dice (CR 706), on top of
//! `r705_706_dice.rs`:
//!
//! - Values: "the result", "that result" / "the other result" (after "roll two d4 and
//!   choose one result"), "the total of those results", "the number of coins that came
//!   up heads".
//! - Instructions: "[player] flips a coin" ("each player flips a coin", "that player
//!   flips a coin"), "for each flip you win, [effect]", "for each even/odd result,
//!   [effect]" / "[effect] for each even result".
//! - Follow-ups: "Each player whose coin comes up tails [does something]".
//! - Conditions: "you've rolled three or more dice this turn", "you've rolled a 20 this
//!   turn", "you rolled 6 or higher" / "the roll was 6 or higher" (the roll that
//!   triggered the ability), "you won five flips this way".
//! - Triggers: "when you roll a natural 20", "whenever you roll a 1 or 2", "whenever you
//!   roll your third die each turn".

use super::{ConditionPattern, EffectPattern, FollowupPattern, TriggerPattern};
use crate::ability::*;
use crate::dice::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"'])
}

fn the_result() -> Value {
    Value::Custom(SmolStr::new(THE_RESULT))
}

/// A value phrase about a die roll or coin flip at the start of `s` (hooked into the value
/// grammar by `value_results::atom_ext`). "The result" is the result of the ability's own
/// roll, or of the roll that triggered it (CR 706.4).
pub fn value(s: &str) -> Option<(Value, String)> {
    for (p, v) in [
        ("the total of those results", the_result()),
        ("the other result", Value::Var(OTHER)),
        ("that result", the_result()),
        ("the result", the_result()),
        ("the number of coins that came up heads", Value::Var(HEADS)),
        ("the number of coins that came up tails", Value::Var(TAILS)),
        ("the number of flips you won", Value::Var(WINS)),
        ("the number of flips you lost", Value::Var(LOSSES)),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) {
                return Some((v, rest.to_string()));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

/// "Each player flips a coin.", "that player flips a coin", "target opponent flips two
/// coins": the flipper calls the flip (CR 705.2).
fn p_player_flips(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.starts_with("you ") || l.starts_with("flip ") {
        return None;
    }
    let (subject, _) = l.split_once(" flips ")?;
    let saved = (b.targets.len(), b.it_player.clone());
    let (who, rest) = player_ref(subject, b)?;
    let undo = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it_player = saved.1.clone();
    };
    if !rest.trim().is_empty() {
        undo(b);
        return None;
    }
    let r = &l[subject.len() + " flips ".len()..];
    let Some((n, rest)) = parse_number(r) else {
        undo(b);
        return None;
    };
    if !matches!(rest.trim(), "coin" | "coins") {
        undo(b);
        return None;
    }
    let mut spec = CoinFlip::new();
    spec.who = who;
    spec.count = n;
    Some(Effect::FlipCoins(Box::new(spec)))
}

inventory::submit! { EffectPattern { name: "a705 player flips coins", priority: 0, parse: p_player_flips } }

/// "For each flip you win, draw a card." / "For each even result, put two -1/-1 counters
/// on that creature." / "Create a tapped Treasure token for each even result."
fn p_for_each_result(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    const COUNTS: [(&str, Var); 6] = [
        ("flip you win", WINS),
        ("flip you won", WINS),
        ("flip you lose", LOSSES),
        ("flip you lost", LOSSES),
        ("even result", EVENS),
        ("odd result", ODDS),
    ];
    let (var, head) = COUNTS.iter().find_map(|(c, v)| {
        if let Some(r) = l.strip_prefix(&format!("for each {c}, ") as &str) {
            return Some((*v, r.to_string()));
        }
        l.strip_suffix(&format!(" for each {c}") as &str)
            .filter(|_| *v == EVENS || *v == ODDS)
            .map(|h| (*v, h.to_string()))
    })?;
    let e = parse_clause(&head, b)?;
    Some(Effect::Repeat {
        times: Value::Var(var),
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "a705-706 for each flip or result", priority: 0, parse: p_for_each_result } }

/// The coin flip the effect ends with.
fn last_flip(e: &mut Effect) -> Option<&mut CoinFlip> {
    match e {
        Effect::FlipCoins(f) => Some(f),
        Effect::Seq(v) => v.last_mut().and_then(last_flip),
        _ => None,
    }
}

/// "Each player flips a coin. Each player whose coin comes up tails sacrifices a creature
/// of their choice.": what happens to each flipper whose coin came up tails (heads).
fn f_whose_coin(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(f) = last_flip(prev) else {
        return false;
    };
    if !matches!(f.who, PlayerRef::EachPlayer) || !matches!(f.count, Value::Const(1)) {
        return false;
    }
    let (heads, rest) = if let Some(r) = l.strip_prefix("each player whose coin comes up tails ")
    {
        (false, r)
    } else if let Some(r) = l.strip_prefix("each player whose coin comes up heads ") {
        (true, r)
    } else {
        return false;
    };
    let old = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let e = parse_clause(&format!("that player {rest}"), b);
    b.it_player = old;
    let Some(e) = e else {
        return false;
    };
    let Some(f) = last_flip(prev) else {
        return false;
    };
    let branch = if heads { &mut f.on_heads } else { &mut f.on_tails };
    if !matches!(branch, Effect::Noop) {
        return false;
    }
    *branch = e;
    // CR 705.2: an effect that cares only about heads or tails has no call.
    f.call = !matches!(f.on_win, Effect::Noop) || !matches!(f.on_lose, Effect::Noop);
    true
}

inventory::submit! { FollowupPattern { name: "a705 each player whose coin comes up", priority: 10, apply: f_whose_coin } }

// ---------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------

fn c_dice_coins(c: &str) -> Option<Condition> {
    let c = end(c);
    // "you've rolled three or more dice this turn"
    if let Some(r) = c.strip_prefix("you've rolled ") {
        if let Some(n) = r.strip_suffix(" or more dice this turn") {
            let (v, rest) = parse_number(n)?;
            let Value::Const(n) = v else { return None };
            if !rest.trim().is_empty() {
                return None;
            }
            return Some(Condition::Custom(SmolStr::new(format!(
                "{DICE_THIS_TURN}:{n}"
            ))));
        }
        // "you've rolled a 20 this turn"
        let n = r
            .strip_prefix("a ")
            .or_else(|| r.strip_prefix("an "))?
            .strip_suffix(" this turn")?;
        let n: u32 = n.parse().ok()?;
        return Some(Condition::Custom(SmolStr::new(format!(
            "{RESULT_THIS_TURN}:{n}"
        ))));
    }
    // "you rolled 6 or higher", "the roll was 6 or higher": the roll the ability is about.
    if let Some(r) = c
        .strip_prefix("you rolled ")
        .or_else(|| c.strip_prefix("the roll was "))
    {
        let (n, cmp) = if let Some(n) = r.strip_suffix(" or higher") {
            (n, Cmp::Ge)
        } else if let Some(n) = r.strip_suffix(" or lower") {
            (n, Cmp::Le)
        } else {
            return None;
        };
        let n: i32 = n.parse().ok()?;
        return Some(Condition::Compare(the_result(), cmp, Value::c(n)));
    }
    // "If you roll a 1, sacrifice this creature", "whenever you roll to visit your
    // Attractions, if you roll a 6, ...": the result of the ability's roll, or of the
    // roll that triggered it.
    if let Some(r) = c.strip_prefix("you roll a ") {
        let (n, cmp) = match r.strip_suffix(" or higher") {
            Some(n) => (n, Cmp::Ge),
            None => (r, Cmp::Eq),
        };
        let n: i32 = n.parse().ok()?;
        return Some(Condition::Compare(the_result(), cmp, Value::c(n)));
    }
    // "you won five flips this way"
    if let Some(r) = c.strip_prefix("you won ") {
        let n = r
            .strip_suffix(" flips this way")
            .or_else(|| r.strip_suffix(" flip this way"))?;
        let (v, rest) = parse_number(n)?;
        if !rest.trim().is_empty() || !matches!(v, Value::Const(_)) {
            return None;
        }
        return Some(Condition::Compare(Value::Var(WINS), Cmp::Eq, v));
    }
    None
}

inventory::submit! { ConditionPattern { name: "a705-706 dice and coin conditions", priority: 0, parse: c_dice_coins } }

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

fn t_rolls(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
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
    let rest = rest
        .strip_prefix("roll ")
        .or_else(|| rest.strip_prefix("rolls "))?;
    let who = rel_word(rel);
    // "roll a natural 20" (CR 706.2).
    if let Some(n) = rest.strip_prefix("a natural ") {
        let n: u32 = n.parse().ok()?;
        return Some((
            TriggerCond::Custom(SmolStr::new(format!("{NATURAL_N}:{n}:{who}"))),
            Sel::None,
            PlayerRef::TriggerPlayer,
        ));
    }
    // "roll your third die each turn".
    if let Some(n) = rest
        .strip_prefix("your ")
        .or_else(|| rest.strip_prefix("their "))
        .and_then(|x| x.strip_suffix(" die each turn"))
    {
        let n = ordinal(n)?;
        return Some((
            TriggerCond::Custom(SmolStr::new(format!("{NTH_DIE}:{n}:{who}"))),
            Sel::None,
            PlayerRef::TriggerPlayer,
        ));
    }
    // "roll a 1 or 2".
    let (a, b) = rest.strip_prefix("a ")?.split_once(" or ")?;
    let (a, b): (i32, i32) = (a.parse().ok()?, b.parse().ok()?);
    let is = |n: i32| Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(n));
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::RollDie(rel)),
            cond: Condition::Or(vec![is(a), is(b)]),
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

fn ordinal(s: &str) -> Option<u32> {
    Some(match s {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    })
}

inventory::submit! { TriggerPattern { name: "a706 die roll triggers", priority: 0, parse: t_rolls } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        assert!(matches!(value("the other result"), Some((Value::Var(v), _)) if v == OTHER));
        assert!(matches!(value("that result to each creature"), Some((Value::Custom(_), r)) if r == " to each creature"));
        assert!(value("the results").is_none());
    }

    #[test]
    fn conditions() {
        assert!(matches!(
            c_dice_coins("you've rolled three or more dice this turn"),
            Some(Condition::Custom(n)) if n.as_str() == "die:rolled dice this turn:3"
        ));
        assert!(matches!(
            c_dice_coins("you've rolled a 20 this turn"),
            Some(Condition::Custom(n)) if n.as_str() == "die:rolled result this turn:20"
        ));
        assert!(c_dice_coins("the roll was 15 or higher").is_some());
        assert!(c_dice_coins("you won five flips this way").is_some());
        assert!(c_dice_coins("you rolled doubles").is_none());
    }

    #[test]
    fn triggers() {
        assert!(t_rolls("you roll a natural 20").is_some());
        assert!(t_rolls("you roll your third die each turn").is_some());
        assert!(t_rolls("you roll a 1 or 2").is_some());
        assert!(t_rolls("you roll a die").is_none());
    }
}
