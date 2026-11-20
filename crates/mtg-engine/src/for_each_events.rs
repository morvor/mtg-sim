//! One "for each" instruction, one event: the tokens, counters and life one instruction
//! creates, puts or gains "for each" of several objects or players are one creation, one
//! putting of counters on each object and one life gain for each player, however the
//! compiler shaped the instruction (register `for-each-tokens-counters-life-one-event`,
//! extending `copy-for-each-one-copy-event` by analogy; engine-gaps G32).
//!
//! Wurmquake's ruling (2023-02-04): "... then you create a number of tokens equal to the
//! number of opponents with three of more poison counters and they enter the battlefield at
//! the same time." Tempting Contract's (2021-04-16): "... Then you create your Treasure
//! tokens all at the same time." Suffer the Past's (2010-06-15): "Suffer the Past’s effect
//! causes a single life-gain event, not a number of separate ones." So a replacement effect
//! that adds to "one or more" tokens or counters, or to life gained (Xorn, Peregrin Took,
//! Hardened Scales, Heron of Hope), applies once to the whole instruction, as it does to
//! "create N tokens".
//!
//! The shapes:
//! * `Effect::CreateToken` with a count ("For each land you control, create a Treasure
//!   token.", Brass's Bounty) is already one creation;
//! * `Effect::Repeat` of a single creation, counter placement or life gain ("For each
//!   opponent, you create a 2/1 ... token", Endless Ranks of HYDRA; Wurmquake's "for each
//!   opponent ..., you create another one of those tokens"): performed once with the
//!   amount multiplied ([`repeat_as_one`]);
//! * `Effect::ForEachPlayer` and `Effect::ForEach`, performed for each player or object
//!   (`simultaneous.rs`): the creations, counters and life gains of all of them are
//!   combined ([`merge_creations`] in `Game::create_tokens_in_groups`; the counter and life
//!   plans in `simultaneous::perform_plans`).
//!
//! What stays several events: what different players do (each player is a different "you":
//! "each player creates a token", the opponents' part of a tempting offer), separate
//! instructions ("create a token. Then create ..."), a keyword action done several times
//! (investigate twice: sequential, Tamiyo Meets the Story Circle's ruling), and each part's
//! own tokens when a later instruction of the part refers to them ("For each opponent,
//! create two 3/3 ... tokens with flying that attack that opponent this turn if able. They
//! gain haste until end of turn.", Furygale Flocking): those are told apart by the part they
//! belong to, as tokens created attacking each opponent are (Adeline, myriad: "If myriad
//! creates more than one token for any given player (due to an effect such as the one
//! Doubling Season creates), ..."), so each part's tokens are a creation of their own, made
//! at the same time as the others' (`simultaneous::Instruction::tokens_apart`), and a
//! doubling effect doubles each opponent's two.
//!
//! Not reached (gaps): tokens of different kinds (a token copy of each of several
//! creatures: one event to the rules, but the replacement pipeline's creation event carries
//! one kind of token), and a part performed for each player as one sequence of several
//! actions ("For each opponent who does, you draw a card and you create a 1/1 white Rabbit
//! creature token.", Tempt with Bunnies, compiled as one sequence performed for each
//! opponent who accepted in turn: a creation each). Seize the Spotlight's "For each player
//! who chose fortune, you draw a card and create a Treasure token." is two instructions,
//! each performed for all of them, so its Treasures are one creation.

use crate::ability::{Effect, Value};
use crate::replacement::TokenCreate;
use crate::types::*;

/// `effect` repeated `times` times as one instruction, if it's a single creation of
/// tokens, putting of counters (on objects chosen by no one) or life gain: the same action
/// with the amount multiplied. (`Effect::Repeat` also carries "Repeat the following process
/// X times." (Torment of Hailfire), whose repetitions are separate events; no card compiles
/// that around a single creation, counter placement or life gain.)
pub(crate) fn repeat_as_one(effect: &Effect, times: i64) -> Option<Effect> {
    if times < 2 {
        return None;
    }
    let mul = |v: &Value| Value::Mul(Box::new(Value::Const(times as i32)), Box::new(v.clone()));
    match effect {
        Effect::CreateToken {
            spec,
            count,
            controller,
            tapped,
            attacking,
        } => Some(Effect::CreateToken {
            spec: spec.clone(),
            count: mul(count),
            controller: controller.clone(),
            tapped: *tapped,
            attacking: *attacking,
        }),
        Effect::GainLife { who, n } => Some(Effect::GainLife {
            who: who.clone(),
            n: mul(n),
        }),
        Effect::AddCounters { what, kind, n } if !crate::simultaneous::has_choice(what) => {
            Some(Effect::AddCounters {
                what: what.clone(),
                kind: kind.clone(),
                n: mul(n),
            })
        }
        _ => None,
    }
}

/// One creation of tokens for [`Game::create_tokens_in_groups`](crate::game::Game): the
/// controller, the tokens, how many, the source, and which of the batch's entries it
/// combines (index and count).
pub(crate) type Creation = (
    PlayerId,
    TokenCreate,
    u32,
    Option<ObjectId>,
    Vec<(usize, u32)>,
);

/// Combines the entries of a batch of token creations made by one instruction that create
/// the same tokens for the same player from the same source into one creation (the count
/// summed), keeping the batch's order of first appearance. Only entries of the same group
/// (`groups[i]`, 0 if missing) are combined: a part of a "for each" whose tokens a later
/// instruction refers to has a group of its own (`simultaneous::Instruction::tokens_apart`).
pub(crate) fn merge_creations(
    batch: Vec<(PlayerId, TokenCreate, u32, Option<ObjectId>)>,
    groups: &[usize],
) -> Vec<Creation> {
    let mut out: Vec<Creation> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    for (i, (controller, spec, count, source)) in batch.into_iter().enumerate() {
        let group = groups.get(i).copied().unwrap_or(0);
        let key = format!("{group}|{}", creation_key(controller, &spec, source));
        match keys.iter().position(|k| *k == key) {
            Some(j) => {
                out[j].2 += count;
                out[j].4.push((i, count));
            }
            None => {
                keys.push(key);
                out.push((controller, spec, count, source, vec![(i, count)]));
            }
        }
    }
    out
}

/// What makes two creations the same: the player, the source and every part of the tokens.
fn creation_key(controller: PlayerId, spec: &TokenCreate, source: Option<ObjectId>) -> String {
    format!(
        "{controller:?}|{source:?}|{:?}|{:?}|{}|{:?}|{:?}|{:?}",
        spec.chars,
        spec.card.as_ref().map(|c| c.name.clone()),
        spec.tapped,
        spec.attacking,
        spec.copy_of,
        spec.copy_exceptions,
    )
}

/// Which entry of a combined creation each of its `made` tokens is credited to: each entry
/// gets as many as it asked for, in order, and the first one any more (the replacement
/// effects' additional tokens).
pub(crate) fn credit(parts: &[(usize, u32)], made: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(made);
    for (i, n) in parts {
        for _ in 0..*n {
            if out.len() < made {
                out.push(*i);
            }
        }
    }
    let first = parts.first().map_or(0, |p| p.0);
    while out.len() < made {
        out.push(first);
    }
    out
}
