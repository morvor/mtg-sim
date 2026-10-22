//! "For each opponent, [you] create a 1/1 black and green Pest creature token with ..."
//! (Eccentric Pestfinder): the instruction is performed once per opponent. Only
//! instructions that don't refer to the opponent ("that player", "they") are read this
//! way; those that do need that player bound to each opponent in turn.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn for_each_opponent(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each opponent, ")?;
    let words: Vec<&str> = r.split([' ', ',']).collect();
    if words
        .iter()
        .any(|w| matches!(*w, "that" | "they" | "their" | "them" | "player" | "opponent"))
    {
        return None;
    }
    let effect = parse_clause(r, b)?;
    // Only token creation: other instructions repeated per opponent may involve choices
    // that ought to be made per opponent.
    if !matches!(effect, Effect::CreateToken { .. }) {
        return None;
    }
    Some(Effect::Repeat {
        times: Value::CountPlayers(PlayerFilter::Opponent),
        effect: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "for each opponent, [create tokens]", priority: 120, parse: for_each_opponent } }
