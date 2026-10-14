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
    // "If you do/don't" after "you may" is the core's.
    if you && optional(last) {
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

/// "they discard a card, they lose 2 life, you draw a card, then you sacrifice ~", "amass
/// Orcs 5, mill five cards, then return a creature card from your graveyard to the
/// battlefield": a series of three or more instructions separated by commas, the last
/// after ", then" (each one performed in order, CR 608.2c).
fn serial_then(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, last) = l.rsplit_once(", then ")?;
    if head.contains('"') || last.contains('"') {
        return None;
    }
    let parts: Vec<&str> = head.split(", ").collect();
    if parts.len() < 2 || parts.iter().any(|p| p.is_empty() || p.starts_with("and ")) {
        return None;
    }
    // Third-person verbs need a subject: "puts it onto the battlefield" isn't an
    // instruction of its own.
    const THIRD: &[&str] = &[
        "puts", "shuffles", "draws", "discards", "mills", "sacrifices", "exiles", "returns",
        "creates", "gains", "loses", "reveals", "searches", "pays", "taps", "untaps",
    ];
    if parts
        .iter()
        .chain(std::iter::once(&last))
        .any(|p| THIRD.contains(&p.split(' ').next().unwrap_or("")))
    {
        return None;
    }
    let saved = (
        b.targets.len(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    let named = b.named.len();
    let mut acc = Effect::Noop;
    let n = parts.len();
    for (i, p) in parts.into_iter().chain(std::iter::once(last)).enumerate() {
        let parsed = if i == n && p.starts_with("you may ") {
            parse_sentence(p, b)
        } else {
            crate::oracle::effects::parse_simple(p, b)
        };
        if let Some(e) = parsed {
            acc = Effect::seq(vec![acc, e]);
            continue;
        }
        if i > 0
            && !matches!(acc, Effect::Noop)
            && crate::oracle_ext::apply_followup_ext(p, &mut acc, b)
        {
            continue;
        }
        b.targets.truncate(saved.0);
        b.named.truncate(named);
        (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
        return None;
    }
    Some(acc)
}

inventory::submit! { super::EffectPattern { name: "followups: A, B, then C (a series of instructions)", priority: 990, parse: serial_then } }

/// "Choose a color. Add one mana of that color unless any player pays {1}." (Rhystic
/// Cave): the chosen color.
fn mana_of_that_color(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" mana of that color") {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let first = l.split(" mana of that color").next()?;
    if !raw.contains(&format!("choose a color. {first} mana of that color")) {
        return None;
    }
    crate::oracle::effects::parse_clause(&l.replace(" mana of that color", " mana of the chosen color"), b)
}

inventory::submit! { super::EffectPattern { name: "followups: add mana of that color (the chosen color)", priority: 980, parse: mana_of_that_color } }

/// "Then that player discards another card at random unless they pay {1}." (Flay), "draw
/// another card" (Hymn of the Faller): a card other than the one an earlier instruction
/// had them discard or draw — which is no longer in their hand or library anyway.
fn another_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let verbs = [
        "discards another card",
        "discard another card",
        "draws another card",
        "draw another card",
    ];
    let v = verbs.iter().find(|v| l.contains(*v))?;
    let raw = crate::oracle::raw_text().to_lowercase();
    // Only after the same instruction with "a card".
    let first = v.replace("another card", "a card");
    let base = first.trim_end_matches(" a card").trim_end_matches('s');
    if !raw.contains(&format!("{base} a card")) && !raw.contains(&format!("{base}s a card")) {
        return None;
    }
    crate::oracle::effects::parse_clause(&l.replace("another card", "a card"), b)
}

inventory::submit! { super::EffectPattern { name: "followups: discard/draw another card", priority: 980, parse: another_card } }

/// "For each delay counter removed this way, you lose 1 life unless you pay {1}{W}."
/// (Delaying Shield): the punisher once for each of them, each with its own choice
/// (CR 118.12a). The count is the one "for each [thing] this way" has without the
/// "unless" part.
fn for_each_this_way_unless(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("for each ") else {
        return false;
    };
    let Some((thing, rest)) = r.split_once(", ") else {
        return false;
    };
    if !thing.ends_with(" this way") || !rest.contains(" unless ") {
        return false;
    }
    // The count, read from the same sentence with a plain instruction.
    let mut counted = prev.clone();
    if !crate::oracle_ext::apply_followup_ext(&format!("for each {thing}, draw a card"), &mut counted, b) {
        return false;
    }
    let Effect::Seq(mut v) = counted else {
        return false;
    };
    let Some(Effect::Draw {
        who: PlayerRef::You,
        n,
    }) = v.pop()
    else {
        return false;
    };
    // "for each" counts with a variable the earlier instructions store.
    if !matches!(n, Value::Var(_)) {
        return false;
    }
    let Some(each) = crate::oracle::effects::parse_clause(rest, b) else {
        return false;
    };
    if !matches!(each, Effect::PayOptional { .. }) {
        return false;
    }
    v.push(Effect::Repeat {
        times: n,
        effect: Box::new(each),
    });
    *prev = Effect::seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "followups: for each [thing] this way, [effect] unless [payment]", priority: 30, apply: for_each_this_way_unless } }

// ---------------------------------------------------------------------------
// "Any opponent may have you [instruction]. If a player does, ... Otherwise, ..."
// ---------------------------------------------------------------------------

/// Whether an opponent has had you do it yet (0 or 1).
const HAD: Var = vars::USER + 9651;
/// The opponent who had you do it.
const HAD_BY: Var = vars::USER + 9652;

/// "Any opponent may have you put that card into your graveyard." (Sin Prodder,
/// Distant Memories): starting with the next opponent in turn order, each opponent may
/// have you do it, until one does (Sin Prodder ruling).
fn any_opponent_may_have_you(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("any opponent may have you ")?;
    let e = crate::oracle::effects::parse_clause(r, b)?;
    Some(Effect::seq(vec![
        Effect::StoreValue {
            var: HAD,
            value: Value::c(0),
        },
        Effect::Store {
            var: HAD_BY,
            sel: Sel::None,
        },
        Effect::InTurnOrder {
            first: TurnOrderStart::NextOpponent,
            who: PlayerFilter::Opponent,
            effect: Box::new(Effect::If {
                cond: Condition::Compare(Value::Var(HAD), Cmp::Eq, Value::c(0)),
                then: Box::new(Effect::May {
                    who: PlayerRef::Iterated,
                    effect: Box::new(Effect::seq(vec![
                        e,
                        Effect::StoreValue {
                            var: HAD,
                            value: Value::c(1),
                        },
                        Effect::Store {
                            var: HAD_BY,
                            sel: Sel::Players(PlayerRef::Iterated),
                        },
                    ])),
                }),
                otherwise: Box::new(Effect::Noop),
            }),
            direction: SeatSide::default(),
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "followups: any opponent may have you [instruction]", priority: 120, parse: any_opponent_may_have_you } }

fn after_had_you(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.iter().any(|x| {
            matches!(x, Effect::StoreValue { var, .. } if *var == HAD)
        }),
        _ => false,
    }
}

/// "If a player does, [effect]" / "Otherwise, [effect]" / "If no player does, [effect]"
/// after "any opponent may have you [instruction]".
fn if_a_player_had_you(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (did, r) = if let Some(r) = l.strip_prefix("if a player does, ") {
        (true, r)
    } else if let Some(r) = l
        .strip_prefix("otherwise, ")
        .or_else(|| l.strip_prefix("if no player does, "))
    {
        (false, r)
    } else {
        return false;
    };
    if !after_had_you(prev) {
        return false;
    }
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Var(HAD_BY));
    let e = crate::oracle::effects::parse_clause(r, b);
    b.it_player = saved;
    let Some(e) = e else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    v.push(Effect::If {
        cond: Condition::Compare(
            Value::Var(HAD),
            Cmp::Eq,
            Value::c(if did { 1 } else { 0 }),
        ),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "followups: if a player does / otherwise (had you)", priority: 30, apply: if_a_player_had_you } }

/// "Then if any player pays {2}, discard three cards." (Rhystic Scrying): each player in
/// turn order gets the option to pay; the effect happens if one does (CR 118.12).
fn if_any_player_pays(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("if any player pays ")?;
    let (cost, rest) = r.split_once(", ")?;
    let cost = super::counters_resources_pay::resolution_cost(cost)?;
    let e = crate::oracle::effects::parse_clause(rest, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::EachPlayer,
        cost,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { super::EffectPattern { name: "followups: if any player pays [cost], [effect]", priority: 120, parse: if_any_player_pays } }

/// "Then for each blight counter on it, each opponent loses 4 life unless that player
/// sacrifices a nonland permanent of their choice or discards a card." (Rottenmouth
/// Viper): the instruction once for each of those counters, each time with its own
/// choices.
fn for_each_counter_on_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("for each ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counter on ")?;
    let (who, rest) = if let Some(x) = r.strip_prefix("it, ") {
        if !matches!(b.it, Sel::This | Sel::TriggerObject) {
            return None;
        }
        (b.it.clone(), x)
    } else {
        (Sel::This, r.strip_prefix("~, ")?)
    };
    // Only an instruction with choices of its own each time.
    if !rest.contains(" unless ") {
        return None;
    }
    // "That player" in it is each opponent, not a player the trigger names.
    let saved = std::mem::replace(
        &mut b.it_player,
        super::oracle_hardening_referents::no_player_referent(),
    );
    let rest = rest.replace(" unless that player ", " unless they ");
    let e = crate::oracle::effects::parse_clause(&rest, b);
    b.it_player = saved;
    let e = e?;
    Some(Effect::Repeat {
        times: Value::CountersOn(Box::new(who), Some(kind)),
        effect: Box::new(e),
    })
}

inventory::submit! { super::EffectPattern { name: "followups: for each [kind] counter on it, [instruction with a choice]", priority: 120, parse: for_each_counter_on_it } }

/// "Tap all untapped permanents of the chosen type target player controls, or untap all
/// tapped permanents of that type that player controls." (Turnabout), "return an
/// enchantment card from your graveyard to your hand or unlock a locked door of a Room
/// you control" (Ghostly Dancers): the controller chooses one of the instructions as the
/// ability resolves (CR 608.2d).
fn either_instruction(l: &str, b: &mut Builder) -> Option<Effect> {
    const VERBS: &[&str] = &[
        "tap", "untap", "return", "unlock", "put", "remove", "exile", "destroy", "draw",
        "create", "sacrifice", "gain", "lose", "discard", "mill", "search",
    ];
    let l = end(l);
    let first = l.split(' ').next()?;
    if !VERBS.contains(&first) || l.contains('"') {
        return None;
    }
    for sep in [", or ", " or "] {
        for (i, _) in l.match_indices(sep) {
            let (a, c) = (&l[..i], &l[i + sep.len()..]);
            let verb = c.split(' ').next().unwrap_or("");
            if !VERBS.contains(&verb) {
                continue;
            }
            let saved = (b.targets.len(), b.it.clone(), b.it_player.clone(), b.group.clone());
            let ea = crate::oracle::effects::parse_clause(a, b);
            let ec = ea
                .as_ref()
                .and_then(|_| crate::oracle::effects::parse_clause(c, b));
            if let (Some(ea), Some(ec)) = (ea, ec) {
                return Some(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options: vec![(a.to_string(), ea), (c.to_string(), ec)],
                });
            }
            b.targets.truncate(saved.0);
            (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
        }
    }
    None
}

inventory::submit! { super::EffectPattern { name: "followups: [instruction] or [instruction]", priority: 985, parse: either_instruction } }



/// The selections "Simultaneously untap ... and tap ..." acts on, fixed before either
/// happens.
const UNTAPPING: Var = vars::USER + 9653;
const TAPPING: Var = vars::USER + 9654;

/// "Simultaneously untap all tapped creatures and tap all untapped creatures." (Breaking
/// Wave): which permanents are untapped and which are tapped is determined before either
/// happens, so each one changes once.
fn simultaneously_untap_and_tap(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("simultaneously ")?;
    let (a, c) = r.split_once(" and ")?;
    let ea = crate::oracle::effects::parse_clause(a, b)?;
    let ec = crate::oracle::effects::parse_clause(c, b)?;
    let (Effect::Untap { what: Sel::All(fa) }, Effect::Tap { what: Sel::All(fc) }) = (&ea, &ec)
    else {
        return None;
    };
    Some(Effect::seq(vec![
        Effect::Store {
            var: UNTAPPING,
            sel: Sel::All(fa.clone()),
        },
        Effect::Store {
            var: TAPPING,
            sel: Sel::All(fc.clone()),
        },
        Effect::Untap {
            what: Sel::Var(UNTAPPING),
        },
        Effect::Tap {
            what: Sel::Var(TAPPING),
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "followups: simultaneously untap ... and tap ...", priority: 120, parse: simultaneously_untap_and_tap } }

/// Whether the last instruction of `e` is a surveil by you.
fn surveils(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(surveils),
        Effect::Surveil {
            who: PlayerRef::You,
            ..
        }
        | Effect::Scry {
            who: PlayerRef::You,
            ..
        } => true,
        _ => false,
    }
}

/// "Surveil 1. If you put a noncreature, nonland card into your graveyard this way, put
/// that card into your hand." (Chandra, Chill of Compliance; Enlightened Confidant): the
/// cards the surveil put into the graveyard (CR 701.25a).
fn if_you_put_into_graveyard_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if you put ") else {
        return false;
    };
    let Some((desc, rest)) = r.split_once(" into your graveyard this way, ") else {
        return false;
    };
    if !matches!(last_instruction(prev), Effect::Surveil { .. }) || !surveils(prev) {
        return false;
    }
    let desc = desc
        .strip_prefix("a ")
        .or_else(|| desc.strip_prefix("an "))
        .unwrap_or(desc);
    let Some(q) = super::card_flow_search::card_filter(desc, b) else {
        return false;
    };
    let those = Sel::Matching(
        Box::new(Sel::Var(crate::scry_rules::SURVEILED_TO_GRAVEYARD)),
        q,
    );
    let named = b.named.len();
    b.named.push(("that card".into(), those.clone()));
    let e = crate::oracle::effects::parse_clause(rest, b);
    b.named.truncate(named);
    let Some(e) = e else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::SelNonEmpty(those),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups: if you put a card into your graveyard this way (surveil)", priority: 120, apply: if_you_put_into_graveyard_this_way } }

/// "Surveil 2. Then for each card you put on top of your library, you draw a card and you
/// lose 3 life." (Starving Revenant).
fn for_each_card_put_on_top(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(rest) = l.strip_prefix("for each card you put on top of your library, ") else {
        return false;
    };
    if !surveils(prev) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_clause(rest, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Repeat {
            times: Value::CountSel(Box::new(Sel::Var(crate::scry_rules::LOOKED_ON_TOP))),
            effect: Box::new(e),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups: for each card you put on top of your library (surveil)", priority: 120, apply: for_each_card_put_on_top } }

/// "Target opponent may choose to put those cards into your hand." (Covenant of Minds):
/// that player decides whether it's done ("may have you put those cards into your
/// hand").
fn may_choose_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = l.split_once(" may choose to ")?;
    if who == "you" || r.contains(" they ") || r.contains(" their ") {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("{who} may have you {r}"), b)
}

inventory::submit! { super::EffectPattern { name: "followups: [player] may choose to [instruction]", priority: 120, parse: may_choose_to } }

#[cfg(test)]
pub(crate) mod tests {
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
            ("Sorcery", "Target opponent discards a card, they lose 2 life, you draw a card, then you gain 2 life."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
        // Nothing records whether a life gain was "done".
        assert!(compiled("Sorcery", "You gain 2 life. If you can't, draw a card.").is_none());
        // "Can't" doesn't follow a choice.
        assert!(compiled("Sorcery", "You may draw a card. If you can't, you gain 2 life.").is_none());
    }
}
