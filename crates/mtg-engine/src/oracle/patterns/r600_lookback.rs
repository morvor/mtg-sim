//! Oracle patterns for trigger conditions that look back in time (CR 603.10): becoming
//! unattached, losing control, phasing out; and "sacrifice that permanent".

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;

fn lookback_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    match r {
        // CR 603.10c: "Whenever this Equipment becomes unattached from a permanent".
        "~ becomes unattached from a permanent" => Some((
            TriggerCond::BecomesUnattached(Filter::Source),
            Sel::TriggerObject,
            PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
        )),
        // CR 603.10d.
        "you lose control of ~" => Some((
            TriggerCond::LoseControl(Filter::Source),
            Sel::This,
            PlayerRef::You,
        )),
        // CR 603.10b.
        "~ phases out" => Some((
            TriggerCond::PhasesOut(Filter::Source),
            Sel::This,
            PlayerRef::You,
        )),
        _ => None,
    }
}

/// "sacrifice that permanent" / "sacrifice it": its controller sacrifices it.
fn sacrifice_that(l: &str, b: &mut Builder) -> Option<Effect> {
    match l {
        "sacrifice that permanent" | "sacrifice it" | "sacrifice that creature" => {
            Some(Effect::SacrificeObjects {
                what: sacrificed_referent(b),
            })
        }
        _ => None,
    }
}

/// What "it" is in "sacrifice it". A player sacrifices only permanents they control
/// (CR 701.21a): after "Target creature you control deals damage equal to its power to any
/// other target." (Burn Together), "it" is the creature you control, not the target that
/// was dealt damage.
fn sacrificed_referent(b: &Builder) -> Sel {
    let yours = |k: usize| {
        b.targets.get(k).is_some_and(|t| match &t.what {
            TargetKind::Object(f) => controlled_by_you(f),
            _ => false,
        })
    };
    match b.it {
        Sel::Target(k) if !yours(k as usize) => {
            let mine: Vec<usize> = (0..b.targets.len()).filter(|j| yours(*j)).collect();
            match mine[..] {
                [j] => Sel::Target(j as u8),
                _ => b.it.clone(),
            }
        }
        _ => b.it.clone(),
    }
}

/// Whether the filter only matches objects controlled by "you".
fn controlled_by_you(f: &Filter) -> bool {
    match f {
        Filter::ControlledBy(PlayerRel::You) => true,
        Filter::And(fs) => fs.iter().any(controlled_by_you),
        _ => false,
    }
}

inventory::submit! { TriggerPattern { name: "look-back triggers", priority: 0, parse: lookback_triggers } }
inventory::submit! { EffectPattern { name: "sacrifice that permanent", priority: 0, parse: sacrifice_that } }
