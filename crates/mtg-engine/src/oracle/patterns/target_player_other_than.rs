//! "you may have target player other than its controller create a token that's a copy of
//! it, then you draw a card if an opponent created a token this way" (Lucy MacLean,
//! Positively Armed): a target player restricted relative to the object the ability is
//! about (CR 115.1), and an instruction conditional on which player the target was.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// "... target player other than its controller ...": parsed as "target player", then the
/// new player target is restricted to players other than the controller of "it".
fn target_player_other_than_its_controller(l: &str, b: &mut Builder) -> Option<Effect> {
    const PHRASE: &str = "target player other than its controller";
    if !l.contains(PHRASE) {
        return None;
    }
    if !matches!(
        b.it,
        Sel::TriggerObject | Sel::Target(_) | Sel::Var(_) | Sel::This
    ) {
        return None;
    }
    let controller = PlayerRef::ControllerOf(Box::new(b.it.clone()));
    let first = b.targets.len();
    let text = l.replacen(PHRASE, "target player", 1);
    let Some(e) = parse_clause(&text, b) else {
        b.targets.truncate(first);
        return None;
    };
    let mut patched = false;
    for t in b.targets[first..].iter_mut() {
        if !patched && matches!(t.what, TargetKind::Player(PlayerFilter::Any)) {
            t.what = TargetKind::Player(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
                controller.clone(),
            )))));
            t.text = PHRASE.into();
            patched = true;
        }
    }
    if !patched {
        b.targets.truncate(first);
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "target player other than its controller", priority: 160, parse: target_player_other_than_its_controller } }

/// "you draw a card if an opponent created a token this way" (after "have target player
/// create a token ..."): only if the target player is an opponent.
fn if_an_opponent_created_a_token(l: &str, b: &mut Builder) -> Option<Effect> {
    let head = end(l).strip_suffix(" if an opponent created a token this way")?;
    let slot = b
        .targets
        .iter()
        .rposition(|t| matches!(t.what, TargetKind::Player(_)))?;
    let e = parse_clause(head, b)?;
    Some(Effect::If {
        cond: Condition::PlayerMatches(PlayerRef::Target(slot as u8), PlayerFilter::Opponent),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "... if an opponent created a token this way", priority: 160, parse: if_an_opponent_created_a_token } }
