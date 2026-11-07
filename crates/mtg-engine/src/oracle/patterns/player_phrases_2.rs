//! More player phrases (read wherever a player is named: subjects, damage recipients),
//! reached through `choice_grammar::player_phrase`:
//!
//! - "each of your opponents";
//! - "each of its controller's opponents" (the opponents of the controller of what "it"
//!   is);
//! - "each player who controls a white creature", "each opponent who doesn't control an
//!   Elf" ([`PlayerFilter::Controls`]);
//! - "target opponent whose turn it is" (a target: the opponent who is the active player);
//! - "another target player" (a player target other than the earlier player targets).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether `r` ends a phrase (nothing more of it follows).
fn ends(r: &str) -> bool {
    r.is_empty() || r.starts_with([' ', ',', '.', '\''])
}

pub fn player_phrase(s: &str, b: &mut Builder) -> Option<(PlayerRef, String)> {
    if let Some(r) = s.strip_prefix("each of your opponents") {
        return ends(r).then(|| (PlayerRef::EachOpponent, r.to_string()));
    }
    if let Some(r) = s.strip_prefix("each of its controller's opponents") {
        if !ends(r) || super::oracle_hardening_referents::is_no_referent(&b.it) {
            return None;
        }
        let ctl = PlayerRef::ControllerOf(Box::new(b.it.clone()));
        return Some((
            PlayerRef::Each(PlayerFilter::OpponentOf(Box::new(ctl))),
            r.to_string(),
        ));
    }
    // "each player who controls a white creature", "each opponent who doesn't control an
    // Elf".
    for (p, base) in [
        ("each player who ", PlayerFilter::Any),
        ("each opponent who ", PlayerFilter::Opponent),
        ("each other player who ", PlayerFilter::NotYou),
    ] {
        let Some(r) = s.strip_prefix(p) else {
            continue;
        };
        let (r, none) = if let Some(r) = r.strip_prefix("controls ") {
            (r, false)
        } else if let Some(r) = r.strip_prefix("doesn't control ") {
            (r, true)
        } else {
            return None;
        };
        let x = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
        let (f, false, rest) = parse_object_phrase(x)? else {
            return None;
        };
        if !ends(rest) {
            return None;
        }
        let f = Filter::and(vec![f, Filter::Permanent]);
        let has = if none {
            PlayerFilter::Controls(Box::new(f), Cmp::Eq, Box::new(Value::c(0)))
        } else {
            PlayerFilter::Controls(Box::new(f), Cmp::Ge, Box::new(Value::c(1)))
        };
        let who = match base {
            PlayerFilter::Any => has,
            base => PlayerFilter::And(vec![base, has]),
        };
        return Some((PlayerRef::Each(who), rest.to_string()));
    }
    // "target opponent whose turn it is"
    if let Some(r) = s.strip_prefix("target opponent whose turn it is") {
        if !ends(r) {
            return None;
        }
        let text = "target opponent whose turn it is";
        let spec = TargetSpec::player(
            PlayerFilter::And(vec![PlayerFilter::Opponent, PlayerFilter::Active]),
            text,
        );
        let slot = b.add_target(spec, text);
        b.it_player = PlayerRef::Target(slot);
        return Some((PlayerRef::Target(slot), r.to_string()));
    }
    // "another target player": a player target other than those chosen before.
    if let Some(r) = s.strip_prefix("another target player") {
        if !ends(r) {
            return None;
        }
        let earlier: Vec<u8> = b
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(t.what, TargetKind::Player(_)))
            .map(|(i, _)| i as u8)
            .collect();
        if earlier.is_empty() {
            return None;
        }
        let text = "another target player";
        let mut spec = TargetSpec::player(PlayerFilter::Any, text);
        spec.distinct_from = earlier;
        let slot = b.add_target(spec, text);
        b.it_player = PlayerRef::Target(slot);
        return Some((PlayerRef::Target(slot), r.to_string()));
    }
    None
}

/// "target player reveals their library": that player reveals every card in it.
fn reveals_their_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" reveals their library")?;
    let targets = b.targets.len();
    let (who, rest) = crate::oracle::effects::player_ref(subj, b)?;
    if !rest.trim().is_empty() || matches!(who, PlayerRef::You) {
        b.targets.truncate(targets);
        return None;
    }
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(Effect::Custom(
            crate::reveal_library::REVEAL_OWN_LIBRARY.into(),
        )),
    })
}

inventory::submit! { super::EffectPattern { name: "[player] reveals their library", priority: 120, parse: reveals_their_library } }

/// "Opponents can't cast spells with the chosen name until your next turn." (Academic
/// Probation): "opponents" alone as a subject means "your opponents".
fn bare_opponents(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("opponents ")?;
    crate::oracle::effects::parse_clause(&format!("your opponents {r}"), b)
}

inventory::submit! { super::EffectPattern { name: "opponents [do something] (your opponents)", priority: 120, parse: bare_opponents } }

/// "~ deals 4 damage to an opponent chosen at random" (Zaffai, Thunder Conductor): the
/// opponent is chosen at random as the effect happens, then dealt the damage.
fn damage_to_random_opponent(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for (suffix, chooser) in [
        (" to an opponent chosen at random", "choose an opponent at random"),
        (" to a player chosen at random", "choose a player at random"),
    ] {
        let Some(head) = l.strip_suffix(suffix) else {
            continue;
        };
        if !head.contains(" deals ") {
            return None;
        }
        let targets = b.targets.len();
        let saved = b.it_player.clone();
        let choose = crate::oracle::effects::parse_clause(chooser, b);
        let deal = choose
            .as_ref()
            .and_then(|_| crate::oracle::effects::parse_clause(&format!("{head} to that player"), b));
        match (choose, deal) {
            (Some(c), Some(d)) if b.targets.len() == targets => {
                return Some(Effect::seq(vec![c, d]))
            }
            _ => {
                b.targets.truncate(targets);
                b.it_player = saved;
                return None;
            }
        }
    }
    None
}

inventory::submit! { super::EffectPattern { name: "deals N damage to an opponent chosen at random", priority: 120, parse: damage_to_random_opponent } }

/// "destroy target nonbasic land defending player controls, and creatures that player
/// controls without flying can't block this turn" (The Motherlode, Excavator): "that
/// player" after "defending player" is the defending player.
fn that_defending_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (before, after) = l.split_once("defending player")?;
    if !after.contains(" that player ") || before.contains("that player") {
        return None;
    }
    let after = after.replace(" that player ", " defending player ");
    crate::oracle::effects::parse_clause(&format!("{before}defending player{after}"), b)
}

inventory::submit! { super::EffectPattern { name: "defending player ... that player (the defending player)", priority: 120, parse: that_defending_player } }
