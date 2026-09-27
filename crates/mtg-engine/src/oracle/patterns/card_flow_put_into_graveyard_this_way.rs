//! "Destroy all creatures. [If the gift was promised,] return a creature card put into
//! your graveyard this way to the battlefield under your control." (Starfall Invocation):
//! after a destroy effect, a card that effect put into your graveyard (the new object in
//! the graveyard, CR 400.7), chosen as the effect happens.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether the effect ends by destroying permanents (which stores the cards they became
/// in `vars::IT`).
fn ends_with_destroy(e: &Effect) -> bool {
    match e {
        Effect::Destroy { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_destroy),
        _ => false,
    }
}

fn return_card_put_into_graveyard(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("return a ") else {
        return false;
    };
    let Some((noun, dest)) = r.split_once(" put into your graveyard this way ") else {
        return false;
    };
    let to = match dest {
        "to the battlefield under your control" => Destination::battlefield().under_your_control(),
        "to the battlefield" => Destination::battlefield().under_your_control(),
        "to your hand" => Destination::zone(ZoneKind::Hand),
        _ => return false,
    };
    let Some((f, false, tail)) = parse_object_phrase(noun) else {
        return false;
    };
    if !end(tail).is_empty() || !ends_with_destroy(prev) {
        return false;
    }
    let chosen = Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::and(vec![
            f,
            Filter::Card,
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
            Filter::In(Box::new(Sel::Var(vars::IT))),
        ]),
        count: Value::c(1),
        up_to: false,
        store: None,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, Effect::Move { what: chosen, to }]);
    true
}

inventory::submit! { FollowupPattern { name: "card_flow: return a card put into your graveyard this way", priority: 80, apply: return_card_put_into_graveyard } }
