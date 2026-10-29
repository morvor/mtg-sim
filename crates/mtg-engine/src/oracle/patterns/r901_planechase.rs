//! Planechase effects (CR 901) that change the game's planar rules for a while or
//! planeswalk in unusual ways:
//!
//! * "each blank roll of the planar die is a {CHAOS} roll until a player planeswalks away
//!   from a plane" (Chaotic Aether) — a duration that ends as a player planeswalks
//!   (CR 901.11);
//! * "until your next turn, if a player would planeswalk as a result of rolling the
//!   planar die, chaos ensues instead" (Fixed Point in Time);
//! * "reveal cards from the top of your planar deck until you reveal two plane cards.
//!   Simultaneously planeswalk to both of them. Put all other cards revealed this way on
//!   the bottom of your planar deck in any order." (Spatial Merging, CR 901.11c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, Builder};
use crate::oracle::phrases::end;
use crate::planechase::{BLANK_ROLLS_ARE_CHAOS, PLANESWALK_ROLLS_ARE_CHAOS, PLANESWALK_TO_PLANES};
use smol_str::SmolStr;

fn rule(name: &str, duration: Duration) -> Effect {
    Effect::AddRestriction {
        restriction: Restriction::Custom(SmolStr::new(name)),
        duration,
    }
}

/// "each blank roll of the planar die is a {chaos} roll [duration]".
fn blank_rolls_are_chaos(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (duration, rest) = duration_suffix(end(l));
    matches!(
        rest.trim(),
        "each blank roll of the planar die is a {chaos} roll"
            | "each blank roll of the planar die is a chaos roll"
    )
    .then(|| rule(BLANK_ROLLS_ARE_CHAOS, duration))
}

inventory::submit! { EffectPattern { name: "r901 blank rolls are chaos", priority: 60, parse: blank_rolls_are_chaos } }

/// "until your next turn, if a player would planeswalk as a result of rolling the planar
/// die, chaos ensues instead".
fn planeswalk_rolls_are_chaos(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (duration, rest) = match end(l).strip_prefix("until your next turn, ") {
        Some(r) => (Duration::UntilYourNextTurn, r),
        None => (Duration::EndOfTurn, end(l).strip_prefix("this turn, ")?),
    };
    (rest == "if a player would planeswalk as a result of rolling the planar die, chaos ensues instead")
        .then(|| rule(PLANESWALK_ROLLS_ARE_CHAOS, duration))
}

inventory::submit! { EffectPattern { name: "r901 planeswalk rolls are chaos", priority: 60, parse: planeswalk_rolls_are_chaos } }

const NUMBERS: [&str; 5] = ["one", "two", "three", "four", "five"];

/// "reveal cards from the top of your planar deck until you reveal N plane cards" (with the
/// two follow-up sentences below): planeswalk to N planes at once.
fn reveal_until_planes(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("reveal cards from the top of your planar deck until you reveal ")?
        .strip_suffix(" plane cards")?;
    let n = NUMBERS.iter().position(|w| *w == r)? + 1;
    Some(Effect::Custom(SmolStr::new(format!(
        "{PLANESWALK_TO_PLANES}{n}"
    ))))
}

inventory::submit! { EffectPattern { name: "r901 reveal until planes", priority: 60, parse: reveal_until_planes } }

/// "Simultaneously planeswalk to both of them." / "Put all other cards revealed this way
/// on the bottom of your planar deck in any order." after the reveal above.
fn planeswalk_to_revealed(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Effect::Custom(n) = prev else {
        return false;
    };
    n.starts_with(PLANESWALK_TO_PLANES)
        && matches!(
            end(l),
            "simultaneously planeswalk to both of them"
                | "simultaneously planeswalk to all of them"
                | "put all other cards revealed this way on the bottom of your planar deck in any order"
        )
}

inventory::submit! { FollowupPattern { name: "r901 planeswalk to revealed planes", priority: 60, apply: planeswalk_to_revealed } }
