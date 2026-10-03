//! Separating the top cards of a library into a face-down pile and a face-up pile
//! (CR 700.3): "Look at the top four cards of your library and separate them into a
//! face-down pile and a face-up pile." (Riddles in the Dark, Curator of Destinies),
//! "Target opponent looks at the top three cards of your library and separates them into
//! a face-down pile and a face-up pile." (Atris, Oracle of Half-Truths; Fortune's Favor),
//! followed by "An opponent chooses one of the piles." and where the piles go (see
//! `r700_piles.rs`). The face-up pile's cards are revealed; the face-down pile's aren't,
//! wherever they go.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;
use crate::piles::PileAction;

/// The top cards looked at, left where they are; "them" names them.
fn look_at_top(n: Value) -> Effect {
    Effect::Dig {
        who: PlayerRef::You,
        n,
        reveal: false,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: false,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to: Destination {
            position: LibraryPosition::FromTop(0),
            ..Destination::library_top()
        },
    }
}

fn face_down_piles(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (separator, r) = if let Some(r) = l.strip_prefix("look at the top ") {
        let r = r.strip_suffix(
            " cards of your library and separate them into a face-down pile and a face-up pile",
        )?;
        (None, r)
    } else {
        let (subject, r) = l.split_once(" looks at the top ")?;
        let r = r.strip_suffix(
            " cards of your library and separates them into a face-down pile and a face-up pile",
        )?;
        if !subject.starts_with("target ") {
            return None;
        }
        (Some(subject), r)
    };
    let (n, rest) = parse_number(r)?;
    if !rest.trim().is_empty() {
        return None;
    }
    n.as_const()?;
    let separator = match separator {
        None => PlayerRef::You,
        Some(subject) => {
            let (who, rest) = player_ref(subject, b)?;
            if !rest.trim().is_empty() {
                return None;
            }
            who
        }
    };
    b.it = Sel::Var(vars::REVEALED);
    Some(Effect::seq(vec![
        look_at_top(n),
        Effect::Piles(Box::new(PileAction::SeparateFaceDown {
            what: Sel::Var(vars::REVEALED),
            separator,
        })),
    ]))
}

inventory::submit! { EffectPattern { name: "r700 separate into a face-down pile and a face-up pile", priority: 100, parse: face_down_piles } }

/// "an opponent chooses one of the piles".
fn choose_one_of_the_piles(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "an opponent chooses one of the piles").then(|| {
        Effect::Piles(Box::new(PileAction::Choose {
            chooser: PlayerRef::EachOpponent,
        }))
    })
}

inventory::submit! { EffectPattern { name: "r700 an opponent chooses one of the piles", priority: 100, parse: choose_one_of_the_piles } }
