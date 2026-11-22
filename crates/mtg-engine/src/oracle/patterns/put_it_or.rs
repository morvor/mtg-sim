//! A choice between two places for one card: "Put that card onto the battlefield or into
//! your hand." (Illuna, Apex of Wishes; Ketria) after a card was found ("that card" is the
//! card the previous instruction found). The controller chooses as the instruction is
//! followed (CR 608.2d). Searches word it inside the search ("put it into your hand or
//! graveyard", `search_grammar.rs`).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::end;

inventory::submit! {
    EffectPattern { name: "put that card [destination] or [destination]", priority: 90, parse: put_it_or }
}

/// One of the places: "onto the battlefield [tapped]", "into your hand", "into your
/// graveyard".
fn place(s: &str) -> Option<(&'static str, Destination)> {
    Some(match s {
        "onto the battlefield" => (
            "Put onto the battlefield",
            Destination::battlefield().under_your_control(),
        ),
        "onto the battlefield tapped" => (
            "Put onto the battlefield tapped",
            Destination::battlefield().under_your_control().tapped(),
        ),
        "into your hand" => ("Put into your hand", Destination::zone(ZoneKind::Hand)),
        "into your graveyard" => (
            "Put into your graveyard",
            Destination::zone(ZoneKind::Graveyard),
        ),
        _ => return None,
    })
}

/// "put that card onto the battlefield or into your hand".
fn put_it_or(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("put that card ")
        .or_else(|| end(l).strip_prefix("put it "))?;
    // Only a card an earlier instruction found.
    if !matches!(b.it, Sel::Var(_)) {
        return None;
    }
    let (a, c) = r.split_once(" or ")?;
    let options = [place(a)?, place(c)?]
        .into_iter()
        .map(|(label, to)| {
            (
                label.to_string(),
                Effect::Move {
                    what: b.it.clone(),
                    to,
                },
            )
        })
        .collect();
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options,
    })
}
