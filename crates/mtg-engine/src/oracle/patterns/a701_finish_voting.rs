//! "Whenever players finish voting, each opponent who voted for a choice you didn't vote
//! for loses 2 life." (Grudge Keeper), "... each opponent who voted for a choice you voted
//! for creates a Treasure token" (Erestor of the Council), "you and each opponent who
//! voted for a choice you voted for may scry 2" (Model of Unity): a trigger on the end of
//! any vote (CR 701.38; `kwa::vote::FINISHED_VOTING`, whatever the vote's controller) and
//! the players who voted like or unlike you in that vote (`PlayerFilter::VotedLikeYou`).

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

fn finish_voting(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "players finish voting").then(|| {
        (
            TriggerCond::PlayerAction {
                name: SmolStr::new(crate::kwa::vote::FINISHED_VOTING),
                who: PlayerRel::Any,
            },
            Sel::None,
            PlayerRef::TriggerPlayer,
        )
    })
}

inventory::submit! { TriggerPattern { name: "a701.38 whenever players finish voting", priority: 60, parse: finish_voting } }

/// "each opponent who voted for a choice you (didn't) vote for [instruction]": the
/// instruction for "each opponent", for just those opponents.
fn opponents_who_voted(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for (has, like) in [
        (" who voted for a choice you voted for", true),
        (" who voted for a choice you didn't vote for", false),
    ] {
        let phrase = format!("each opponent{has}");
        let Some(i) = l.find(&phrase) else {
            continue;
        };
        let plain = format!("{}each opponent{}", &l[..i], &l[i + phrase.len()..]);
        if plain.matches("each opponent").count() != 1 {
            return None;
        }
        let e = parse_clause(&plain, b)?;
        let group = PlayerRef::Each(PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::VotedLikeYou(like),
        ]));
        let from = serde_json::to_string(&PlayerRef::EachOpponent).ok()?;
        let to = serde_json::to_string(&group).ok()?;
        let json = serde_json::to_string(&e).ok()?;
        if json.matches(&from).count() != 1 {
            return None;
        }
        return serde_json::from_str(&json.replace(&from, &to)).ok();
    }
    None
}

inventory::submit! { EffectPattern { name: "a701.38 each opponent who voted for a choice you (didn't) vote for", priority: 200, parse: opponents_who_voted } }
