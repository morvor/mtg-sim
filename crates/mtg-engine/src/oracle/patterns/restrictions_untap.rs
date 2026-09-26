//! Untapping restrictions and choices: "You may choose not to untap ~ during your untap
//! step." (CR 502.3), "[It] doesn't untap during its controller's untap step for as long
//! as ~ remains tapped." and the older "[creature] gets +2/+2 and has trample for as long
//! as ~ remains tapped." (CR 611.2b).

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, keyword_mods, object_ref, parse_simple, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::untap_choice::MAY_CHOOSE_NOT_TO_UNTAP;

/// "You may choose not to untap ~ during your untap step."
fn may_choose_not_to_untap(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "you may choose not to untap ~ during your untap step" {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Restriction(Restriction::Custom(
        MAY_CHOOSE_NOT_TO_UNTAP.into(),
    )));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "restrictions: may choose not to untap ~", priority: 100, parse: may_choose_not_to_untap } }

/// "It doesn't untap during its controller's untap step for as long as ~ remains tapped",
/// "that permanent doesn't untap during its controller's untap step for as long as you
/// control ~": a rule-modifying effect locked onto the objects it names (CR 611.2c) that
/// lasts for the stated duration.
fn doesnt_untap_for_as_long_as(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, main) = duration_suffix(l);
    if !matches!(
        dur,
        Duration::WhileCondition(_)
            | Duration::WhileSourceOnBattlefield
            | Duration::WhileYouControlSource
    ) {
        return None;
    }
    let subject = [
        " doesn't untap during its controller's untap step",
        " don't untap during their controllers' untap steps",
        " don't untap during their controller's untap step",
    ]
    .iter()
    .find_map(|s| main.strip_suffix(s))?;
    let what = match subject {
        // The objects the previous sentence named ("Tap all other artifacts. They ...").
        "it" | "that creature" | "that permanent" | "that land" | "that artifact" | "they"
        | "those creatures" | "those permanents" | "those artifacts" | "those lands" => {
            if matches!(b.it, Sel::None | Sel::This) {
                return None;
            }
            b.it.clone()
        }
        _ => {
            let (what, tail) = object_ref(subject, b)?;
            if !end(&tail).is_empty() || matches!(what, Sel::This | Sel::None) {
                return None;
            }
            what
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::DoesntUntap(Filter::In(Box::new(what))),
        duration: dur,
    })
}

inventory::submit! { EffectPattern { name: "restrictions: doesn't untap for as long as", priority: 100, parse: doesnt_untap_for_as_long_as } }

/// "Tap all other artifacts. They don't untap during their controllers' untap steps for as
/// long as ~ remains tapped.": the plural pronoun refers to the objects the previous
/// sentence tapped, fixed as the effect begins (CR 611.2c).
fn f_they_dont_untap_for_as_long_as(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    fn tapped(e: &Effect) -> Option<Sel> {
        match e {
            Effect::Tap { what } => Some(what.clone()),
            Effect::Seq(v) => v.last().and_then(tapped),
            _ => None,
        }
    }
    let (dur, main) = duration_suffix(l);
    if !matches!(dur, Duration::WhileCondition(_)) {
        return false;
    }
    let Some(subject) = [
        " don't untap during their controllers' untap steps",
        " don't untap during their controller's untap step",
    ]
    .iter()
    .find_map(|s| main.strip_suffix(s)) else {
        return false;
    };
    if !matches!(
        subject,
        "they" | "those creatures" | "those permanents" | "those artifacts"
    ) {
        return false;
    }
    let Some(what) = tapped(prev) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::AddRestriction {
            restriction: Restriction::DoesntUntap(Filter::In(Box::new(what))),
            duration: dur,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "restrictions: they don't untap for as long as", priority: 0, apply: f_they_dont_untap_for_as_long_as } }

/// "Target Elf creature gets +2/+2 and has trample for as long as ~ remains tapped",
/// "target creature you control has shroud for as long as ~ remains tapped": in an effect
/// with a duration, "has [keyword]" grants the keyword just like "gains".
fn has_keyword_for_a_duration(l: &str, b: &mut Builder) -> Option<Effect> {
    let t = l.trim();
    let (dur, main) = duration_suffix(t);
    if matches!(dur, Duration::Permanent) {
        return None;
    }
    let suffix = &t[main.len()..];
    let (a, verb, k) = [
        (" and has ", " and gains "),
        (" and have ", " and gain "),
        (" has ", " gains "),
        (" have ", " gain "),
    ]
    .iter()
    .find_map(|(from, to)| main.split_once(from).map(|(a, k)| (a, *to, k)))?;
    // What follows "has" must be keywords only.
    keyword_mods(k)?;
    let rewritten = format!("{a}{verb}{k}");
    let e = parse_simple(&format!("{rewritten}{suffix}"), b)?;
    // Only keyword grants (and P/T changes with them) read this way.
    match &e {
        Effect::Modify { mods, .. }
            if mods
                .iter()
                .all(|m| matches!(m, Modification::AddKeyword(_) | Modification::ModifyPT(..))) =>
        {
            Some(e)
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "restrictions: has [keyword] for a duration", priority: 100, parse: has_keyword_for_a_duration } }
