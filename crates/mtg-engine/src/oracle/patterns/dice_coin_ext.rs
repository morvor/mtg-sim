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

use super::{
    ConditionPattern, EffectPattern, FilterSuffixPattern, FollowupPattern, StaticPattern,
    TriggerPattern,
};
use crate::ability::*;
use crate::dice::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::CompileContext;
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
    // "draw two cards for each flip" (after flips that all won): every flip.
    if let Some(head) = l.strip_suffix(" for each flip") {
        let e = parse_clause(head, b)?;
        return Some(Effect::Repeat {
            times: Value::Sum(vec![Value::Var(WINS), Value::Var(LOSSES)]),
            effect: Box::new(e),
        });
    }
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

use super::r705_706_dice::last_flip;

/// The object the current flip of "flip a coin for each [object]" is for.
const FLIPPED_FOR: Var = vars::USER + 8540;

/// "Flip a coin for each creature that isn't a Demon", "for each blocking creature, flip a
/// coin", "flip a coin for each opponent you have": one flip for each, all as one
/// instruction (what's done to the objects whose coins come up tails happens to them at
/// once, `simultaneous`); "that creature" / "that player" is the one the flip is for.
fn p_flip_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let x = l
        .strip_prefix("flip a coin for each ")
        .or_else(|| l.strip_prefix("for each ")?.strip_suffix(", flip a coin"))?;
    let flip = CoinFlip::new();
    if matches!(x, "opponent you have" | "opponent") {
        b.it_player = PlayerRef::Iterated;
        return Some(Effect::ForEachPlayer {
            who: PlayerRef::EachOpponent,
            effect: Box::new(Effect::FlipCoins(Box::new(flip))),
        });
    }
    let f = match parse_object_phrase(x) {
        Some((f, _, tail)) if tail.trim().is_empty() => f,
        _ => {
            let (f, rest) = super::value_grammar::objects(x, b)?;
            if !rest.trim().is_empty() {
                return None;
            }
            f
        }
    };
    let mut flip = flip;
    // Only heads or tails matters until a win/lose part says otherwise (CR 705.2).
    flip.call = false;
    b.it = Sel::Var(FLIPPED_FOR);
    Some(Effect::ForEach {
        sel: Sel::All(f),
        var: FLIPPED_FOR,
        effect: Box::new(Effect::FlipCoins(Box::new(flip))),
    })
}

inventory::submit! { EffectPattern { name: "a705 flip a coin for each", priority: 0, parse: p_flip_for_each } }

/// Whether the effect's last flip is one of several flips made for each object or player.
fn flips_in_loop(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(flips_in_loop),
        Effect::ForEach { effect, .. } | Effect::ForEachPlayer { effect, .. } => {
            matches!(**effect, Effect::FlipCoins(_))
        }
        _ => false,
    }
}

/// After one flip for each object or player: "For each flip you win, draw a card. For each
/// flip you lose, ~ deals 3 damage to that player." — what happens after each flip.
fn f_for_each_flip_in_loop(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !flips_in_loop(prev) {
        return false;
    }
    let (win, rest) = if let Some(r) = l.strip_prefix("for each flip you win, ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("for each flip you lose, ") {
        (false, r)
    } else {
        return false;
    };
    let Some(e) = parse_clause(rest, b) else {
        return false;
    };
    let Some(f) = last_flip(prev) else {
        return false;
    };
    let branch = if win { &mut f.on_win } else { &mut f.on_lose };
    if !matches!(branch, Effect::Noop) {
        return false;
    }
    *branch = e;
    f.call = true;
    true
}

inventory::submit! { FollowupPattern { name: "a705 for each flip you win (one flip each)", priority: 5, apply: f_for_each_flip_in_loop } }

/// "Destroy each creature whose coin comes up tails." after "flip a coin for each
/// creature".
fn f_whose_coin_objects(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !flips_in_loop(prev) {
        return false;
    }
    let Some(i) = l.find(" whose coin comes up ") else {
        return false;
    };
    let after = &l[i + " whose coin comes up ".len()..];
    let (heads, after) = if let Some(r) = after.strip_prefix("tails") {
        (false, r)
    } else if let Some(r) = after.strip_prefix("heads") {
        (true, r)
    } else {
        return false;
    };
    // "each creature" / "each permanent": the objects the flips were for.
    let Some(j) = l[..i].rfind("each ") else {
        return false;
    };
    let rewritten = format!("{}it{after}", &l[..j]);
    let saved = std::mem::replace(&mut b.it, Sel::Var(FLIPPED_FOR));
    let e = parse_clause(&rewritten, b);
    b.it = saved;
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
    true
}

inventory::submit! { FollowupPattern { name: "a705 each object whose coin comes up", priority: 10, apply: f_whose_coin_objects } }

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

/// "Flip a coin X times or until you lose a flip, whichever comes first."
fn p_flip_until(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("flip a coin ")?;
    let (n, rest) = parse_number(r)?;
    if rest.trim() != "times or until you lose a flip, whichever comes first" {
        return None;
    }
    let mut spec = CoinFlip::new();
    spec.until_lose = true;
    spec.max_flips = Some(n);
    Some(Effect::FlipCoins(Box::new(spec)))
}

inventory::submit! { EffectPattern { name: "a705 flip until you lose or N times", priority: 0, parse: p_flip_until } }

/// "Until end of turn, you may have ~ become an X/X Insect artifact creature with flying":
/// optionally, "~ becomes ...".
fn p_may_have_become(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find("you may have ~ become ")?;
    let rewritten = format!(
        "{}~ becomes {}",
        &l[..i],
        &l[i + "you may have ~ become ".len()..]
    );
    let e = parse_clause(&rewritten, b)?;
    Some(Effect::May {
        who: PlayerRef::You,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "a706 you may have ~ become", priority: 0, parse: p_may_have_become } }

/// The token an effect creates last, if it ends creating one.
fn last_create(e: &Effect) -> Option<&Effect> {
    match e {
        Effect::CreateToken { .. } => Some(e),
        Effect::Seq(v) => v.last().and_then(last_create),
        _ => None,
    }
}

/// "Create a 3/3 colorless Robot artifact creature token. If you rolled 6 or higher,
/// instead create that token and a Treasure token.": the token, and more.
fn f_instead_that_token_and(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((cond, rest)) = r.split_once(", instead create that token and ") else {
        return false;
    };
    let Some(token) = last_create(prev).cloned() else {
        return false;
    };
    let Some(cond) = crate::oracle::statics::parse_condition(cond, b.ctx) else {
        return false;
    };
    let Some(more) = parse_clause(&format!("create {rest}"), b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::If {
        cond,
        then: Box::new(Effect::seq(vec![token, more])),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "a706 instead create that token and", priority: 10, apply: f_instead_that_token_and } }

/// "permanents named Wirefly": an object with that name (CR 201.2); the name is as printed
/// (capitalized).
fn named_suffix<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s.strip_prefix("named ")?;
    let n = r.find([',', '.', ';']).unwrap_or(r.len());
    let (name, rest) = (&r[..n], &r[n..]);
    if name.is_empty() || name.contains('~') || name.contains(" and ") || name.contains(" or ")
    {
        return None;
    }
    let printed = super::cost_parts::printed_name(name)?;
    Some((Filter::Named(printed), rest))
}

inventory::submit! { FilterSuffixPattern { name: "a705 named [printed name]", priority: 100, parse: named_suffix } }

/// "creature that isn't a Demon, Devil, or Imp": none of the listed subtypes (the
/// zone-move grammar reads a single type).
fn not_subtype_list<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s
        .strip_prefix("that isn't a ")
        .or_else(|| s.strip_prefix("that isn't an "))?;
    let n = r.find(['.', ';']).unwrap_or(r.len());
    let (list, rest) = (&r[..n], &r[n..]);
    if !list.contains(" or ") {
        return None;
    }
    let words: Vec<&str> = list
        .split([','])
        .flat_map(|w| w.split(" or "))
        .map(|w| w.trim().trim_start_matches("or ").trim())
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < 2 {
        return None;
    }
    let subtypes = words
        .iter()
        .map(|w| subtype_word(w).map(Filter::Subtype))
        .collect::<Option<Vec<_>>>()?;
    Some((Filter::not(Filter::Or(subtypes)), rest))
}

inventory::submit! { FilterSuffixPattern { name: "a705 that isn't a [subtype], [subtype], or [subtype]", priority: 960, parse: not_subtype_list } }

/// "After you roll a die, you may pay 1 life. If you do, increase or decrease the result by
/// 1. Do this only once each turn." (CR 706.2a)
fn s_after_you_roll(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("after you roll a die, you may ")?;
    let (cost, rest) = r.split_once(". if you do, increase or decrease the result by ")?;
    let (n, once) = match rest.split_once(". ") {
        Some((n, "do this only once each turn")) => (n, true),
        None => (rest, false),
        _ => return None,
    };
    let n: i64 = n.trim().parse().ok()?;
    let (cost, _) = crate::oracle::costs::parse_cost(cost)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Dice(DiceStatic::Modifier(
            DieModifier {
                whose: PlayerRel::You,
                kind: ModifierKind::AddOrSubtract(n),
                cost,
                once_per_turn: once,
                natural: None,
                sides: None,
                text: text.to_string(),
            },
        )))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a706 after you roll a die, you may", priority: 0, parse: s_after_you_roll } }

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
    // "you rolled 6 exactly seven times": that many of the roll's results were 6.
    if let Some(r) = c.strip_prefix("you rolled ") {
        if let Some((n, times)) = r.split_once(" exactly ") {
            let n: u32 = n.parse().ok().filter(|n| (1..=20).contains(n))?;
            let (m, rest) = parse_number(times)?;
            if rest.trim() != "times" && rest.trim() != "time" {
                return None;
            }
            return Some(Condition::Compare(
                Value::Var(RESULT_COUNT + n as Var),
                Cmp::Eq,
                m,
            ));
        }
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
    // "if you win all the flips": none lost.
    if c == "you win all the flips" || c == "you won all the flips" {
        return Some(Condition::Compare(Value::Var(LOSSES), Cmp::Eq, Value::c(0)));
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
