//! "Exile all creature cards from target player's graveyard in a face-down pile, shuffle
//! that pile, then manifest those cards." (Ghastly Conscription): the cards are exiled face
//! down (CR 406.3), so no one can tell them apart, and manifested (CR 701.40a) in an order
//! no one knows (see `kwa/manifest.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn manifest_face_down_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let objects = end(l)
        .strip_prefix("exile ")?
        .strip_suffix(" in a face-down pile, shuffle that pile, then manifest those cards")?;
    let slot = b.targets.len();
    let saved_it = b.it.clone();
    // "exile it and the top card of your library": the objects of each part together.
    let mut parts = Vec::new();
    for part in objects.split(" and ") {
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
            action: KeywordAction::Manifest,
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            n: Value::c(1),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "a701 exile in a face-down pile, then manifest those cards", priority: 90, parse: manifest_face_down_pile } }
