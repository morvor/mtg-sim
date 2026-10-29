//! Player grammar VI (cards5-choices-players): players picked by their results.
//!
//! - "Whenever one or more Pirates you control deal damage to your opponents, you create a
//!   Treasure token for each opponent dealt damage." (Malcolm, Keen-Eyed Navigator): one
//!   token for each player of the triggering batch of damage events, each counted once
//!   (CR 603.2c; `PlayerRef::TriggerPlayers`).
//! - "Each player shuffles all creatures they own into their library. Each player who
//!   shuffled a nontoken creature into their library this way reveals cards ..."
//!   (Collision of Realms): the players whose own instruction moved such an object (the
//!   objects each player's instruction moved are kept, CR 608.2c), each performing the
//!   rest as themselves, in APNAP order (CR 101.4).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

/// "[you] create [tokens] for each opponent dealt damage" in a "one or more [objects] deal
/// damage to [players]" trigger: read as "create X [tokens], where X is the number of
/// opponents dealt damage this way".
fn create_for_each_opponent_dealt_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let body = ["opponent", "player"].iter().find_map(|p| {
        l.strip_suffix(&format!(" for each {p} dealt damage"))
            .or_else(|| l.strip_suffix(&format!(" for each {p} dealt combat damage")))
            .map(|b| (b, *p, l.ends_with("combat damage")))
    });
    let (body, who, combat) = body?;
    if !b.in_trigger
        || !b
            .named
            .iter()
            .any(|(n, _)| n == super::referents_values_5::BATCH_PLAYERS)
    {
        return None;
    }
    let body = body.strip_prefix("you ").unwrap_or(body);
    let token = body.strip_prefix("create a ").or_else(|| body.strip_prefix("create an "))?;
    // "a Treasure token" → "X Treasure tokens".
    let plural = token.strip_suffix(" token")?;
    let dmg = if combat { "combat damage" } else { "damage" };
    let text = format!(
        "create x {plural} tokens, where x is the number of {who}s dealt {dmg} this way"
    );
    parse_sentence(&text, b)
}

inventory::submit! { EffectPattern { name: "create [token] for each opponent dealt damage (batch trigger)", priority: 90, parse: create_for_each_opponent_dealt_damage } }

/// The objects moved by each player's instruction of an "each player [instruction]", for
/// "each player who [did it to an object] this way".
const MOVED_BY_EACH: Var = vars::USER + 9531;

/// Records the objects each player's instruction in `e` (the last "each player ..."
/// instruction) moved, if it's one that keeps them (`vars::IT`). Returns whether it did.
fn record_each_players_results(e: &mut Effect, verb: &str) -> bool {
    match e {
        Effect::Seq(v) => v
            .last_mut()
            .is_some_and(|x| record_each_players_results(x, verb)),
        Effect::ForEachPlayer { effect, .. } => {
            let Effect::AsPlayer { effect: inner, .. } = effect.as_mut() else {
                return false;
            };
            let ok = match (verb, inner.as_ref()) {
                ("shuffled", Effect::ShuffleIntoLibrary { .. }) => true,
                ("exiled", Effect::Exile { .. }) => true,
                _ => false,
            };
            if !ok {
                return false;
            }
            let body = std::mem::replace(inner.as_mut(), Effect::Noop);
            **inner = Effect::seq(vec![
                body,
                Effect::Store {
                    var: MOVED_BY_EACH,
                    sel: Sel::Union(vec![Sel::Var(MOVED_BY_EACH), Sel::Var(vars::IT)]),
                },
            ]);
            let each = std::mem::replace(e, Effect::Noop);
            *e = Effect::seq(vec![
                Effect::Store {
                    var: MOVED_BY_EACH,
                    sel: Sel::Union(vec![]),
                },
                each,
            ]);
            true
        }
        _ => false,
    }
}

/// "Each player who shuffled a nontoken creature into their library this way [does
/// something]" after "Each player shuffles ...".
fn each_player_who_did_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("each player who ") else {
        return false;
    };
    let Some((did, pred)) = r.split_once(" this way ") else {
        return false;
    };
    // "shuffled a nontoken creature into their library".
    let Some((verb, obj)) = did.split_once(' ') else {
        return false;
    };
    let obj = match verb {
        "shuffled" => obj.strip_suffix(" into their library"),
        "exiled" => Some(obj),
        _ => None,
    };
    let Some(obj) = obj
        .and_then(|o| o.strip_prefix("a ").or_else(|| o.strip_prefix("an ")))
    else {
        return false;
    };
    let Some((f, false, tail)) = parse_object_phrase(obj) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    // The object's qualities, wherever it is now.
    let f = match f {
        Filter::And(v) => Filter::and(
            v.into_iter()
                .filter(|x| !matches!(x, Filter::Permanent | Filter::InZone(_)))
                .collect(),
        ),
        Filter::Permanent => Filter::Any,
        f => f,
    };
    let mut prev2 = prev.clone();
    if !record_each_players_results(&mut prev2, verb) {
        return false;
    }
    let Some(e) = parse_sentence(&format!("each player {pred}"), b) else {
        return false;
    };
    let Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect,
    } = e
    else {
        return false;
    };
    let did_it = Condition::Compare(
        Value::CountSel(Box::new(Sel::Matching(
            Box::new(Sel::Var(MOVED_BY_EACH)),
            Filter::and(vec![f, Filter::OwnedBy(PlayerRel::Iterated)]),
        ))),
        Cmp::Ge,
        Value::c(1),
    );
    *prev = Effect::seq(vec![
        prev2,
        Effect::ForEachPlayer {
            who: PlayerRef::Each(PlayerFilter::History(Box::new(did_it))),
            effect,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "each player who [did it to an object] this way [does something]", priority: 60, apply: each_player_who_did_this_way } }

/// The cards the instructions of a "you and defending player each discard a card or
/// sacrifice a permanent" put into graveyards, for "for each land card put into a
/// graveyard this way".
const PUT_INTO_GRAVEYARDS: Var = vars::USER + 9532;

/// Adds "and keep what it moved" after each discard and sacrifice instruction in `v` (a
/// compiled effect as JSON). Returns how many there were.
fn keep_discarded_and_sacrificed(v: &mut serde_json::Value) -> usize {
    use serde_json::Value as J;
    let mut n = 0;
    match v {
        J::Object(m) => {
            for x in m.values_mut() {
                n += keep_discarded_and_sacrificed(x);
            }
            let is_action = m.len() == 1
                && (m.get("Discard").is_some_and(|d| d.get("who").is_some() && d.get("n").is_some())
                    || m.get("Sacrifice").is_some_and(|d| {
                        d.get("who").is_some() && d.get("filter").is_some() && d.get("count").is_some()
                    }));
            if is_action {
                let keep = serde_json::to_value(Effect::Store {
                    var: PUT_INTO_GRAVEYARDS,
                    sel: Sel::Union(vec![Sel::Var(PUT_INTO_GRAVEYARDS), Sel::Var(vars::IT)]),
                })
                .unwrap_or(J::Null);
                let action = J::Object(std::mem::take(m));
                m.insert("Seq".into(), J::Array(vec![action, keep]));
                n += 1;
            }
        }
        J::Array(a) => {
            for x in a {
                n += keep_discarded_and_sacrificed(x);
            }
        }
        _ => {}
    }
    n
}

/// "You draw a card for each land card put into a graveyard this way." after "you and
/// defending player each discard a card or sacrifice a permanent" (Eumidian Wastewaker):
/// the cards those instructions put into graveyards (a discarded or sacrificed card that
/// went elsewhere instead, or a token, isn't one).
fn draw_for_each_card_put_into_graveyards(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(noun) = l
        .strip_prefix("you draw a card for each ")
        .or_else(|| l.strip_prefix("draw a card for each "))
        .and_then(|r| r.strip_suffix(" put into a graveyard this way"))
    else {
        return false;
    };
    let Some((f, false, tail)) = parse_object_phrase(noun) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    let Ok(mut j) = serde_json::to_value(&*prev) else {
        return false;
    };
    if keep_discarded_and_sacrificed(&mut j) == 0 {
        return false;
    }
    let Ok(kept) = serde_json::from_value::<Effect>(j) else {
        return false;
    };
    let f = match f {
        Filter::And(v) => Filter::and(
            v.into_iter()
                .filter(|x| !matches!(x, Filter::InZone(_)))
                .collect(),
        ),
        f => f,
    };
    let n = Value::CountSel(Box::new(Sel::Matching(
        Box::new(Sel::Var(PUT_INTO_GRAVEYARDS)),
        Filter::and(vec![f, Filter::Card, Filter::InZone(ZoneKind::Graveyard)]),
    )));
    *prev = Effect::seq(vec![
        Effect::Store {
            var: PUT_INTO_GRAVEYARDS,
            sel: Sel::Union(vec![]),
        },
        kept,
        Effect::Draw {
            who: PlayerRef::You,
            n,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "you draw a card for each [card] put into a graveyard this way (after discards and sacrifices)", priority: 60, apply: draw_for_each_card_put_into_graveyards } }

/// Replaces each discard of a constant number of cards in `v` (JSON of an effect) with a
/// manifest of that many cards from the hand. Returns how many were replaced.
fn discard_to_manifest(v: &mut serde_json::Value) -> usize {
    use serde_json::Value as J;
    let mut n = 0;
    match v {
        J::Object(m) => {
            if let Some(d) = m.get("Discard") {
                let k = d.get("n").and_then(|x| x.get("Const")).and_then(|x| x.as_u64());
                let plain = d.get("random") == Some(&J::Bool(false))
                    && d.get("filter") == Some(&J::String("Any".into()));
                if let (Some(k), true, 1) = (k, plain, m.len()) {
                    let e = crate::kw::manifest_from_hand_5::manifest_from_hand(k as u32);
                    *v = serde_json::to_value(e).unwrap_or(J::Null);
                    return 1;
                }
            }
            for x in m.values_mut() {
                n += discard_to_manifest(x);
            }
        }
        J::Array(a) => {
            for x in a {
                n += discard_to_manifest(x);
            }
        }
        _ => {}
    }
    n
}

/// "up to two target players each manifest two cards from their hands" (Kozilek, the
/// Broken Reality): see `kw/manifest_from_hand_5.rs`; "each card manifested this way"
/// afterward.
fn players_manifest_from_hands(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subj, n) = l.split_once(" manifest ")?;
    let n = n
        .strip_suffix(" cards from their hands")
        .or_else(|| n.strip_suffix(" cards from their hand"))
        .or_else(|| n.strip_suffix(" card from their hand"))?;
    let first = b.targets.len();
    let e = parse_sentence(&format!("{subj} discard {n} cards"), b)?;
    let mut j = serde_json::to_value(&e).ok()?;
    if discard_to_manifest(&mut j) == 0 {
        b.targets.truncate(first);
        return None;
    }
    let e: Effect = serde_json::from_value(j).ok()?;
    let var = crate::kw::manifest_from_hand_5::MANIFESTED_THIS_WAY;
    b.named
        .push(("cards manifested this way".into(), Sel::Var(var)));
    Some(Effect::seq(vec![
        Effect::Store {
            var,
            sel: Sel::Union(vec![]),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "[players] each manifest N cards from their hands", priority: 90, parse: players_manifest_from_hands } }

/// "For each card manifested this way, you draw a card."
fn draw_for_each_manifested(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !matches!(
        l,
        "for each card manifested this way, you draw a card"
            | "for each card manifested this way, draw a card"
    ) {
        return None;
    }
    let sel = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == "cards manifested this way")
        .map(|(_, s)| s.clone())?;
    Some(Effect::Draw {
        who: PlayerRef::You,
        n: Value::CountSel(Box::new(sel)),
    })
}

inventory::submit! { EffectPattern { name: "for each card manifested this way, you draw a card", priority: 90, parse: draw_for_each_manifested } }

/// The spells a "you may cast any number of spells from among ..." instruction cast.
const CAST_THIS_WAY: Var = vars::USER + 9533;

/// Whether `e` (or the last instruction of a sequence) casts cards.
fn ends_with_cast(e: &Effect) -> bool {
    match e {
        Effect::CastCard { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_cast),
        _ => false,
    }
}

/// "Then each player who owns a spell you cast this way loses life equal to its mana
/// value." (Kefka, Dancing Mad, after "You may cast any number of spells from among cards
/// exiled this way ..."): for each player, the spells of theirs you cast this way.
fn owners_of_spells_cast_lose_life(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player who owns a spell you cast this way loses life equal to its mana value" {
        return false;
    }
    if !ends_with_cast(prev) {
        return false;
    }
    let theirs = || {
        Sel::Matching(
            Box::new(Sel::Var(CAST_THIS_WAY)),
            Filter::OwnedBy(PlayerRel::Iterated),
        )
    };
    let cast = std::mem::take(prev);
    *prev = Effect::seq(vec![
        cast,
        // The spells cast, if any were (otherwise "it" is still the cards).
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::Store {
                var: CAST_THIS_WAY,
                sel: Sel::Var(vars::IT),
            }),
            otherwise: Box::new(Effect::Store {
                var: CAST_THIS_WAY,
                sel: Sel::Union(vec![]),
            }),
        },
        Effect::ForEachPlayer {
            who: PlayerRef::Each(PlayerFilter::History(Box::new(Condition::Compare(
                Value::CountSel(Box::new(theirs())),
                Cmp::Ge,
                Value::c(1),
            )))),
            effect: Box::new(Effect::LoseLife {
                who: PlayerRef::Iterated,
                n: Value::ManaValueOf(Box::new(theirs())),
            }),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "each player who owns a spell you cast this way loses life equal to its mana value", priority: 60, apply: owners_of_spells_cast_lose_life } }
