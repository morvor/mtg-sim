//! "Put it into your hand" / "put that card into your hand" about a card an earlier
//! instruction named (looked at, revealed, exiled, chosen in a graveyard): it goes to its
//! owner's hand (a card can only be put into its owner's hand, CR 400.3), as in "If you
//! don't, put it into your hand." (Reason) or "Otherwise, put that card into your hand."
//! (Matter Reshaper).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::patterns::oracle_hardening_referents::is_no_referent;
use crate::oracle::phrases::end;

fn put_it_into_your_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("put ")?
        .strip_suffix(" into your hand")?;
    if !matches!(r, "it" | "that card" | "the card" | "the exiled card") {
        return None;
    }
    let r = if r == "the card" || r == "the exiled card" {
        "that card"
    } else {
        r
    };
    let before = b.targets.len();
    let (what, rest) = object_ref(r, b)?;
    // Only an object an earlier instruction named, not the source.
    if b.targets.len() != before
        || !rest.trim().is_empty()
        || is_no_referent(&what)
        || matches!(what, Sel::This | Sel::None)
    {
        b.targets.truncate(before);
        return None;
    }
    Some(Effect::Move {
        what,
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "put it into your hand", priority: 300, parse: put_it_into_your_hand } }
