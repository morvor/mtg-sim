//! Mana added to a player the ability's controller chooses (CR 106.4, 605.1a: no target,
//! so these are still mana abilities):
//! - "A player of your choice adds {C}." (Victory Chimes)
//! - "Choose a player. That player adds one mana of any color they choose." (Spectral
//!   Searchlight): the player adding the mana chooses its color.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// "that player adds one mana of any color they choose": the player adding it chooses the
/// color ([`ManaProduction::AnyOneColor`] asks the player whose pool it goes to).
fn that_player_adds_color_they_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("that player adds ")?;
    if r != "one mana of any color they choose" || matches!(b.it_player, PlayerRef::You) {
        return None;
    }
    Some(Effect::AddMana {
        who: b.it_player.clone(),
        mana: ManaProduction::AnyOneColor(Value::c(1)),
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "that player adds one mana of any color they choose", priority: 55, parse: that_player_adds_color_they_choose } }

/// "a player of your choice adds {C}": the controller chooses a player (any player,
/// themselves included) as the ability resolves, who adds the mana.
fn player_of_your_choice_adds(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("a player of your choice adds ")?;
    let add = parse_sentence(&format!("add {r}"), b)?;
    let Effect::AddMana {
        who: PlayerRef::You,
        mana,
        restriction,
    } = add
    else {
        return None;
    };
    if matches!(mana, ManaProduction::AnyOneColor(_) | ManaProduction::AnyCombination(_)) {
        // Who would choose the color isn't said: not supported.
        return None;
    }
    b.it_player = PlayerRef::ChosenOpponent;
    Some(Effect::Seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Player,
        },
        Effect::AddMana {
            who: PlayerRef::ChosenOpponent,
            mana,
            restriction,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "a player of your choice adds [mana]", priority: 55, parse: player_of_your_choice_adds } }
