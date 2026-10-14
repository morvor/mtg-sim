//! Extra turns (CR 500.7): "Take two extra turns after this one." (Teferi, Master of
//! Time): two extra turns, each added directly after this one (the second one taken
//! first, CR 500.7 — they're identical, so the order doesn't matter).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn take_n_extra_turns(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("take ")?;
    let (n, r) = parse_number(r)?;
    if end(r) != "extra turns after this one" || matches!(n, Value::X) {
        return None;
    }
    Some(Effect::Repeat {
        times: n,
        effect: Box::new(Effect::ExtraTurn {
            who: PlayerRef::You,
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: take N extra turns after this one", priority: 100, parse: take_n_extra_turns } }

/// "Until your next upkeep, ~ can't phase out.", "Until the end of your next turn, they
/// can't phase in.": the same instruction with its duration written after it (read by
/// the duration suffixes).
fn leading_duration_moved(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (rest, suffix) = [
        ("until your next upkeep, ", " until your next upkeep"),
        ("until the end of your next turn, ", " until the end of your next turn"),
    ]
    .into_iter()
    .find_map(|(p, s)| l.strip_prefix(p).map(|r| (r, s)))?;
    if rest.contains(", ") || rest.contains(" and ") {
        return None;
    }
    crate::oracle::effects::parse_simple(&format!("{rest}{suffix}"), b)
}

inventory::submit! { EffectPattern { name: "game rules: until your next upkeep, [instruction]", priority: 100, parse: leading_duration_moved } }

/// "~ and that creature phase out" (Dream Fighter): both phase out (CR 702.26).
fn source_and_it_phase_out(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "~ and that creature phase out" || matches!(b.it, Sel::This | Sel::None) {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::PhaseOut { what: Sel::This },
        Effect::PhaseOut { what: b.it.clone() },
    ]))
}

inventory::submit! { EffectPattern { name: "game rules: ~ and that creature phase out", priority: 100, parse: source_and_it_phase_out } }

/// "~ can't attack during extra turns." (Medomai the Ageless): the restriction applies
/// while the current turn is an extra turn (CR 500.7).
fn during_extra_turns(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_suffix(" during extra turns")?;
    let parsed = crate::oracle::statics::parse_static(r, ctx)?;
    parsed
        .into_iter()
        .map(|a| {
            let AbilityKind::Static(s) = &a.kind else {
                return None;
            };
            if s.condition.is_some() {
                return None;
            }
            let mut s = s.clone();
            s.condition = Some(Condition::Custom(crate::kw::game_rules::EXTRA_TURN.into()));
            Some(AbilityDef::new(AbilityKind::Static(s), text))
        })
        .collect()
}

inventory::submit! { super::StaticPattern { name: "game rules: [static] during extra turns", priority: 100, parse: during_extra_turns } }
