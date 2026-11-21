//! Oracle patterns for rad counters (CR 728): "You gain life rather than lose life from
//! radiation." (CR 728.1a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

fn gain_rather_than_lose(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l.trim().trim_end_matches('.') != "you gain life rather than lose life from radiation" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(SmolStr::new(
            crate::radiation::GAIN_RATHER_THAN_LOSE,
        )))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r728 gain life rather than lose life from radiation", priority: 60, parse: gain_rather_than_lose } }
