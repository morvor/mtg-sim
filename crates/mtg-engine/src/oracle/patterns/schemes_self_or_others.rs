//! "Target opponent chooses self or others. If that player chooses self, [effect]. If the
//! player chooses others, [effect]." (the Nicol Bolas schemes of Archenemy: Feed the
//! Machine, The Fate of the Flammable, May Civilization Collapse, Surrender Your
//! Thoughts). The targeted player chooses one of the two words as the ability resolves
//! (`Effect::ChooseOne`), even if that choice would do nothing; "others" are the ability
//! controller's other opponents, each of whom performs the instruction (or is dealt the
//! damage) in turn.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::end;

const SELF: &str = "self";
const OTHERS: &str = "others";
const OTHER_OPPONENTS: &str = "each of your other opponents";

/// "target opponent chooses self or others".
fn chooses_self_or_others(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" chooses self or others")?;
    let (who, rest) = player_ref(r, b)?;
    if !rest.trim().is_empty() || !matches!(who, PlayerRef::Target(_)) {
        return None;
    }
    b.it_player = who.clone();
    Some(Effect::ChooseOne {
        who,
        options: vec![(SELF.into(), Effect::Noop), (OTHERS.into(), Effect::Noop)],
    })
}

inventory::submit! { EffectPattern { name: "chooses self or others", priority: 100, parse: chooses_self_or_others } }

/// "If that player chooses self, [effect]." / "If the player chooses others, [effect]."
/// after "... chooses self or others": the effect of that choice.
fn self_or_others_branch(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::ChooseOne { who, options } = prev else {
        return false;
    };
    if options.len() != 2 || options[0].0 != SELF || options[1].0 != OTHERS {
        return false;
    }
    let branch = |word: &str| {
        ["if that player chooses ", "if the player chooses "]
            .iter()
            .find_map(|p| l.strip_prefix(p)?.strip_prefix(word)?.strip_prefix(", "))
    };
    let chooser = who.clone();
    let saved = b.it_player.clone();
    let parsed = if let Some(body) = branch(SELF) {
        // "the player sacrifices two creatures of their choice": the chooser.
        b.it_player = chooser.clone();
        let body = match body.strip_prefix("the player ") {
            Some(r) => format!("that player {r}"),
            None => body.to_string(),
        };
        parse_clause(&body, b).map(|e| (0, e))
    } else if let Some(body) = branch(OTHERS) {
        // "each of your other opponents discards two cards", "~ deals 3 damage to each of
        // your other opponents": each opponent other than the chooser, one at a time.
        let others = PlayerRef::Each(PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(chooser.clone())))),
        ]));
        let body = if let Some(r) = body.strip_prefix(&format!("{OTHER_OPPONENTS} ")) {
            Some(format!("that player {r}"))
        } else if body.contains(OTHER_OPPONENTS) {
            Some(body.replace(OTHER_OPPONENTS, "that player"))
        } else {
            None
        };
        b.it_player = PlayerRef::Iterated;
        body.and_then(|body| parse_clause(&body, b)).map(|e| {
            (
                1,
                Effect::ForEachPlayer {
                    who: others,
                    effect: Box::new(e),
                },
            )
        })
    } else {
        None
    };
    b.it_player = saved;
    match parsed {
        Some((i, e)) => {
            options[i].1 = e;
            true
        }
        None => false,
    }
}

inventory::submit! { FollowupPattern { name: "self or others: branch", priority: 100, apply: self_or_others_branch } }
