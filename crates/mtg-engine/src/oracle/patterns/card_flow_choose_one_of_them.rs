//! "Exile the top three cards of your library. Choose one of them. You may play that card
//! this turn." (Case of the Burning Masks, Jaya, Fiery Negotiator; "Until the end of your
//! next turn, you may play that card.", Riverwheel Sweep, Mishra's Research Desk, Strongbox
//! Raider): after the exile, "choose one of them" chooses one of the exiled cards, which
//! "that card" then refers to; only it may be played.

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::FollowupPattern;
use crate::oracle::phrases::*;

/// The variable iterating over the chosen card (it's stored in `vars::IT`).
const CHOICE: Var = vars::USER + 1771;

/// Whether the effect ends by exiling the top cards of a library.
fn exiles_top_cards(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(..),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(exiles_top_cards),
        Effect::If {
            then, otherwise, ..
        } => exiles_top_cards(then) && exiles_top_cards(otherwise),
        _ => false,
    }
}

/// Whether the effect ends by choosing one of the exiled cards (see [`choose_one`]).
fn ends_with_choice(e: &Effect) -> bool {
    match e {
        Effect::ForEach { var, .. } => *var == CHOICE,
        Effect::Seq(v) => v.last().is_some_and(ends_with_choice),
        _ => false,
    }
}

/// "choose one of them" after exiling the top cards of a library.
fn choose_one(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "choose one of them" || !exiles_top_cards(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::ForEach {
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::In(Box::new(Sel::Var(vars::IT))),
                count: Value::c(1),
                up_to: false,
                store: Some(vars::IT),
            },
            var: CHOICE,
            effect: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "card_flow: choose one of the exiled cards", priority: 80, apply: choose_one } }

/// "you may play that card this turn", "until the end of your next turn, you may play that
/// card", "until end of turn, you may play that card" after choosing one of the exiled
/// cards.
fn may_play_chosen(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let (duration, r) = if let Some(r) = l.strip_prefix("until the end of your next turn, ") {
        (Some(Duration::UntilEndOfYourNextTurn), r)
    } else if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Some(Duration::EndOfTurn), r)
    } else {
        (None, l)
    };
    let Some(r) = r.strip_prefix("you may play that card") else {
        return false;
    };
    let duration = match (duration, end(r)) {
        (Some(d), "") => d,
        (None, "this turn") => Duration::EndOfTurn,
        (None, "until the end of your next turn") => Duration::UntilEndOfYourNextTurn,
        _ => return false,
    };
    if !ends_with_choice(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            duration,
            free: false,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "card_flow: you may play the chosen exiled card", priority: 80, apply: may_play_chosen } }
