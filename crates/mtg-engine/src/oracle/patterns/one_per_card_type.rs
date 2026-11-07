//! "Reveal the top ten cards of your library. For each card type, you may put a card of
//! that type from among the revealed cards into your hand. Put the rest on the bottom of
//! your library in a random order." (Atraxa, Grand Unifier): any number of the revealed
//! cards are taken, each for a different card type it has — a card with several card
//! types counts as any one of them (`TargetGroup::OnePerCardType`, chosen together, see
//! `target_groups::choose_together`).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn one_card_per_card_type(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    if !matches!(
        l,
        "for each card type, you may put a card of that type from among the revealed cards into your hand"
            | "for each card type, you may put a card of that type from among them into your hand"
    ) {
        return false;
    }
    let Effect::Dig {
        who: PlayerRef::You,
        reveal: true,
        filter,
        take,
        take_up_to,
        take_to,
        rest_to,
        ..
    } = prev
    else {
        return false;
    };
    if !matches!(take, Value::Const(0))
        || !super::card_flow_dig::is_in_place(rest_to)
    {
        return false;
    }
    *filter = Filter::Together(TargetGroup::OnePerCardType);
    // At most one card for each card type there is; the relationship limits it further.
    *take = Value::c(crate::types::CardType::ALL.len() as i32);
    *take_up_to = true;
    *take_to = Destination::zone(ZoneKind::Hand);
    true
}

inventory::submit! { FollowupPattern { name: "for each card type, you may put a card of that type from among the revealed cards into your hand", priority: 80, apply: one_card_per_card_type } }
