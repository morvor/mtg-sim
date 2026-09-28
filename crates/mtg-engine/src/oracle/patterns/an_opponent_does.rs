//! "An opponent draws a card." (Baleful Mastery), "an opponent creates two Treasure tokens
//! and they scry 2" (Ingenious Mastery): an instruction for one opponent, who isn't
//! targeted — the spell's controller chooses which opponent as the effect happens (in a
//! two-player game, the only one). Parsed as the same instruction for "target opponent",
//! with that player replaced by the opponent chosen.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};

/// Replaces player target slot `slot` in `e` with the chosen opponent. `None` if the slot
/// is used other than as a player.
fn target_to_chosen_opponent(e: &Effect, slot: usize) -> Option<Effect> {
    fn subst(v: serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> serde_json::Value {
        use serde_json::Value as J;
        if &v == from {
            return to.clone();
        }
        match v {
            J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, subst(x, from, to))).collect()),
            J::Array(a) => J::Array(a.into_iter().map(|x| subst(x, from, to)).collect()),
            other => other,
        }
    }
    let json = serde_json::to_value(e).ok()?;
    let from = serde_json::to_value(PlayerRef::Target(slot as u8)).ok()?;
    let to = serde_json::to_value(PlayerRef::ChosenOpponent).ok()?;
    serde_json::from_value(subst(json, &from, &to)).ok()
}

fn an_opponent_does(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("an opponent ")?;
    // "... and they scry N": the same opponent scries.
    let (r, scry) = match r.rsplit_once(" and they scry ") {
        Some((a, n)) => {
            let (n, t) = parse_number(n)?;
            if !end(t).is_empty() {
                return None;
            }
            (a, Some(n))
        }
        None => (r, None),
    };
    let slot = b.targets.len();
    let (saved_it, saved_player) = (b.it.clone(), b.it_player.clone());
    let parsed = parse_clause(&format!("target opponent {r}"), b);
    let targeted_opponent = b.targets.len() == slot + 1
        && matches!(
            b.targets[slot].what,
            TargetKind::Player(PlayerFilter::Opponent)
        );
    b.targets.truncate(slot);
    b.it = saved_it;
    b.it_player = saved_player;
    let e = parsed.filter(|_| targeted_opponent)?;
    let e = target_to_chosen_opponent(&e, slot)?;
    let mut v = vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        },
        e,
    ];
    if let Some(n) = scry {
        v.push(Effect::Scry {
            who: PlayerRef::ChosenOpponent,
            n,
        });
    }
    b.it_player = PlayerRef::ChosenOpponent;
    Some(Effect::Seq(v))
}

inventory::submit! { EffectPattern { name: "an opponent [does something]", priority: 120, parse: an_opponent_does } }
