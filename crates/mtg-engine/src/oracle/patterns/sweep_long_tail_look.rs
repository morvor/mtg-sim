//! Looking at another player's hand: "Look at target player's hand.", "look at that
//! player's hand", "you may look at defending player's hand", and "Look at target
//! opponent's hand and choose a card from it. That player discards that card." — the
//! choice is made like one from a revealed hand (see `card_flow_reveal_hand`), but only
//! the player looking sees the cards (unlike revealing, CR 701.20a).

use super::card_flow_reveal_hand::choose_from_it;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "sweep: look at [player]'s hand", priority: 95, parse: look_at_hand }
}

fn look_at_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("look at ")?;
    let saved = b.targets.len();
    let saved_player = b.it_player.clone();
    let parsed = (|| {
        // "target player's hand": the player phrase ends at the possessive.
        let (who_text, rest) = r.split_once("'s hand")?;
        let (who, extra) = player_ref(who_text, b)?;
        if !extra.trim().is_empty() {
            return None;
        }
        // Only a single player whose hand "it" can name.
        if !matches!(
            who,
            PlayerRef::Target(_) | PlayerRef::TriggerPlayer | PlayerRef::DefendingPlayer
        ) {
            return None;
        }
        b.it_player = who.clone();
        let mut e = Effect::LookAtHand { who };
        match rest.trim() {
            "" => {}
            // "and choose a card from it", "and choose up to two cards from it".
            r => {
                let choose = r.strip_prefix("and choose ")?;
                if !choose_from_it(&format!("you choose {choose}"), &mut e, b) {
                    return None;
                }
            }
        }
        Some(e)
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
        b.it_player = saved_player;
    }
    parsed
}
