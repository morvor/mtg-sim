//! Oracle patterns for subgames (CR 729): "Players play a Magic subgame, using their
//! libraries as their decks." and "Each player who doesn't win the subgame loses half
//! their life, rounded up." (Shahrazad).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn subgame(l: &str, _b: &mut Builder) -> Option<Effect> {
    let name = match end(l) {
        "players play a magic subgame, using their libraries as their decks" => {
            crate::subgame::PLAY_SUBGAME
        }
        "each player who doesn't win the subgame loses half their life, rounded up" => {
            crate::subgame::NON_WINNERS_LOSE_HALF
        }
        _ => return None,
    };
    Some(Effect::Custom(SmolStr::new(name)))
}

inventory::submit! { EffectPattern { name: "r729 play a subgame", priority: 60, parse: subgame } }
