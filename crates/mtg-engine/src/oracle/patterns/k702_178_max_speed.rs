//! Max speed (CR 702.178a): "Max speed — [Ability]" means "As long as your speed is 4,
//! this object has '[Ability].'" It's a keyword ability, not an ability word (CR 207.2c),
//! so the ability it grants only functions while its controller has max speed
//! (CR 702.179e):
//!
//! - a static ability applies only while you have max speed;
//! - an activated ability can be activated only while you have max speed;
//! - a triggered ability triggers only if you have max speed as its trigger event occurs
//!   (it isn't an intervening "if" clause, so it isn't checked again on resolution).
//!
//! The granted ability keeps the zones it functions from (CR 702.178b): "Max speed —
//! {3}, Exile this card from your graveyard: Draw a card." works from the graveyard.

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use std::sync::Arc;

fn and_max_speed(c: Option<Condition>) -> Condition {
    match c {
        None => Condition::MaxSpeed,
        Some(Condition::And(mut v)) => {
            v.push(Condition::MaxSpeed);
            Condition::And(v)
        }
        Some(c) => Condition::And(vec![c, Condition::MaxSpeed]),
    }
}

/// The granted ability, functioning only while its controller has max speed.
fn gated(a: &Ability, text: &str) -> Option<Ability> {
    let mut def = (**a).clone();
    match &mut def.kind {
        AbilityKind::Static(s) => s.condition = Some(and_max_speed(s.condition.take())),
        AbilityKind::Activated(act) => act.condition = Some(and_max_speed(act.condition.take())),
        AbilityKind::Triggered(t) => {
            t.trigger = TriggerCond::Where {
                trigger: Box::new(t.trigger.clone()),
                cond: Condition::MaxSpeed,
            };
        }
        // Keyword lines and anything else can't be made conditional here.
        _ => return None,
    }
    def.text = text.to_string();
    Some(Arc::new(def))
}

fn max_speed(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = text.trim();
    let rest = text.strip_prefix("Max speed — ")?;
    let inner = crate::oracle::parse_ability(rest, ctx)?;
    if inner.is_empty() {
        return None;
    }
    inner.iter().map(|a| gated(a, text)).collect()
}

inventory::submit! { AbilityPattern { name: "k702.178 max speed", priority: 50, parse: max_speed } }

/// "each player who doesn't have max speed", "each opponent who has max speed" (CR
/// 702.179e: a player has max speed if their speed is 4): the instruction for "each
/// player" / "each opponent", for just those players ("It deals 2 damage to each player
/// who doesn't have max speed.").
fn players_with_max_speed(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for (who, all, base) in [
        ("each player", PlayerRef::EachPlayer, PlayerFilter::Any),
        ("each opponent", PlayerRef::EachOpponent, PlayerFilter::Opponent),
    ] {
        for (has, negate) in [(" who has max speed", false), (" who doesn't have max speed", true)] {
            let phrase = format!("{who}{has}");
            let Some(i) = l.find(&phrase) else {
                continue;
            };
            // Only one such group, and no other mention of the same players.
            let plain = format!("{}{who}{}", &l[..i], &l[i + phrase.len()..]);
            if plain.matches(who).count() != 1 {
                return None;
            }
            let e = parse_clause(&plain, b)?;
            let fast = if negate {
                PlayerFilter::Not(Box::new(PlayerFilter::MaxSpeed))
            } else {
                PlayerFilter::MaxSpeed
            };
            let group = PlayerRef::Each(PlayerFilter::And(vec![base, fast]));
            // Those players stand in for "each player" wherever the instruction names them.
            let from = serde_json::to_string(&all).ok()?;
            let to = serde_json::to_string(&group).ok()?;
            let json = serde_json::to_string(&e).ok()?;
            if json.matches(&from).count() != 1 {
                return None;
            }
            return serde_json::from_str(&json.replace(&from, &to)).ok();
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "k702.179e each player who has max speed", priority: 200, parse: players_with_max_speed } }
