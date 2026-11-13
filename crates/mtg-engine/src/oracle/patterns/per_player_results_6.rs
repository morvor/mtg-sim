//! Each player's own result of a step every player (or several players) performed, read
//! by the sentences after it (cards6-per-player-results; CR 101.4, 608.2c, 608.2h):
//!
//! - "Each player exiles the top card of their library. The player who exiled the card
//!   with the greatest mana value takes an extra turn after this one." (Timesifter): the
//!   cards the step moved are compared; "the player who" is the one player whose card has
//!   the greatest mana value (with two or more tied, no player is: Psychic Battle's "If two
//!   or more cards are tied for greatest, the target or targets remain unchanged"), "each
//!   player who" is every player whose card does. A player who moved no card has none (a
//!   player who fails to exile a card can't take the extra turn, Timesifter's rulings).
//! - "If two or more players' cards are tied for greatest, the tied players repeat this
//!   process until the tie is broken.": the step and the comparison again, among only the
//!   tied players (Timesifter's rulings), until one player's card is greatest or no
//!   player is tied (`repeat_process.rs`).
//! - "Those permanents' owners shuffle them into their libraries." (Guff Rewrites
//!   History): the owners of those permanents.
//! - "Each player who controlled one of those permanents [instruction]": the players who
//!   controlled them as they last existed on the battlefield (CR 608.2h), each performing
//!   the instruction in APNAP order (CR 101.4).
//! - "Each player may cast the nonland card they exiled without paying its mana cost."
//!   after each of several players exiled cards from their own library: each player, in
//!   APNAP order, may cast the card they exiled themselves (CR 608.2g), not another
//!   player's.
//! - "Each opponent may search their library for up to three basic land cards. They each
//!   put one of those cards onto the battlefield tapped under your control and the rest
//!   onto the battlefield tapped under their control." (Rootweaver Druid): each opponent
//!   in turn, starting with the next one in turn order (the active player first when it's
//!   an opponent's turn, CR 101.4), decides whether to search and, if they do, searches
//!   and splits the cards they found, knowing what the opponents before them did
//!   (Rootweaver Druid's rulings; one card found goes under your control); "Then each
//!   player who searched their library this way shuffles." after all of them.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

// ---------------------------------------------------------------------------
// The player who exiled the card with the greatest mana value
// ---------------------------------------------------------------------------

/// The players performing the step (the tied players, on a repeat).
pub const CONTENDERS: Var = vars::USER + 6602;
/// The cards the step moved or revealed.
pub const COMPARED: Var = vars::USER + 6603;
/// The players whose card has the greatest mana value among [`COMPARED`].
pub const GREATEST: Var = vars::USER + 6604;

/// The players among `among` with a card among `cards` that has the greatest mana value
/// among `cards`.
pub fn greatest_mana_value_players(among: PlayerRef, cards: Sel) -> Sel {
    let greatest = Value::GreatestManaValue(Filter::In(Box::new(cards.clone())));
    let theirs = Sel::Matching(
        Box::new(cards),
        Filter::and(vec![
            Filter::OwnedBy(PlayerRel::Iterated),
            Filter::ManaValue(Cmp::Eq, Box::new(greatest)),
        ]),
    );
    Sel::Players(PlayerRef::Each(PlayerFilter::And(vec![
        PlayerFilter::Ref(Box::new(among)),
        PlayerFilter::History(Box::new(Condition::Compare(
            Value::CountSel(Box::new(theirs)),
            Cmp::Ge,
            Value::c(1),
        ))),
    ])))
}

/// The number of players whose card has the greatest mana value compared with `n`.
fn greatest_count(c: Cmp, n: i32) -> Condition {
    Condition::Compare(
        Value::CountSel(Box::new(Sel::Var(GREATEST))),
        c,
        Value::c(n),
    )
}

/// "[the player who / each player who] [verb] the card with the greatest mana value" after
/// the step the cards came from, `body` being performed for (by) that player: the cards
/// compared ([`COMPARED`]) and the players whose card has the greatest mana value
/// ([`GREATEST`]); for `unique`, only when that's one player.
pub fn greatest_mana_value_comparison(
    cards: Sel,
    among: PlayerRef,
    body: Effect,
    unique: bool,
) -> Effect {
    let each = Effect::ForEachPlayer {
        who: PlayerRef::Var(GREATEST),
        effect: Box::new(body),
    };
    Effect::seq(vec![
        Effect::Store {
            var: COMPARED,
            sel: cards,
        },
        Effect::Store {
            var: GREATEST,
            sel: greatest_mana_value_players(among, Sel::Var(COMPARED)),
        },
        if unique {
            Effect::If {
                cond: greatest_count(Cmp::Eq, 1),
                then: Box::new(each),
                otherwise: Box::new(Effect::Noop),
            }
        } else {
            each
        },
    ])
}

/// The step of each player of a group with the top card of their library, at the end of
/// `e`: the players and the verbs the text says it with.
fn top_card_step(e: &Effect) -> Option<(PlayerRef, &'static [&'static str])> {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(who, Value::Const(1)),
            face_down: false,
            ..
        } if matches!(who, PlayerRef::EachPlayer | PlayerRef::EachOpponent) => {
            Some((who.clone(), &["exiled", "exiles"]))
        }
        Effect::Custom(n) if n.as_str() == crate::kw::parley::REVEAL_TOPS => {
            Some((PlayerRef::EachPlayer, &["revealed", "reveals"]))
        }
        _ => None,
    }
}

/// "the player who exiled the card with the greatest mana value [instruction]", "each
/// player who revealed the card with the greatest mana value [instruction]" after each
/// player exiled (revealed) the top card of their library.
fn the_player_with_the_greatest(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (unique, r) = if let Some(r) = l.strip_prefix("the player who ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("each player who ") {
        (false, r)
    } else {
        return false;
    };
    let Some((verb, r)) = r.split_once(' ') else {
        return false;
    };
    let Some(pred) = r.strip_prefix("the card with the greatest mana value ") else {
        return false;
    };
    let Some((players, verbs)) = top_card_step(prev) else {
        return false;
    };
    if !verbs.contains(&verb) {
        return false;
    }
    // "It", "that card", "its mana value" in the instruction: that player's own card (the
    // one compared), not every card the step moved.
    let theirs = Sel::Matching(
        Box::new(Sel::Var(COMPARED)),
        Filter::OwnedBy(PlayerRel::Iterated),
    );
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let saved_it = std::mem::replace(&mut b.it, theirs);
    let body = parse_sentence(&format!("that player {pred}"), b);
    b.it_player = saved;
    b.it = saved_it;
    let Some(body) = body else {
        return false;
    };
    // About that player only.
    if format!("{body:?}").contains("TriggerPlayer") {
        return false;
    }
    let step = std::mem::take(prev);
    let (step, cards, among) = match step {
        // The players performing it are kept: the tied players repeat it.
        Effect::Exile {
            what: Sel::TopOfLibrary(_, n),
            face_down,
            link,
        } => (
            Effect::seq(vec![
                Effect::Store {
                    var: CONTENDERS,
                    sel: Sel::Players(players.clone()),
                },
                Effect::Exile {
                    what: Sel::TopOfLibrary(PlayerRef::Var(CONTENDERS), n),
                    face_down,
                    link,
                },
            ]),
            Sel::Var(vars::IT),
            PlayerRef::Var(CONTENDERS),
        ),
        other => (other, Sel::Var(crate::kw::parley::REVEALED), players),
    };
    *prev = Effect::seq(vec![
        step,
        greatest_mana_value_comparison(cards, among, body, unique),
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "per-player results 6: the player who exiled the card with the greatest mana value", priority: 55, apply: the_player_with_the_greatest } }

/// "If two or more players' cards are tied for greatest, the tied players repeat this
/// process until the tie is broken." after the comparison of an exile step.
fn tied_players_repeat(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l)
        != "if two or more players' cards are tied for greatest, the tied players repeat this process until the tie is broken"
    {
        return false;
    }
    // Store CONTENDERS, Exile, Store COMPARED, Store GREATEST, If(unique)
    let Effect::Seq(v) = prev else {
        return false;
    };
    let [Effect::Store {
        var: c,
        sel: players,
    }, exile @ Effect::Exile { .. }, s1 @ Effect::Store { var: COMPARED, .. }, s2 @ Effect::Store { var: GREATEST, .. }, Effect::If {
        cond,
        then,
        otherwise,
    }] = v.as_mut_slice()
    else {
        return false;
    };
    if *c != CONTENDERS
        || format!("{cond:?}") != format!("{:?}", greatest_count(Cmp::Eq, 1))
        || !matches!(**otherwise, Effect::Noop)
    {
        return false;
    }
    let tied = Effect::If {
        cond: greatest_count(Cmp::Ge, 2),
        then: Box::new(Effect::seq(vec![
            Effect::Store {
                var: CONTENDERS,
                sel: Sel::Var(GREATEST),
            },
            Effect::RepeatThisProcess,
        ])),
        otherwise: Box::new(Effect::Noop),
    };
    let pass = Effect::seq(vec![
        exile.clone(),
        s1.clone(),
        s2.clone(),
        Effect::If {
            cond: cond.clone(),
            then: then.clone(),
            otherwise: Box::new(tied),
        },
    ]);
    let start = Effect::Store {
        var: CONTENDERS,
        sel: players.clone(),
    };
    *prev = Effect::seq(vec![
        start,
        Effect::RepeatProcess {
            body: Box::new(pass),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "per-player results 6: the tied players repeat this process until the tie is broken", priority: 55, apply: tied_players_repeat } }

// ---------------------------------------------------------------------------
// Those permanents' owners; each player who controlled one of those permanents
// ---------------------------------------------------------------------------

/// "those permanents' owners shuffle them into their libraries": "the owners of those
/// permanents ...".
fn possessive_owners(l: &str, b: &mut Builder) -> Option<crate::ability::Effect> {
    let l = end(l);
    let r = l.strip_prefix("those ")?;
    let (noun, pred) = r.split_once("' owners ")?;
    if noun.is_empty() || noun.contains(' ') || !noun.ends_with('s') {
        return None;
    }
    parse_sentence(&format!("the owners of those {noun} {pred}"), b)
}

inventory::submit! { super::EffectPattern { name: "per-player results 6: those [objects]' owners [instruction]", priority: 55, parse: possessive_owners } }

/// What the last instruction of `e` moved (the objects "those permanents" names).
fn moved_objects(e: &Effect) -> Option<Sel> {
    match e {
        Effect::ShuffleIntoLibrary { what, .. }
        | Effect::ShuffleInto { what }
        | Effect::Exile { what, .. }
        | Effect::Destroy { what, .. }
        | Effect::Move { what, .. } => Some(what.clone()),
        Effect::Seq(v) => v.last().and_then(moved_objects),
        _ => None,
    }
}

/// The controllers of the objects an instruction moved, as they were just before it.
pub const CONTROLLED_MOVED: Var = vars::USER + 6606;

/// Keeps, just before the last instruction of `e` (which moves `those`), the players who
/// control `those` ([`CONTROLLED_MOVED`]).
fn keep_controllers_before_move(e: &mut Effect, those: &Sel) -> bool {
    match e {
        Effect::Seq(v) => v
            .last_mut()
            .is_some_and(|x| keep_controllers_before_move(x, those)),
        Effect::ShuffleIntoLibrary { .. }
        | Effect::ShuffleInto { .. }
        | Effect::Exile { .. }
        | Effect::Destroy { .. }
        | Effect::Move { .. } => {
            let moving = std::mem::take(e);
            *e = Effect::seq(vec![
                Effect::Store {
                    var: CONTROLLED_MOVED,
                    sel: Sel::Players(PlayerRef::ControllerOf(Box::new(those.clone()))),
                },
                moving,
            ]);
            true
        }
        _ => false,
    }
}

/// The cards each player's own reveal-until found, over all of them.
pub const FOUND_BY_EACH: Var = vars::USER + 6605;

/// The instruction recording, in [`FOUND_BY_EACH`], the card a reveal-until found.
fn store_found() -> Effect {
    Effect::Store {
        var: FOUND_BY_EACH,
        sel: Sel::Union(vec![Sel::Var(FOUND_BY_EACH), Sel::Var(vars::DUG_FOUND)]),
    }
}

/// Records, in [`FOUND_BY_EACH`], the card each player's reveal-until in `e` found: an
/// instruction of its own right after it (each player's results are their own until
/// they're collected, `simultaneous.rs`). Returns how many it recorded.
fn record_found(e: &mut Effect) -> usize {
    match e {
        Effect::RevealUntil { .. } => {
            let until = std::mem::take(e);
            *e = Effect::Seq(vec![until, store_found()]);
            1
        }
        Effect::Seq(v) => {
            let mut n = 0;
            let mut i = 0;
            while i < v.len() {
                if matches!(v[i], Effect::RevealUntil { .. }) {
                    v.insert(i + 1, store_found());
                    n += 1;
                    i += 1;
                } else {
                    n += record_found(&mut v[i]);
                }
                i += 1;
            }
            n
        }
        Effect::AsPlayer { effect, .. } => record_found(effect),
        _ => 0,
    }
}

/// "each player who controlled one of those permanents [instruction]", "each opponent who
/// controlled one or more of those creatures [instruction]" after an instruction that moved
/// them.
fn each_player_who_controlled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("each ") else {
        return false;
    };
    let Some((noun, r)) = r.split_once(" who controlled ") else {
        return false;
    };
    let among = match noun {
        "player" => None,
        "opponent" => Some(PlayerFilter::Opponent),
        _ => return false,
    };
    let Some(r) = r
        .strip_prefix("one of those ")
        .or_else(|| r.strip_prefix("one or more of those "))
    else {
        return false;
    };
    let Some((objects, pred)) = r.split_once(' ') else {
        return false;
    };
    if !matches!(objects, "permanents" | "creatures" | "artifacts" | "lands") {
        return false;
    }
    let Some(those) = moved_objects(prev) else {
        return false;
    };
    // The instruction for each of them: "each player [pred]", or "that player [pred]" for
    // each of them ("each player draws a card" is one instruction for all players). "It",
    // "that creature" and "them" in it would mean the permanent (or permanents) that
    // player controlled, which no referent of the builder says (the targets or the group
    // are all of them): such an instruction isn't supported.
    let saved_it = std::mem::replace(
        &mut b.it,
        super::oracle_hardening_referents::no_referent(),
    );
    let saved_group = b.group.take();
    let saved_named = std::mem::take(&mut b.named);
    let saved_chosen = b.chosen_creature.take();
    let targets = b.targets.len();
    let effect = match parse_sentence(&format!("each player {pred}"), b) {
        Some(Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect,
        }) => Some(effect),
        _ => {
            let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
            let e = parse_sentence(&format!("that player {pred}"), b);
            b.it_player = saved;
            e.filter(|e| !format!("{e:?}").contains("TriggerPlayer"))
                .map(Box::new)
        }
    };
    b.it = saved_it;
    b.group = saved_group;
    b.named = saved_named;
    b.chosen_creature = saved_chosen;
    // No referent, and none of the targets ("those permanents" are all of them).
    let refers = |e: &Effect| {
        serde_json::to_string(e).is_ok_and(|j| {
            j.contains(&format!(
                "\"Var\":{}",
                super::oracle_hardening_referents::NO_REFERENT
            )) || j.contains("\"Target\":")
        })
    };
    let Some(effect) = effect.filter(|e| b.targets.len() == targets && !refers(e)) else {
        b.targets.truncate(targets);
        return false;
    };
    // As they last existed on the battlefield (CR 608.2h): their controllers just before
    // they moved.
    if !keep_controllers_before_move(prev, &those) {
        return false;
    }
    let mut filter = vec![PlayerFilter::Ref(Box::new(PlayerRef::Var(
        CONTROLLED_MOVED,
    )))];
    filter.extend(among);
    let mut effect = effect;
    let found = record_found(&mut effect) > 0;
    let before = std::mem::take(prev);
    let mut v = vec![before];
    if found {
        v.push(Effect::Store {
            var: FOUND_BY_EACH,
            sel: Sel::Union(vec![]),
        });
    }
    v.push(Effect::ForEachPlayer {
        who: PlayerRef::Each(PlayerFilter::And(filter)),
        effect,
    });
    *prev = Effect::seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "per-player results 6: each player who controlled one of those permanents [instruction]", priority: 55, apply: each_player_who_controlled } }

/// Whether `e` recorded the card each player's reveal-until found.
fn recorded_found(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|j| j.contains(&format!("\"var\":{FOUND_BY_EACH},")))
}

/// "each player may cast the nonland card they exiled without paying its mana cost" after
/// each of several players exiled cards from the top of their own library until they
/// found one: each player's own card.
fn each_may_cast_the_card_they_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("each player may cast the ") else {
        return false;
    };
    let (r, free) = match r.strip_suffix(" without paying its mana cost") {
        Some(r) => (r, true),
        None => (r, false),
    };
    let r = r.strip_suffix(" this way").unwrap_or(r);
    let Some(desc) = r.strip_suffix(" they exiled") else {
        return false;
    };
    let Some((quality, false, tail)) = parse_object_phrase(desc) else {
        return false;
    };
    if !tail.trim().is_empty() {
        return false;
    }
    // The cards found: recorded for each player ("Each player who ... exiles cards ...
    // until ..."), or found by the "each player" instruction just before ("Each player
    // exiles cards from the top of their library until ...").
    if !recorded_found(prev) && !record_last(prev) {
        return false;
    }
    finish_cast(prev, quality, free)
}

/// Records the cards found by the "each player" reveal-until `e` ends with.
fn record_last(e: &mut Effect) -> bool {
    let init = Effect::Store {
        var: FOUND_BY_EACH,
        sel: Sel::Union(vec![]),
    };
    match e {
        Effect::Seq(v) => {
            let Some(last) = v.last_mut() else {
                return false;
            };
            if !record_each(last) {
                return false;
            }
            let n = v.len();
            v.insert(n - 1, init);
            true
        }
        Effect::ForEachPlayer { .. } => {
            if !record_each(e) {
                return false;
            }
            let each = std::mem::take(e);
            *e = Effect::seq(vec![init, each]);
            true
        }
        _ => false,
    }
}

/// Records the found card of the reveal-until each player of a "for each player"
/// instruction performs from their own library.
fn record_each(e: &mut Effect) -> bool {
    let Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer | PlayerRef::EachOpponent,
        effect,
    } = e
    else {
        return false;
    };
    let own = matches!(
        &**effect,
        Effect::AsPlayer {
            who: PlayerRef::Iterated,
            ..
        }
    ) && format!("{effect:?}").contains("RevealUntil { who: You");
    own && record_found(effect) > 0
}

fn finish_cast(prev: &mut Effect, quality: Filter, free: bool) -> bool {
    // A card's qualities wherever it is now (in exile).
    let quality = match quality {
        Filter::And(v) => Filter::and(
            v.into_iter()
                .filter(|x| !matches!(x, Filter::Card | Filter::InZone(_)))
                .collect(),
        ),
        Filter::Card => Filter::Any,
        f => f,
    };
    let cast = Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::CastCard {
            who: PlayerRef::Iterated,
            what: Sel::Matching(
                Box::new(Sel::Var(FOUND_BY_EACH)),
                Filter::and(vec![quality, Filter::OwnedBy(PlayerRel::Iterated)]),
            ),
            free,
            optional: true,
        }),
    };
    let before = std::mem::take(prev);
    *prev = Effect::seq(vec![before, cast]);
    true
}

inventory::submit! { FollowupPattern { name: "per-player results 6: each player may cast the [card] they exiled", priority: 55, apply: each_may_cast_the_card_they_exiled } }

// ---------------------------------------------------------------------------
// They each put one of those cards ... and the rest ...
// ---------------------------------------------------------------------------

/// "They each put one of those cards onto the battlefield tapped under your control and the
/// rest onto the battlefield tapped under their control." after "Each opponent may search
/// their library for [cards]." (`each_opponent_may`): each opponent in turn decides, then
/// searches and puts the cards they found.
fn they_each_put(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    use super::each_opponent_may::ACCEPTED;
    use crate::scry_rules::{OPTED, OPT_IN};
    let l = end(l);
    let Some(x) = l.strip_prefix("they each put ") else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    let n = v.len();
    if n < 4 {
        return false;
    }
    let ok = matches!(
        &v[n - 4],
        Effect::Store {
            var: OPTED,
            sel: Sel::None
        }
    ) && matches!(
        &v[n - 3],
        Effect::ForEachPlayer {
            who: PlayerRef::EachOpponent,
            ..
        }
    ) && matches!(
        &v[n - 2],
        Effect::Store {
            var: ACCEPTED,
            sel: Sel::Var(OPTED)
        }
    );
    if !ok {
        return false;
    }
    let Effect::ForEachPlayer {
        who: PlayerRef::Var(ACCEPTED),
        effect,
    } = &v[n - 1]
    else {
        return false;
    };
    let Effect::AsPlayer {
        who: PlayerRef::Iterated,
        effect: inner,
    } = &**effect
    else {
        return false;
    };
    let Effect::SearchCards(spec) = &**inner else {
        return false;
    };
    if !matches!((&spec.who, &spec.whose), (PlayerRef::You, PlayerRef::You))
        || !spec.dests.is_empty()
        || spec.shuffle != SearchShuffle::No
    {
        return false;
    }
    let Some(x) = x
        .strip_prefix("one of those cards ")
        .map(|r| format!("one of those cards {r}"))
        .or_else(|| x.strip_prefix("those cards ").map(|r| format!("them {r}")))
    else {
        return false;
    };
    let Some(dests) = super::search_grammar::each_searchers_dests(&x, b) else {
        return false;
    };
    let mut spec = (**spec).clone();
    spec.who = PlayerRef::Iterated;
    spec.whose = PlayerRef::Iterated;
    spec.dests = dests;
    // One opponent at a time, in APNAP order (Rootweaver Druid's rulings).
    let each = Effect::InTurnOrder {
        first: TurnOrderStart::Active,
        who: PlayerFilter::Opponent,
        direction: SeatSide::Left,
        effect: Box::new(Effect::seq(vec![
            Effect::May {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::Custom(OPT_IN.into())),
            },
            Effect::If {
                cond: Condition::PlayerMatches(
                    PlayerRef::Iterated,
                    PlayerFilter::Ref(Box::new(PlayerRef::Var(OPTED))),
                ),
                then: Box::new(Effect::SearchCards(Box::new(spec))),
                otherwise: Box::new(Effect::Noop),
            },
        ])),
    };
    v.truncate(n - 4);
    v.push(Effect::Store {
        var: OPTED,
        sel: Sel::None,
    });
    v.push(each);
    v.push(Effect::Store {
        var: ACCEPTED,
        sel: Sel::Var(OPTED),
    });
    true
}

inventory::submit! { FollowupPattern { name: "per-player results 6: they each put one of those cards ... and the rest ...", priority: 55, apply: they_each_put } }

/// "Then each player who searched their library this way shuffles." after [`they_each_put`].
fn searchers_shuffle(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    use super::each_opponent_may::ACCEPTED;
    use crate::scry_rules::OPTED;
    let l = end(l);
    if l.strip_prefix("then ").unwrap_or(l)
        != "each player who searched their library this way shuffles"
    {
        return false;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let n = v.len();
    if n < 2
        || !matches!(
            &v[n - 1],
            Effect::Store {
                var: ACCEPTED,
                sel: Sel::Var(OPTED)
            }
        )
    {
        return false;
    }
    let searches = matches!(&v[n - 2], Effect::InTurnOrder { first: TurnOrderStart::Active, effect, .. }
        if format!("{effect:?}").contains("SearchCards(SearchSpec { who: Iterated, whose: Iterated"));
    if !searches {
        return false;
    }
    v.push(Effect::ForEachPlayer {
        who: PlayerRef::Var(ACCEPTED),
        effect: Box::new(Effect::Shuffle {
            who: PlayerRef::Iterated,
        }),
    });
    true
}

inventory::submit! { FollowupPattern { name: "per-player results 6: then each player who searched their library this way shuffles (after they each put)", priority: 55, apply: searchers_shuffle } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn per_player_results() {
        for (ty, text) in [
            ("Artifact", "At the beginning of each upkeep, each player exiles the top card of their library. The player who exiled the card with the greatest mana value takes an extra turn after this one. If two or more players' cards are tied for greatest, the tied players repeat this process until the tie is broken."),
            ("Sorcery", "Each player exiles the top card of their library. The player who exiled the card with the greatest mana value draws a card."),
            ("Sorcery", "Each player reveals the top card of their library. Each player who revealed the card with the greatest mana value draws a card."),
            ("Sorcery", "Each opponent exiles the top card of their library. The player who exiled the card with the greatest mana value loses 3 life."),
            ("Instant", "For each player, choose target nonenchantment, nonland permanent that player controls. Those permanents' owners shuffle them into their libraries. Each player who controlled one of those permanents exiles cards from the top of their library until they exile a nonland card, then puts the rest on the bottom of their library in a random order. Each player may cast the nonland card they exiled without paying its mana cost."),
            ("Instant", "For each player, choose target nonenchantment, nonland permanent that player controls. Shuffle those permanents into their owners' libraries. Each player who controlled one of those permanents draws a card."),
            ("Sorcery", "Each player exiles cards from the top of their library until they exile a nonland card. Each player may cast the nonland card they exiled without paying its mana cost."),
            ("Creature — Elf Druid", "When this creature enters, each opponent may search their library for up to three basic land cards. They each put one of those cards onto the battlefield tapped under your control and the rest onto the battlefield tapped under their control. Then each player who searched their library this way shuffles."),
            ("Creature — Elf Druid", "When this creature enters, each opponent may search their library for up to three basic land cards. They each put one of those cards onto the battlefield tapped under your control and the rest into their hand. Then each player who searched their library this way shuffles."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
        for (ty, text) in [
            // A comparison of cards no instruction moved.
            ("Sorcery", "Draw a card. The player who exiled the card with the greatest mana value draws a card."),
            // A tie loop needs the comparison.
            ("Sorcery", "Each player exiles the top card of their library. If two or more players' cards are tied for greatest, the tied players repeat this process until the tie is broken."),
            // "It" would be the permanent that player controlled, which no referent says.
            ("Sorcery", "For each player, choose target creature that player controls. Exile those creatures. Each player who controlled one of those creatures returns it to the battlefield."),
            ("Sorcery", "For each player, choose target creature that player controls. Exile those creatures. Each player who controlled one of those creatures gains life equal to its power."),
            ("Sorcery", "For each player, choose target creature that player controls. Destroy those creatures. Each player who controlled one of those creatures creates a token that's a copy of that creature."),
        ] {
            assert!(compiled(ty, text).is_none(), "{text}");
        }
    }
}
