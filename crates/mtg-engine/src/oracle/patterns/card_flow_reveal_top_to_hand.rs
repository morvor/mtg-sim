//! "Reveal the top card of your library and put that card into your hand. You lose life
//! equal to its mana value." (Dark Confidant, Dark Tutelage, Caustic Bronco): the revealed
//! card goes to your hand, and it's what "it"/"that card" refers to afterward.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn reveal_top_to_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal the top card of your library and put ")?;
    if !matches!(
        r,
        "it into your hand" | "that card into your hand" | "the revealed card into your hand"
    ) {
        return None;
    }
    let mut rest_to = Destination::library_top();
    rest_to.position = LibraryPosition::FromTop(0);
    b.it = Sel::Var(vars::IT);
    Some(Effect::Dig {
        who: PlayerRef::You,
        n: Value::c(1),
        reveal: true,
        filter: Filter::Any,
        take: Value::c(1),
        take_up_to: false,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to,
    })
}

inventory::submit! { EffectPattern { name: "card_flow: reveal the top card and put it into your hand", priority: 80, parse: reveal_top_to_hand } }
