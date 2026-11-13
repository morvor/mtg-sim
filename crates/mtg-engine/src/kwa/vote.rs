//! CR 701.38: vote.
//!
//! * To vote, each player, starting with a specified player and proceeding in turn order,
//!   chooses one of the listed choices (CR 701.38a). The choices are words (each connected
//!   to an effect) or objects (CR 701.38b): a [`Spec`] with `options` (the words) or with
//!   `what` (the objects that may be voted for).
//! * Only an actual vote is "voting" (CR 701.38c): the extra votes of [`EXTRA_VOTE`] and
//!   [`OPTIONAL_EXTRA_VOTE`] ("While voting, you get an additional vote" / "you may vote an
//!   additional time") apply only here, not to other choices players make.
//! * A player with several votes casts them all at the time they'd otherwise vote
//!   (CR 701.38d).
//!
//! * In a secret vote ("each player secretly votes for ..., then those votes are
//!   revealed", secret council), no player learns another's vote while voting: the votes
//!   are revealed together once every player has voted.
//!
//! The result is stored for the instructions that follow: the number of votes for each
//! word (`Value::Var(word_var(word))`), the most votes any choice got ([`MOST_VOTES`]),
//! how many choices got that many ([`CHOICES_WITH_MOST`]), the objects with the most
//! votes or tied for most ([`WINNERS`]), and each object voted for once per vote it got
//! ([`VOTES_CAST`]; "for each creature with one or more votes, put that many stun
//! counters on it": [`GOT_VOTES`], [`VOTES_FOR_IT`]).

use super::*;

/// `StaticEffect::Custom` names: "While voting, you get an additional vote." / "While
/// voting, you may vote an additional time."
pub const EXTRA_VOTE: &str = "vote: you get an additional vote";
pub const OPTIONAL_EXTRA_VOTE: &str = "vote: you may vote an additional time";
/// `Event::Custom` name reported for each vote cast (the object voted for, if any; the
/// amount is the index of the word voted for, or -1).
pub const VOTED: &str = "vote";
/// `Event::Custom` name reported once all players have voted ("whenever players finish
/// voting").
pub const FINISHED_VOTING: &str = "finished voting";

/// `Effect::Custom` name: "You choose how each player votes this turn." (Illusion of
/// Choice): recorded among the turn's player actions; the player who most recently did
/// so makes every player's voting choices this turn, while each player still decides
/// whether to take an optional additional vote (Illusion of Choice's rulings).
pub const CHOOSES_VOTES: &str = "vote: you choose how each player votes this turn";

/// Who chooses how `voter` votes: the controller of the most recent "you choose how each
/// player votes this turn" effect still in the game, or the voter.
fn vote_chooser(g: &Game, voter: PlayerId) -> PlayerId {
    g.history
        .custom_actions
        .iter()
        .rev()
        .find(|(p, n)| n == CHOOSES_VOTES && g.player(*p).in_game())
        .map_or(voter, |(p, _)| *p)
}

/// The events of this turn so far, in order.
fn events_this_turn(g: &Game) -> impl Iterator<Item = &Event> {
    g.turn_events.iter().chain(g.events.iter())
}

/// How many votes have finished this turn.
fn finished_votes(g: &Game) -> i32 {
    events_this_turn(g)
        .filter(|e| matches!(e, Event::Custom { name, .. } if name == FINISHED_VOTING))
        .count() as i32
}

/// The votes cast in the vote that finished as the `serial`th this turn: (voter, the
/// object voted for or the index of the word).
fn votes_in(g: &Game, serial: i32) -> Vec<(PlayerId, i64)> {
    let mut finished = 0;
    let mut votes = Vec::new();
    for e in events_this_turn(g) {
        let Event::Custom {
            name,
            player: Some(p),
            obj,
            amount,
        } = e
        else {
            continue;
        };
        if name == VOTED {
            let choice = obj.map_or(*amount as i64, |o| (1i64 << 40) + o.0 as i64);
            votes.push((*p, choice));
        } else if name == FINISHED_VOTING {
            finished += 1;
            if finished == serial {
                return votes;
            }
            votes.clear();
        }
    }
    vec![]
}

/// For an ability that triggered on players finishing a vote (its event's amount is the
/// vote's number this turn): whether `p` voted for a choice the ability's controller voted
/// for (`Some(true)`), voted only for choices they didn't vote for (`Some(false)`), or
/// didn't vote (`None`). Only that vote counts, not earlier ones (Grudge Keeper's
/// rulings).
pub fn voted_like(g: &Game, ctx: &Ctx, p: PlayerId) -> Option<bool> {
    let serial = ctx.event.as_ref().map(|e| e.amount)?;
    let votes = votes_in(g, serial);
    let mine: Vec<i64> = votes
        .iter()
        .filter(|(v, _)| *v == ctx.controller)
        .map(|(_, c)| *c)
        .collect();
    let theirs: Vec<i64> = votes
        .iter()
        .filter(|(v, _)| *v == p)
        .map(|(_, c)| *c)
        .collect();
    if theirs.is_empty() {
        return None;
    }
    Some(theirs.iter().any(|c| mine.contains(c)))
}

/// Result variables (`Ctx::nums` / `Ctx::vars`).
pub const MOST_VOTES: Var = vars::USER + 1038;
pub const CHOICES_WITH_MOST: Var = vars::USER + 1039;
pub const WINNERS: Var = vars::USER + 1040;
/// Each object voted for, once per vote it got.
pub const VOTES_CAST: Var = vars::USER + 1041;
/// The object an instruction about "each [object] with one or more votes" is performed
/// for.
pub const VOTED_FOR: Var = vars::USER + 1042;
/// `Filter::Custom`: an object that got one or more votes in the vote earlier in the
/// spell or ability.
pub const GOT_VOTES: &str = "vote: got one or more votes";
/// `Value::Custom`: the number of votes the object in [`VOTED_FOR`] got.
pub const VOTES_FOR_IT: &str = "vote: votes for it";

/// The variable holding the number of votes for `word`.
pub fn word_var(word: &str) -> Var {
    vars::USER + 2000 + (word_hash(word) % 20000) as Var
}

/// The variable holding the voters of the votes for `word`, once per vote ("for each
/// money vote, choose a permanent owned by the voter").
pub fn word_voters_var(word: &str) -> Var {
    vars::USER + 40000 + (word_hash(word) % 20000) as Var
}

fn word_hash(word: &str) -> u32 {
    // FNV-1a.
    let mut h: u32 = 0x811c9dc5;
    for b in word.to_lowercase().bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// `Value::Custom`: the number of votes the player performing the instruction ("you")
/// received in the vote earlier in the spell or ability ("Each player draws a card for
/// each vote they received").
pub const VOTES_RECEIVED: &str = "vote: votes received";

/// How many votes `p` casts (CR 701.38d).
fn votes_of(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> u32 {
    let customs = g.statics.customs.clone();
    let mut n = 1;
    for (_, ctl, name) in customs {
        if ctl != p {
            continue;
        }
        if name == EXTRA_VOTE
            || (name == OPTIONAL_EXTRA_VOTE
                && g.ask_yes_no(p, source, "Vote an additional time?", true))
        {
            n += 1;
        }
    }
    n
}

pub struct Vote;

impl KeywordActionRules for Vote {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Vote]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let words: Vec<String> = a
            .spec
            .map(|s| s.options.iter().map(|(w, _)| w.clone()).collect())
            .unwrap_or_default();
        // Votes for players ("each player secretly votes for a player"; "for another
        // player": a player other than the voter).
        let players: Option<&PlayerRef> = match a.what {
            Sel::Players(r) if words.is_empty() => Some(r),
            _ => None,
        };
        let objects: Vec<ObjectId> = if words.is_empty() && players.is_none() {
            g.resolve_objects(a.what, ctx)
        } else {
            vec![]
        };
        // "votes for up to one creature": a voter may vote for none (n = 0).
        let up_to = matches!(a.n, Value::Const(0));
        // Starting with the specified player, in turn order.
        let first = g.eval_player(a.who, ctx).unwrap_or(ctx.controller);
        let mut order = vec![first];
        let mut p = g.next_player(first);
        while p != first && !order.contains(&p) {
            order.push(p);
            p = g.next_player(p);
        }
        let secret = a.spec.is_some_and(|s| s.secret);
        let mut word_votes = vec![0i64; words.len()];
        let mut object_votes: Vec<(ObjectId, i64)> = objects.iter().map(|o| (*o, 0)).collect();
        let mut player_votes: Vec<(PlayerId, i64)> = match players {
            Some(_) => g.players_in_game().into_iter().map(|p| (p, 0)).collect(),
            None => vec![],
        };
        // The votes cast: (voter, word index, object voted for).
        let mut cast: Vec<(PlayerId, usize, Option<ObjectId>)> = Vec::new();
        // The players voted for: (voter, player).
        let mut cast_for_players: Vec<(PlayerId, PlayerId)> = Vec::new();
        for voter in order {
            if !g.player(voter).in_game() {
                continue;
            }
            for _ in 0..votes_of(g, voter, ctx.source) {
                let chooser = vote_chooser(g, voter);
                if let Some(r) = players {
                    let cands: Vec<Entity> = match r {
                        PlayerRef::EachOtherPlayer => g
                            .players_in_game()
                            .into_iter()
                            .filter(|p| *p != voter)
                            .collect(),
                        r => g.eval_players(r, ctx),
                    }
                    .into_iter()
                    .map(Entity::Player)
                    .collect();
                    let Some(pick) = g
                        .ask_entities(chooser, ctx.source, "Vote", cands.clone(), 1, 1)
                        .first()
                        .and_then(|e| e.player())
                        .or_else(|| cands.first().and_then(|e| e.player()))
                    else {
                        continue;
                    };
                    if let Some(e) = player_votes.iter_mut().find(|(q, _)| *q == pick) {
                        e.1 += 1;
                    }
                    cast_for_players.push((voter, pick));
                    if !secret {
                        reveal_player_vote(g, voter, pick);
                    }
                    continue;
                }
                let vote = if !words.is_empty() {
                    let i = g.ask_option(chooser, ctx.source, "Vote", words.clone());
                    let i = i.min(words.len() - 1);
                    word_votes[i] += 1;
                    (voter, i, None)
                } else if !objects.is_empty() {
                    let picked = g
                        .ask_objects(
                            chooser,
                            ctx.source,
                            "Vote",
                            objects.clone(),
                            if up_to { 0 } else { 1 },
                            1,
                        )
                        .first()
                        .copied();
                    let pick = match picked {
                        Some(o) => o,
                        // Voting for no object ("up to one").
                        None if up_to => continue,
                        None => objects[0],
                    };
                    if let Some(e) = object_votes.iter_mut().find(|(o, _)| *o == pick) {
                        e.1 += 1;
                    }
                    (voter, 0, Some(pick))
                } else {
                    continue;
                };
                cast.push(vote);
                if !secret {
                    reveal(g, &words, vote);
                }
            }
        }
        // A secret vote: all the votes are revealed at the same time.
        if secret {
            g.log(|_| "The votes are revealed".to_string());
            for vote in &cast {
                reveal(g, &words, *vote);
            }
            for (voter, pick) in &cast_for_players {
                reveal_player_vote(g, *voter, *pick);
            }
        }
        ctx.set_var(
            VOTES_CAST,
            cast.iter()
                .filter_map(|(_, _, o)| o.map(Entity::Object))
                .chain(cast_for_players.iter().map(|(_, p)| Entity::Player(*p)))
                .collect(),
        );
        for (i, w) in words.iter().enumerate() {
            ctx.set_var(
                word_voters_var(w),
                cast.iter()
                    .filter(|(_, j, o)| o.is_none() && *j == i)
                    .map(|(v, _, _)| Entity::Player(*v))
                    .collect(),
            );
        }
        let counts: Vec<i64> = if players.is_some() {
            player_votes.iter().map(|(_, n)| *n).collect()
        } else if words.is_empty() {
            object_votes.iter().map(|(_, n)| *n).collect()
        } else {
            word_votes.clone()
        };
        let most = counts.iter().copied().max().unwrap_or(0);
        let with_most = counts.iter().filter(|n| **n == most).count() as i64;
        for (w, n) in words.iter().zip(&word_votes) {
            ctx.nums.insert(word_var(w), *n);
        }
        ctx.nums.insert(MOST_VOTES, most);
        ctx.nums.insert(CHOICES_WITH_MOST, with_most);
        let winners: Vec<Entity> = object_votes
            .iter()
            .filter(|(_, n)| *n == most && most > 0)
            .map(|(o, _)| Entity::Object(*o))
            .chain(
                player_votes
                    .iter()
                    .filter(|(_, n)| *n == most && most > 0)
                    .map(|(p, _)| Entity::Player(*p)),
            )
            .collect();
        ctx.set_var(WINNERS, winners);
        // The vote's number this turn identifies its votes for the abilities that
        // trigger on it (`voted_like`).
        let serial = finished_votes(g) + 1;
        emit(g, FINISHED_VOTING, ctx.controller, None, serial);
    }
}

inventory::submit! { KeywordActionRegistration(&Vote) }

/// Logs and reports one vote (`words` are the choices of a vote on words).
fn reveal(g: &mut Game, words: &[String], (voter, i, pick): (PlayerId, usize, Option<ObjectId>)) {
    match pick {
        Some(o) => {
            g.log(|g| format!("{voter} votes for {}", g.describe(o)));
            emit(g, VOTED, voter, Some(o), -1);
        }
        None => {
            g.log(|_| format!("{voter} votes for {}", words[i]));
            emit(g, VOTED, voter, None, i as i32);
        }
    }
}

/// Logs and reports one vote for a player.
fn reveal_player_vote(g: &mut Game, voter: PlayerId, pick: PlayerId) {
    g.log(|_| format!("{voter} votes for {pick}"));
    emit(g, VOTED, voter, None, -2 - pick.0 as i32);
}

/// The objects in `ctx`'s [`VOTES_CAST`], once per vote.
fn votes_cast(ctx: &Ctx) -> &[Entity] {
    ctx.vars.get(&VOTES_CAST).map_or(&[], |v| v.as_slice())
}

/// The results of a vote on objects, for the instructions that follow it: [`GOT_VOTES`]
/// and [`VOTES_FOR_IT`].
pub struct VoteResults;

impl crate::kw::KeywordRules for VoteResults {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CHOOSES_VOTES {
            return false;
        }
        g.history
            .custom_actions
            .push((ctx.controller, SmolStr::new(CHOOSES_VOTES)));
        true
    }

    fn custom_filter(&self, _g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == GOT_VOTES).then(|| votes_cast(ctx).contains(&Entity::Object(id)))
    }

    fn custom_value(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == VOTES_RECEIVED {
            let me = Entity::Player(ctx.controller);
            return Some(votes_cast(ctx).iter().filter(|e| **e == me).count() as i64);
        }
        if name != VOTES_FOR_IT {
            return None;
        }
        let it = ctx.vars.get(&VOTED_FOR).and_then(|v| v.first().copied());
        Some(it.map_or(0, |it| {
            votes_cast(ctx).iter().filter(|e| **e == it).count() as i64
        }))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&VoteResults) }
