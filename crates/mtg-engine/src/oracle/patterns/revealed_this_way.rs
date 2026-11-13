//! "Whenever you scry, you may reveal the top card of your library. If a land card is
//! revealed this way, put it onto the battlefield tapped." (Galadriel of Lothlórien),
//! "Scry 2, then you may reveal the top card of your library. If a creature card is
//! revealed this way, draw a card." (Elven Farsight): after revealing the top card of a
//! library (CR 701.20a), what happens if it's a card of the given kind. The revealed card
//! stays on top of the library unless the effect moves it ("it").

use super::card_flow_search::card_filter;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::patterns::FollowupPattern;
use crate::oracle::phrases::*;

/// Whether `e` ends by revealing the top card of a library and leaving it there.
fn ends_with_single_reveal(e: &Effect) -> bool {
    match e {
        Effect::Dig {
            n: Value::Const(1),
            reveal: true,
            take: Value::Const(0),
            rest_to,
            ..
        } => {
            rest_to.zone == ZoneKind::Library
                && matches!(rest_to.position, LibraryPosition::FromTop(0))
        }
        Effect::May { effect, .. } => ends_with_single_reveal(effect),
        Effect::Seq(v) => v.last().is_some_and(ends_with_single_reveal),
        _ => false,
    }
}

fn if_revealed_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some(r) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) else {
        return false;
    };
    let Some((kind, rest)) = r.split_once(" is revealed this way, ") else {
        return false;
    };
    if !kind.ends_with(" card") || !ends_with_single_reveal(prev) {
        return false;
    }
    let Some(f) = card_filter(kind, b) else {
        return false;
    };
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::Var(vars::IT);
    let Some(then) = parse_clause(rest, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        // Nothing is revealed if the player chooses not to reveal.
        Effect::Store {
            var: vars::IT,
            sel: Sel::None,
        },
        old,
        Effect::If {
            cond: Condition::SelMatches(Sel::Var(vars::IT), f),
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if a [card] is revealed this way, [effect]", priority: 90, apply: if_revealed_this_way } }
