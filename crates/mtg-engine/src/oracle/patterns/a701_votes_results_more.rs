//! What follows a vote (CR 701.38; `kwa/vote.rs`), beyond `a701_choices_votes.rs`:
//!
//! - votes for players: "each player draws a card for each vote they received", "each
//!   player who received no votes may [instruction]", "for each vote an opponent
//!   received, ~ deals 2 damage to that player and each creature that player controls",
//!   "for each vote you received, draw a card";
//! - "for each money vote, choose a permanent owned by the voter and gain control of it":
//!   the instruction once per vote, "the voter" being who cast it;
//! - "return each card with the most votes or tied for most votes to your hand", "~ gains
//!   protection from each color with the most votes or tied for most votes", "if no
//!   creature got votes, [instruction]";
//! - after "whenever players finish voting": "you scry X, where X is the number of
//!   opponents who voted for a choice you didn't vote for", "you and each opponent who
//!   voted for a choice you voted for may scry 2".

use super::EffectPattern;
use crate::ability::*;
use crate::kwa::vote::{word_var, word_voters_var, MOST_VOTES, VOTES_CAST, VOTES_RECEIVED, WINNERS};
use crate::oracle::effects::{parse_clause, parse_effect_text, Builder};
use crate::oracle::phrases::*;
use crate::types::Color;
use smol_str::SmolStr;

/// The player one vote was cast for or by, in a "for each vote" instruction.
const THE_VOTE: Var = vars::USER + 30526;

fn received() -> Value {
    Value::Custom(SmolStr::new(VOTES_RECEIVED))
}

/// Parses `text` with `phrase` naming `who`.
fn with_named(text: &str, phrase: &str, who: PlayerRef, b: &mut Builder) -> Option<Effect> {
    b.named.push((phrase.to_string(), Sel::Players(who.clone())));
    let saved = std::mem::replace(&mut b.it_player, who);
    let targets = b.targets.len();
    let e = parse_effect_text(text, b);
    b.it_player = saved;
    b.named.pop();
    if e.is_none() || b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    e
}

fn vote_results_more(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "each player draws a card for each vote they received"
    if let Some(r) = l.strip_prefix("each player ") {
        if let Some(pred) = r.strip_suffix(" for each vote they received") {
            let one = super::player_subjects::as_you(pred)?;
            let e = parse_clause(&format!("you {one}"), b)?;
            let e = match e {
                Effect::Draw { who, .. } => Effect::Draw { who, n: received() },
                _ => return None,
            };
            return Some(Effect::ForEachPlayer {
                who: PlayerRef::EachPlayer,
                effect: Box::new(Effect::AsPlayer {
                    who: PlayerRef::Iterated,
                    effect: Box::new(e),
                }),
            });
        }
        // "each player who received no votes may put a permanent card from their hand
        // onto the battlefield"
        if let Some(pred) = r.strip_prefix("who received no votes ") {
            let e = parse_clause(&format!("each player {pred}"), b)?;
            let from = serde_json::to_value(PlayerRef::EachPlayer).ok()?;
            let to = serde_json::to_value(PlayerRef::Each(PlayerFilter::Not(Box::new(
                PlayerFilter::Ref(Box::new(PlayerRef::Var(VOTES_CAST))),
            ))))
            .ok()?;
            let mut n = 0;
            let v = replace_json(serde_json::to_value(&e).ok()?, &from, &to, &mut n);
            return (n == 1).then(|| serde_json::from_value(v).ok()).flatten();
        }
    }
    // "for each vote you received, draw a card"
    if let Some(r) = l.strip_prefix("for each vote you received, ") {
        let e = parse_clause(r, b)?;
        return Some(Effect::Repeat {
            times: received(),
            effect: Box::new(e),
        });
    }
    // "for each vote an opponent received, ~ deals 2 damage to that player and each
    // creature that player controls"
    if let Some(r) = l.strip_prefix("for each vote an opponent received, ") {
        let e = with_named(r, "that player", PlayerRef::Var(THE_VOTE), b)?;
        return Some(Effect::ForEach {
            sel: Sel::Var(VOTES_CAST),
            var: THE_VOTE,
            effect: Box::new(Effect::If {
                cond: Condition::PlayerMatches(PlayerRef::Var(THE_VOTE), PlayerFilter::Opponent),
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            }),
        });
    }
    // "for each money vote, choose a permanent owned by the voter and gain control of it"
    if let Some(r) = l.strip_prefix("for each ") {
        if let Some((word, rest)) = r.split_once(" vote, ") {
            if rest.contains("the voter") && word.split(' ').count() <= 3 {
                let e = with_named(rest, "the voter", PlayerRef::Var(THE_VOTE), b)?;
                return Some(Effect::ForEach {
                    sel: Sel::Var(word_voters_var(word)),
                    var: THE_VOTE,
                    effect: Box::new(e),
                });
            }
        }
    }
    // "return each card with the most votes or tied for most votes to your hand"
    if let Some(r) = l.strip_prefix("return each ") {
        let (noun, to) = r.split_once(" with the most votes or tied for most votes to ")?;
        if !matches!(noun, "card" | "creature card" | "permanent") || to != "your hand" {
            return None;
        }
        return parse_clause("return target card to your hand", &mut Builder::new(b.ctx)).and_then(
            |e| {
                let from = serde_json::to_value(Sel::Target(0)).ok()?;
                let to = serde_json::to_value(Sel::Var(WINNERS)).ok()?;
                let mut n = 0;
                let v = replace_json(serde_json::to_value(&e).ok()?, &from, &to, &mut n);
                (n == 1).then(|| serde_json::from_value(v).ok()).flatten()
            },
        );
    }
    // "~ gains protection from each color with the most votes or tied for most votes"
    if let Some(who) = l.strip_suffix(" gains protection from each color with the most votes or tied for most votes") {
        if who != "~" {
            return None;
        }
        let mut v = Vec::new();
        for c in Color::ALL {
            let e = parse_clause(&format!("~ gains protection from {}", c.word()), b)?;
            v.push(Effect::If {
                cond: Condition::And(vec![
                    Condition::Compare(Value::Var(MOST_VOTES), Cmp::Gt, Value::c(0)),
                    Condition::Compare(Value::Var(word_var(c.word())), Cmp::Eq, Value::Var(MOST_VOTES)),
                ]),
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            });
        }
        return Some(Effect::seq(v));
    }
    None
}

inventory::submit! { EffectPattern { name: "a701 vote results (players, voters, colors, cards)", priority: 55, parse: vote_results_more } }

/// "If no creature got votes, [instruction]".
fn if_no_votes(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if no ")?;
    let (noun, rest) = r.split_once(" got votes, ")?;
    if !matches!(noun, "creature" | "permanent" | "card" | "player") {
        return None;
    }
    let e = parse_clause(rest, b)?;
    Some(Effect::If {
        cond: Condition::Compare(Value::Var(MOST_VOTES), Cmp::Eq, Value::c(0)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if no creature got votes", priority: 55, parse: if_no_votes } }

/// "[instruction with X], where X is the number of opponents who voted for a choice you
/// (didn't) vote(d) for" (after "whenever players finish voting").
fn x_is_opponents_who_voted(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (x, like) = if let Some(x) =
        l.strip_suffix(", where x is the number of opponents who voted for a choice you didn't vote for")
    {
        (x, false)
    } else {
        (
            l.strip_suffix(", where x is the number of opponents who voted for a choice you voted for")?,
            true,
        )
    };
    let e = parse_clause(x, b)?;
    Some(Effect::seq(vec![
        Effect::SetX {
            value: Value::CountPlayers(PlayerFilter::And(vec![
                PlayerFilter::Opponent,
                PlayerFilter::VotedLikeYou(like),
            ])),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "where X is the number of opponents who voted ...", priority: 55, parse: x_is_opponents_who_voted } }

/// "you and each opponent who voted for a choice you voted for may scry 2".
fn you_and_opponents_who_voted(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (pred, like) = if let Some(p) =
        l.strip_prefix("you and each opponent who voted for a choice you voted for ")
    {
        (p, true)
    } else {
        (
            l.strip_prefix("you and each opponent who voted for a choice you didn't vote for ")?,
            false,
        )
    };
    if pred.contains("their") || pred.contains("they") {
        return None;
    }
    let targets = b.targets.len();
    let e = parse_clause(&format!("you {pred}"), b);
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Each(PlayerFilter::Or(vec![
            PlayerFilter::You,
            PlayerFilter::And(vec![PlayerFilter::Opponent, PlayerFilter::VotedLikeYou(like)]),
        ])),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e?),
        }),
    })
}

inventory::submit! { EffectPattern { name: "you and each opponent who voted for a choice you voted for", priority: 55, parse: you_and_opponents_who_voted } }

fn replace_json(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
    n: &mut usize,
) -> serde_json::Value {
    use serde_json::Value as J;
    if v == *from {
        *n += 1;
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| replace_json(x, from, to, n)).collect()),
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, replace_json(x, from, to, n)))
                .collect(),
        ),
        x => x,
    }
}

/// "owned by the voter" (in "for each money vote, choose a permanent owned by the voter"):
/// owned by the player who cast the vote the instruction is performed for.
fn owned_by_the_voter<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s.strip_prefix("owned by the voter")?;
    (r.is_empty() || r.starts_with([' ', ',', '.'])).then(|| {
        (
            Filter::OwnedByPlayer(Box::new(PlayerRef::Var(THE_VOTE))),
            r,
        )
    })
}

inventory::submit! { super::FilterSuffixPattern { name: "owned by the voter", priority: 120, parse: owned_by_the_voter } }
