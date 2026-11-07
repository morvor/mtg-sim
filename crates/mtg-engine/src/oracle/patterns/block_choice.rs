//! "You choose which creatures block this combat and how those creatures block" (Odric,
//! Master Tactician; Master Warcraft; Melee) and "[creatures] block this turn if able, and
//! you choose how those creatures block" (Brutal Hordechief): the effect's controller
//! declares blockers instead of the defending players (see [`crate::block_choice`]).
//! Also "You choose which creatures attack this turn" (Master Warcraft): the effect's
//! controller chooses the attacking creatures (see [`crate::attack_choice`]).

use super::EffectPattern;
use crate::ability::*;
use crate::attack_choice::CHOOSES_ATTACKERS;
use crate::block_choice::{CHOOSES_BLOCKS, CHOOSES_OPPONENTS_BLOCKS};
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn chooses_blocks(name: &str, duration: Duration) -> Effect {
    Effect::AddRestriction {
        restriction: Restriction::Custom(SmolStr::new(name)),
        duration,
    }
}

fn you_choose_blocks(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for (text, duration) in [
        (
            "you choose which creatures block this combat and how those creatures block",
            Duration::EndOfCombat,
        ),
        (
            "you choose which creatures block this turn and how those creatures block",
            Duration::EndOfTurn,
        ),
    ] {
        if l == text {
            return Some(chooses_blocks(CHOOSES_BLOCKS, duration));
        }
    }
    // "Creatures your opponents control block this turn if able, and you choose how those
    // creatures block."
    let requirement = l.strip_suffix(", and you choose how those creatures block")?;
    let (duration, suffix) = if requirement.ends_with(" this turn if able") {
        (Duration::EndOfTurn, " this turn if able")
    } else if requirement.ends_with(" this combat if able") {
        (Duration::EndOfCombat, " this combat if able")
    } else {
        return None;
    };
    // Whose creatures "those creatures" are: all creatures, or those of the controller's
    // opponents.
    let name = match requirement.strip_suffix(suffix)?.strip_suffix(" block")? {
        "creatures your opponents control" => CHOOSES_OPPONENTS_BLOCKS,
        "all creatures" => CHOOSES_BLOCKS,
        _ => return None,
    };
    let first = parse_simple(requirement, b)?;
    let Effect::AddRestriction {
        restriction: Restriction::MustBlock(_),
        ..
    } = &first
    else {
        return None;
    };
    Some(Effect::seq(vec![first, chooses_blocks(name, duration)]))
}

inventory::submit! { EffectPattern { name: "block choice: you choose how creatures block", priority: 100, parse: you_choose_blocks } }

fn you_choose_attackers(l: &str, _b: &mut Builder) -> Option<Effect> {
    let duration = match end(l) {
        "you choose which creatures attack this turn" => Duration::EndOfTurn,
        "you choose which creatures attack this combat" => Duration::EndOfCombat,
        _ => return None,
    };
    Some(chooses_blocks(CHOOSES_ATTACKERS, duration))
}

inventory::submit! { EffectPattern { name: "attack choice: you choose which creatures attack", priority: 100, parse: you_choose_attackers } }
