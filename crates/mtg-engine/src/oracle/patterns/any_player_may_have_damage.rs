//! "Any player may have ~ deal 5 damage to them. If no one does, target player draws three
//! cards." (Browbeat, Book Burning, Breaking Point) and "When ~ enters, any opponent may
//! have it deal 4 damage to them. If a player does, sacrifice ~." (Vexing Devil): as the
//! spell or ability resolves, each of those players in turn order, starting with the
//! active player (CR 101.4), chooses whether to be dealt the damage; each one who does is
//! dealt it. Afterwards, the follow-up checks whether anyone did ("No player may take
//! actions between the time an opponent chooses to be dealt damage by Vexing Devil and the
//! time you sacrifice Vexing Devil.").

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};

/// Whether a player chose to be dealt the damage (0 or 1).
const CHOSE: Var = vars::USER + 1774;

fn any_player_may_have_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = [
        ("any player may have ", PlayerRef::EachPlayer),
        ("any opponent may have ", PlayerRef::EachOpponent),
    ]
    .into_iter()
    .find_map(|(p, who)| l.strip_prefix(p).map(|r| (who, r)))?;
    // The source: the spell itself, or the permanent whose ability this is ("it" in its
    // own trigger).
    let r = match r.strip_prefix("~ deal ") {
        Some(r) => r,
        None if matches!(b.it, Sel::This) => r.strip_prefix("it deal ")?,
        None => return None,
    };
    let (n, rest) = parse_number(r)?;
    if rest.trim() != "damage to them" {
        return None;
    }
    let choice = Effect::seq(vec![
        Effect::DealDamage {
            source: Sel::This,
            amount: n,
            to: Sel::Players(PlayerRef::Iterated),
        },
        Effect::StoreValue {
            var: CHOSE,
            value: Value::c(1),
        },
    ]);
    Some(Effect::Seq(vec![
        Effect::StoreValue {
            var: CHOSE,
            value: Value::c(0),
        },
        Effect::ForEachPlayer {
            who,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::May {
                    who: PlayerRef::You,
                    effect: Box::new(choice),
                }),
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "any player may have ~ deal N damage to them", priority: 100, parse: any_player_may_have_damage } }

/// "If no one does, [effect]." / "If a player does, [effect]." after it.
fn if_someone_chose(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (r, chose) = if let Some(r) = l.strip_prefix("if no one does, ") {
        (r, 0)
    } else if let Some(r) = l.strip_prefix("if a player does, ") {
        (r, 1)
    } else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    if !matches!(v.first(), Some(Effect::StoreValue { var: CHOSE, .. })) || v.len() != 2 {
        return false;
    }
    let Some(e) = parse_clause(end(r), b) else {
        return false;
    };
    v.push(Effect::If {
        cond: Condition::Compare(Value::Var(CHOSE), Cmp::Eq, Value::c(chose)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "if no one does / if a player does (damage to them)", priority: 50, apply: if_someone_chose } }
