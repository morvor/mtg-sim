//! "You may look at that player's hand. If you do, you may cast a spell from among those
//! cards without paying its mana cost." (Mindleech Mass): after looking at a player's
//! hand, one nonland card from it may be cast as the ability resolves (CR 608.2g), for
//! the alternative cost of nothing (CR 118.9); the player casting it controls the spell.
//! "If you do": only if the player looked at the hand.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The player whose hand the effect ends by looking at.
fn looked_at_hand(e: &Effect) -> Option<PlayerRel> {
    match e {
        Effect::LookAtHand { who } => match who {
            PlayerRef::TriggerPlayer => Some(PlayerRel::TriggerPlayer),
            PlayerRef::Target(n) => Some(PlayerRel::Target(*n)),
            _ => None,
        },
        Effect::May { effect, .. } => looked_at_hand(effect),
        Effect::Seq(v) => v.last().and_then(looked_at_hand),
        _ => None,
    }
}

fn cast_from_looked_at_hand(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l.trim());
    let (if_you_do, l) = match l.strip_prefix("if you do, ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    if l != "you may cast a spell from among those cards without paying its mana cost" {
        return false;
    }
    let Some(whose) = looked_at_hand(prev) else {
        return false;
    };
    let cast = Effect::CastCard {
        who: PlayerRef::You,
        // Choosing none is not casting one ("you may"); a land can't be cast.
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(whose),
                Filter::not(Filter::Type(crate::types::CardType::Land)),
            ]),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    };
    let cast = if if_you_do {
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(cast),
            otherwise: Box::new(Effect::Noop),
        }
    } else {
        cast
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, cast]);
    true
}

inventory::submit! { FollowupPattern { name: "r608 you may cast a spell from among the cards in the hand looked at without paying its mana cost", priority: 100, apply: cast_from_looked_at_hand } }
