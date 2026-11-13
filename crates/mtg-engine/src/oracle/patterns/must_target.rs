//! "While an opponent is choosing targets as part of casting a spell they control or
//! activating an ability they control, that player must choose at least one Flagbearer on
//! the battlefield if able." (Coalition Honor Guard): a requirement on choosing targets
//! while casting or activating (CR 601.2c, 602.2b). Targets changed later, and new targets
//! chosen for copies, aren't chosen as part of casting or activating, so it doesn't apply
//! to them (CR 115.7, 707.10c).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn must_target(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    // (Enroll in the Coalition's "your opponents must choose at least one Flagbearer"
    // includes players who are Flagbearers, which isn't modeled.)
    let r = end(l).strip_prefix(
        "while an opponent is choosing targets as part of casting a spell they control or activating an ability they control, that player must choose at least one ",
    )?;
    let chooser = PlayerFilter::Opponent;
    let what = r.strip_suffix(" if able")?;
    let what = what.strip_suffix(" on the battlefield").unwrap_or(what);
    let (f, plural, tail) = parse_object_phrase(what)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Restriction(Restriction::MustTarget {
        chooser,
        what: Filter::And(vec![Filter::Permanent, f]),
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "while choosing targets ..., [players] must choose at least one [permanent] if able", priority: 100, parse: must_target } }
