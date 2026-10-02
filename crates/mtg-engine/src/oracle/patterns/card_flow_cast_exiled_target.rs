//! "Exile target noncreature, nonland card from your graveyard. Until the end of your next
//! turn, you may cast that card." (Practiced Scrollsmith), "... You may cast it this
//! turn.": a permission to cast the exiled card (a new object, CR 400.7) for a while,
//! paying its costs and following the normal timing rules (CR 601.2, 307.1). Only a card
//! that isn't a land can be cast (CR 305.9): an exiled land card gets no permission.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// Whether the effect ends by exiling the targeted cards face up.
fn ends_with_exiling_targets(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::Target(_),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exiling_targets),
        _ => false,
    }
}

/// The duration of "[until ...,] you may cast that card [this turn | until ...]".
fn cast_duration(l: &str) -> Option<Duration> {
    let (lead, r) = if let Some(r) = l.strip_prefix("until the end of your next turn, ") {
        (Some(Duration::UntilEndOfYourNextTurn), r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Some(Duration::EndOfTurn), r)
    } else {
        (None, l)
    };
    let r = r.strip_prefix("you may cast ")?;
    let r = ["that card", "it", "the exiled card"]
        .iter()
        .find_map(|p| {
            r.strip_prefix(p)
                .filter(|x| x.is_empty() || x.starts_with(' '))
        })?;
    match (lead, r.trim()) {
        (Some(d), "") => Some(d),
        (None, "this turn") => Some(Duration::EndOfTurn),
        (None, "until the end of your next turn") => Some(Duration::UntilEndOfYourNextTurn),
        _ => None,
    }
}

fn may_cast_exiled_target(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(duration) = cast_duration(end(l)) else {
        return false;
    };
    if !ends_with_exiling_targets(prev) {
        return false;
    }
    let card = Sel::Var(CARD);
    let each = Effect::ForEach {
        sel: Sel::Var(vars::IT),
        var: CARD,
        effect: Box::new(Effect::If {
            cond: Condition::SelMatches(
                card.clone(),
                Filter::Not(Box::new(Filter::Type(CardType::Land))),
            ),
            // A permission to cast it, not to play a land (CR 305.9).
            then: Box::new(
                Effect::GrantPlayPermission {
                    who: PlayerRef::You,
                    what: card,
                    duration,
                    free: false,
                }
                .cast_only(),
            ),
            otherwise: Box::new(Effect::Noop),
        }),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), each]);
    true
}

/// The variable iterating over the exiled cards.
const CARD: Var = vars::USER + 3131;

inventory::submit! { FollowupPattern { name: "card_flow: exile target card; you may cast that card for a while", priority: 80, apply: may_cast_exiled_target } }
