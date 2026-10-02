//! "Exile that card. [If the gift was promised,] you may cast that card for as long as it
//! remains exiled, and mana of any type can be spent to cast it." (Cruelclaw's Heist): a
//! permission to cast the exiled card (a new object, CR 400.7) that lasts while it stays
//! in exile, and mana of any type can be spent to cast it (CR 118.14; see
//! `conditional_followups.rs` for the condition). Only a card that isn't a land can be
//! cast (CR 305.9): an exiled land card gets no permission.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// Whether the effect ends by exiling cards face up.
fn ends_with_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            face_down: false, ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile),
        _ => false,
    }
}

fn may_cast_while_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(rest) = end(l)
        .strip_prefix("you may cast that card for as long as it remains exiled")
        .or_else(|| end(l).strip_prefix("you may cast it for as long as it remains exiled"))
    else {
        return false;
    };
    let any_type = match rest {
        "" => false,
        ", and mana of any type can be spent to cast it"
        | ", and mana of any type can be spent to cast that spell" => true,
        _ => return false,
    };
    if !ends_with_exile(prev) {
        return false;
    }
    // Each exiled card that isn't a land: a land card is played, never cast (CR 305.9).
    let card = Sel::Var(CARD);
    let mut grant = vec![Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: card.clone(),
        // The permission is for that object: it ends when the card leaves exile.
        duration: Duration::Permanent,
        free: false,
    }
    // A permission to cast it, not to play a land (CR 305.9).
    .cast_only()];
    if any_type {
        grant.push(Effect::SpendAnyTypeMana {
            who: PlayerRef::You,
            what: card.clone(),
            duration: Duration::Permanent,
        });
    }
    let each = Effect::ForEach {
        sel: Sel::Var(vars::IT),
        var: CARD,
        effect: Box::new(Effect::If {
            cond: Condition::SelMatches(card, Filter::Not(Box::new(Filter::Type(CardType::Land)))),
            then: Box::new(Effect::seq(grant)),
            otherwise: Box::new(Effect::Noop),
        }),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), each]);
    true
}

/// The variable iterating over the exiled cards.
const CARD: Var = vars::USER + 1772;

inventory::submit! { FollowupPattern { name: "card_flow: you may cast that card for as long as it remains exiled", priority: 80, apply: may_cast_while_exiled } }
