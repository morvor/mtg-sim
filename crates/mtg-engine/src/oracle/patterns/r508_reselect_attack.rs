//! Reselecting what attacking creatures are attacking (CR 508.7):
//!
//! * "When ~ enters during the declare attackers step, ..." — a trigger event qualified by
//!   the step it happens in;
//! * "you may reselect which player or permanent target attacking creature is attacking"
//!   (Portal Mage, Misleading Signpost);
//! * "for each attacking creature, you may reselect which player or permanent that
//!   creature is attacking" (Windshaper Planetar).
//!
//! The effects are performed by `kw/reselect_attack.rs`.

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::reselect_attack::{RESELECT_EACH, RESELECT_TARGET};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// "[object] enters during the declare attackers step".
fn enters_during_declare_attackers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let subject = r.strip_suffix(" enters during the declare attackers step")?;
    let (trigger, it, player) =
        crate::oracle::triggers::parse_trigger_condition(&format!("when {subject} enters"))?;
    Some((
        TriggerCond::Where {
            trigger: Box::new(trigger),
            cond: Condition::Phase(PhaseCond::DeclareAttackers),
        },
        it,
        player,
    ))
}

inventory::submit! { TriggerPattern { name: "[object] enters during the declare attackers step", priority: 100, parse: enters_during_declare_attackers } }

fn reselect_attack(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l).trim();
    if l == "for each attacking creature, you may reselect which player or permanent that creature is attacking"
    {
        return Some(Effect::Custom(RESELECT_EACH.into()));
    }
    // "you may" is optional here: the reselection itself lets the player keep the
    // creature attacking what it's attacking.
    let l = l.strip_prefix("you may ").unwrap_or(l);
    if l != "reselect which player or permanent target attacking creature is attacking" {
        return None;
    }
    let spec = TargetSpec::one(
        TargetKind::Object(Filter::and(vec![
            Filter::Type(CardType::Creature),
            Filter::Attacking,
        ])),
        "target attacking creature",
    );
    let slot = b.add_target(spec, "target attacking creature");
    Some(Effect::Custom(format!("{RESELECT_TARGET}{slot}").into()))
}

inventory::submit! { EffectPattern { name: "reselect which player or permanent an attacking creature is attacking", priority: 100, parse: reselect_attack } }
