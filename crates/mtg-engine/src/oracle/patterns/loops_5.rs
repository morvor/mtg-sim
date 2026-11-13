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

/// The cards "that player" and you exiled from the revealed hand (Struggle for Sanity),
/// over all the passes, and each pass's pick.
const THEY_EXILED: Var = vars::USER + 9553;
const YOU_EXILED: Var = vars::USER + 9554;
const THEIR_PICK: Var = vars::USER + 9555;
const YOUR_PICK: Var = vars::USER + 9556;

/// The player target an earlier sentence named ("Target opponent reveals their hand.").
fn last_player_target(b: &Builder) -> Option<u8> {
    b.targets
        .iter()
        .rposition(|t| matches!(t.what, TargetKind::Player(_)))
        .map(|i| i as u8)
}

/// The cards in the target player's hand.
fn hand_of(slot: u8) -> Filter {
    Filter::and(vec![
        Filter::Card,
        Filter::InZone(ZoneKind::Hand),
        Filter::OwnedByPlayer(Box::new(PlayerRef::Target(slot))),
    ])
}

/// "That player exiles a card from it, then you exile a card from it." (Struggle for
/// Sanity, after "Target opponent reveals their hand."): each chooses a card from that
/// hand in turn and exiles it.
fn they_then_you_exile_from_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "that player exiles a card from it, then you exile a card from it" {
        return None;
    }
    let slot = last_player_target(b)?;
    let pick = |var: Var, chooser: PlayerRef, kept: Var| {
        vec![
            Effect::Store {
                var,
                sel: Sel::Choose {
                    chooser,
                    filter: hand_of(slot),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
            },
            Effect::Exile {
                what: Sel::Var(var),
                face_down: false,
                link: false,
            },
            crate::kw::choices_5_filters::keep_affected(kept),
        ]
    };
    let mut v = pick(THEIR_PICK, PlayerRef::Target(slot), THEY_EXILED);
    v.extend(pick(YOUR_PICK, PlayerRef::You, YOU_EXILED));
    b.named
        .push(("the cards they exiled this way".into(), Sel::Var(THEY_EXILED)));
    b.named.push(("the rest".into(), Sel::Var(YOU_EXILED)));
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "that player exiles a card from it, then you exile a card from it", priority: 90, parse: they_then_you_exile_from_hand } }

/// "Repeat this process until all cards in that hand have been exiled." after the
/// alternating exiles: the exiles again while the hand has a card.
fn repeat_until_hand_exiled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if end(l) != "repeat this process until all cards in that hand have been exiled" {
        return false;
    }
    let Some(slot) = last_player_target(b) else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    // The pass: from the first pick to what you exiled being kept (the sequence the
    // sentence compiled to, flattened into the ability's).
    let store_of = |e: &Effect, var: Var| matches!(e, Effect::Store { var: x, .. } if *x == var);
    let keep_of = |e: &Effect, var: Var| {
        matches!(e, Effect::Custom(n) if *n == format!("{}{var}", crate::kw::choices_5_filters::KEEP_AFFECTED))
    };
    let Some(i) = v.iter().position(|e| store_of(e, THEIR_PICK)) else {
        return false;
    };
    let Some(j) = v.iter().skip(i).position(|e| keep_of(e, YOU_EXILED)).map(|k| i + k) else {
        return false;
    };
    let pass = Effect::Seq(v.drain(i..=j).collect());
    v.insert(i, Effect::Noop);
    v[i] = Effect::seq(vec![
        Effect::Store {
            var: THEY_EXILED,
            sel: Sel::Union(vec![]),
        },
        Effect::Store {
            var: YOU_EXILED,
            sel: Sel::Union(vec![]),
        },
        Effect::RepeatProcess {
            body: Box::new(Effect::seq(vec![
                pass,
                Effect::If {
                    cond: Condition::Compare(Value::Count(hand_of(slot)), Cmp::Ge, Value::c(1)),
                    then: Box::new(Effect::RepeatThisProcess),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "repeat this process until all cards in that hand have been exiled", priority: 60, apply: repeat_until_hand_exiled } }

/// "That player returns the cards they exiled this way to their hand and puts the rest
/// into their graveyard." (Struggle for Sanity).
fn return_theirs_rest_to_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l)
        != "that player returns the cards they exiled this way to their hand and puts the rest into their graveyard"
    {
        return None;
    }
    let find = |k: &str| {
        b.named
            .iter()
            .rev()
            .find(|(n, _)| n == k)
            .map(|(_, s)| s.clone())
    };
    let theirs = find("the cards they exiled this way")?;
    let rest = find("the rest")?;
    if !matches!(rest, Sel::Var(YOU_EXILED)) {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Move {
            what: theirs,
            to: Destination::zone(ZoneKind::Hand),
        },
        Effect::Move {
            what: rest,
            to: Destination::zone(ZoneKind::Graveyard),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "that player returns the cards they exiled this way to their hand and puts the rest into their graveyard", priority: 90, parse: return_theirs_rest_to_graveyard } }
