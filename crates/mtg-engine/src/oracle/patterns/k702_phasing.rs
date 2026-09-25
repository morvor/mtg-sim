//! Oracle patterns around phasing (CR 702.26):
//!
//! * "[objects] phase(s) out": "target creature phases out", "it phases out", "~ phases
//!   out", "enchanted creature phases out", "any number of target creatures you control
//!   phase out", "all permanents you control phase out";
//! * "Players skip their untap steps." / "Skip your untap step." (no phasing happens in a
//!   skipped untap step, CR 702.26m).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase, parse_target};
use crate::oracle::CompileContext;

fn phase_out(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let subject = l
        .strip_suffix(" phases out")
        .or_else(|| l.strip_suffix(" phase out"))?;
    let what = match subject {
        "~" | "this creature" | "this permanent" => Sel::This,
        "it" | "that creature" | "that permanent" => b.it.clone(),
        "enchanted creature" | "equipped creature" | "enchanted permanent" => Sel::AttachedTo,
        _ => {
            if subject.contains("target") {
                let (spec, tail) = parse_target(subject)?;
                if !end(tail).is_empty() {
                    return None;
                }
                Sel::Target(b.add_target(spec, subject))
            } else {
                let r = subject
                    .strip_prefix("all ")
                    .or_else(|| subject.strip_prefix("each "))?;
                let (f, _, tail) = parse_object_phrase(r)?;
                if !end(tail).is_empty() {
                    return None;
                }
                Sel::All(f)
            }
        }
    };
    Some(Effect::PhaseOut { what })
}

inventory::submit! { EffectPattern { name: "k702.26 phases out", priority: 60, parse: phase_out } }

fn skip_untap_steps(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let whose = match end(l) {
        "players skip their untap steps" => PlayerRel::Any,
        "skip your untap step" | "you skip your untap step" => PlayerRel::You,
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::SkipStep {
                    step: StepKind::Untap,
                    whose,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "k702.26 skip untap steps", priority: 60, parse: skip_untap_steps } }
