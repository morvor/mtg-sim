//! Optional loops over the cards looked at (cards6-per-player-results, CR 608.2c):
//!
//! - "Look at the top five cards of your library. As many times as you choose, you may pay
//!   1 life, put those cards on the bottom of your library in any order, then look at the
//!   top five cards of your library." (Lim-Dûl's Vault): after the look, the player
//!   chooses whether to perform the process (`repeat_process.rs`); each pass begins with
//!   its cost (paying life: only with that much life, CR 119.4) and goes on with
//!   its instructions in order, "those cards" being the cards looked at last. Without a
//!   cost ("As many times as you choose, you may put those cards ...") each pass is
//!   optional the same way.
//! - "Then shuffle and put the last cards you looked at this way on top in any order.":
//!   the cards looked at last stay out of the shuffle and go on top (CR 701.24b): the
//!   library is shuffled, then those cards (still in it, CR 400.7) are put on top in the
//!   order the player chooses (CR 401.4). After a plain look, "those cards" are the cards
//!   looked at.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::costs::parse_cost;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// The cards looked at last by a look and the passes of a loop after it.
pub const LOOKED_AT_LAST: Var = vars::USER + 6601;

/// The look at the top of your own library that leaves the cards where they are, at the
/// end of `e`.
fn ends_with_look(e: &Effect) -> bool {
    match e {
        Effect::Dig {
            who: PlayerRef::You,
            reveal: false,
            take: Value::Const(0),
            rest_to,
            ..
        } => super::card_flow_dig::is_in_place(rest_to),
        Effect::Seq(v) => v.last().is_some_and(ends_with_look),
        _ => false,
    }
}

/// Whether `e` keeps the cards looked at last ([`LOOKED_AT_LAST`]).
fn keeps_looked(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|j| j.contains(&format!("\"var\":{LOOKED_AT_LAST},")))
}

/// The look, followed by keeping the cards it looked at.
fn look_and_keep(look: Effect) -> Effect {
    Effect::seq(vec![
        look,
        Effect::Store {
            var: LOOKED_AT_LAST,
            sel: Sel::Var(vars::DUG),
        },
    ])
}

/// One instruction of a pass: putting the cards looked at last somewhere ("put those
/// cards on the bottom of your library in any order"), or a new look at the top of your
/// library (whose cards are kept).
fn pass_step(s: &str, b: &mut Builder) -> Option<Effect> {
    for those in ["put those cards ", "put them "] {
        if let Some(r) = s.strip_prefix(those) {
            let to = super::card_flow_dig::rest_destination(r, false)?;
            if to.zone != ZoneKind::Library {
                return None;
            }
            return Some(Effect::DigStep(Box::new(DigStep::Rest {
                from: Sel::Var(LOOKED_AT_LAST),
                to,
            })));
        }
    }
    if s.starts_with("look at the top ") {
        let e = parse_sentence(s, b)?;
        return ends_with_look(&e).then(|| look_and_keep(e));
    }
    None
}

/// "as many times as you choose, you may [pay N life, ][instruction], then [instruction]"
/// after a look at the top of your library.
fn as_many_times_over_looked(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("as many times as you choose, you may ") else {
        return false;
    };
    if !ends_with_look(prev) {
        return false;
    }
    // The cost each pass begins with, if any.
    let (cost, r) = match r.split_once(", ") {
        Some((c, rest)) if c.starts_with("pay ") => match parse_cost(c) {
            Some((cost, false)) if cost.mana.is_none() => (Some(cost), rest),
            _ => return false,
        },
        _ => (None, r),
    };
    // The instructions, in order: "A, B, then C" / "A, then B".
    let Some((head, last)) = r.rsplit_once(", then ") else {
        return false;
    };
    let mut steps = Vec::new();
    for s in head.split(", ").chain(std::iter::once(last)) {
        let Some(e) = pass_step(s, b) else {
            return false;
        };
        steps.push(e);
    }
    // A pass must look again (else it would repeat over the same cards it moved away).
    if !steps.iter().any(keeps_looked) || keeps_looked(&steps[0]) {
        return false;
    }
    steps.push(Effect::RepeatThisProcess);
    let body = Effect::seq(steps);
    let pass = match cost {
        Some(cost) => Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(body),
            otherwise: Box::new(Effect::Noop),
        },
        None => Effect::May {
            who: PlayerRef::You,
            effect: Box::new(body),
        },
    };
    let look = std::mem::take(prev);
    *prev = Effect::seq(vec![
        look_and_keep(look),
        Effect::RepeatProcess {
            body: Box::new(pass),
        },
    ]);
    b.it = Sel::Var(LOOKED_AT_LAST);
    true
}

inventory::submit! { FollowupPattern { name: "loops 6: as many times as you choose, you may [pay N life,] [instructions over the cards looked at]", priority: 55, apply: as_many_times_over_looked } }

/// "Then shuffle and put the last cards you looked at this way on top in any order.",
/// "Then shuffle and put those cards on top in any order." after a look.
fn shuffle_and_put_looked_on_top(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l.strip_prefix("shuffle and put ") else {
        return false;
    };
    let Some(which) = r
        .strip_suffix(" on top in any order")
        .or_else(|| r.strip_suffix(" on top of your library in any order"))
    else {
        return false;
    };
    let looped = keeps_looked(prev);
    match which {
        "the last cards you looked at this way" if looped => {}
        "those cards" | "them" if !looped && ends_with_look(prev) => {
            let look = std::mem::take(prev);
            *prev = look_and_keep(look);
        }
        _ => return false,
    }
    let before = std::mem::take(prev);
    *prev = Effect::seq(vec![
        before,
        Effect::Shuffle {
            who: PlayerRef::You,
        },
        Effect::DigStep(Box::new(DigStep::Rest {
            from: Sel::Var(LOOKED_AT_LAST),
            to: Destination::library_top(),
        })),
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "loops 6: then shuffle and put the last cards you looked at this way on top in any order", priority: 55, apply: shuffle_and_put_looked_on_top } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn loops_over_the_cards_looked_at() {
        for text in [
            "Look at the top five cards of your library. As many times as you choose, you may pay 1 life, put those cards on the bottom of your library in any order, then look at the top five cards of your library. Then shuffle and put the last cards you looked at this way on top in any order.",
            "Look at the top five cards of your library. As many times as you choose, you may put those cards on the bottom of your library in any order, then look at the top five cards of your library. Then shuffle and put the last cards you looked at this way on top in any order.",
            "Look at the top three cards of your library. As many times as you choose, you may pay 2 life, put them on the bottom of your library in a random order, then look at the top three cards of your library.",
            "Look at the top five cards of your library. Then shuffle and put those cards on top in any order.",
        ] {
            assert!(compiled("Instant", text).is_some(), "{text}");
        }
        for text in [
            // Nothing new to look at: not a loop over the cards looked at.
            "Look at the top five cards of your library. As many times as you choose, you may pay 1 life, put those cards on the bottom of your library in any order, then draw a card.",
            // "The last cards you looked at" needs a loop.
            "Look at the top five cards of your library. Then shuffle and put the last cards you looked at this way on top in any order.",
        ] {
            assert!(compiled("Instant", text).is_none(), "{text}");
        }
    }
}
