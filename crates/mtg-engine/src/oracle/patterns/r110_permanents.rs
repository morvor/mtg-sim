//! Putting objects onto the battlefield (CR 110.2a): "each player puts a creature card
//! from their graveyard onto the battlefield" (Exhume). Each player is instructed to put
//! a card onto the battlefield, so it enters under that player's control.

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;

/// "each player puts a [filter] card from their graveyard/hand onto the battlefield".
fn each_player_puts(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player puts ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (filter, plural, rest) = parse_object_phrase(r)?;
    if plural {
        return None;
    }
    let rest = rest.trim();
    let (zone, rest) = if let Some(x) = rest.strip_prefix("from their graveyard ") {
        (ZoneKind::Graveyard, x)
    } else if let Some(x) = rest.strip_prefix("from their hand ") {
        (ZoneKind::Hand, x)
    } else {
        return None;
    };
    if end(rest) != "onto the battlefield" {
        return None;
    }
    let mut to = Destination::battlefield();
    to.controller = Some(PlayerRef::Iterated);
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::Move {
            what: Sel::Choose {
                chooser: PlayerRef::Iterated,
                filter: Filter::and(vec![
                    filter,
                    Filter::Card,
                    Filter::InZone(zone),
                    Filter::OwnedBy(PlayerRel::Iterated),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            to,
        }),
    })
}

inventory::submit! { EffectPattern { name: "r110 each player puts onto the battlefield", priority: 100, parse: each_player_puts } }
