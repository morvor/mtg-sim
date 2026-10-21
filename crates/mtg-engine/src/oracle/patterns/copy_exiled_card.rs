//! Copying a card the effect exiled: "Exile up to one target instant or sorcery card with
//! mana value 2 or less from your graveyard. Copy it. You may cast the copy without paying
//! its mana cost." (Roving Actuator), "exile target noncreature, nonland card with mana
//! value less than ~'s power from a graveyard and copy it" (Narset, Enlightened Exile).
//! The copy is created in exile (CR 707.12) and "the copy" is `vars::CREATED` (see
//! `r707_copy_cards.rs` for casting it).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};

/// Whether the effect ends by exiling targeted cards face up.
fn ends_with_exiling_a_target(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::Target(_),
            face_down: false,
            ..
        } => true,
        // "Exile target [card] ... and target [card] ..." (Spelltwine).
        Effect::Exile {
            what: Sel::Union(v),
            face_down: false,
            ..
        } => v.iter().all(|s| matches!(s, Sel::Target(_))),
        Effect::Seq(v) => v.last().is_some_and(ends_with_exiling_a_target),
        _ => false,
    }
}

/// "That player exiles an instant or sorcery card from their graveyard. Copy that card."
/// (Wildfire Devils): the variable holding the card another player chose to exile from a
/// graveyard (kept for "that card" after that player's instruction).
fn exiled_by_another_player(e: &Effect) -> Option<Var> {
    match e {
        Effect::AsPlayer { effect, .. } => match effect.as_ref() {
            Effect::Seq(v) => match v.as_slice() {
                [Effect::Exile {
                    what: Sel::Choose { filter, .. },
                    face_down: false,
                    ..
                }, Effect::Store {
                    var,
                    sel: Sel::Var(vars::IT),
                }] if filter.zone() == Some(ZoneKind::Graveyard) => Some(*var),
                _ => None,
            },
            _ => None,
        },
        Effect::Seq(v) => v.last().and_then(exiled_by_another_player),
        _ => None,
    }
}

fn copy_of_exiled() -> Effect {
    Effect::CopyCard {
        what: Sel::Var(vars::IT),
        named: None,
    }
}

/// The copies of `n` card copy effects made one after another, gathered as "the copies"
/// (`vars::CREATED`) for "You may cast the copies".
const COPIES: [Var; 5] = [
    vars::USER + 7351,
    vars::USER + 7352,
    vars::USER + 7353,
    vars::USER + 7354,
    vars::USER + 7355,
];

/// "Copy it." / "Copy that card." after exiling a target card; "Copy those cards." after
/// exiling several (Spelltwine); "Copy that card three times." (Mnemonic Deluge: three
/// copies of it, CR 707.12).
fn copy_it(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let times = if matches!(
        l,
        "copy it" | "copy that card" | "copy the exiled card" | "copy those cards" | "copy them"
    ) {
        1
    } else {
        let Some(t) = l
            .strip_prefix("copy that card ")
            .or_else(|| l.strip_prefix("copy it "))
        else {
            return false;
        };
        match t {
            "twice" => 2,
            _ => match t.strip_suffix(" times").and_then(parse_number) {
                Some((Value::Const(n), rest)) if rest.trim().is_empty() && (2..=5).contains(&n) => {
                    n as usize
                }
                _ => return false,
            },
        }
    };
    let by_other = exiled_by_another_player(prev);
    if !ends_with_exiling_a_target(prev) && !(by_other.is_some() && times == 1) {
        return false;
    }
    let mut seq = vec![std::mem::take(prev)];
    if let Some(v) = by_other {
        seq.push(Effect::CopyCard {
            what: Sel::Var(v),
            named: None,
        });
    } else if times == 1 {
        seq.push(copy_of_exiled());
    } else {
        for v in COPIES.iter().take(times) {
            seq.push(copy_of_exiled());
            seq.push(Effect::Store {
                var: *v,
                sel: Sel::Var(vars::CREATED),
            });
        }
        seq.push(Effect::Store {
            var: vars::CREATED,
            sel: Sel::Union(COPIES.iter().take(times).map(|v| Sel::Var(*v)).collect()),
        });
    }
    *prev = Effect::seq(seq);
    true
}

/// "exile target [card] from a graveyard and copy it".
fn exile_and_copy_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" and copy it")?;
    // "exile up to one target legendary or Rat card from your graveyard and copy it"
    // (Nashi, Moon's Legacy): nothing is copied if nothing was exiled.
    if !r.starts_with("exile target ") && !r.starts_with("exile up to one target ") {
        return None;
    }
    let e = parse_clause(r, b)?;
    if !ends_with_exiling_a_target(&e) {
        return None;
    }
    Some(Effect::seq(vec![e, copy_of_exiled()]))
}

inventory::submit! { FollowupPattern { name: "copy it (the exiled card)", priority: 90, apply: copy_it } }
inventory::submit! { EffectPattern { name: "exile target card and copy it", priority: 90, parse: exile_and_copy_it } }

/// Whether the effect ends by exiling cards from the top of a library face up ("the
/// cards exiled this way" are `vars::IT`).
fn ends_with_exiling_from_library(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(..),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exiling_from_library),
        _ => false,
    }
}

/// The card chosen by "choose an instant or sorcery card exiled this way and copy it".
const CHOSEN_EXILED: Var = vars::USER + 7356;

/// "Exile the top ten cards of your library. Choose an instant or sorcery card exiled this
/// way and copy it three times." (Chandra, Pyromaster): a card is chosen among them (if
/// any qualifies) and copied (CR 707.12).
fn choose_exiled_and_copy_it(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("choose an ")
        .or_else(|| l.strip_prefix("choose a "))
    else {
        return false;
    };
    let Some((card, copy)) = r.split_once(" exiled this way and ") else {
        return false;
    };
    let times = match copy {
        "copy it" => 1,
        "copy it twice" => 2,
        _ => match copy
            .strip_prefix("copy it ")
            .and_then(|t| t.strip_suffix(" times"))
            .and_then(parse_number)
        {
            Some((Value::Const(n), rest)) if rest.trim().is_empty() && (2..=5).contains(&n) => {
                n as usize
            }
            _ => return false,
        },
    };
    let Some((f, false, tail)) = crate::oracle::phrases::parse_object_phrase(card) else {
        return false;
    };
    if !end(tail).is_empty() || !ends_with_exiling_from_library(prev) {
        return false;
    }
    let mut seq = vec![
        std::mem::take(prev),
        Effect::Store {
            var: CHOSEN_EXILED,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    f,
                    Filter::In(Box::new(Sel::Var(vars::IT))),
                    Filter::InZone(ZoneKind::Exile),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
    ];
    let copy = Effect::CopyCard {
        what: Sel::Var(CHOSEN_EXILED),
        named: None,
    };
    if times == 1 {
        seq.push(copy);
    } else {
        for v in COPIES.iter().take(times) {
            seq.push(copy.clone());
            seq.push(Effect::Store {
                var: *v,
                sel: Sel::Var(vars::CREATED),
            });
        }
        seq.push(Effect::Store {
            var: vars::CREATED,
            sel: Sel::Union(COPIES.iter().take(times).map(|v| Sel::Var(*v)).collect()),
        });
    }
    *prev = Effect::seq(seq);
    true
}

inventory::submit! { FollowupPattern { name: "choose a card exiled this way and copy it", priority: 90, apply: choose_exiled_and_copy_it } }
