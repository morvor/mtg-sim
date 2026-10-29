//! "Exile all creature cards from target player's graveyard in a face-down pile, shuffle
//! that pile, then manifest those cards." (Ghastly Conscription): the cards are exiled face
//! down (CR 406.3), so no one can tell them apart, and manifested (CR 701.40a) in an order
//! no one knows (see `kwa/manifest.rs`). The same for "..., then cloak those cards. They
//! enter tapped." (Become Anonymous) and "Exile any number of face-up creatures you control
//! with disguise in a face-down pile, shuffle that pile, then cloak them." (Expose the
//! Culprit).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn manifest_face_down_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile ")?;
    let (objects, action) = [
        (
            " in a face-down pile, shuffle that pile, then manifest those cards",
            KeywordAction::Manifest,
        ),
        (
            " in a face-down pile, shuffle that pile, then cloak those cards",
            KeywordAction::Cloak,
        ),
        (
            " in a face-down pile, shuffle that pile, then cloak them",
            KeywordAction::Cloak,
        ),
    ]
    .iter()
    .find_map(|(suffix, a)| r.strip_suffix(suffix).map(|o| (o, *a)))?;
    let slot = b.targets.len();
    let saved_it = b.it.clone();
    // "exile it and the top card of your library": the objects of each part together.
    let mut parts = Vec::new();
    for part in objects.split(" and ") {
        // "any number of face-up creatures you control with disguise": chosen as it
        // resolves.
        if let Some(sel) = any_number_of(part) {
            parts.push(sel);
            continue;
        }
        match crate::oracle::effects::parse_clause(&format!("exile {part}"), b) {
            Some(Effect::Exile { what, .. }) => parts.push(what),
            _ => {
                b.targets.truncate(slot);
                b.it = saved_it;
                return None;
            }
        }
        b.it = saved_it.clone();
    }
    let what = if parts.len() == 1 {
        parts.pop()?
    } else {
        Sel::Union(parts)
    };
    b.it = Sel::Var(crate::kwa::kvars::MANIFESTED);
    Some(Effect::seq(vec![
        Effect::Exile {
            what,
            face_down: true,
            link: false,
        },
        Effect::KeywordAction {
            action,
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            n: Value::c(1),
        },
    ]))
}

/// "any number of [face-up] [objects]".
fn any_number_of(part: &str) -> Option<Sel> {
    let r = part.strip_prefix("any number of ")?;
    let (face_up, r) = match r.strip_prefix("face-up ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut v = vec![f];
    if face_up {
        v.push(Filter::Not(Box::new(Filter::FaceDown)));
    }
    Some(Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::and(v),
        count: Value::c(99),
        up_to: true,
        store: None,
    })
}

/// "They enter tapped." after "..., then cloak those cards" / "manifest those cards".
fn they_enter_tapped(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l != "they enter tapped" {
        return false;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(last) = v.last_mut() else {
        return false;
    };
    let Effect::KeywordAction {
        action: action @ (KeywordAction::Manifest | KeywordAction::Cloak),
        who,
        what: what @ Sel::Var(vars::IT),
        n,
    } = last
    else {
        return false;
    };
    let mut spec = crate::kwa::Spec::new(*action, who.clone(), what.clone(), n.clone());
    spec.tapped = true;
    *last = spec.effect();
    true
}

inventory::submit! { FollowupPattern { name: "a701 they enter tapped (manifested or cloaked pile)", priority: 90, apply: they_enter_tapped } }

inventory::submit! { EffectPattern { name: "a701 exile in a face-down pile, then manifest those cards", priority: 90, parse: manifest_face_down_pile } }
