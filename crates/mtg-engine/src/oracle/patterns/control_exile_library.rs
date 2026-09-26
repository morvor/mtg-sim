//! Putting objects into libraries (CR 401, 400.3):
//!
//! - "Put target creature on top of its owner's library", "put target card from a
//!   graveyard on the bottom of its owner's library", "put target card from your graveyard
//!   on top of your library": each object goes to its owner's library (CR 400.3).
//! - "Put target creature into its owner's library second from the top": the Nth position
//!   from the top, or the bottom of a library with fewer cards (CR 401.7).
//! - "Put any number of target creature cards from your graveyard on top of your library":
//!   the owner arranges cards put into the same position at the same time (CR 401.4; see
//!   `zones::order_simultaneous`).
//! - A choice of positions: "target creature's owner puts it on their choice of the top or
//!   bottom of their library", "the owner of target nonland permanent puts it into their
//!   library second from the top or on the bottom", "put target card from a graveyard on
//!   your choice of the top or bottom of its owner's library".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "control_exile: put into a library", priority: 90, parse: p_put_into_library }
}
inventory::submit! {
    EffectPattern { name: "control_exile: owner puts it into their library", priority: 90, parse: p_owner_puts }
}

/// "its owner's library", "your library": the library of each object's owner (CR 400.3
/// sends an object put into a library to its owner's). "Their library" names the owner's
/// library only where the owner is the subject ("its owner puts it on top of their
/// library").
fn library_phrase(s: &str, owner_subject: bool) -> Option<&str> {
    let s = s.trim_start();
    for p in [
        "its owner's library",
        "their owner's library",
        "their owners' library",
        "their owners' libraries",
        "your library",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some(r);
        }
    }
    if owner_subject {
        return s.strip_prefix("their library");
    }
    None
}

fn ordinal(w: &str) -> Option<u32> {
    Some(match w {
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        _ => return None,
    })
}

/// One position: "on top of [library]", "on the bottom of [library]", "into [library]
/// second from the top". Returns the position and the rest of the text.
fn one_position(s: &str, owner_subject: bool) -> Option<(LibraryPosition, &str)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("on top of ") {
        return Some((LibraryPosition::Top, library_phrase(r, owner_subject)?));
    }
    if let Some(r) = s.strip_prefix("on the bottom of ") {
        let r = library_phrase(r, owner_subject)?;
        if let Some(x) = r.strip_prefix(" in a random order") {
            return Some((LibraryPosition::BottomRandom, x));
        }
        return Some((LibraryPosition::Bottom, r));
    }
    if let Some(r) = s.strip_prefix("into ") {
        let r = library_phrase(r, owner_subject)?;
        let (w, r) = split_word(r.trim_start());
        let n = ordinal(w)?;
        let r = r.trim_start().strip_prefix("from the top")?;
        return Some((LibraryPosition::FromTop(n - 1), r));
    }
    None
}

/// The positions offered: one position, or a choice between two ("on [your/their] choice
/// of the top or bottom of [library]", "on the top or bottom of their library", "into their
/// library second from the top or on the bottom").
fn positions(s: &str, owner_subject: bool) -> Option<Vec<LibraryPosition>> {
    let s = end(s);
    let choice = s
        .strip_prefix("on your choice of the top or bottom of ")
        .or_else(|| s.strip_prefix("on their choice of the top or bottom of "))
        .or_else(|| s.strip_prefix("on the top or bottom of "));
    if let Some(r) = choice {
        if !library_phrase(r, owner_subject)?.is_empty() {
            return None;
        }
        return Some(vec![LibraryPosition::Top, LibraryPosition::Bottom]);
    }
    let (first, rest) = one_position(s, owner_subject)?;
    let rest = rest.trim();
    // "in any order": the owner arranges them anyway (CR 401.4).
    if rest.is_empty() || rest == "in any order" {
        return Some(vec![first]);
    }
    let alt = rest.strip_prefix("or ")?;
    let second = if alt == "on the bottom" {
        LibraryPosition::Bottom
    } else if alt == "on top" {
        LibraryPosition::Top
    } else {
        let (p, r) = one_position(alt, owner_subject)?;
        if !r.trim().is_empty() {
            return None;
        }
        p
    };
    Some(vec![first, second])
}

fn to_library(pos: LibraryPosition) -> Destination {
    let mut d = Destination::zone(ZoneKind::Library);
    d.position = pos;
    d
}

fn label(pos: LibraryPosition) -> String {
    match pos {
        LibraryPosition::Top => "Top of library".into(),
        LibraryPosition::Bottom | LibraryPosition::BottomRandom => "Bottom of library".into(),
        LibraryPosition::FromTop(n) => format!("{} from the top of library", n + 1),
        LibraryPosition::Shuffled => "Shuffled into library".into(),
    }
}

/// Moves `what` to the position, or lets `who` choose among the positions.
fn place(what: Sel, who: PlayerRef, positions: Vec<LibraryPosition>) -> Effect {
    if positions.len() == 1 {
        return Effect::Move {
            what,
            to: to_library(positions[0]),
        };
    }
    Effect::ChooseOne {
        who,
        options: positions
            .into_iter()
            .map(|p| {
                (
                    label(p),
                    Effect::Move {
                        what: what.clone(),
                        to: to_library(p),
                    },
                )
            })
            .collect(),
    }
}

fn is_object_sel(s: &Sel) -> bool {
    !matches!(s, Sel::Players(_) | Sel::None)
}

/// "put [objects] on top of its owner's library", "put [objects] on your choice of the top
/// or bottom of its owner's library".
fn p_put_into_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("put ")?;
    let (what, tail) = object_ref(r, b)?;
    if !is_object_sel(&what) {
        return None;
    }
    let tail = tail.trim();
    // "Their library" isn't the owner's here unless the owner is the subject.
    let ps = positions(tail, false)?;
    Some(place(what, PlayerRef::You, ps))
}

/// "[object]'s owner puts it on their choice of the top or bottom of their library", "the
/// owner of [object] puts it into their library second from the top or on the bottom",
/// "its owner puts it on top of their library".
fn p_owner_puts(l: &str, b: &mut Builder) -> Option<Effect> {
    let (what, rest) = if let Some(r) = l.strip_prefix("the owner of ") {
        let (obj, rest) = r.split_once(" puts ")?;
        let (what, tail) = object_ref(obj, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (what, rest)
    } else if let Some(rest) = l.strip_prefix("its owner puts ") {
        (b.it.clone(), rest)
    } else {
        let (obj, rest) = l.split_once("'s owner puts ")?;
        let (what, tail) = object_ref(obj, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (what, rest)
    };
    if !is_object_sel(&what) || matches!(what, Sel::All(_)) {
        return None;
    }
    let rest = ["it ", "that card ", "that creature ", "that permanent "]
        .iter()
        .find_map(|p| rest.strip_prefix(p))?;
    let ps = positions(rest, true)?;
    let owner = PlayerRef::OwnerOf(Box::new(what.clone()));
    Some(place(what, owner, ps))
}
