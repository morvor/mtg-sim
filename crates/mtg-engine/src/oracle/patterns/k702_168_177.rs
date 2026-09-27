//! Oracle text of the keywords of CR 702.168–702.177 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Exhaust — [cost]: [effect]" (CR 702.177a), "Whenever you activate an exhaust
//!   ability", "During your turn, as long as you haven't activated an exhaust ability this
//!   turn, you may activate exhaust abilities as though they haven't been activated"
//!   (CR 702.177b); "[Vehicle] becomes an artifact creature" (what many exhaust abilities
//!   of Vehicles do).

use super::{AbilityPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Exhaust (CR 702.177)
// ---------------------------------------------------------------------------

/// "Exhaust — [cost]: [effect]" (CR 702.177a): the activated ability, which can be
/// activated only once (see `kw/exhaust.rs`). Its text keeps the "Exhaust" label, which
/// makes it an exhaust ability.
fn exhaust(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest = t.strip_prefix("Exhaust — ")?;
    let abilities = crate::oracle::parse_ability(rest, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Activated(act.clone()),
        t,
    )])
}

inventory::submit! { AbilityPattern { name: "k702.177 exhaust", priority: 100, parse: exhaust } }

/// "Whenever you activate an exhaust ability" (Rangers' Refueler).
fn exhaust_activated(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "you activate an exhaust ability").then(|| {
        (
            TriggerCond::Custom(SmolStr::new(crate::kw::exhaust::EXHAUST_ACTIVATED)),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "k702.177 you activate an exhaust ability", priority: 100, parse: exhaust_activated } }

/// "During your turn, as long as you haven't activated an exhaust ability this turn, you
/// may activate exhaust abilities as though they haven't been activated." (Elvish
/// Refueler; CR 702.177b).
fn exhaust_again(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    (end(l)
        == "during your turn, as long as you haven't activated an exhaust ability this turn, \
            you may activate exhaust abilities as though they haven't been activated")
        .then(|| {
            vec![AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
                    crate::kw::exhaust::AS_THOUGH_NOT_ACTIVATED.into(),
                ))),
                text,
            )]
        })
}

inventory::submit! { StaticPattern { name: "k702.177b activate exhaust abilities as though they haven't been activated", priority: 100, parse: exhaust_again } }

/// "~ becomes an artifact creature" (an exhaust ability of a Vehicle: "Exhaust — {3}:
/// This Vehicle becomes an artifact creature. Put a +1/+1 counter on it."): for as long as
/// it remains on the battlefield.
fn becomes_artifact_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" becomes an artifact creature")?;
    let saved = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subj, b)?;
    if !end(&rest).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddTypes(vec![
            CardType::Artifact,
            CardType::Creature,
        ])],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "k702.177 ~ becomes an artifact creature", priority: 100, parse: becomes_artifact_creature } }
