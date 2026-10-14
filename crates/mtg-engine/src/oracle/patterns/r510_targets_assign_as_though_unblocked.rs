//! "X target blocked creatures assign their combat damage this turn as though they
//! weren't blocked." (Outmaneuver): each target assigns all its combat damage this turn to
//! the player, planeswalker, or battle it's attacking, none to the creatures blocking it
//! (CR 510.1c). The targets gain the ability `StaticEffect::Custom(ASSIGNS_UNBLOCKED)` until
//! end of turn; see `kw::assign_as_though_unblocked`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::assign_as_though_unblocked::ASSIGNS_UNBLOCKED;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_target};

fn targets_assign_unblocked(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l
        .strip_suffix(" assign their combat damage this turn as though they weren't blocked")
        .or_else(|| {
            l.strip_suffix(" assigns its combat damage this turn as though it weren't blocked")
        })?;
    let (spec, rest) = parse_target(head)?;
    if !end(rest).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    let slot = b.add_target(spec, head);
    let ability = AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            ASSIGNS_UNBLOCKED.into(),
        ))),
        "~ assigns its combat damage as though it weren't blocked",
    );
    Some(Effect::Modify {
        what: Sel::Target(slot),
        mods: vec![Modification::AddAbility(ability)],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "r510 target creatures assign combat damage as though unblocked this turn", priority: 60, parse: targets_assign_unblocked } }
