//! Life and player-resource phrases:
//!
//! * drain: "[Each opponent loses N life.] You gain life equal to the life lost this way."
//!   (CR 119.3: the amount is the life actually lost by the preceding instruction);
//! * one player doing several things: "target player draws two cards and loses 2 life",
//!   "each opponent discards a card and loses 2 life", "you gain 2 life and get {E}{E}",
//!   "target player draws three cards, loses 3 life, and gets three poison counters".

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_simple, player_ref, Builder};
use crate::oracle::phrases::*;

/// Adds `gain` right after the life-loss instruction that ends `e` (inside the branch of
/// an "if"/"you may" that contains it). Returns false if `e` doesn't end in one.
fn attach_after_life_loss(e: &mut Effect, gain: Effect) -> bool {
    match e {
        Effect::LoseLife { .. } => {
            let lose = std::mem::replace(e, Effect::Noop);
            *e = Effect::Seq(vec![lose, gain]);
            true
        }
        // "Target opponent loses half their life": one player, so the amount recorded is
        // that player's loss.
        Effect::ForEachPlayer { who, effect }
            if matches!(**effect, Effect::LoseLife { .. })
                && matches!(
                    who,
                    PlayerRef::Target(_) | PlayerRef::You | PlayerRef::DefendingPlayer
                ) =>
        {
            let lose = std::mem::replace(e, Effect::Noop);
            *e = Effect::Seq(vec![lose, gain]);
            true
        }
        Effect::Seq(v) => v
            .last_mut()
            .is_some_and(|last| attach_after_life_loss(last, gain)),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => attach_after_life_loss(then, gain),
        Effect::May { effect, .. } => attach_after_life_loss(effect, gain),
        _ => false,
    }
}

/// "You gain life equal to the life lost this way." following a life-loss instruction;
/// also "If [condition], you gain life equal to the life lost this way."
fn gain_life_lost_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    const GAIN: &str = "you gain life equal to the life lost this way";
    let cond = if l == GAIN {
        None
    } else if let Some(c) = l
        .strip_prefix("if ")
        .and_then(|r| r.strip_suffix(&format!(", {GAIN}")))
    {
        match crate::oracle::statics::parse_condition(c, b.ctx) {
            Some(c) => Some(c),
            None => return false,
        }
    } else {
        return false;
    };
    let gain = Effect::GainLife {
        who: PlayerRef::You,
        n: Value::Prev,
    };
    let gain = match cond {
        Some(cond) => Effect::If {
            cond,
            then: Box::new(gain),
            otherwise: Box::new(Effect::Noop),
        },
        None => gain,
    };
    attach_after_life_loss(prev, gain)
}

inventory::submit! { FollowupPattern { name: "counters_resources: gain the life lost this way", priority: 100, apply: gain_life_lost_this_way } }

/// "[player] loses N life and you gain life equal to the life lost this way" in one
/// sentence.
fn lose_and_gain_life_lost(l: &str, b: &mut Builder) -> Option<Effect> {
    let (lose, rest) = end(l).split_once(" and you gain life equal to the life lost this way")?;
    if !rest.is_empty() {
        return None;
    }
    let mut e = parse_simple(lose, b)?;
    let gain = Effect::GainLife {
        who: PlayerRef::You,
        n: Value::Prev,
    };
    attach_after_life_loss(&mut e, gain).then_some(e)
}

inventory::submit! { EffectPattern { name: "counters_resources: loses life and you gain that much", priority: 100, parse: lose_and_gain_life_lost } }

/// Third-person verbs a player subject can take in a list of instructions.
const VERBS_3P: &[&str] = &[
    "draws",
    "loses",
    "gains",
    "discards",
    "sacrifices",
    "mills",
    "gets",
    "exiles",
    "shuffles",
    "reveals",
    "puts",
    "creates",
    "scries",
    "surveils",
];
/// The same verbs after "you".
const VERBS_YOU: &[&str] = &[
    "draw",
    "lose",
    "gain",
    "discard",
    "sacrifice",
    "mill",
    "get",
    "exile",
    "shuffle",
    "reveal",
    "put",
    "create",
    "scry",
    "surveil",
];

/// Splits "draws two cards, loses 2 life, and gets {E}" into its instructions.
fn split_instructions(s: &str) -> Vec<String> {
    let s = s
        .replace(", and then ", "\u{1}")
        .replace(", and ", "\u{1}")
        .replace(", then ", "\u{1}")
        .replace(" and then ", "\u{1}")
        .replace(" and ", "\u{1}")
        .replace(", ", "\u{1}");
    s.split('\u{1}').map(|p| p.trim().to_string()).collect()
}

/// One player subject with several instructions: "target player draws two cards and
/// loses 2 life", "each opponent sacrifices a creature of their choice, discards a card,
/// and loses 3 life", "you gain 2 life and get {E}{E}". Each instruction is performed in
/// order by the same player(s).
fn player_does_several(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // The subject: "you", or a player phrase parsed by the player parser (it adds a
    // player target for "target player"/"target opponent").
    let (who, subject, rest): (PlayerRef, String, String) = if let Some(r) = l.strip_prefix("you ")
    {
        (PlayerRef::You, "you".into(), r.to_string())
    } else {
        let before = b.targets.len();
        let (who, rest) = player_ref(l, b)?;
        let subject_text = &l[..l.len() - rest.len()];
        // A player target just added becomes "that player" for every instruction.
        let subject = if b.targets.len() > before {
            "that player".to_string()
        } else {
            subject_text.trim().to_string()
        };
        (who, subject, rest.trim().to_string())
    };
    let verbs = if subject == "you" {
        VERBS_YOU
    } else {
        VERBS_3P
    };
    let parts = split_instructions(&rest);
    if parts.len() < 2 {
        return None;
    }
    let mut effects = Vec::new();
    for p in &parts {
        let (verb, _) = split_word(p);
        if !verbs.contains(&verb) {
            return None;
        }
        b.it_player = who.clone();
        let clause = format!("{subject} {p}");
        let e = parse_simple(&clause, b).or_else(|| third_person_counters(&who, p))?;
        effects.push(e);
    }
    b.it_player = who;
    Some(Effect::seq(effects))
}

/// "gets three poison counters" for a subject the counter pattern doesn't know by name
/// (a target player).
fn third_person_counters(who: &PlayerRef, p: &str) -> Option<Effect> {
    let r = p.strip_prefix("gets ")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = r.trim_start().split_once(' ')?;
    if !matches!(kind, "poison" | "experience" | "rad") {
        return None;
    }
    if !matches!(end(r), "counter" | "counters") {
        return None;
    }
    Some(Effect::AddPlayerCounters {
        who: who.clone(),
        kind: kind.into(),
        n,
    })
}

inventory::submit! { EffectPattern { name: "counters_resources: player does several things", priority: 100, parse: player_does_several } }
