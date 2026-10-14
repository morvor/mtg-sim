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
        "copy it" | "copy that card" | "copy the exiled card" | "copy those cards"
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
    if !ends_with_exiling_a_target(prev) {
        return false;
    }
    let mut seq = vec![std::mem::take(prev)];
    if times == 1 {
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
