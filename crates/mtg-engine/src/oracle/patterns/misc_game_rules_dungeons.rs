//! Completing dungeons (CR 309.7): the condition "you've completed a dungeon" (Cloister
//! Gargoyle, Nadaar, Sarevok's Tome, ...) and the trigger "whenever you complete a
//! dungeon" (Varis, Dungeon Crawler, ...).

use super::{ConditionPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::dungeon_completion::{DUNGEONS_COMPLETED, DUNGEON_NAMED_COMPLETED};
use crate::oracle::phrases::end;
use smol_str::SmolStr;

/// "you've completed a dungeon", "you haven't completed a dungeon".
fn completed_a_dungeon(c: &str) -> Option<Condition> {
    let completed = Value::Custom(DUNGEONS_COMPLETED.into());
    match end(c) {
        "you've completed a dungeon" | "you have completed a dungeon" => {
            Some(Condition::Compare(completed, Cmp::Ge, Value::c(1)))
        }
        "you haven't completed a dungeon" | "you have not completed a dungeon" => {
            Some(Condition::Compare(completed, Cmp::Eq, Value::c(0)))
        }
        c => completed_named_dungeon(c),
    }
}

/// "you've completed Tomb of Annihilation", "you haven't completed Tomb of Annihilation":
/// a dungeon card's name.
fn completed_named_dungeon(c: &str) -> Option<Condition> {
    let (name, done) = if let Some(n) = c
        .strip_prefix("you've completed ")
        .or_else(|| c.strip_prefix("you have completed "))
    {
        (n, true)
    } else if let Some(n) = c
        .strip_prefix("you haven't completed ")
        .or_else(|| c.strip_prefix("you have not completed "))
    {
        (n, false)
    } else {
        return None;
    };
    let card = crate::card::CardDb::global().get(name.trim())?;
    if !card
        .front()
        .chars
        .card_types
        .contains(crate::types::CardType::Dungeon)
    {
        return None;
    }
    let count = Value::Custom(
        format!(
            "{DUNGEON_NAMED_COMPLETED}{}",
            card.front().chars.name.to_lowercase()
        )
        .into(),
    );
    Some(if done {
        Condition::Compare(count, Cmp::Ge, Value::c(1))
    } else {
        Condition::Compare(count, Cmp::Eq, Value::c(0))
    })
}

inventory::submit! { ConditionPattern { name: "misc: you've completed a dungeon", priority: 100, parse: completed_a_dungeon } }

/// "whenever you complete a dungeon" (the player completes it as the dungeon card is
/// removed from the game, CR 309.7).
fn complete_a_dungeon_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let who = match end(r) {
        "you complete a dungeon" => PlayerRel::You,
        "a player completes a dungeon" => PlayerRel::Any,
        "an opponent completes a dungeon" => PlayerRel::Opponent,
        _ => return None,
    };
    Some((
        TriggerCond::PlayerAction {
            name: SmolStr::new(crate::dungeons::COMPLETED),
            who,
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "misc: whenever you complete a dungeon", priority: 100, parse: complete_a_dungeon_trigger } }
