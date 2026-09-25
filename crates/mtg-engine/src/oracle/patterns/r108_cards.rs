//! Cards from outside the game (CR 108.3b, 400.11b): "reveal a sorcery card you own from
//! outside the game and put it into your hand" (wishes), "put a card you own from outside
//! the game into your hand".

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;

/// "reveal a [filter] card you own from outside the game and put it into your hand",
/// "put a card you own from outside the game into your hand",
/// "choose a [filter] card you own from outside the game and put it into your hand".
fn wish(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (verb, r) = l.split_once(' ')?;
    if !matches!(verb, "reveal" | "put" | "choose") {
        return None;
    }
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (filter, plural, rest) = parse_object_phrase(r)?;
    if plural {
        return None;
    }
    let rest = rest.trim().strip_prefix("from outside the game")?.trim();
    let ok = match verb {
        "put" => rest == "into your hand",
        _ => rest == "and put it into your hand" || rest == "and put that card into your hand",
    };
    if !ok {
        return None;
    }
    // The card must be one the player owns outside the game (their sideboard, CR 108.3b).
    let filter = Filter::and(vec![
        filter,
        Filter::Card,
        Filter::InZone(ZoneKind::Outside),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "r108 wish from outside the game", priority: 100, parse: wish } }
