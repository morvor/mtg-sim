//! Processors (the Eldrazi of the Battle for Zendikar block): "put a card an opponent owns
//! from exile into that player's graveyard" and "put two cards your opponents own from
//! exile into their owners' graveyards", as an effect ("you may ... If you do, ...") and
//! as a cost. The player putting them there chooses the cards (they may be owned by
//! different opponents); each goes to its owner's graveyard (CR 400.3). Face-down exiled
//! cards the player can't look at are chosen by pile, at random within the pile
//! (CR 406.4, see `zones::choose_objects`).

use super::{CostPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;

/// The effect of putting `n` cards opponents own from exile into their owners' graveyards.
pub fn process(n: i32) -> Effect {
    Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Exile),
                Filter::OwnedBy(PlayerRel::Opponent),
            ]),
            count: Value::c(n),
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Graveyard),
    }
}

/// The number of cards the instruction puts from exile into graveyards.
fn process_count(l: &str) -> Option<i32> {
    let l = l.trim().trim_end_matches('.');
    if l == "put a card an opponent owns from exile into that player's graveyard" {
        return Some(1);
    }
    let r = l.strip_prefix("put ")?;
    let (num, rest) = r.split_once(' ')?;
    let n = match num {
        "two" => 2,
        "three" => 3,
        _ => return None,
    };
    (rest == "cards your opponents own from exile into their owners' graveyards").then_some(n)
}

fn process_effect(l: &str, _b: &mut Builder) -> Option<Effect> {
    process_count(l).map(process)
}

fn process_cost(l: &str) -> Option<CostPart> {
    process_count(l).map(|n| CostPart::Effect(Box::new(process(n))))
}

inventory::submit! { EffectPattern { name: "processors", priority: 60, parse: process_effect } }
inventory::submit! { CostPattern { name: "processors", priority: 60, parse: process_cost } }
