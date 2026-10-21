//! Reveal-until variants (CR 701.20a) beyond `dig_grammar.rs`'s sources: what happens if
//! the card looked for was found ("If an artifact card is revealed this way, put it onto
//! the battlefield under your control and sacrifice ~.", Thought Dissector) — a reveal can
//! stop without finding it (the library runs out, or a cap is reached: "until an artifact
//! card or X cards are revealed, whichever comes first").

use super::dig_grammar::{dig_card_filter, ends_with_until};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::patterns::FollowupPattern;
use crate::oracle::phrases::end;

/// "If a [card] is revealed this way, [effect]" / "If you reveal a [card] this way,
/// [effect]" after revealing cards until such a card is revealed: whether one was found
/// ([`vars::DUG_FOUND`]); "it" is that card.
fn if_found_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some(r) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) else {
        return false;
    };
    let Some((kind, rest)) = r.split_once(" is revealed this way, ") else {
        return false;
    };
    if !kind.ends_with(" card") || !ends_with_until(prev) {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone());
    let Some(f) = dig_card_filter(kind, b) else {
        return false;
    };
    b.it = Sel::Var(vars::DUG_FOUND);
    let Some(then) = parse_clause(rest, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    let found = Condition::Exists(Filter::and(vec![
        Filter::In(Box::new(Sel::Var(vars::DUG_FOUND))),
        f,
    ]));
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: found,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "reveal until: if a [card] is revealed this way, [effect]", priority: 95, apply: if_found_this_way } }

/// "If it is, [effect]" after "... until a card with that name is revealed" (Tunnel
/// Vision): whether the card looked for was found (the library can run out first).
fn if_it_is(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("if it is, ") else {
        return false;
    };
    if !ends_with_until(prev) {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::Var(vars::DUG_FOUND);
    let Some(then) = parse_clause(rest, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    let found = Condition::Compare(
        Value::CountSel(Box::new(Sel::Var(vars::DUG_FOUND))),
        Cmp::Ge,
        Value::c(1),
    );
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: found,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "reveal until: if it is, [effect]", priority: 95, apply: if_it_is } }
