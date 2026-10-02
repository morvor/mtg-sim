//! Target players described by a comparison with you ("choose target opponent who has
//! more life than you do as you activate this ability", the Keepers): the requirement is
//! a [`PlayerFilter`]; "as you activate this ability" makes it apply only as the target is
//! chosen, not as the ability resolves (`PlayerFilter::AsChosen`; the Keepers' rulings:
//! "It is only necessary that the condition be true as you activate the ability").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "who has more life than you [do]", "who has at least two more cards in hand than you
/// [do]", "who controls more creatures than you [do]": a comparison of the player with
/// you.
pub fn compared_with_you(s: &str) -> Option<PlayerFilter> {
    let s = s.trim();
    let s = s.strip_suffix(" do").unwrap_or(s);
    let r = s.strip_suffix(" than you")?;
    if let Some(r) = r.strip_prefix("who has ") {
        // "at least two more", "more"
        let (cmp, extra, r) = if let Some(x) = r.strip_prefix("at least ") {
            let (n, x) = parse_number(x)?;
            let x = x.trim_start().strip_prefix("more ")?;
            (Cmp::Ge, Some(n), x)
        } else {
            (Cmp::Gt, None, r.strip_prefix("more ")?)
        };
        let theirs_vs = |you: Value| match extra.clone() {
            Some(n) => Value::Sum(vec![you, n]),
            None => you,
        };
        return match r {
            "life" => Some(PlayerFilter::Life(
                cmp,
                Box::new(theirs_vs(Value::LifeTotal(PlayerRef::You))),
            )),
            "cards in hand" => Some(PlayerFilter::HandSize(
                cmp,
                Box::new(theirs_vs(Value::HandSize(PlayerRef::You))),
            )),
            _ => None,
        };
    }
    let noun = r.strip_prefix("who controls more ")?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some(PlayerFilter::Controls(
        Box::new(f.clone()),
        Cmp::Gt,
        Box::new(Value::Count(Filter::and(vec![
            f,
            Filter::ControlledBy(PlayerRel::You),
        ]))),
    ))
}

/// "Choose target opponent who has more life than you do as you activate this ability."
fn choose_target_player_who(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (spec, tail) = parse_target(r)?;
    let TargetKind::Player(base) = &spec.what else {
        return None;
    };
    if spec.fixed_min() != Some(1) || !matches!(spec.max, Value::Const(1)) {
        return None;
    }
    let tail = tail.trim();
    let (pred, as_chosen) = match tail.strip_suffix(" as you activate this ability") {
        Some(p) => (p, true),
        None => (tail, false),
    };
    let f = compared_with_you(pred)?;
    let f = if as_chosen {
        PlayerFilter::AsChosen(Box::new(f))
    } else {
        f
    };
    let what = match base {
        PlayerFilter::Any => f,
        other => PlayerFilter::And(vec![other.clone(), f]),
    };
    let text = r.trim().to_string();
    let spec = TargetSpec {
        what: TargetKind::Player(what),
        ..spec
    };
    let slot = b.add_target(spec, &text);
    b.it_player = PlayerRef::Target(slot);
    b.named.push((
        "the chosen player".into(),
        Sel::Players(PlayerRef::Target(slot)),
    ));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose target player who [compared with you]", priority: 86, parse: choose_target_player_who } }
