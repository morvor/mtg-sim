//! Oracle patterns around banding (CR 702.22):
//!
//! * "[target] loses banding and all "bands with other" abilities": losing banding
//!   already removes every "bands with other" ability (CR 702.22b), which is a banding
//!   keyword with a quality;
//! * "Target unblocked attacking creature becomes blocked." (CR 509.1h; with banding the
//!   whole band becomes blocked, CR 702.22i).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_target};

/// The custom effect making the attacking creature bound to [`vars::AFFECTED`] become
/// blocked.
pub const TARGET_BECOMES_BLOCKED: &str = "target becomes blocked";

fn loses_banding_and_bands_with_other(l: &str, b: &mut Builder) -> Option<Effect> {
    const PHRASE: &str = " and all \"bands with other\" abilities";
    let i = l.find(PHRASE)?;
    let rest = format!("{}{}", &l[..i], &l[i + PHRASE.len()..]);
    if !rest.contains("loses banding") && !rest.contains("lose banding") {
        return None;
    }
    parse_clause(&rest, b)
}

inventory::submit! { EffectPattern { name: "k702.22 loses banding and bands with other", priority: 60, parse: loses_banding_and_bands_with_other } }

fn becomes_blocked(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" becomes blocked")?;
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let slot = b.add_target(spec, r);
    Some(Effect::ForEach {
        sel: Sel::Target(slot),
        var: vars::AFFECTED,
        effect: Box::new(Effect::Custom(TARGET_BECOMES_BLOCKED.into())),
    })
}

inventory::submit! { EffectPattern { name: "k702.22 target becomes blocked", priority: 60, parse: becomes_blocked } }
