//! "Put a card from your hand on top of your library." / "put two cards from your hand on
//! top of your library in any order" (Brainstorm, Jace, the Mind Sculptor, Cavalier of
//! Gales, Conch Horn): the player chooses that many cards from their hand (all of them if
//! they have fewer) and puts them on top of their library in the order they choose
//! (CR 401.4).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};

fn put_from_hand_on_top(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    let r = r
        .strip_suffix(" in any order")
        .unwrap_or(r)
        .strip_suffix(" from your hand on top of your library")?;
    let count = match r {
        "a card" => Value::c(1),
        _ => {
            let (n, rest) = parse_number(r)?;
            n.as_const()?;
            if end(rest).trim() != "cards" {
                return None;
            }
            n
        }
    };
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::You),
            ]),
            count,
            up_to: false,
            store: None,
        },
        to: Destination::library_top(),
    })
}

inventory::submit! { EffectPattern { name: "r401 put cards from your hand on top of your library", priority: 100, parse: put_from_hand_on_top } }
