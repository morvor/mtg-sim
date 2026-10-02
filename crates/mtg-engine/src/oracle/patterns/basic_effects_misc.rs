//! Smaller basic-effect constructs:
//!
//! - "shuffle target nontoken permanent you control into its owner's library" (CR 701.24):
//!   the object is shuffled into its owner's library.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// "shuffle [object] into its owner's library", "shuffle [objects] into their owners'
/// libraries".
fn shuffle_into_owners_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("shuffle ")?;
    let (what, tail) = object_ref(r, b)?;
    match tail.trim() {
        "into its owner's library" | "into their owners' libraries" => {
            Some(Effect::ShuffleInto { what })
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "basic effects: shuffle [object] into its owner's library", priority: 60, parse: shuffle_into_owners_library } }
