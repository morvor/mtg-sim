//! Triggers on the events of the player an Aura (a Curse) enchants: "Whenever enchanted
//! player draws a card", "Whenever enchanted opponent draws a card", "Whenever enchanted
//! player is dealt damage" (CR 303.4: the Aura is attached to that player). They're
//! parsed as the same event of "a player", limited to the enchanted player; "they" and
//! "that player" are that player.

use super::TriggerPattern;
use crate::ability::*;

fn enchanted_player_event(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = r.trim();
    let rest = r
        .strip_prefix("enchanted player ")
        .or_else(|| r.strip_prefix("enchanted opponent "))?;
    // "is attacked" has its own form (triggers_attacks.rs).
    if rest.starts_with("is attacked") {
        return None;
    }
    let (cond, it, player) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever a player {rest}"))?;
    if !matches!(player, PlayerRef::TriggerPlayer) {
        return None;
    }
    let enchanted =
        PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))));
    // A batched trigger ("is dealt damage", once per batch of simultaneous damage) is
    // matched event by event inside the batch.
    let cond = super::trigger_grammar_events::where_events(
        cond,
        Condition::PlayerMatches(PlayerRef::TriggerPlayer, enchanted),
    );
    Some((cond, it, player))
}

inventory::submit! { TriggerPattern { name: "enchanted player/opponent [player event]", priority: 150, parse: enchanted_player_event } }
