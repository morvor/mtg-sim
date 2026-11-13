//! Numbers chosen as an effect happens and what later sentences do with them (CR
//! 608.2d): "Target opponent chooses a number. You may have that player lose that much
//! life. If you don't, that player sacrifices all but that many permanents of their
//! choice." (Choice of Damnations): "that much" / "that many" is the chosen number
//! (`Value::Chosen`).
//!
//! Also "[player] sacrifices all but N [permanents] of their choice": the player chooses N
//! of them to keep and sacrifices the rest (CR 701.21a).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, player_ref, Builder};
use crate::oracle::phrases::*;

/// The permanents kept by "sacrifices all but N".
const KEPT: Var = vars::USER + 4434;

/// Whether the effect chose a number ([`ChoiceKind::Number`]).
fn chose_number(e: &Effect) -> bool {
    let Ok(j) = serde_json::to_string(e) else {
        return false;
    };
    j.contains("\"Choose\":{") && j.contains("\"kind\":{\"Number\"")
}

/// "that much" / "that many" after a number was chosen: the chosen number.
fn that_much_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !(l.contains("that much") || l.contains("that many")) || !chose_number(prev) {
        return false;
    }
    let text = l
        .replace("that much life", "x life")
        .replace("that much damage", "x damage")
        .replace("that many", "x");
    if text.contains("that much") {
        return false;
    }
    let Some(e) = parse_sentence(&text, b) else {
        return false;
    };
    let Some(e) = super::r107_numbers::substitute_x(&e, &Value::Chosen) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: that much / that many (the chosen number)", priority: 45, apply: that_much_chosen } }

/// "that player sacrifices all but three permanents of their choice".
fn sacrifices_all_but(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let k = l.find(" sacrifices all but ")?;
    let (who, rest) = player_ref(&l[..k], b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let r = &l[k + " sacrifices all but ".len()..];
    let r = r.strip_suffix(" of their choice")?;
    let (n, r) = parse_number(r)?;
    let (f, true, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    // Each player keeps their own.
    let each = matches!(
        who,
        PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
    );
    let owner = if each {
        Filter::ControlledBy(PlayerRel::Iterated)
    } else {
        Filter::ControlledByPlayer(Box::new(who.clone()))
    };
    let theirs = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield), owner]);
    let chooser = if each {
        PlayerRef::Iterated
    } else {
        who.clone()
    };
    let body = Effect::seq(vec![
        Effect::Store {
            var: KEPT,
            sel: Sel::Choose {
                chooser,
                filter: theirs.clone(),
                count: n,
                up_to: false,
                store: None,
            },
        },
        Effect::SacrificeObjects {
            what: Sel::All(Filter::and(vec![
                theirs,
                Filter::not(Filter::In(Box::new(Sel::Var(KEPT)))),
            ])),
        },
    ]);
    Some(if each {
        Effect::ForEachPlayer {
            who,
            effect: Box::new(body),
        }
    } else {
        body
    })
}

inventory::submit! { EffectPattern { name: "choice grammar: sacrifices all but N of their choice", priority: 120, parse: sacrifices_all_but } }
