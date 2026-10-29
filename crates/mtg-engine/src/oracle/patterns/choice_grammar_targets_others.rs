//! Targets chosen by a player other than the controller (CR 601.2c, 601.7):
//!
//! - "put a +1/+1 counter on target creature of defending player's choice" (Erithizon:
//!   "Erithizon's controller controls the triggered ability, but their opponent chooses
//!   the target");
//! - "target nonbasic land of an opponent's choice you don't control" (Volcanic
//!   Offering): an opponent the controller picks chooses it (CR 601.7a);
//! - "its controller chooses target creature one of their opponents controls" (Necrotic
//!   Plague), "an opponent chooses target creature they control" (Echo Chamber): a
//!   sentence that only chooses the target; "that creature" is it afterwards.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, player_ref, Builder};
use crate::oracle::phrases::*;

/// "target [object] of defending player's choice", "... of an opponent's choice".
fn target_of_players_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (phrase, chooser) = [
        (" of defending player's choice", Some(PlayerRef::DefendingPlayer)),
        (" of an opponent's choice", None),
    ]
    .into_iter()
    .find(|(p, _)| l.contains(p))?;
    let k = l.find(phrase)?;
    let before = &l[..k];
    let t = before.rfind("target ")?;
    if before[t..].contains(", ") || before[t..].contains(" and ") {
        return None;
    }
    let text = format!("{before}{}", &l[k + phrase.len()..]);
    if text.contains(phrase) {
        return None;
    }
    let first = b.targets.len();
    let e = parse_sentence(&text, b)?;
    let Some(slot) = (first..b.targets.len())
        .rev()
        .find(|i| matches!(b.targets[*i].what, TargetKind::Object(_)))
    else {
        b.targets.truncate(first);
        return None;
    };
    match chooser {
        Some(who) => b.targets[slot].chosen_by = Some(who),
        None => b.targets[slot].chosen_by_opponent = true,
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "target ... of defending player's / an opponent's choice", priority: 84, parse: target_of_players_choice } }

/// "its controller chooses target creature one of their opponents controls", "an
/// opponent chooses target creature they control": only the target is chosen, by that
/// player.
fn player_chooses_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subj, r) = l.split_once(" chooses target ")?;
    let first = b.targets.len();
    let (chooser, opponent) = if subj == "an opponent" {
        (None, true)
    } else {
        let (who, rest) = player_ref(subj, b)?;
        if !rest.trim().is_empty() || b.targets.len() != first || matches!(who, PlayerRef::You) {
            b.targets.truncate(first);
            return None;
        }
        (Some(who), false)
    };
    // Whose objects, relative to the chooser; qualifiers may follow ("target permanent
    // another player controls that shares a card type with it", Confusion in the Ranks).
    let split = |r: &str, p: &str| -> Option<String> {
        let (head, tail) = r.split_once(p)?;
        (tail.is_empty() || tail.starts_with(" that ")).then(|| format!("{head}{tail}"))
    };
    let (noun, owner) = if let Some(n) = split(r, " one of their opponents controls") {
        (n, Some(1))
    } else if let Some(n) = split(r, " another player controls") {
        (n, Some(2))
    } else if let Some(n) = r.strip_suffix(" they control") {
        (n.to_string(), Some(0))
    } else {
        (r.to_string(), None)
    };
    let target_text = format!("target {noun}");
    let (mut spec, tail) = parse_target(&target_text)?;
    if !end(tail).is_empty() {
        return None;
    }
    let TargetKind::Object(f) = &spec.what else {
        return None;
    };
    let chooser_ref = match (&chooser, opponent) {
        (Some(c), _) => Some(c.clone()),
        // The opponent choosing (CR 601.7a), as known while choosing and on resolution.
        (None, true) => Some(PlayerRef::ChosenOpponent),
        _ => None,
    };
    let f = match (owner, &chooser_ref) {
        (Some(1), Some(c)) => Filter::and(vec![
            f.clone(),
            Filter::ControllerMatches(Box::new(PlayerFilter::OpponentOf(Box::new(c.clone())))),
        ]),
        // "another player controls": any player but the chooser.
        (Some(2), Some(c)) if !matches!(c, PlayerRef::ChosenOpponent) => Filter::and(vec![
            f.clone(),
            Filter::ControllerMatches(Box::new(PlayerFilter::Not(Box::new(
                PlayerFilter::Ref(Box::new(c.clone())),
            )))),
        ]),
        (Some(0), Some(PlayerRef::ChosenOpponent)) => {
            Filter::and(vec![f.clone(), Filter::ControlledBy(PlayerRel::Chosen)])
        }
        (Some(0), Some(c)) => {
            Filter::and(vec![f.clone(), Filter::ControlledByPlayer(Box::new(c.clone()))])
        }
        (None, _) => f.clone(),
        _ => return None,
    };
    spec.what = TargetKind::Object(f);
    if opponent {
        spec.chosen_by_opponent = true;
    } else {
        spec.chosen_by = chooser;
    }
    let slot = b.add_target(spec, &target_text);
    b.it = Sel::Target(slot);
    b.chosen_creature = Some((slot, "that creature".into()));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "[player] chooses target [object]", priority: 84, parse: player_chooses_target } }
