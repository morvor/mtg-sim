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
//! The result is stored for the instructions that follow: the number of votes for each
//! word (`Value::Var(word_var(word))`), the most votes any choice got ([`MOST_VOTES`]),
//! how many choices got that many ([`CHOICES_WITH_MOST`]), and the objects with the most
//! votes or tied for most ([`WINNERS`]).

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

/// Result variables (`Ctx::nums` / `Ctx::vars`).
pub const MOST_VOTES: Var = vars::USER + 1038;
pub const CHOICES_WITH_MOST: Var = vars::USER + 1039;
pub const WINNERS: Var = vars::USER + 1040;

/// The variable holding the number of votes for `word`.
pub fn word_var(word: &str) -> Var {
    // FNV-1a, into a range of its own.
    let mut h: u32 = 0x811c9dc5;
    for b in word.to_lowercase().bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    vars::USER + 2000 + (h % 20000) as Var
}

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
        let objects: Vec<ObjectId> = if words.is_empty() {
            g.resolve_objects(a.what, ctx)
        } else {
            vec![]
        };
        // Starting with the specified player, in turn order.
        let first = g.eval_player(a.who, ctx).unwrap_or(ctx.controller);
        let mut order = vec![first];
        let mut p = g.next_player(first);
        while p != first && !order.contains(&p) {
            order.push(p);
            p = g.next_player(p);
        }
        let mut word_votes = vec![0i64; words.len()];
        let mut object_votes: Vec<(ObjectId, i64)> = objects.iter().map(|o| (*o, 0)).collect();
        for voter in order {
            if !g.player(voter).in_game() {
                continue;
            }
            for _ in 0..votes_of(g, voter, ctx.source) {
                if !words.is_empty() {
                    let i = g.ask_option(voter, ctx.source, "Vote", words.clone());
                    let i = i.min(words.len() - 1);
                    word_votes[i] += 1;
                    g.log(|_| format!("{voter} votes for {}", words[i]));
                    emit(g, VOTED, voter, None, i as i32);
                } else if !objects.is_empty() {
                    let pick = g
                        .ask_objects(voter, ctx.source, "Vote", objects.clone(), 1, 1)
                        .first()
                        .copied()
                        .unwrap_or(objects[0]);
                    if let Some(e) = object_votes.iter_mut().find(|(o, _)| *o == pick) {
                        e.1 += 1;
                    }
                    g.log(|g| format!("{voter} votes for {}", g.describe(pick)));
                    emit(g, VOTED, voter, Some(pick), -1);
                }
            }
        }
        let counts: Vec<i64> = if words.is_empty() {
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
            .collect();
        ctx.set_var(WINNERS, winners);
        emit(g, FINISHED_VOTING, ctx.controller, None, 0);
    }
}

inventory::submit! { KeywordActionRegistration(&Vote) }
