//! Follow-up sentences about whether the player an earlier instruction was given to did
//! it (CR 118.12, 608.2c, 609.3):
//!
//! - "If [you / they / that player / the player] [does / doesn't / can't / didn't],
//!   [effect]" after an instruction that player performs and that can fail or be
//!   declined: "Remove a pupa counter from ~. If you can't, sacrifice it" (Cocoon), "That
//!   player discards a card with that name. If they can't, you draw a card" (Brain Pry),
//!   "that player sacrifices it. If the player does, ~ deals 7 damage to the player"
//!   (Bronze Bombshell), "that creature's controller sacrifices it at end of combat. If the
//!   player does, they create ... token" (Basalt Golem: part of the delayed ability).
//!   "Does" is whether the player did it (an instruction a player can't follow isn't
//!   done, CR 609.3); for an optional instruction, whether they chose to (CR 118.12).
//! - The same after an instruction several players perform ("each opponent sacrifices a
//!   creature. If they can't, ..."): each player's own result, in one "each player"
//!   instruction (see `simultaneous.rs`).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// What the follow-up asks about the instruction's result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Did {
    Does,
    Doesnt,
    Cant,
}

/// "if [subject] [does|doesn't|can't], [rest]": the subject, the question, the rest.
fn split_if(l: &str) -> Option<(&str, Did, &str)> {
    let r = l.strip_prefix("if ")?;
    let (head, rest) = r.split_once(", ")?;
    for (suffix, did) in [
        (" does", Did::Does),
        (" do", Did::Does),
        (" did", Did::Does),
        (" doesn't", Did::Doesnt),
        (" don't", Did::Doesnt),
        (" didn't", Did::Doesnt),
        (" can't", Did::Cant),
    ] {
        if let Some(s) = head.strip_suffix(suffix) {
            if matches!(s, "you" | "they" | "that player" | "the player" | "that opponent") {
                return Some((s, did, rest));
            }
        }
    }
    None
}

/// The players a primitive's `who` names, if it has one.
fn who_of(e: &Effect) -> Option<&PlayerRef> {
    match e {
        Effect::Sacrifice { who, .. }
        | Effect::Discard { who, .. }
        | Effect::DiscardHand { who }
        | Effect::May { who, .. }
        | Effect::PayOptional { who, .. }
        | Effect::AsPlayer { who, .. } => Some(who),
        _ => None,
    }
}

/// Whether `e`, performed by "you", records whether it was done as the last thing it
/// does (`ctx.prev_happened`): an optional instruction, or one that can fail.
fn records(e: &Effect) -> bool {
    matches!(
        e,
        Effect::May { .. }
            | Effect::PayOptional { .. }
            | Effect::Exile { .. }
            | Effect::Sacrifice { .. }
            | Effect::SacrificeObjects { .. }
            | Effect::Move { .. }
            | Effect::Discard { .. }
            | Effect::RemoveCounters { .. }
            | Effect::ShuffleIntoLibrary { .. }
            | Effect::TurnFaceUp { .. }
            | Effect::CastCard { .. }
    )
}

/// Whether the instruction is optional ("can't" doesn't follow a "may").
fn optional(e: &Effect) -> bool {
    match e {
        Effect::May { .. } | Effect::PayOptional { .. } => true,
        Effect::AsPlayer { effect, .. } => optional(effect),
        _ => false,
    }
}

/// The last instruction of `e` (inside sequences and the delayed triggered ability an
/// instruction "at end of combat" creates).
fn last_instruction(e: &mut Effect) -> &mut Effect {
    let descend = match &*e {
        Effect::Seq(v) => v.last().is_some_and(|x| !bookkeeping(x)),
        Effect::DelayedTrigger { body, .. } => {
            body.modal.is_none() && !matches!(body.effect, Effect::Noop)
        }
        Effect::AtNext { .. } => true,
        _ => false,
    };
    if !descend {
        return e;
    }
    match e {
        Effect::Seq(v) => {
            let n = v.len() - 1;
            last_instruction(&mut v[n])
        }
        Effect::DelayedTrigger { body, .. } => last_instruction(&mut body.effect),
        Effect::AtNext { effect, .. } => last_instruction(effect),
        e => e,
    }
}

/// "That creature's controller sacrifices it at end of combat. If the player does, ...":
/// the player performs the instruction when it's performed, later, and the follow-up is
/// part of what happens then. Moves the player into the later instruction.
fn delay_inside(e: &mut Effect) {
    let Effect::AsPlayer { who, effect } = e else {
        return;
    };
    let (stores, at): (Vec<Effect>, Effect) = match &**effect {
        Effect::AtNext { .. } => (vec![], (**effect).clone()),
        Effect::Seq(v)
            if matches!(v.last(), Some(Effect::AtNext { .. }))
                && v[..v.len() - 1]
                    .iter()
                    .all(|x| matches!(x, Effect::Store { .. })) =>
        {
            (v[..v.len() - 1].to_vec(), v[v.len() - 1].clone())
        }
        _ => return,
    };
    let Effect::AtNext { step, effect: inner } = at else {
        return;
    };
    let mut out = stores;
    out.push(Effect::AtNext {
        step,
        effect: Box::new(Effect::AsPlayer {
            who: who.clone(),
            effect: inner,
        }),
    });
    *e = Effect::seq(out);
}

/// An instruction that only records results for later ones (and leaves whether the
/// previous one was done as it was).
fn bookkeeping(e: &Effect) -> bool {
    match e {
        Effect::Store { .. } | Effect::StoreValue { .. } | Effect::Note { .. } => true,
        Effect::Custom(n) => n == crate::kw::hand_graveyard_actions::RECORD_THAT_MANY,
        _ => false,
    }
}

/// The instruction a sequence ends with, before the bookkeeping after it.
fn last_action(v: &[Effect]) -> Option<&Effect> {
    v.iter().rev().find(|x| !bookkeeping(x))
}

/// Who performs `e`, if it records whether it was done.
fn performer(e: &Effect) -> Option<PlayerRef> {
    match e {
        Effect::Seq(v) => match last_action(v)? {
            x if records(x) => performer(x),
            _ => None,
        },
        Effect::AsPlayer { who, effect } if matches!(**effect, Effect::Seq(_)) => {
            let Effect::Seq(v) = &**effect else {
                return None;
            };
            last_action(v)
                .filter(|x| records(x) && matches!(who_of(x), None | Some(PlayerRef::You)))
                .map(|_| who.clone())
        }
        Effect::AsPlayer { who, effect } => {
            // "that player sacrifices it" performed by that player.
            match &**effect {
                Effect::If {
                    then, otherwise, ..
                } if matches!(**otherwise, Effect::Noop) && records(then) => Some(who.clone()),
                inner if records(inner) => Some(who.clone()),
                _ => None,
            }
        }
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => performer(then),
        e if records(e) => Some(match who_of(e) {
            Some(PlayerRef::You) | None => PlayerRef::You,
            Some(w) => w.clone(),
        }),
        _ => None,
    }
}

fn is_multi(p: &PlayerRef) -> bool {
    matches!(
        p,
        PlayerRef::EachOpponent
            | PlayerRef::EachPlayer
            | PlayerRef::EachOtherPlayer
            | PlayerRef::Each(_)
    )
}

/// The condition for `did`.
fn cond(did: Did) -> Condition {
    match did {
        Did::Does => Condition::PrevHappened,
        Did::Doesnt | Did::Cant => Condition::Not(Box::new(Condition::PrevHappened)),
    }
}

/// `x` followed by the effect for `did`. An instruction that's only performed if a
/// condition holds ("if it's still under that player's control") isn't done when it
/// doesn't: then "doesn't" and "can't" hold too.
fn follow(x: Effect, did: Did, then: Effect) -> Option<Effect> {
    let gated = |inner: Effect| {
        Effect::seq(vec![
            inner,
            Effect::If {
                cond: cond(did),
                then: Box::new(then.clone()),
                otherwise: Box::new(Effect::Noop),
            },
        ])
    };
    Some(match x {
        Effect::AsPlayer { who, effect } => match *effect {
            Effect::If {
                cond: c,
                then: t,
                otherwise,
            } if matches!(*otherwise, Effect::Noop) => Effect::If {
                cond: as_player_cond(&c, &who)?,
                then: Box::new(gated(Effect::AsPlayer { who, effect: t })),
                otherwise: Box::new(if did == Did::Does {
                    Effect::Noop
                } else {
                    then.clone()
                }),
            },
            inner => gated(Effect::AsPlayer {
                who,
                effect: Box::new(inner),
            }),
        },
        Effect::If {
            cond: c,
            then: t,
            otherwise,
        } if matches!(*otherwise, Effect::Noop) => Effect::If {
            cond: c,
            then: Box::new(follow(*t, did, then.clone())?),
            otherwise: Box::new(if did == Did::Does {
                Effect::Noop
            } else {
                then
            }),
        },
        x => gated(x),
    })
}

/// The condition `c` of an instruction `who` performs, read outside it: "if it's still
/// under your control" there is "if it's still under that player's control". None if it
/// refers to "you" any other way.
fn as_player_cond(c: &Condition, who: &PlayerRef) -> Option<Condition> {
    let mut j = serde_json::to_value(c).ok()?;
    let w = serde_json::to_value(Filter::ControlledByPlayer(Box::new(who.clone()))).ok()?;
    fn walk(j: &mut serde_json::Value, w: &serde_json::Value) {
        match j {
            serde_json::Value::Object(m) => {
                if m.len() == 1 && m.get("ControlledBy").is_some_and(|x| x == "You") {
                    *j = w.clone();
                    return;
                }
                m.values_mut().for_each(|x| walk(x, w));
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, w)),
            _ => {}
        }
    }
    walk(&mut j, &w);
    if j.to_string().contains("\"You\"") {
        return None;
    }
    serde_json::from_value(j).ok()
}

/// `e` with its players `who` replaced by [`PlayerRef::Iterated`].
fn for_iterated(e: &Effect) -> Option<Effect> {
    let mut e = e.clone();
    match &mut e {
        Effect::Sacrifice { who, .. } | Effect::Discard { who, .. } => *who = PlayerRef::Iterated,
        _ => return None,
    }
    Some(e)
}

/// "If [player] does/doesn't/can't, [effect]".
fn if_player_did(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((subject, did, rest)) = split_if(l) else {
        return false;
    };
    let last = last_instruction(prev);
    delay_inside(last);
    let last = last_instruction(last);
    let Some(who) = performer(last) else {
        return false;
    };
    if did == Did::Cant && optional(last) {
        return false;
    }
    let you = subject == "you";
    if you != matches!(who, PlayerRef::You) {
        return false;
    }
    // "If you do" after "you may" is the core's.
    if you && did == Did::Does && optional(last) {
        return false;
    }
    // Whether the player did it is whether they did all of it: an instruction for
    // several cards they can't all choose isn't done ("exile that many cards from your
    // graveyard. If you can't, ...": the cards are exiled only if there are enough, as
    // for a cost, CR 118.12). Other amounts aren't recorded that way.
    let mut enough = None;
    match action_of(last) {
        Some(Effect::Exile {
            what:
                Sel::Choose {
                    filter,
                    count,
                    up_to: false,
                    chooser: PlayerRef::You,
                    ..
                },
            ..
        })
        | Some(Effect::Move {
            what:
                Sel::Choose {
                    filter,
                    count,
                    up_to: false,
                    chooser: PlayerRef::You,
                    ..
                },
            ..
        }) if !matches!(count, Value::Const(1)) => {
            if !you || matches!(last, Effect::AsPlayer { .. }) {
                return false;
            }
            enough = Some(Condition::Compare(
                Value::Count(filter.clone()),
                Cmp::Ge,
                count.clone(),
            ));
        }
        Some(Effect::Exile { what, .. }) | Some(Effect::Move { what, .. })
            if matches!(what, Sel::Choose { .. }) =>
        {
            if !matches!(what, Sel::Choose { count: Value::Const(1), up_to: false, .. }) {
                return false;
            }
        }
        Some(Effect::Sacrifice { count: n, .. }) | Some(Effect::Discard { n, .. })
            if !matches!(n, Value::Const(1)) =>
        {
            return false;
        }
        _ => {}
    }
    let multi = is_multi(&who);
    let player = if multi {
        PlayerRef::Iterated
    } else {
        who.clone()
    };
    // "~ deals 7 damage to the player": that player.
    let rest = format!(" {rest}")
        .replace(" the player's ", " that player's ")
        .replace(" the player ", " that player ")
        .replace(" to the player", " to that player");
    let rest = rest.trim();
    let saved = std::mem::replace(&mut b.it_player, player);
    let parsed = if you {
        b.it_player = saved.clone();
        parse_sentence(rest, b)
    } else {
        parse_sentence(rest, b)
    };
    b.it_player = saved;
    let Some(then) = parsed else {
        return false;
    };
    let x = std::mem::take(last);
    if multi {
        // Each player's own result (CR 101.4, 608.2e).
        let Some(inner) = for_iterated(&x) else {
            *last = x;
            return false;
        };
        let Some(body) = follow(inner, did, then) else {
            *last = x;
            return false;
        };
        *last = Effect::ForEachPlayer {
            who,
            effect: Box::new(body),
        };
        return true;
    }
    *last = match enough {
        Some(c) => Effect::If {
            cond: c,
            then: Box::new(if did == Did::Does {
                Effect::seq(vec![x, then.clone()])
            } else {
                x
            }),
            otherwise: Box::new(if did == Did::Does { Effect::Noop } else { then }),
        },
        None => match follow(x.clone(), did, then) {
            Some(e) => e,
            None => {
                *last = x;
                return false;
            }
        },
    };
    true
}

/// The primitive `e` performs last (through players performing it, conditions and the
/// bookkeeping after it).
fn action_of(e: &Effect) -> Option<&Effect> {
    match e {
        Effect::Seq(v) => last_action(v).and_then(action_of),
        Effect::AsPlayer { effect, .. } => action_of(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => action_of(then),
        e => Some(e),
    }
}

inventory::submit! { FollowupPattern { name: "followups: if [player] does/doesn't/can't", priority: 970, apply: if_player_did } }

#[cfg(test)]
mod tests {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    /// The compiled abilities of `text` on a card of type `ty`, or None if any of it is
    /// unsupported.
    pub(crate) fn compiled(ty: &str, text: &str) -> Option<String> {
        let tl = TypeLine::parse(ty);
        let ctx = CompileContext {
            card_name: "Test Card",
            full_name: "Test Card",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let r = compile(text, &ctx);
        r.unsupported
            .is_empty()
            .then(|| format!("{:?}", r.abilities))
    }

    #[test]
    fn if_the_player_did() {
        for (ty, text) in [
            ("Enchantment", "At the beginning of your upkeep, remove a pupa counter from ~. If you can't, sacrifice it."),
            ("Sorcery", "Target player discards a card. If they can't, you draw a card."),
            ("Sorcery", "Target opponent sacrifices a creature. If they can't, they lose 2 life and you draw a card."),
            ("Sorcery", "Each opponent sacrifices a creature. If they can't, they lose 2 life."),
            ("Sorcery", "Target opponent sacrifices a creature. If that player does, you gain 2 life."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
        // Nothing records whether a life gain was "done".
        assert!(compiled("Sorcery", "You gain 2 life. If you can't, draw a card.").is_none());
        // "Can't" doesn't follow a choice.
        assert!(compiled("Sorcery", "You may draw a card. If you can't, you gain 2 life.").is_none());
    }
}
