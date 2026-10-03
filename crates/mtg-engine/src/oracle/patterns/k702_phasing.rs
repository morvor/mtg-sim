//! Oracle patterns around phasing (CR 702.26):
//!
//! * "[objects] phase(s) out": "target creature phases out", "it phases out", "~ phases
//!   out", "enchanted creature phases out", "any number of target creatures you control
//!   phase out", "all permanents you control phase out"; with "until ~ leaves the
//!   battlefield", they phase in again right after it does (CR 610.4);
//! * "Players skip their untap steps." / "Skip your untap step." (no phasing happens in a
//!   skipped untap step, CR 702.26m).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase, parse_target};
use crate::oracle::CompileContext;

fn phase_out(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "It phases out until ~ leaves the battlefield." (CR 610.4).
    let (l, until) = match l.strip_suffix(" until ~ leaves the battlefield") {
        Some(r) => (r, Some(UntilEvent::SourceLeavesBattlefield)),
        None => (l, None),
    };
    let subject = l
        .strip_suffix(" phases out")
        .or_else(|| l.strip_suffix(" phase out"))?;
    let what = match subject {
        "~" | "this creature" | "this permanent" => Sel::This,
        "it" | "that creature" | "that permanent" => b.it.clone(),
        "enchanted creature" | "equipped creature" | "enchanted permanent" => Sel::AttachedTo,
        // "each creature target player controls phases out" (Galadriel's Dismissal).
        _ if subject.starts_with("each ") && subject.ends_with(" target player controls") => {
            let r = subject.strip_prefix("each ")?;
            let (f, _, tail) = parse_object_phrase(r)?;
            let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
            let (f, rest) = crate::oracle::effects::bind_target_player(f, tail, b);
            if !end(&rest).is_empty() || b.targets.len() == saved.0 {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
                return None;
            }
            Sel::All(f)
        }
        // "those creatures", "the chosen permanents": objects the text named earlier.
        _ if subject.starts_with("those ") || subject.starts_with("the chosen ") => {
            let (sel, rest) = crate::oracle::effects::object_ref(subject, b)?;
            if !end(&rest).is_empty() {
                return None;
            }
            sel
        }
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
    Some(match until {
        Some(until) => Effect::PhaseOutUntil { what, until },
        None => Effect::PhaseOut { what },
    })
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
