//! Per-player outcomes of an instruction several players were given (CR 101.4, 118.12,
//! 608.2c, 608.2e):
//!
//! - after "each [players] may [instruction]" (the players decide in APNAP order, then
//!   those who accepted do it, see `player_subjects`): "Each player who doesn't loses 3
//!   life" (The Death of Gwen Stacy), "If they don't, they lose 3 life" (Professor Onyx),
//!   "For each opponent who doesn't, that player loses 2 life and you draw a card" (Braids,
//!   Arisen Nightmare), "~ deals 3 damage to each opponent who didn't" (Zoyowa
//!   Lava-Tongue), "If a player does, creatures that player controls can't attack you ..."
//!   (Orzhov Advokist), "For each opponent who does, you gain protection from that player
//!   ..." (Noble Heritage). "Does" is whether the player chose to (CR 118.12).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_sentence, Builder};
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use crate::scry_rules::{OPTED, OPT_IN};

/// The "each [players] may [instruction]" a sequence ends with.
struct OptIn {
    /// The players offered the choice.
    offered: PlayerFilter,
    /// The variable holding the players who accepted.
    accepted: Var,
    /// Where the "those who accepted do it" step is.
    at: usize,
}

fn offered_filter(p: &PlayerRef) -> Option<PlayerFilter> {
    Some(match p {
        PlayerRef::EachPlayer => PlayerFilter::Any,
        PlayerRef::EachOpponent => PlayerFilter::Opponent,
        PlayerRef::EachOtherPlayer => PlayerFilter::NotYou,
        PlayerRef::Each(f) => f.clone(),
        _ => return None,
    })
}

fn opt_in_of(v: &[Effect]) -> Option<OptIn> {
    let n = v.len();
    if n < 3 {
        return None;
    }
    let Effect::ForEachPlayer {
        who: PlayerRef::Var(accepted),
        ..
    } = &v[n - 1]
    else {
        return None;
    };
    let Effect::Store {
        var,
        sel: Sel::Var(OPTED),
    } = &v[n - 2]
    else {
        return None;
    };
    if var != accepted {
        return None;
    }
    let Effect::ForEachPlayer { who, effect } = &v[n - 3] else {
        return None;
    };
    if !format!("{effect:?}").contains(OPT_IN) {
        return None;
    }
    Some(OptIn {
        offered: offered_filter(who)?,
        accepted: *accepted,
        at: n - 1,
    })
}

/// The sequence an "each [players] may [instruction]" ends, at the end of `e` (also
/// inside "If you do, ...").
fn opt_in_seq(e: &mut Effect) -> Option<&mut Vec<Effect>> {
    if matches!(&*e, Effect::Seq(v) if opt_in_of(v).is_some()) {
        return match e {
            Effect::Seq(v) => Some(v),
            _ => None,
        };
    }
    match e {
        Effect::Seq(v) => v.last_mut().and_then(opt_in_seq),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => opt_in_seq(then),
        _ => None,
    }
}

/// Who the follow-up is about: the players who did it (`did`) or those who didn't, among
/// the players a noun names ("player", "opponent"); `what`: what they did, as the text
/// words it ("discarded a card", "drew a card"), if it says.
#[derive(Debug)]
struct Who {
    did: bool,
    what: Option<String>,
    among: Option<PlayerFilter>,
}

fn noun_filter(noun: &str) -> Option<Option<PlayerFilter>> {
    Some(match noun {
        "player" => None,
        "opponent" => Some(PlayerFilter::Opponent),
        "of those opponents" => Some(PlayerFilter::Opponent),
        _ => return None,
    })
}

/// After "who": whether they did it, what they did (if said), and the rest.
fn who_did(r: &str) -> Option<(bool, Option<String>, &str)> {
    let word_end = |r: &str, w: &str| -> Option<usize> {
        let rest = r.strip_prefix(w)?;
        (rest.is_empty() || rest.starts_with(' ') || rest.starts_with(',')).then_some(w.len())
    };
    for (w, did) in [
        ("doesn't", false),
        ("didn't", false),
        ("does", true),
        ("did", true),
    ] {
        let Some(n) = word_end(r, w) else {
            continue;
        };
        let rest = &r[n..];
        // "didn't discard a card this way".
        if !did {
            if let Some(phrase) = rest.trim_start().split(" this way").next() {
                if rest.contains(" this way") && !phrase.contains(',') {
                    let after = &rest[rest.find(" this way")? + " this way".len()..];
                    return Some((false, Some(phrase.trim().to_string()), after));
                }
            }
        }
        return Some((did, None, rest));
    }
    // "discarded a card this way".
    let at = r.find(" this way")?;
    let phrase = &r[..at];
    if phrase.contains(',') {
        return None;
    }
    Some((true, Some(phrase.to_string()), &r[at + " this way".len()..]))
}

/// The follow-up's form: who it's about and the instruction, as a clause about "that
/// player" (the player it's performed for).
fn split(l: &str) -> Option<(Who, String)> {
    let l = l.strip_prefix("then ").unwrap_or(l);
    // "if they don't, [clause]", "if a player does, [clause]".
    for (p, did) in [
        ("if they don't, ", false),
        ("if they do, ", true),
        ("if a player does, ", true),
        ("if a player doesn't, ", false),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            let who = Who {
                did,
                what: None,
                among: None,
            };
            return Some((who, r.to_string()));
        }
    }
    // "for each opponent who doesn't, [clause]".
    if let Some(r) = l.strip_prefix("for each ") {
        let (noun, r) = r.split_once(" who ")?;
        let (did, what, rest) = who_did(r)?;
        let rest = rest.strip_prefix(", ")?;
        let who = Who {
            did,
            what,
            among: noun_filter(noun)?,
        };
        return Some((who, rest.to_string()));
    }
    // "each player who doesn't loses 3 life": the predicate about that player.
    if let Some(r) = l.strip_prefix("each ") {
        let (noun, r) = r.split_once(" who ")?;
        let (did, what, pred) = who_did(r)?;
        let pred = pred.strip_prefix(' ')?;
        let who = Who {
            did,
            what,
            among: noun_filter(noun)?,
        };
        return Some((who, format!("that player {pred}")));
    }
    // "~ deals 3 damage to each opponent who didn't", "... to each opponent who discarded
    // their hand this way".
    let (head, r) = l.rsplit_once(" who ")?;
    for noun in ["opponent", "player"] {
        let Some(h) = head.strip_suffix(&format!(" each {noun}")) else {
            continue;
        };
        let (did, what, rest) = who_did(r)?;
        if !rest.is_empty() {
            return None;
        }
        let who = Who {
            did,
            what,
            among: noun_filter(noun)?,
        };
        return Some((who, format!("{h} that player")));
    }
    None
}

/// The verbs of the instruction a player performs (as the text words what they did).
fn act_verbs(e: &Effect, out: &mut Vec<&'static str>) {
    match e {
        Effect::Discard { .. } | Effect::DiscardHand { .. } => out.push("discard"),
        Effect::Draw { .. } => out.push("draw"),
        Effect::Sacrifice { .. } | Effect::SacrificeObjects { .. } => out.push("sacrifice"),
        Effect::KeywordAction {
            action: KeywordAction::Investigate,
            ..
        } => out.push("investigate"),
        Effect::ChooseOne { options, .. } => options.iter().for_each(|(_, e)| act_verbs(e, out)),
        Effect::AsPlayer { effect, .. } => act_verbs(effect, out),
        Effect::Seq(v) => v.iter().for_each(|e| act_verbs(e, out)),
        _ => {}
    }
}

/// Whether "[what] this way" says what the instruction `act` has the player do: its
/// verbs ("discarded", "didn't sacrifice a permanent or discard a card"), each with an
/// object any such instruction acts on ("a card", "their hand", "a permanent").
fn says_what(what: &str, act: &Effect) -> bool {
    let mut verbs = vec![];
    act_verbs(act, &mut verbs);
    what.split(" or ").all(|part| {
        let (verb, obj) = part.split_once(' ').unwrap_or((part, ""));
        let base = match verb {
            "discarded" | "discard" => "discard",
            "drew" | "draw" => "draw",
            "sacrificed" | "sacrifice" => "sacrifice",
            "investigated" | "investigate" => "investigate",
            _ => return false,
        };
        let generic = matches!(
            obj,
            "" | "a card" | "cards" | "their hand" | "a permanent" | "permanents"
        );
        verbs.contains(&base) && generic
    })
}

/// The follow-up of an "each [players] may [instruction]".
fn after_each_may(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(v) = opt_in_seq(prev) else {
        return false;
    };
    let Some(opt) = opt_in_of(v) else {
        return false;
    };
    let Some((who, clause)) = split(end(l)) else {
        return false;
    };
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let body = parse_sentence(&clause, b).or_else(|| parse_clause(&clause, b));
    b.it_player = saved;
    let Some(body) = body else {
        return false;
    };
    let body = players_objects_later(body);
    // Only about that player: no other player is "that player".
    let text = format!("{body:?}");
    if text.contains("TriggerPlayer") {
        return false;
    }
    // Who did it: a player who accepted but couldn't do it didn't (Zoyowa Lava-Tongue
    // ruling: an opponent with no cards in hand can't choose to discard), when the
    // instruction records that; otherwise who accepted (CR 118.12).
    if let Some(what) = &who.what {
        let Effect::ForEachPlayer { effect, .. } = &v[opt.at] else {
            return false;
        };
        if !says_what(what, effect) {
            return false;
        }
    }
    let recorded = already_recorded(v);
    let did = record_done(v, opt.at);
    let at = opt.at + usize::from(did.is_some() && !recorded);
    let did_var = did.unwrap_or(opt.accepted);
    let mut filter = vec![opt.offered];
    filter.extend(who.among);
    let did_it = PlayerFilter::Ref(Box::new(PlayerRef::Var(did_var)));
    if who.did {
        // After doing it, in the same "each player" instruction.
        let Effect::ForEachPlayer { effect, .. } = &mut v[at] else {
            return false;
        };
        filter.push(did_it);
        let old = std::mem::take(&mut **effect);
        **effect = Effect::seq(vec![
            old,
            Effect::If {
                cond: Condition::PlayerMatches(PlayerRef::Iterated, PlayerFilter::And(filter)),
                then: Box::new(body),
                otherwise: Box::new(Effect::Noop),
            },
        ]);
        return true;
    }
    filter.push(PlayerFilter::Not(Box::new(did_it)));
    v.push(Effect::ForEachPlayer {
        who: PlayerRef::Each(PlayerFilter::And(filter)),
        effect: Box::new(body),
    });
    true
}

/// "If a player does, creatures that player controls can't attack you ... until your
/// next turn": the creatures that player controls at any time during the effect, not
/// only those they control now (Orzhov Advokist ruling).
fn players_objects_later(e: Effect) -> Effect {
    match e {
        Effect::AddRestriction {
            mut restriction,
            duration,
        } => {
            if let Restriction::CantAttackPlayer { attackers, .. } = &mut restriction {
                if let Filter::In(inner) = attackers {
                    if let Sel::All(f) = &**inner {
                        if format!("{f:?}").contains("ControlledByPlayer(Iterated)") {
                            *attackers = f.clone();
                        }
                    }
                }
            }
            Effect::AddRestriction {
                restriction,
                duration,
            }
        }
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(players_objects_later).collect()),
        e => e,
    }
}

/// The players who did what they accepted to do ("each opponent who didn't").
pub const DID: Var = vars::USER + 9650;

/// Whether the instruction a player performs was done, if it records that: an
/// instruction that can fail to do anything, or a draw (some cards drawn).
fn done_cond(e: &Effect) -> Option<Condition> {
    match e {
        Effect::Discard { .. }
        | Effect::Sacrifice { .. }
        | Effect::SacrificeObjects { .. }
        | Effect::AddCounters { .. }
        | Effect::Exile { .. }
        | Effect::Move { .. }
        | Effect::PayOptional { .. } => Some(Condition::PrevHappened),
        Effect::Draw { .. } => Some(Condition::Compare(Value::Prev, Cmp::Ge, Value::c(1))),
        Effect::ChooseOne { options, .. } => options
            .iter()
            .all(|(_, e)| matches!(done_cond(e), Some(Condition::PrevHappened)))
            .then_some(Condition::PrevHappened),
        Effect::AsPlayer { effect, .. } => done_cond(effect),
        _ => None,
    }
}

fn already_recorded(v: &[Effect]) -> bool {
    v.iter()
        .any(|e| matches!(e, Effect::Store { var, sel: Sel::None } if *var == DID))
}

/// Records, in [`DID`], the players who did the accepted instruction at `v[at]`, if it
/// records whether it was done; returns [`DID`] then.
fn record_done(v: &mut Vec<Effect>, at: usize) -> Option<Var> {
    if already_recorded(v) {
        return Some(DID);
    }
    let Effect::ForEachPlayer { effect, .. } = &mut v[at] else {
        return None;
    };
    let cond = done_cond(effect)?;
    let old = std::mem::take(&mut **effect);
    **effect = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(Effect::Store {
                var: DID,
                sel: Sel::Union(vec![Sel::Var(DID), Sel::Players(PlayerRef::Iterated)]),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    v.insert(
        at,
        Effect::Store {
            var: DID,
            sel: Sel::None,
        },
    );
    Some(DID)
}

inventory::submit! { FollowupPattern { name: "followups: each player who doesn't (after each player may)", priority: 140, apply: after_each_may } }

// ---------------------------------------------------------------------------
// After an instruction several players had to follow ("each opponent discards two
// cards"): what each of them did, from the instruction's record of the cards discarded
// or permanents sacrificed (as they last existed).
// ---------------------------------------------------------------------------

/// What an instruction several players perform records: its variable, how a recorded
/// object belongs to the player, the verb, and the players.
struct Record {
    var: Var,
    discard: bool,
    players: PlayerRef,
}

fn record_of(e: &Effect) -> Option<Record> {
    match e {
        Effect::Discard { who, .. } if !matches!(who, PlayerRef::You) => Some(Record {
            var: crate::discard_rules::DISCARDED,
            discard: true,
            players: who.clone(),
        }),
        Effect::Sacrifice { who, .. } if !matches!(who, PlayerRef::You) => Some(Record {
            var: vars::SACRIFICED,
            discard: false,
            players: who.clone(),
        }),
        Effect::ForEachPlayer { who, effect } => match &**effect {
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect,
            } => match &**effect {
                Effect::Discard {
                    who: PlayerRef::You,
                    ..
                } => Some(Record {
                    var: crate::discard_rules::DISCARDED,
                    discard: true,
                    players: who.clone(),
                }),
                Effect::Sacrifice {
                    who: PlayerRef::You,
                    ..
                } => Some(Record {
                    var: vars::SACRIFICED,
                    discard: false,
                    players: who.clone(),
                }),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// "You and that player each sacrifice a creature" (each performed in turn): performed
/// by both at the same time (CR 101.4), so what each did is recorded together.
fn together(v: &mut Vec<Effect>) -> bool {
    let n = v.len();
    if n < 2 {
        return false;
    }
    let (Effect::AsPlayer { who, effect }, first) = (&v[n - 1], &v[n - 2]) else {
        return false;
    };
    let same = format!("{effect:?}") == format!("{first:?}");
    let one = matches!(
        first,
        Effect::Sacrifice {
            who: PlayerRef::You,
            ..
        } | Effect::Discard {
            who: PlayerRef::You,
            ..
        }
    );
    if !same || !one {
        return false;
    }
    let other = who.clone();
    let e = first.clone();
    v.truncate(n - 2);
    v.push(Effect::ForEachPlayer {
        who: PlayerRef::Each(PlayerFilter::Or(vec![
            PlayerFilter::You,
            PlayerFilter::Ref(Box::new(other)),
        ])),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    });
    true
}

/// The objects of the record that belonged to the iterated player and match `q`.
fn theirs(rec: &Record, q: Filter) -> Sel {
    let owner = if rec.discard {
        Filter::OwnedBy(PlayerRel::Iterated)
    } else {
        Filter::ControlledBy(PlayerRel::Iterated)
    };
    // As they last existed: a sacrificed permanent is no longer on the battlefield.
    Sel::Matching(Box::new(Sel::Var(rec.var)), Filter::and(vec![q, owner]))
}

/// "[verb] [number] [objects] [this way]" for the record: how many of which objects.
/// Returns the number, the quality, and a value the instruction may name ("that mana
/// value").
fn did_objects(phrase: &str, rec: &Record, b: &Builder) -> Option<(i32, Filter, Option<Value>)> {
    let r = if rec.discard {
        phrase
            .strip_prefix("discarded ")
            .or_else(|| phrase.strip_prefix("discard "))?
    } else {
        phrase
            .strip_prefix("sacrificed ")
            .or_else(|| phrase.strip_prefix("sacrifice "))?
    };
    let greatest = " with the greatest mana value among cards discarded this way";
    let r = match r.strip_suffix(greatest) {
        Some(h) => format!("{h}{greatest}"),
        None => r.strip_suffix(" this way").unwrap_or(r).to_string(),
    };
    let r = r.as_str();
    let (n, rest) = match r.split_once(' ') {
        Some(("a" | "an", rest)) => (1, rest),
        _ => {
            let (n, rest) = parse_number(r)?;
            (n.as_const()?, rest)
        }
    };
    let all = Filter::In(Box::new(Sel::Var(rec.var)));
    // "a card with the greatest mana value among cards discarded this way".
    if let Some(head) = rest.strip_suffix(" with the greatest mana value among cards discarded this way") {
        if !rec.discard || head != "card" {
            return None;
        }
        let greatest = Value::GreatestManaValue(all);
        return Some((
            n,
            Filter::ManaValue(Cmp::Eq, Box::new(greatest.clone())),
            Some(greatest),
        ));
    }
    // "a card that shares a card type with the card you discarded".
    if let Some(head) = rest.strip_suffix(" that shares a card type with the card you discarded") {
        if !rec.discard || head != "card" {
            return None;
        }
        let yours = Sel::All(Filter::and(vec![all, Filter::OwnedBy(PlayerRel::You)]));
        return Some((n, Filter::SharesCardType(Box::new(yours)), None));
    }
    let q = if rec.discard {
        super::card_flow_search::card_filter(rest, b)?
    } else {
        let (f, _, tail) = parse_object_phrase(rest)?;
        if !tail.trim().is_empty() {
            return None;
        }
        f
    };
    Some((n, q, None))
}

/// "for each opponent who didn't discard two nonland cards this way, you draw a card",
/// "each player who sacrificed a creature this way draws two cards", "each player who
/// discarded a card with the greatest mana value among cards discarded this way loses
/// life equal to that mana value", "for each of those opponents who didn't discard a
/// card with mana value 4 or greater, draw a card".
fn after_each_did(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (form_for, r) = if let Some(r) = l.strip_prefix("for each ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("each ") {
        (false, r)
    } else {
        return false;
    };
    let Some((noun, r)) = r.split_once(" who ") else {
        return false;
    };
    let (did, r) = match r.strip_prefix("didn't ") {
        Some(r) => (false, r),
        None => (true, r),
    };
    // The instruction the follow-up is about (the pattern registry restores `prev` if
    // this returns false).
    if !matches!(prev, Effect::Seq(_)) {
        *prev = Effect::Seq(vec![std::mem::take(prev)]);
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    together(v);
    let Some(rec) = v.last().and_then(record_of) else {
        return false;
    };
    let among = match noun {
        "player" => PlayerFilter::Any,
        "opponent" => PlayerFilter::Opponent,
        "of those opponents" | "of those players" => {
            PlayerFilter::Ref(Box::new(rec.players.clone()))
        }
        _ => return false,
    };
    // Where the description of the objects ends and the instruction begins.
    let mut found = None;
    if form_for {
        if let Some((phrase, clause)) = r.split_once(", ") {
            if let Some(x) = did_objects(phrase, &rec, b) {
                found = Some((x, clause.to_string()));
            }
        }
    } else {
        for (i, _) in r.match_indices(' ').collect::<Vec<_>>().into_iter().rev() {
            let (phrase, pred) = (&r[..i], &r[i + 1..]);
            if let Some(x) = did_objects(phrase, &rec, b) {
                found = Some((x, format!("that player {pred}")));
                break;
            }
        }
    }
    let Some(((n, q, named), clause)) = found else {
        return false;
    };
    let saved_player = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let body = match (&named, clause.as_str()) {
        (Some(mv), "that player loses life equal to that mana value") => Some(Effect::LoseLife {
            who: PlayerRef::Iterated,
            n: mv.clone(),
        }),
        _ => parse_sentence(&clause, b),
    };
    b.it_player = saved_player;
    let Some(body) = body else {
        return false;
    };
    let count = Value::CountSel(Box::new(theirs(&rec, q)));
    let cond = Condition::Compare(count, if did { Cmp::Ge } else { Cmp::Lt }, Value::c(n));
    v.push(Effect::ForEachPlayer {
        who: PlayerRef::Each(among),
        effect: Box::new(Effect::If {
            cond,
            then: Box::new(body),
            otherwise: Box::new(Effect::Noop),
        }),
    });
    true
}

inventory::submit! { FollowupPattern { name: "followups: each player who discarded/sacrificed [objects] (this way)", priority: 145, apply: after_each_did } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn after_each_player_may() {
        for (ty, text) in [
            ("Sorcery", "Each player may discard a card. Each player who doesn't loses 3 life."),
            ("Sorcery", "Each opponent may discard a card. If they don't, they lose 3 life."),
            ("Sorcery", "Each opponent may sacrifice a creature. For each opponent who doesn't, that player loses 2 life and you draw a card."),
            ("Sorcery", "Each opponent may discard a card. ~ deals 3 damage to each opponent who didn't."),
            ("Sorcery", "Each player may sacrifice a creature. If a player does, that player draws a card."),
            ("Sorcery", "Each player may discard a card. Each player who discarded a card this way draws a card."),
            ("Sorcery", "Each opponent discards two cards. For each opponent who didn't discard two nonland cards this way, you draw a card."),
            ("Sorcery", "Choose an opponent. You and that player each sacrifice a creature. Each player who sacrificed a creature this way draws two cards."),
            ("Sorcery", "Each player discards a card. Each opponent who discarded a card that shares a card type with the card you discarded loses 3 life."),
            ("Sorcery", "Each player may draw a card. Then each player who drew a card this way gains 1 life."),
            ("Sorcery", "Each opponent may sacrifice a permanent or discard a card. Then ~ deals 2 damage to each opponent who didn't sacrifice a permanent or discard a card this way."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
