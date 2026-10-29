//! "Whenever a player casts their first spell each turn, choose another target player.
//! ~ deals damage equal to that spell's mana value to the chosen player." (The Lord of
//! Pain): in a trigger with a triggering player, "another target player" is a target
//! player other than that one (CR 115.1), and "the chosen player" in the next sentence is
//! that target.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// "choose another target player": a target player other than the triggering player.
fn choose_another_target_player(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "choose another target player"
        || !b.in_trigger
        || !matches!(b.it_player, PlayerRef::TriggerPlayer)
    {
        return None;
    }
    let other = PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
        PlayerRef::TriggerPlayer,
    ))));
    let spec = TargetSpec::one(TargetKind::Player(other), "");
    // The triggering player isn't a target: no other target to differ from.
    let slot = b.add_target(spec, "target player");
    b.targets[slot as usize].text = "another target player".into();
    b.it_player = PlayerRef::Target(slot);
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choose another target player", priority: 80, parse: choose_another_target_player } }

/// "[effect] to the chosen player" after "choose another target player": the chosen
/// player is that target.
fn to_the_chosen_player(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let slot = match (&*prev, &b.it_player) {
        (Effect::Noop, PlayerRef::Target(slot)) => *slot,
        _ => return false,
    };
    if b.targets.get(slot as usize).map(|t| t.text.as_str()) != Some("another target player")
        || !l.contains("the chosen player")
    {
        return false;
    }
    let rewritten = l.replace("the chosen player", "that player");
    match parse_clause(&rewritten, b) {
        Some(e) => {
            *prev = e;
            true
        }
        None => false,
    }
}

inventory::submit! { FollowupPattern { name: "[effect] to the chosen player (another target player)", priority: 80, apply: to_the_chosen_player } }
