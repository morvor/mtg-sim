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

/// "The player to your right gains control of ~." (Hithlain Rope, Bucknard's Everfull
/// Purse): the player seated to your right (CR 101.4).
fn neighbor_gains_control(l: &str, _b: &mut Builder) -> Option<Effect> {
    let name = match end(l) {
        "the player to your right gains control of ~" => crate::kw::game_rules::RIGHT_GAINS_CONTROL,
        "the player to your left gains control of ~" => crate::kw::game_rules::LEFT_GAINS_CONTROL,
        _ => return None,
    };
    Some(Effect::Custom(name.into()))
}

inventory::submit! { EffectPattern { name: "game rules: the player to your right gains control of ~", priority: 100, parse: neighbor_gains_control } }

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
