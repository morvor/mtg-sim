//! Player grammar VI (cards5-choices-players): players picked by their results.
//!
//! - "Whenever one or more Pirates you control deal damage to your opponents, you create a
//!   Treasure token for each opponent dealt damage." (Malcolm, Keen-Eyed Navigator): one
//!   token for each player of the triggering batch of damage events, each counted once
//!   (CR 603.2c; `PlayerRef::TriggerPlayers`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// "[you] create [tokens] for each opponent dealt damage" in a "one or more [objects] deal
/// damage to [players]" trigger: read as "create X [tokens], where X is the number of
/// opponents dealt damage this way".
fn create_for_each_opponent_dealt_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let body = ["opponent", "player"].iter().find_map(|p| {
        l.strip_suffix(&format!(" for each {p} dealt damage"))
            .or_else(|| l.strip_suffix(&format!(" for each {p} dealt combat damage")))
            .map(|b| (b, *p, l.ends_with("combat damage")))
    });
    let (body, who, combat) = body?;
    if !b.in_trigger
        || !b
            .named
            .iter()
            .any(|(n, _)| n == super::referents_values_5::BATCH_PLAYERS)
    {
        return None;
    }
    let body = body.strip_prefix("you ").unwrap_or(body);
    let token = body.strip_prefix("create a ").or_else(|| body.strip_prefix("create an "))?;
    // "a Treasure token" → "X Treasure tokens".
    let plural = token.strip_suffix(" token")?;
    let dmg = if combat { "combat damage" } else { "damage" };
    let text = format!(
        "create x {plural} tokens, where x is the number of {who}s dealt {dmg} this way"
    );
    parse_sentence(&text, b)
}

inventory::submit! { EffectPattern { name: "create [token] for each opponent dealt damage (batch trigger)", priority: 90, parse: create_for_each_opponent_dealt_damage } }
