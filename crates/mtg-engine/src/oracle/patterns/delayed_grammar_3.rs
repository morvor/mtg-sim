//! Delayed triggered abilities, grammar III (cards3-triggers-durations).
//!
//! The cards a player exiled, returned later by that player: "Target player exiles all
//! cards from their hand face down. At the beginning of the end step of that player's next
//! turn, that player returns those cards to their hand." (Suppress; CR 603.7a, 603.7c: the
//! delayed ability refers to the cards the first instruction exiled).
//!
//! A card from the graveyard of the trigger object's owner: "Choose target creature. When
//! that creature dies this turn, return a creature card from its owner's graveyard to the
//! battlefield under the control of that creature's owner." (Reincarnation: the creature
//! card is chosen as the delayed ability resolves, by its controller; it may be the
//! creature that died).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether the effect ends with an exile instruction (possibly performed by another
/// player).
fn ends_with_exile(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile),
        Effect::AsPlayer { effect, .. } => ends_with_exile(effect),
        Effect::Exile { .. } => true,
        _ => false,
    }
}

fn those_exiled_cards_returned_later(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_exile(prev) {
        return false;
    }
    let s = end(s);
    let Some((delay, instr)) = s.split_once(", ") else {
        return false;
    };
    if !delay.starts_with("at the beginning of ")
        || !matches!(
            instr,
            "that player returns those cards to their hand" | "they return those cards to their hand"
        )
    {
        return false;
    }
    // "Those cards": what the exile instruction exiled (its "it", CR 603.7c).
    let saved = b.it.clone();
    b.it = Sel::Var(vars::IT);
    let e = super::delayed_grammar::delayed_instruction(
        &format!("{delay}, return those cards to their owner's hand"),
        b,
    );
    b.it = saved;
    let Some(e) = e else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar 3: that player returns those (exiled) cards to their hand later", priority: 100, apply: those_exiled_cards_returned_later } }

/// Whether the instruction is the one [`return_card_from_its_owners_graveyard`] reads.
pub(crate) fn reads_from_its_owners_graveyard(eff: &str) -> bool {
    end(eff).starts_with("return a creature card from its owner's graveyard to the battlefield")
}

fn return_card_from_its_owners_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("return a creature card from its owner's graveyard to the battlefield")?;
    if !b.in_trigger || !matches!(b.it, Sel::TriggerObject | Sel::TriggerLki) {
        return None;
    }
    // "its owner" / "that creature's owner": the trigger object's.
    let owner = PlayerRef::OwnerOf(Box::new(Sel::TriggerLki));
    let controller = match r {
        " under the control of that creature's owner" | " under its owner's control" => {
            owner.clone()
        }
        " under your control" => PlayerRef::You,
        _ => return None,
    };
    let mut to = Destination::battlefield();
    to.controller = Some(controller);
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::creature(),
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedByPlayer(Box::new(owner)),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to,
    })
}

inventory::submit! { EffectPattern { name: "delayed grammar 3: return a creature card from its owner's graveyard", priority: 100, parse: return_card_from_its_owners_graveyard } }
