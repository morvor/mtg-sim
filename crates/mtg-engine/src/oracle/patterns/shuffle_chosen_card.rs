//! "You choose a card from it. That player shuffles that card into their library."
//! (Perish the Thought): if no card could be chosen (the hand was empty), there's no card
//! to shuffle and no shuffle happens (unlike shuffling a set of objects that turns out to
//! be empty, CR 701.24d).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// The variable holding the single object an earlier "choose a [card]" stored, if the
/// effect ends with such a choice.
fn chosen_one(e: &Effect) -> Option<Var> {
    match e {
        Effect::Store {
            var,
            sel:
                Sel::Choose {
                    count: Value::Const(1),
                    up_to: false,
                    ..
                },
        } => Some(*var),
        Effect::Seq(v) => v.last().and_then(chosen_one),
        _ => None,
    }
}

fn shuffle_the_chosen_card(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !(l.contains("shuffles that card into") || l.contains("shuffle that card into")) {
        return false;
    }
    let Some(var) = chosen_one(prev) else {
        return false;
    };
    let Some(e) = parse_sentence(l, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::SelNonEmpty(Sel::Var(var)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "shuffle the chosen card (if one was chosen)", priority: 60, apply: shuffle_the_chosen_card } }
