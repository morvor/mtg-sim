//! Follow-up grammar IV: per-player outcomes of an instruction several players performed
//! (CR 101.4, 608.2c, 608.2e).
//!
//! - Each player's own number of cards from the instruction, in a value read for each
//!   player in turn: "Each player may discard up to three cards. ~ deals damage to each
//!   player equal to 3 minus the number of cards they discarded this way." (Mind Bomb).
//! - "For each card less than two a player draws this way, that player gains 2 life."
//!   after "Each player may draw up to two cards." (Truce).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::scry_rules::OPT_IN;

/// Each player's number of cards drawn by the latest "each player [may] draw" instruction
/// (see [`record_draws`]).
pub const CARDS_DRAWN: Var = vars::USER + 41000;

thread_local! {
    /// Set while a value is read for each player in turn (so "they" in "the number of
    /// cards they discarded this way" is the player it's read for).
    static PER_PLAYER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The players who accepted an earlier "each [players] may [instruction]" and what
    /// the instruction's verb is, while the next sentence is read ("the number of
    /// opponents who investigated this way").
    static ACCEPTED: std::cell::Cell<Option<(Var, &'static str)>> = const { std::cell::Cell::new(None) };
}

fn with_per_player<T>(f: impl FnOnce() -> T) -> T {
    let saved = PER_PLAYER.with(|c| c.replace(true));
    let out = f();
    PER_PLAYER.with(|c| c.set(saved));
    out
}

/// Which per-player record a verb reads: discards record each player's number of cards
/// acted on (`hand_graveyard_grammar`), draws are recorded by [`record_draws`].
fn tally_of_verb(verb: &str) -> Option<Var> {
    Some(match verb {
        "discarded" => crate::tallies::CARDS_ACTED_ON,
        "drew" => CARDS_DRAWN,
        _ => return None,
    })
}

/// The value grammar's atom (only while a value is read for each player): "the number of
/// cards they discarded this way", "the number of cards that player drew this way".
pub fn own_count_atom(s: &str, _b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = accepted_count_atom(s) {
        return Some(v);
    }
    if let Some(v) = super::notes_hidden_4::noted_atom(s) {
        return Some(v);
    }
    if !PER_PLAYER.with(|c| c.get()) {
        return None;
    }
    let r = s.strip_prefix("the number of cards ")?;
    let r = r
        .strip_prefix("they ")
        .or_else(|| r.strip_prefix("that player "))?;
    let (verb, rest) = r.split_once(' ')?;
    let rest = rest.strip_prefix("this way")?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    let var = tally_of_verb(verb)?;
    Some((
        Value::PlayerTally {
            var,
            who: Box::new(PlayerRef::Iterated),
        },
        rest.to_string(),
    ))
}

/// "the number of opponents who investigated this way" after "each opponent may
/// investigate": the players who accepted (an instruction that can always be followed,
/// so each player who accepted did it, CR 118.12).
fn accepted_count_atom(s: &str) -> Option<(Value, String)> {
    let (var, past) = ACCEPTED.with(|c| c.get())?;
    let r = s.strip_prefix("the number of ")?;
    let r = r
        .strip_prefix("opponents who ")
        .or_else(|| r.strip_prefix("players who "))?;
    let rest = r.strip_prefix(past)?.strip_prefix(" this way")?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((Value::CountSel(Box::new(Sel::Var(var))), rest.to_string()))
}

/// The instructions an opt-in may offer that can always be followed, by their past tense.
fn always_doable(e: &Effect) -> Option<&'static str> {
    match e {
        Effect::AsPlayer { effect, .. } => always_doable(effect),
        Effect::KeywordAction {
            action: KeywordAction::Investigate,
            ..
        } => Some("investigated"),
        _ => None,
    }
}

/// The variable holding the players who accepted the "each [players] may [instruction]"
/// in `v`, and the instruction's past tense.
fn accepted_in(v: &[Effect]) -> Option<(Var, &'static str)> {
    let i = v.iter().position(
        |e| matches!(e, Effect::Store { sel: Sel::Var(o), .. } if *o == crate::scry_rules::OPTED),
    )?;
    let Effect::Store { var, .. } = &v[i] else {
        return None;
    };
    let Effect::ForEachPlayer {
        who: PlayerRef::Var(w),
        effect,
    } = v.get(i + 1)?
    else {
        return None;
    };
    if w != var {
        return None;
    }
    Some((*var, always_doable(effect)?))
}

/// A sentence after "each [players] may [instruction]" that counts the players who did:
/// "You investigate X times, where X is one plus the number of opponents who investigated
/// this way." (Wernog, Rider's Chaplain).
fn counts_accepted(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !l.contains(" who ") || !l.contains(" this way") {
        return false;
    }
    let Effect::Seq(v) = prev else {
        return false;
    };
    let Some(acc) = accepted_in(v) else {
        return false;
    };
    let saved = ACCEPTED.with(|c| c.replace(Some(acc)));
    let e = crate::oracle::effects::parse_sentence(l, b);
    ACCEPTED.with(|c| c.set(saved));
    let Some(e) = e else {
        return false;
    };
    if !format!("{e:?}").contains(&format!("CountSel(Var({}))", acc.0)) {
        return false;
    }
    v.push(e);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: the number of players who did it this way", priority: 135, apply: counts_accepted } }

fn uses_tally(v: &Value, var: Var) -> bool {
    format!("{v:?}").contains(&format!("PlayerTally {{ var: {var}, who: Iterated }}"))
}

/// The opt-in sequence ("Store OPTED; for each player: may ...; Store accepted; for each
/// accepted player: ...") or plain "each player" instruction `prev` ends with: the
/// instruction each player performs.
fn each_player_step(prev: &mut Effect) -> Option<&mut Effect> {
    match prev {
        Effect::Seq(v) => {
            let last = v.last_mut()?;
            if matches!(last, Effect::ForEachPlayer { .. }) {
                each_player_step(last)
            } else {
                None
            }
        }
        Effect::ForEachPlayer { effect, .. } => {
            if format!("{effect:?}").contains(OPT_IN) {
                return None;
            }
            Some(effect)
        }
        _ => None,
    }
}

/// Whether the instruction each player performs is a card action that records each
/// player's number (`hand_graveyard_grammar::record`), or a draw (recorded here).
fn records(e: &Effect, verb_var: Var) -> bool {
    let d = format!("{e:?}");
    if verb_var == crate::tallies::CARDS_ACTED_ON {
        return d.contains(crate::kw::hand_graveyard_actions::RECORD_THAT_MANY)
            && d.contains("Discard");
    }
    d.contains("Draw {")
}

/// Appends, after each draw in `e`, the number of cards the player drew to their
/// [`CARDS_DRAWN`] total. Returns whether there was exactly one draw.
fn record_draws(e: &mut Effect) -> usize {
    match e {
        Effect::Seq(v) => {
            let mut n = 0;
            let mut i = 0;
            while i < v.len() {
                if matches!(v[i], Effect::Draw { .. }) {
                    v.insert(
                        i + 1,
                        Effect::AddToPlayerTally {
                            var: CARDS_DRAWN,
                            who: PlayerRef::Iterated,
                            value: Value::Prev,
                        },
                    );
                    n += 1;
                    i += 2;
                } else {
                    n += record_draws(&mut v[i]);
                    i += 1;
                }
            }
            n
        }
        Effect::AsPlayer { effect, .. } => {
            if matches!(**effect, Effect::Draw { .. }) {
                let d = std::mem::take(&mut **effect);
                **effect = Effect::Seq(vec![
                    d,
                    Effect::AddToPlayerTally {
                        var: CARDS_DRAWN,
                        who: PlayerRef::Iterated,
                        value: Value::Prev,
                    },
                ]);
                return 1;
            }
            record_draws(effect)
        }
        _ => 0,
    }
}

/// Makes sure the instruction each player performed records the per-player number `var`
/// reads.
fn ensure_recorded(prev: &mut Effect, var: Var) -> bool {
    let Some(step) = each_player_step(prev) else {
        return false;
    };
    if var == CARDS_DRAWN {
        if format!("{step:?}").contains(&format!("var: {CARDS_DRAWN}")) {
            return true;
        }
        return records(step, var) && record_draws(step) == 1;
    }
    records(step, var)
}

/// The players an "each [players]" phrase names.
fn each_group(w: &str) -> Option<PlayerRef> {
    Some(match w {
        "each player" => PlayerRef::EachPlayer,
        "each opponent" => PlayerRef::EachOpponent,
        _ => return None,
    })
}

/// Parses `clause` (about "that player", with one "x") for each player in turn, with x
/// the per-player value `v`.
fn per_player_clause(clause: &str, v: &Value, b: &mut Builder) -> Option<Effect> {
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let e = super::value_grammar::with_x_defined(true, || parse_clause(clause, b));
    b.it_player = saved;
    let e = e?;
    let d = format!("{e:?}");
    // Only about that player.
    if d.contains("EachPlayer") || d.contains("EachOpponent") {
        return None;
    }
    super::r107_numbers::substitute_x(&e, v)
}

/// "~ deals damage to each player equal to 3 minus the number of cards they discarded
/// this way", "each player loses life equal to the number of cards they drew this way":
/// for each of those players, their own number.
fn own_count_equal_to(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some((head, value)) = l.split_once(" equal to ") else {
        return false;
    };
    let Some((v, rest)) = with_per_player(|| super::value_grammar::parse_value(value, b)) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let Some(var) = [crate::tallies::CARDS_ACTED_ON, CARDS_DRAWN]
        .into_iter()
        .find(|var| uses_tally(&v, *var))
    else {
        return false;
    };
    // The group: the subject ("each player loses life") or the recipient ("~ deals
    // damage to each player").
    let (group, clause) = if let Some((g, r)) = ["each player ", "each opponent "]
        .iter()
        .find_map(|g| head.strip_prefix(g).map(|r| (g.trim_end(), r)))
    {
        let pred = match r {
            "loses life" => "loses x life",
            "gains life" => "gains x life",
            "draws cards" => "draws x cards",
            "mills cards" => "mills x cards",
            _ => return false,
        };
        (g, format!("that player {pred}"))
    } else {
        let Some((h, g)) = head.rsplit_once(" to ") else {
            return false;
        };
        let Some(h) = h.strip_suffix(" deals damage") else {
            return false;
        };
        (g, format!("{h} deals x damage to that player"))
    };
    let Some(who) = each_group(group) else {
        return false;
    };
    if !ensure_recorded(prev, var) {
        return false;
    }
    let Some(e) = per_player_clause(&clause, &v, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::ForEachPlayer {
            who,
            effect: Box::new(e),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: each player's own count equal to", priority: 130, apply: own_count_equal_to } }

/// "For each card less than two a player draws this way, that player gains 2 life."
/// (Truce): for each player, the shortfall of their own draw.
fn for_each_card_less_than(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("for each card ") else {
        return false;
    };
    let Some(r) = r
        .strip_prefix("less than ")
        .or_else(|| r.strip_prefix("fewer than "))
    else {
        return false;
    };
    let Some((n, r)) = parse_number(r) else {
        return false;
    };
    let Some((verb, clause)) = r.trim_start().split_once(" this way, ") else {
        return false;
    };
    let var = match verb {
        "a player draws" | "a player drew" => CARDS_DRAWN,
        "a player discards" | "a player discarded" => crate::tallies::CARDS_ACTED_ON,
        _ => return false,
    };
    if !ensure_recorded(prev, var) {
        return false;
    }
    let short = Value::Max(
        Box::new(Value::c(0)),
        Box::new(Value::Diff(
            Box::new(n),
            Box::new(Value::PlayerTally {
                var,
                who: Box::new(PlayerRef::Iterated),
            }),
        )),
    );
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let e = parse_clause(clause, b);
    b.it_player = saved;
    let Some(e) = e else {
        return false;
    };
    let Some(e) = super::hand_graveyard_grammar::scale(e, short) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(e),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: for each card less than N a player draws this way", priority: 130, apply: for_each_card_less_than } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn per_player_counts() {
        for (ty, text) in [
            ("Sorcery", "Each player may discard up to three cards. ~ deals damage to each player equal to 3 minus the number of cards they discarded this way."),
            ("Sorcery", "Each player may discard up to two cards. Each player loses life equal to the number of cards they discarded this way."),
            ("Sorcery", "Each player may draw up to two cards. For each card less than two a player draws this way, that player gains 2 life."),
            ("Sorcery", "Each player may draw up to three cards. Each opponent loses life equal to the number of cards they drew this way."),
            ("Creature", "When ~ enters, each opponent may investigate. Each opponent who doesn't loses 1 life. You investigate X times, where X is one plus the number of opponents who investigated this way."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
