//! Oracle text of the keywords of CR 702.168–702.177 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Exhaust — [cost]: [effect]" (CR 702.177a), "Whenever you activate an exhaust
//!   ability", "During your turn, as long as you haven't activated an exhaust ability this
//!   turn, you may activate exhaust abilities as though they haven't been activated"
//!   (CR 702.177b); "[Vehicle] becomes an artifact creature" (what many exhaust abilities
//!   of Vehicles do);
//! * saddle (CR 702.171): "~ is saddled", "whenever ~ becomes saddled [for the first time
//!   each turn]", "whenever ~ saddles a Mount [or crews a Vehicle] [during your main
//!   phase]", "[Mount] becomes saddled until end of turn" (and "creature that saddled it
//!   this turn" in `oracle/phrases.rs`).

use super::{AbilityPattern, ConditionPattern, EffectPattern, StaticPattern, TriggerPattern};
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

// ---------------------------------------------------------------------------
// Saddle (CR 702.171)
// ---------------------------------------------------------------------------

/// The saddled designation of the permanent itself (CR 702.171b).
fn this_saddled() -> Condition {
    Condition::SelMatches(
        Sel::This,
        Filter::Custom(SmolStr::new(crate::kw::saddle::SADDLED)),
    )
}

/// "~ is saddled", "~ isn't saddled" (Caustic Bronco, Archmage's Newt).
fn saddled_condition(c: &str) -> Option<Condition> {
    match end(c) {
        "~ is saddled" | "it's saddled" | "it is saddled" => Some(this_saddled()),
        "~ isn't saddled" | "~ is not saddled" | "it isn't saddled" | "it's not saddled" => {
            Some(Condition::Not(Box::new(this_saddled())))
        }
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "k702.171b ~ is saddled", priority: 100, parse: saddled_condition } }

/// "Whenever ~ becomes saddled [for the first time each turn]" (Stubborn Burrowfiend):
/// the permanent became saddled (CR 702.171a–b). "Whenever ~ saddles a Mount [or crews a
/// Vehicle] [during your main phase]" (Canyon Vaulter): it was tapped to pay for a saddle
/// ability (CR 702.171c); "that Mount" is the trigger object.
fn saddle_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    use crate::kw::crew::CREWS_A_VEHICLE;
    use crate::kw::saddle::{BECAME_SADDLED, SADDLES_A_MOUNT};
    let r = end(r);
    if let Some(rest) = r.strip_prefix("~ saddles a mount") {
        let (crews, rest) = match rest.strip_prefix(" or crews a vehicle") {
            Some(x) => (true, x),
            None => (false, rest),
        };
        let main = match rest {
            "" => false,
            " during your main phase" => true,
            _ => return None,
        };
        let saddles = TriggerCond::Custom(SmolStr::new(SADDLES_A_MOUNT));
        let mut trigger = if crews {
            TriggerCond::AnyOf(vec![
                saddles,
                TriggerCond::Custom(SmolStr::new(CREWS_A_VEHICLE)),
            ])
        } else {
            saddles
        };
        if main {
            trigger = TriggerCond::Where {
                trigger: Box::new(trigger),
                cond: Condition::And(vec![
                    Condition::YourTurn,
                    Condition::Phase(PhaseCond::MainPhase),
                ]),
            };
        }
        return Some((trigger, Sel::TriggerObject, PlayerRef::TriggerPlayer));
    }
    let (first_time, subj) = match r.strip_suffix(" becomes saddled for the first time each turn") {
        Some(s) => (true, s),
        None => (false, r.strip_suffix(" becomes saddled")?),
    };
    if subj != "~" {
        return None;
    }
    let cond = TriggerCond::Where {
        trigger: Box::new(TriggerCond::PlayerAction {
            name: SmolStr::new(BECAME_SADDLED),
            who: PlayerRel::Any,
        }),
        cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
    };
    Some((
        if first_time {
            TriggerCond::FirstTimeEachTurn(Box::new(cond))
        } else {
            cond
        },
        Sel::This,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.171 becomes saddled, saddles a mount", priority: 100, parse: saddle_triggers } }

/// "[Mount] becomes saddled until end of turn" (Guidelight Matrix, Kolodin, Triumph
/// Caster): the saddled designation from an effect (CR 702.171b).
fn becomes_saddled(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" becomes saddled until end of turn")?;
    let saved = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subj, b)?;
    if !end(&rest).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(crate::kw::saddle::becomes_saddled(what))
}

inventory::submit! { EffectPattern { name: "k702.171 becomes saddled until end of turn", priority: 100, parse: becomes_saddled } }

// ---------------------------------------------------------------------------
// Freerunning (CR 702.173)
// ---------------------------------------------------------------------------

/// "this spell's freerunning cost was paid" (Monastery Raid).
fn freerunning_paid(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "this spell's freerunning cost was paid"
            | "~'s freerunning cost was paid"
            | "its freerunning cost was paid"
    )
    .then(|| Condition::CostPaid(crate::kw::freerunning::FREERUNNING.into()))
}

inventory::submit! { ConditionPattern { name: "k702.173 freerunning cost was paid", priority: 100, parse: freerunning_paid } }
