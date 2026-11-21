//! Cards exiled with a permanent (CR 607.2a), checked and then put into graveyards:
//!
//! * "if there are cards exiled with ~" — an intervening-if condition (CR 603.4).
//! * "Whenever a player plays a land or casts a spell, if it shares a card type with the
//!   exiled card, ..." (Cemetery Gatekeeper, Cemetery Protector): the land or spell has a
//!   card type (CR 205.2a) that a card exiled with ~ has.
//! * "At the beginning of your end step, if there are cards exiled with ~, put them into
//!   their owner's graveyard, then ~ deals that much damage to each opponent." (Valakut
//!   Exploration): "them" are the cards exiled with ~, and "that much" is the number of
//!   cards put into a graveyard this way.
//!
//! Every ability of a permanent exiles cards "with" it (they're linked to its abilities,
//! see `Effect::Exile`), so the cards are those the permanent's abilities exiled that are
//! still in exile.

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The cards exiled with the source that are still in exile.
fn exiled_with_source() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

fn there_are_cards_exiled_with(c: &str) -> Option<Condition> {
    match end(c.trim()) {
        "there are cards exiled with ~" | "there are one or more cards exiled with ~" => {
            Some(Condition::Exists(exiled_with_source()))
        }
        _ => None,
    }
}

fn shares_a_card_type_with_the_exiled_card(c: &str) -> Option<Condition> {
    match end(c.trim()) {
        "it shares a card type with the exiled card"
        | "it shares a card type with a card exiled with ~" => Some(Condition::SelMatches(
            Sel::TriggerObject,
            Filter::SharesCardType(Box::new(Sel::All(exiled_with_source()))),
        )),
        _ => None,
    }
}

fn put_them_into_graveyards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (them, rest) = if let Some(r) = l.strip_prefix("put the cards exiled with ~ into ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("put them into ") {
        // "Them" with no other antecedent (no group was mentioned, and "it" is at most the
        // source): the cards the condition was about.
        let none = b.group.is_none()
            && (matches!(b.it, Sel::None | Sel::This)
                || super::oracle_hardening_referents::is_no_referent(&b.it));
        (none, r)
    } else {
        return None;
    };
    if !them {
        return None;
    }
    let rest = rest
        .strip_prefix("their owner's graveyard")
        .or_else(|| rest.strip_prefix("their owners' graveyards"))?;
    let put = Effect::Move {
        what: Sel::All(exiled_with_source()),
        to: Destination::zone(ZoneKind::Graveyard),
    };
    match rest {
        "" => Some(put),
        ", then ~ deals that much damage to each opponent" => Some(Effect::seq(vec![
            put,
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::CountSel(Box::new(Sel::Var(vars::IT))),
                to: Sel::Players(PlayerRef::EachOpponent),
            },
        ])),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "there are cards exiled with ~", priority: 100, parse: there_are_cards_exiled_with } }
inventory::submit! { ConditionPattern { name: "it shares a card type with the exiled card", priority: 100, parse: shares_a_card_type_with_the_exiled_card } }
inventory::submit! { EffectPattern { name: "put the cards exiled with ~ into their owners' graveyards", priority: 100, parse: put_them_into_graveyards } }
