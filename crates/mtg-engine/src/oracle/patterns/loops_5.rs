//! Loops a player repeats (cards5-choices-players, CR 608.2c):
//!
//! - "As many times as you choose, you may exile the top card of your library." (Dance
//!   with Calamity): the instruction, then the choice to perform it again, until the
//!   player stops (`repeat_process.rs`; each card is seen before the next choice, its
//!   ruling). The cards the passes exiled are kept: "the cards exiled this way", "those
//!   cards".
//! - "If the total mana value of the cards exiled this way is 13 or less, [instruction]":
//!   the sum of their mana values (CR 202.3).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;

/// The cards the passes of an "as many times as you choose" exile instruction exiled.
pub const EXILED_BY_PASSES: Var = vars::USER + 9551;

/// "as many times as you choose, you may exile the top card of your library".
fn as_many_times_as_you_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("as many times as you choose, you may ")?;
    // Only an exile of the top card of the library (the cards it exiled are kept).
    if !r.starts_with("exile the top card of ") {
        return None;
    }
    let pass = parse_sentence(r, b)?;
    if !matches!(pass, Effect::Exile { what: Sel::TopOfLibrary(..), .. }) {
        return None;
    }
    let body = Effect::May {
        who: PlayerRef::You,
        effect: Box::new(Effect::seq(vec![
            pass,
            Effect::Store {
                var: EXILED_BY_PASSES,
                sel: Sel::Union(vec![Sel::Var(EXILED_BY_PASSES), Sel::Var(vars::DUG)]),
            },
            Effect::RepeatThisProcess,
        ])),
    };
    for n in ["the cards exiled this way", "those cards", "cards exiled this way"] {
        b.named.push((n.into(), Sel::Var(EXILED_BY_PASSES)));
    }
    b.it = Sel::Var(EXILED_BY_PASSES);
    Some(Effect::seq(vec![
        Effect::Store {
            var: EXILED_BY_PASSES,
            sel: Sel::Union(vec![]),
        },
        Effect::RepeatProcess {
            body: Box::new(body),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "as many times as you choose, you may exile the top card of your library", priority: 90, parse: as_many_times_as_you_choose } }

/// "If the total mana value of the cards exiled this way is N or less, [instruction]."
/// after an "as many times as you choose" exile.
fn if_total_mana_value_exiled(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("if the total mana value of the cards exiled this way is ")?;
    let (cond, rest) = r.split_once(", ")?;
    let n = cond.strip_suffix(" or less")?;
    let (n, tail) = parse_number(n)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let exiled = b
        .named
        .iter()
        .rev()
        .find(|(k, _)| k == "the cards exiled this way")
        .map(|(_, s)| s.clone())?;
    if !matches!(exiled, Sel::Var(EXILED_BY_PASSES)) {
        return None;
    }
    b.it = exiled.clone();
    // "you may cast any number of spells from among those cards without paying their mana
    // costs": each of them may be cast, in the order chosen, as this resolves (Dance with
    // Calamity's rulings); a land card can't be (it's not a spell).
    let then = if matches!(
        rest,
        "you may cast any number of spells from among those cards without paying their mana costs"
            | "you may cast any number of spells from among them without paying their mana costs"
    ) {
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Matching(
                Box::new(exiled.clone()),
                Filter::not(Filter::Type(crate::types::CardType::Land)),
            ),
            free: true,
            optional: true,
        }
    } else {
        parse_sentence(rest, b)?
    };
    Some(Effect::If {
        cond: Condition::Compare(Value::ManaValueOf(Box::new(exiled)), Cmp::Le, n),
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if the total mana value of the cards exiled this way is N or less, ...", priority: 90, parse: if_total_mana_value_exiled } }

/// Set (to 1) when the put of a "you may put that card into your hand unless ..." pass
/// happened, for "until you put a card into your hand".
const PUT_INTO_HAND: Var = vars::USER + 9552;

/// "You may put that card into your hand unless it has the same name as another card
/// exiled this way." after "Exile the top card of your library." (Tainted Pact): the
/// cards exiled so far are kept (over the passes of a process, CR 608.2c).
fn put_unless_same_name(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(put) = l.strip_suffix(" unless it has the same name as another card exiled this way")
    else {
        return false;
    };
    if !matches!(prev, Effect::Exile { what: Sel::TopOfLibrary(..), .. }) {
        return false;
    }
    let Some(e) = parse_sentence(put, b) else {
        return false;
    };
    let Effect::May { who, effect } = e else {
        return false;
    };
    if !matches!(effect.as_ref(), Effect::Move { to, .. } if to.zone == ZoneKind::Hand) {
        return false;
    }
    let exile = std::mem::take(prev);
    *prev = Effect::seq(vec![
        exile,
        Effect::Store {
            var: EXILED_BY_PASSES,
            sel: Sel::Union(vec![Sel::Var(EXILED_BY_PASSES), Sel::Var(vars::DUG)]),
        },
        Effect::If {
            cond: Condition::Custom(
                crate::kw::choices_5_filters::SAME_NAME_AS_ANOTHER_EXILED.into(),
            ),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::May {
                who,
                effect: Box::new(Effect::seq(vec![
                    *effect,
                    Effect::If {
                        cond: Condition::PrevHappened,
                        then: Box::new(Effect::StoreValue {
                            var: PUT_INTO_HAND,
                            value: Value::c(1),
                        }),
                        otherwise: Box::new(Effect::Noop),
                    },
                ])),
            }),
        },
    ]);
    b.named
        .push(("the cards exiled this way".into(), Sel::Var(EXILED_BY_PASSES)));
    true
}

inventory::submit! { super::FollowupPattern { name: "you may put that card into your hand unless it has the same name as another card exiled this way", priority: 60, apply: put_unless_same_name } }

/// "Repeat this process until you put a card into your hand or you exile two cards with
/// the same name, whichever comes first." (Tainted Pact): the process again after each
/// pass that did neither.
fn repeat_until_put_or_same_name(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l)
        != "repeat this process until you put a card into your hand or you exile two cards with the same name, whichever comes first"
    {
        return false;
    }
    let Ok(j) = serde_json::to_string(&*prev) else {
        return false;
    };
    if !j.contains(crate::kw::choices_5_filters::SAME_NAME_AS_ANOTHER_EXILED) {
        return false;
    }
    let done = Condition::Or(vec![
        Condition::Compare(Value::Var(PUT_INTO_HAND), Cmp::Ge, Value::c(1)),
        Condition::Custom(crate::kw::choices_5_filters::TWO_EXILED_SHARE_A_NAME.into()),
    ]);
    let body = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: EXILED_BY_PASSES,
            sel: Sel::Union(vec![]),
        },
        Effect::StoreValue {
            var: PUT_INTO_HAND,
            value: Value::c(0),
        },
        Effect::RepeatProcess {
            body: Box::new(Effect::seq(vec![
                body,
                Effect::If {
                    cond: done,
                    then: Box::new(Effect::Noop),
                    otherwise: Box::new(Effect::RepeatThisProcess),
                },
            ])),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "repeat this process until you put a card into your hand or you exile two cards with the same name", priority: 60, apply: repeat_until_put_or_same_name } }
