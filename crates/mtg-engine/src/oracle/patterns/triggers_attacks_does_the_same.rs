//! "Whenever enchanted player is attacked, [instruction]. Each opponent attacking that
//! player does the same." (Curse of Vitality, Curse of Opulence, ...), and "Each opponent
//! attacking that player untaps all nonland permanents they control." (Curse of Bounty):
//! after the ability's controller, each of their opponents who is attacking the attacked
//! player (as the ability resolves) performs the instruction as its "you"
//! (`Effect::AsPlayer`).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::attacking_that_player::opponents_attacking_that_player;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

const SUBJECT: &str = "each opponent attacking that player ";

/// Each opponent attacking the event's player performs `e`.
fn each_attacking_opponent(e: Effect) -> Effect {
    Effect::ForEachPlayer {
        who: opponents_attacking_that_player(),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    }
}

/// "Each opponent attacking that player does the same.": the previous instruction again,
/// for each of them.
fn does_the_same(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(s).strip_prefix(SUBJECT) != Some("does the same") {
        return false;
    }
    let e = prev.clone();
    *prev = Effect::seq(vec![e.clone(), each_attacking_opponent(e)]);
    true
}

inventory::submit! { FollowupPattern { name: "each opponent attacking that player does the same", priority: 100, apply: does_the_same } }

/// "each opponent attacking that player untaps all nonland permanents they control": the
/// instruction as each of them would give it to themself ("untap all nonland permanents
/// you control").
fn each_attacking_opponent_does(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix(SUBJECT)?;
    let (verb, rest) = r.split_once(' ')?;
    if verb == "does" {
        return None;
    }
    let verb = verb.strip_suffix('s')?;
    let rest = format!(" {rest} ")
        .replace(" they control ", " you control ")
        .replace(" their ", " your ")
        .replace(" they ", " you ");
    let e = parse_clause(&format!("{verb}{}", rest.trim_end()), b)?;
    Some(each_attacking_opponent(e))
}

inventory::submit! { EffectPattern { name: "each opponent attacking that player [does something]", priority: 100, parse: each_attacking_opponent_does } }
