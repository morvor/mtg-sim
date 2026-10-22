//! Follow-up grammar IV: per-player outcomes of an instruction several players performed
//! (CR 101.4, 608.2c, 608.2e).
//!
//! - Each player's own number of cards from the instruction, in a value read for each
//!   player in turn: "Each player may discard up to three cards. ~ deals damage to each
//!   player equal to 3 minus the number of cards they discarded this way." (Mind Bomb).
//! - "For each card less than two a player draws this way, that player gains 2 life."
//!   after "Each player may draw up to two cards." (Truce).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::scry_rules::OPT_IN;

/// The power of the creature each player exiled, as it last existed ("the power of the
/// creature they exiled", Olórin's Searing Light).
pub const EXILED_POWER: Var = vars::USER + 41006;

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
    /// The objects the previous instruction moved, stored as they last existed, while the
    /// next sentence is read ("the total power of creatures they controlled that were
    /// exiled this way").
    static MOVED: std::cell::Cell<Option<Var>> = const { std::cell::Cell::new(None) };
}

/// The objects the previous instruction moved (exiled, returned, destroyed), as they
/// last existed, kept for a sentence about each player's share of them.
const MOVED_THIS_WAY: Var = vars::USER + 41005;

/// The verbs of "[objects] [verb] this way" an instruction that moves objects is read
/// with.
fn moved_verb(e: &Effect) -> Option<&'static str> {
    match e {
        Effect::Exile {
            what: Sel::All(_), ..
        } => Some("exiled"),
        Effect::Move {
            what: Sel::All(_),
            to,
        } if to.zone == ZoneKind::Hand => Some("returned"),
        Effect::Destroy {
            what: Sel::All(_), ..
        } => Some("destroyed"),
        _ => None,
    }
}

/// The value grammar's atom (while [`MOVED`] is set): "the total power of creatures they
/// controlled that were exiled this way" (Oversimplify): each player's own share, judged
/// as the creatures last existed (CR 608.2h).
fn moved_share_atom(s: &str) -> Option<(Value, String)> {
    let v = MOVED.with(|c| c.get())?;
    let r = s.strip_prefix("the total power of ")?;
    // "they" (each player), read as "you" by a player performing the instruction.
    let (noun, rest) = r
        .split_once(" they controlled that were ")
        .or_else(|| r.split_once(" you controlled that were "))?;
    let (_verb, rest) = rest.split_once(" this way")?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(noun)?;
    if !tail.trim().is_empty() {
        return None;
    }
    Some((
        Value::Aggregate(
            AggOp::Sum,
            Stat::Power,
            Box::new(Sel::Matching(
                Box::new(Sel::Var(v)),
                Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
            )),
        ),
        rest.to_string(),
    ))
}

/// A sentence about each player's share of the objects the previous instruction moved:
/// "Each player creates a 0/0 ... token and puts a number of +1/+1 counters on it equal to
/// the total power of creatures they controlled that were exiled this way." (Oversimplify),
/// "For each opponent who controlled a creature returned this way, you create two 1/1 ...
/// tokens" (Faerie Slumber Party).
fn players_share_of_moved(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(verb) = moved_verb(last_instruction(prev)) else {
        return false;
    };
    if !l.contains(&format!(" {verb} this way")) {
        return false;
    }
    let store = Effect::Store {
        var: MOVED_THIS_WAY,
        sel: Sel::Before(Box::new(Sel::Var(vars::IT))),
    };
    // "for each opponent who controlled a creature returned this way, [clause]".
    if let Some(r) = l.strip_prefix("for each opponent who controlled ") {
        let Some((noun, clause)) = r.split_once(&format!(" {verb} this way, ")) else {
            return false;
        };
        let noun = noun
            .strip_prefix("a ")
            .or_else(|| noun.strip_prefix("an "))
            .unwrap_or(noun);
        let Some((f, _, tail)) = crate::oracle::phrases::parse_object_phrase(noun) else {
            return false;
        };
        if !tail.trim().is_empty() {
            return false;
        }
        let Some(e) = parse_clause(clause, b) else {
            return false;
        };
        let who = PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::History(Box::new(Condition::SelNonEmpty(Sel::Matching(
                Box::new(Sel::Var(MOVED_THIS_WAY)),
                Filter::and(vec![f, Filter::ControlledBy(PlayerRel::Iterated)]),
            )))),
        ]);
        let old = std::mem::take(prev);
        *prev = Effect::seq(vec![
            old,
            store,
            Effect::ForEachPlayer {
                who: PlayerRef::Each(who),
                effect: Box::new(e),
            },
        ]);
        return true;
    }
    if !l.contains(" they controlled that were ") {
        return false;
    }
    let saved = MOVED.with(|c| c.replace(Some(MOVED_THIS_WAY)));
    let e = crate::oracle::effects::parse_sentence(l, b);
    MOVED.with(|c| c.set(saved));
    let Some(e) = e else {
        return false;
    };
    if !format!("{e:?}").contains(&format!("Var({MOVED_THIS_WAY})")) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, store, e]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: each player's share of the objects moved this way", priority: 125, apply: players_share_of_moved } }

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
pub fn own_count_atom(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = super::choices_4::attached_to_player_atom(s, b) {
        return Some(v);
    }
    if let Some(v) = accepted_count_atom(s) {
        return Some(v);
    }
    if let Some(v) = super::notes_hidden_4::noted_atom(s) {
        return Some(v);
    }
    if let Some(v) = super::reflexive_modal_4::excess_atom(s) {
        return Some(v);
    }
    if let Some(v) = moved_share_atom(s) {
        return Some(v);
    }
    if !PER_PLAYER.with(|c| c.get()) {
        // One player's draw: "~ deals damage to the player equal to the number of cards
        // they drew this way" (Malignant Growth), the cards the latest draw instruction
        // drew.
        for p in [
            "the number of cards they drew this way",
            "the number of cards that player drew this way",
        ] {
            if let Some(r) = s.strip_prefix(p) {
                if r.is_empty() || r.starts_with([' ', ',', '.']) {
                    return Some((
                        Value::Count(Filter::In(Box::new(Sel::Var(vars::REVEALED)))),
                        r.to_string(),
                    ));
                }
            }
        }
        return None;
    }
    for p in [
        "the power of the creature they exiled",
        "the power of the creature that player exiled",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((
                    Value::PlayerTally {
                        var: EXILED_POWER,
                        who: Box::new(PlayerRef::Iterated),
                    },
                    r.to_string(),
                ));
            }
        }
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
    if var == EXILED_POWER {
        // One creature each player exiles: its power as it last existed.
        let Effect::Exile {
            what: Sel::Choose { count, filter, .. },
            ..
        } = &*step
        else {
            return false;
        };
        if count.as_const() != Some(1) || !format!("{filter:?}").contains("Creature") {
            return false;
        }
        let exile = std::mem::take(step);
        *step = Effect::seq(vec![
            exile,
            Effect::AddToPlayerTally {
                var: EXILED_POWER,
                who: PlayerRef::Iterated,
                value: Value::PowerOf(Box::new(Sel::Before(Box::new(Sel::Var(vars::IT))))),
            },
        ]);
        return true;
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
    // "If there are two or more instant and/or sorcery cards in your graveyard, ...".
    let (cond, l) = match l.strip_prefix("if ").and_then(|r| r.split_once(", ")) {
        Some((c, r)) => match crate::oracle::statics::parse_condition(c, b.ctx) {
            Some(cond) => (Some(cond), r),
            None => return false,
        },
        None => (None, l),
    };
    let Some((head, value)) = l.split_once(" equal to ") else {
        return false;
    };
    let Some((v, rest)) = with_per_player(|| super::value_grammar::parse_value(value, b)) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let Some(var) = [crate::tallies::CARDS_ACTED_ON, CARDS_DRAWN, EXILED_POWER]
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
    let each = Effect::ForEachPlayer {
        who,
        effect: Box::new(e),
    };
    let each = match cond {
        Some(cond) => Effect::If {
            cond,
            then: Box::new(each),
            otherwise: Box::new(Effect::Noop),
        },
        None => each,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, each]);
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

/// "they" / "that opponent" for the player an earlier part of the text names ("that
/// player"), in a clause the core reads with "that player": "creatures they control can't
/// attack you this turn" (Web of Inertia), "they can't attack you this combat", "that
/// opponent may pay {X}, where X is the number of cards in their hand" (Champions of Minas
/// Tirith).
fn that_player_words(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::oracle_hardening_referents::is_no_player_referent(&b.it_player)
        || matches!(b.it_player, PlayerRef::You)
    {
        return None;
    }
    let l = end(l);
    let rewritten = if let Some(r) = l.strip_prefix("creatures they control can't attack you ") {
        format!("creatures that player controls can't attack you {r}")
    } else if let Some(r) = l
        .strip_prefix("they can't attack you ")
        .or_else(|| l.strip_prefix("that player can't attack you "))
    {
        format!("creatures that player controls can't attack you {r}")
    } else if let Some(r) = l.strip_prefix("that opponent may pay ") {
        format!("that player may pay {r}")
    } else {
        return None;
    };
    let e = parse_clause(&rewritten, b)?;
    Some(unlock_attackers(e))
}

/// "Creatures that player controls can't attack you this turn" from a resolving ability
/// modifies the rules, not the creatures: it applies to creatures that player controls
/// at any time during the duration (CR 611.2c), not only those they control now.
fn unlock_attackers(e: Effect) -> Effect {
    match e {
        Effect::AddRestriction {
            mut restriction,
            duration,
        } => {
            if let Restriction::CantAttackPlayer { attackers, .. } = &mut restriction {
                if let Filter::In(inner) = attackers {
                    if let Sel::All(f) = &**inner {
                        if let Some(f) = by_player(f) {
                            *attackers = f;
                        }
                    }
                }
            }
            Effect::AddRestriction {
                restriction,
                duration,
            }
        }
        e => e,
    }
}

/// "Creatures [that player] controls" with the player as a reference bound as the effect
/// begins (the triggering player, the iterated player); None for other filters.
fn by_player(f: &Filter) -> Option<Filter> {
    let Filter::And(v) = f else {
        return None;
    };
    let mut found = false;
    let v = v
        .iter()
        .map(|x| match x {
            Filter::ControlledBy(PlayerRel::TriggerPlayer) => {
                found = true;
                Filter::ControlledByPlayer(Box::new(PlayerRef::TriggerPlayer))
            }
            Filter::ControlledByPlayer(_) => {
                found = true;
                x.clone()
            }
            x => x.clone(),
        })
        .collect();
    found.then_some(Filter::And(v))
}

inventory::submit! { EffectPattern { name: "followups 4: they / that opponent as that player", priority: 120, parse: that_player_words } }

/// "you mill three cards, then ~ deals damage to that player equal to the total mana value
/// of those cards" (Combustible Gearhulk), "you mill X cards, where X is ..., and that
/// player loses life equal to the total mana value of those cards" (Palantír of
/// Orthanc): "those cards" are the milled cards.
fn mill_then_those_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for sep in [", then ", ", and "] {
        let Some((first, second)) = l.split_once(sep) else {
            continue;
        };
        if !(first.starts_with("mill ") || first.starts_with("you mill "))
            || !second.contains("those cards")
        {
            continue;
        }
        let saved = (
            b.targets.len(),
            b.it.clone(),
            b.it_player.clone(),
            b.named.len(),
        );
        let Some(e1) = parse_clause(first, b) else {
            continue;
        };
        if !format!("{e1:?}").contains("Mill") {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            b.named.truncate(saved.3);
            continue;
        }
        let player = b.it_player.clone();
        super::dig_grammar::note_source(Some(&e1), b);
        // The mill doesn't change who "that player" is.
        b.it_player = player;
        let Some(e2) = parse_clause(&second.replace("those cards", "the milled cards"), b) else {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            b.named.truncate(saved.3);
            continue;
        };
        return Some(Effect::seq(vec![e1, e2]));
    }
    None
}

inventory::submit! { EffectPattern { name: "followups 4: mill, then ... those cards", priority: 120, parse: mill_then_those_cards } }

/// "that player taps an untapped artifact, creature, or land they control for each fade
/// counter on ~" (Tangle Wire), "tap two untapped creatures you control": the player
/// chooses that many of the objects and taps them (as many as they can, CR 609.3).
fn taps_untapped(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r, own) = if let Some(r) = l.strip_prefix("that player taps ") {
        if super::oracle_hardening_referents::is_no_player_referent(&b.it_player)
            || matches!(b.it_player, PlayerRef::You)
        {
            return None;
        }
        (b.it_player.clone(), r, " they control")
    } else if let Some(r) = l.strip_prefix("tap ") {
        (PlayerRef::You, r, " you control")
    } else {
        return None;
    };
    let (n, r) = if let Some(r) = r.strip_prefix("an ").or_else(|| r.strip_prefix("a ")) {
        (Value::c(1), r)
    } else {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (n, r.trim_start())
    };
    if !r.starts_with("untapped ") {
        return None;
    }
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(r)?;
    let tail = tail.trim_start();
    let tail = tail.strip_prefix(own.trim_start())?;
    let count = if tail.is_empty() {
        n
    } else {
        let thing = tail.strip_prefix(" for each ")?;
        let (v, rest) = super::value_grammar::parse_value(&format!("the number of {thing}"), b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        match n.as_const() {
            Some(1) => v,
            _ => Value::Mul(Box::new(n), Box::new(v)),
        }
    };
    let f = Filter::and(vec![
        f,
        Filter::Untapped,
        Filter::ControlledByPlayer(Box::new(who.clone())),
    ]);
    Some(Effect::Tap {
        what: Sel::Choose {
            chooser: who,
            filter: f,
            count,
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "followups 4: [player] taps N untapped [objects] they control", priority: 120, parse: taps_untapped } }

/// "If the last card exiled isn't a land card, repeat this process." (Rally the Horde):
/// after several instructions that each exile one card, "the last card exiled" is the
/// card the latest of them exiled, what "it" refers to.
fn the_last_card_exiled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if the last card exiled ") else {
        return false;
    };
    if !matches!(b.it, Sel::Var(_)) || !format!("{prev:?}").contains("Exile") {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_sentence(&format!("if it {r}"), b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: if the last card exiled ...", priority: 120, apply: the_last_card_exiled } }

/// The last instruction of an effect (looking into sequences and players' instructions).
fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        Effect::AsPlayer { effect, .. } => last_instruction(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_instruction(then),
        _ => e,
    }
}

/// "If a nonland permanent is sacrificed this way, repeat this process." after "[player]
/// chooses a permanent they control at random and sacrifices it" (Tyrant of Discord): the
/// one permanent that instruction sacrificed, as "if it was a nonland permanent".
fn if_the_sacrificed_one_was(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if a ").or_else(|| l.strip_prefix("if an ")) else {
        return false;
    };
    let Some((kind, rest)) = r.split_once(" is sacrificed this way, ") else {
        return false;
    };
    let Effect::SacrificeObjects { what: Sel::Var(v) } = last_instruction(prev) else {
        return false;
    };
    if !matches!(b.it, Sel::Var(w) if w == *v) {
        return false;
    }
    let article = if l.starts_with("if an ") { "an" } else { "a" };
    let Some(e) =
        crate::oracle::effects::parse_sentence(&format!("if it was {article} {kind}, {rest}"), b)
    else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: if a [kind] is sacrificed this way (one chosen permanent)", priority: 120, apply: if_the_sacrificed_one_was } }

/// Every card exiled by a process of single-card exiles, over all its passes.
const EXILED_IN_PROCESS: Var = vars::USER + 41001;

/// Adds, after each exile of `e`, the card it exiled to [`EXILED_IN_PROCESS`]; the number
/// of exiles.
fn accumulate_exiles(e: &mut Effect) -> usize {
    match e {
        Effect::Seq(v) => {
            let mut n = 0;
            let mut i = 0;
            while i < v.len() {
                if matches!(v[i], Effect::Exile { .. }) {
                    v.insert(
                        i + 1,
                        Effect::Store {
                            var: EXILED_IN_PROCESS,
                            sel: Sel::Union(vec![Sel::Var(EXILED_IN_PROCESS), Sel::Var(vars::IT)]),
                        },
                    );
                    n += 1;
                    i += 2;
                } else {
                    n += accumulate_exiles(&mut v[i]);
                    i += 1;
                }
            }
            n
        }
        Effect::RepeatProcess { body } => accumulate_exiles(body),
        _ => 0,
    }
}

/// "Create a 1/1 red Warrior creature token for each nonland card exiled this way." after
/// a process of exiles that repeats (Rally the Horde): the cards every exile of every
/// pass exiled.
fn for_each_exiled_in_process(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.ends_with(" exiled this way") || !l.contains(" for each ") {
        return false;
    }
    let Effect::RepeatProcess { body } = prev else {
        return false;
    };
    let d = format!("{body:?}");
    if d.matches("Exile {").count() < 2 || d.contains(&format!("var: {EXILED_IN_PROCESS}")) {
        return false;
    }
    let Some(e) = crate::oracle::effects::parse_sentence(l, b) else {
        return false;
    };
    let Ok(json) = serde_json::to_string(&e) else {
        return false;
    };
    let last = r#""Before":{"Var":0}"#;
    if json.matches(last).count() != 1 {
        return false;
    }
    let json = json.replace(last, &format!(r#""Before":{{"Var":{EXILED_IN_PROCESS}}}"#));
    let Ok(e) = serde_json::from_str::<Effect>(&json) else {
        return false;
    };
    let mut process = std::mem::take(prev);
    accumulate_exiles(&mut process);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: EXILED_IN_PROCESS,
            sel: Sel::None,
        },
        process,
        e,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: for each card exiled this way, after a repeated process", priority: 120, apply: for_each_exiled_in_process } }

/// Every card put into a hand by a repeated process, over all its passes.
const PUT_IN_HAND_IN_PROCESS: Var = vars::USER + 41003;

/// Adds, after each instruction of `e` that puts a card into a hand, that card to
/// [`PUT_IN_HAND_IN_PROCESS`]; the number of such instructions.
fn accumulate_to_hand(e: &mut Effect) -> usize {
    match e {
        Effect::Seq(v) => {
            let mut n = 0;
            let mut i = 0;
            while i < v.len() {
                if matches!(&v[i], Effect::Move { to, .. } if to.zone == ZoneKind::Hand) {
                    v.insert(
                        i + 1,
                        Effect::Store {
                            var: PUT_IN_HAND_IN_PROCESS,
                            sel: Sel::Union(vec![
                                Sel::Var(PUT_IN_HAND_IN_PROCESS),
                                Sel::Var(vars::IT),
                            ]),
                        },
                    );
                    n += 1;
                    i += 2;
                } else {
                    n += accumulate_to_hand(&mut v[i]);
                    i += 1;
                }
            }
            n
        }
        Effect::RepeatProcess { body } => accumulate_to_hand(body),
        _ => 0,
    }
}

/// "~ deals 1 damage to you for each card put into your hand this way." after a process
/// that repeats (Demonlord Belzenlok): the cards every pass put into your hand, the
/// damage dealt all at once.
fn for_each_put_into_hand_in_process(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(head) = l.strip_suffix(" for each card put into your hand this way") else {
        return false;
    };

    let Effect::RepeatProcess { body } = prev else {
        return false;
    };
    if format!("{body:?}").contains(&format!("var: {PUT_IN_HAND_IN_PROCESS}")) {
        return false;
    }
    let Some((before, after)) = head.split_once(" deals ") else {
        return false;
    };
    let Some((n, rest)) = parse_number(after) else {
        return false;
    };
    let rest = rest.trim_start();
    if n.as_const().is_none() || !rest.starts_with("damage to ") {
        return false;
    }
    let clause = format!("{before} deals x {rest}");
    let Some(e) = super::value_grammar::with_x_defined(true, || parse_clause(&clause, b)) else {
        return false;
    };
    let count = Value::CountSel(Box::new(Sel::Var(PUT_IN_HAND_IN_PROCESS)));
    let total = match n.as_const() {
        Some(1) => count,
        _ => Value::Mul(Box::new(n), Box::new(count)),
    };
    let Some(e) = super::r107_numbers::substitute_x(&e, &total) else {
        return false;
    };
    let mut process = std::mem::take(prev);
    if accumulate_to_hand(&mut process) == 0 {
        *prev = process;
        return false;
    }
    *prev = Effect::seq(vec![
        Effect::Store {
            var: PUT_IN_HAND_IN_PROCESS,
            sel: Sel::None,
        },
        process,
        e,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: for each card put into your hand this way, after a repeated process", priority: 120, apply: for_each_put_into_hand_in_process } }

/// Whether the player chose to do the optional instruction in any pass of a process
/// repeated once (Claim Jumper: "If you search your library this way, shuffle").
const DID_IN_PROCESS: Var = vars::USER + 41002;

/// "Then if an opponent controls more lands than you, repeat this process once." (Claim
/// Jumper): the instructions of the previous sentence happen again, with new choices, if
/// the condition is true then (CR 608.2c).
fn repeat_once_if(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some(c) = r.strip_suffix(", repeat this process once") else {
        return false;
    };
    if matches!(prev, Effect::Noop) {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let did = || Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(Effect::StoreValue {
            var: DID_IN_PROCESS,
            value: Value::c(1),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    let once = prev.clone();
    let first = std::mem::take(prev);
    let optional = matches!(last_instruction(&first), Effect::May { .. });
    let mut v = vec![];
    if optional {
        v.push(Effect::StoreValue {
            var: DID_IN_PROCESS,
            value: Value::c(0),
        });
    }
    v.push(first);
    if optional {
        v.push(did());
    }
    let again = if optional {
        Effect::seq(vec![once, did()])
    } else {
        once
    };
    v.push(Effect::If {
        cond,
        then: Box::new(again),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: then if [condition], repeat this process once", priority: 120, apply: repeat_once_if } }

/// "If you search your library this way, shuffle." after an optional search repeated once:
/// if you searched in either pass.
fn if_you_searched_in_the_process(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "if you search your library this way, shuffle" {
        return false;
    }
    let d = format!("{prev:?}");
    if !d.contains(&format!("var: {DID_IN_PROCESS}")) || !d.contains("Search {") {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::Compare(Value::Var(DID_IN_PROCESS), Cmp::Ge, Value::c(1)),
            then: Box::new(Effect::Shuffle {
                who: PlayerRef::You,
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: if you search your library this way, shuffle (after a repeated search)", priority: 120, apply: if_you_searched_in_the_process } }

/// The objects a per-opponent exile exiled, as they last existed, and the one iterated.
const EXILED_EACH: Var = vars::USER + 41007;
const EXILED_ONE: Var = vars::USER + 41008;

/// "For each opponent, exile up to one target creature that player controls and that
/// player gains life equal to its power." (Luminate Primordial): each exiled creature's
/// controller, as it last existed on the battlefield, gains life equal to its power then
/// (Luminate Primordial rulings).
fn per_opponent_exile_and_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("for each opponent, exile ") {
        return None;
    }
    let (first, second) = l.split_once(" and that player ")?;
    let gains = match second {
        "gains life equal to its power" => true,
        "loses life equal to its power" => false,
        _ => return None,
    };
    let e = parse_clause(first, b)?;
    if !matches!(
        e,
        Effect::Exile {
            what: Sel::Target(_),
            ..
        }
    ) {
        return None;
    }
    let one = Sel::Var(EXILED_ONE);
    let who = PlayerRef::ControllerOf(Box::new(one.clone()));
    let n = Value::PowerOf(Box::new(one));
    let life = if gains {
        Effect::GainLife { who, n }
    } else {
        Effect::LoseLife { who, n }
    };
    Some(Effect::seq(vec![
        e,
        Effect::Store {
            var: EXILED_EACH,
            sel: Sel::Before(Box::new(Sel::Var(vars::IT))),
        },
        Effect::ForEach {
            sel: Sel::Var(EXILED_EACH),
            var: EXILED_ONE,
            effect: Box::new(life),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "followups 4: for each opponent, exile ... and that player gains life equal to its power", priority: 120, parse: per_opponent_exile_and_that_player } }

/// The player another player's optional instruction ("Its owner may put it on top of
/// their library") is given to, when `prev` ends with it.
fn optional_for_other_player(prev: &Effect) -> Option<PlayerRef> {
    let last = match prev {
        Effect::Seq(v) => v.iter().rev().find(|e| !matches!(e, Effect::If { .. }))?,
        e => e,
    };
    match last {
        Effect::AsPlayer { who, effect } if matches!(**effect, Effect::May { .. }) => {
            (!matches!(who, PlayerRef::You)).then(|| who.clone())
        }
        _ => None,
    }
}

/// "Its owner may put it on top of their library. If they do, ~ deals 2 damage to them. If
/// they didn't put the card on top of their library, they put it on the bottom." (Clash of
/// Elements): "they" and "them" are the player who was given the choice (CR 118.12).
fn other_players_choice_followups(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(who) = optional_for_other_player(prev) else {
        return false;
    };
    let (did, rest) = if let Some(r) = l.strip_prefix("if they do, ") {
        (true, r.to_string())
    } else if let Some(r) = l.strip_prefix("if they don't, ") {
        (false, r.to_string())
    } else if let Some(r) = l.strip_prefix("if they didn't ") {
        // "If they didn't put the card on top of their library, ...": what they declined.
        let Some((what, r)) = r.split_once(", ") else {
            return false;
        };
        if !what.starts_with("put ") {
            return false;
        }
        (false, r.to_string())
    } else {
        return false;
    };
    let clause = if let Some(r) = rest.strip_prefix("they ") {
        let r = if r.ends_with(" on the bottom") {
            format!("{r} of their library")
        } else {
            r.to_string()
        };
        format!("that player {}", super::choices_4::third_person(&r))
    } else {
        // "deals 2 damage to them": the player.
        match rest.strip_suffix(" damage to them") {
            Some(h) => format!("{h} damage to that player"),
            None => return false,
        }
    };
    let saved = std::mem::replace(&mut b.it_player, who);
    let e = parse_clause(&clause, b);
    b.it_player = saved;
    let Some(e) = e else {
        return false;
    };
    // "If they don't" after "If they do": the other branch of that condition.
    if !did {
        if let Effect::Seq(v) = prev {
            if let Some(Effect::If {
                cond: Condition::PrevHappened,
                otherwise,
                ..
            }) = v.last_mut()
            {
                if matches!(**otherwise, Effect::Noop) {
                    **otherwise = e;
                    return true;
                }
            }
        }
    }
    let cond = if did {
        Condition::PrevHappened
    } else {
        Condition::Not(Box::new(Condition::PrevHappened))
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: if they do / didn't, after another player's optional instruction", priority: 20, apply: other_players_choice_followups } }

/// "each player loses life and discards cards equal to the number of permanents they
/// control but don't own" (Blim, Comedic Genius): both amounts are that player's number;
/// each instruction is performed by all the players before the next (CR 608.2e).
fn loses_life_and_discards_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (head, value) = l.split_once(" loses life and discards cards equal to ")?;
    if !matches!(head, "each player" | "each opponent") {
        return None;
    }
    let e = parse_clause(&format!("{head} loses life equal to {value}"), b)?;
    let Effect::ForEachPlayer { who, effect } = e else {
        return None;
    };
    let Effect::AsPlayer {
        who: PlayerRef::Iterated,
        effect: inner,
    } = *effect
    else {
        return None;
    };
    let Effect::LoseLife {
        who: PlayerRef::You,
        n,
    } = *inner
    else {
        return None;
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::seq(vec![
                Effect::LoseLife {
                    who: PlayerRef::You,
                    n: n.clone(),
                },
                Effect::Discard {
                    who: PlayerRef::You,
                    n,
                    random: false,
                    filter: Filter::Any,
                },
            ])),
        }),
    })
}

inventory::submit! { EffectPattern { name: "followups 4: each player loses life and discards cards equal to", priority: 120, parse: loses_life_and_discards_equal_to } }

/// The card a "for each [card] revealed this way" punisher is about.
const EACH_REVEALED: Var = vars::USER + 41009;

/// "Target player reveals their hand. For each blue instant card revealed this way, that
/// player discards that card unless they pay 4 life." (Sirocco): for each of those cards,
/// one at a time, the player chooses whether to pay (CR 118.12a); if they don't, they
/// discard that card.
fn for_each_revealed_discard_unless(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("for each ") else {
        return false;
    };
    let Some((thing, rest)) = r.split_once(" revealed this way, ") else {
        return false;
    };
    let Some(cost) = rest
        .strip_prefix("that player discards that card unless they pay ")
        .or_else(|| rest.strip_prefix("they discard that card unless they pay "))
    else {
        return false;
    };
    let who = b.it_player.clone();
    if super::oracle_hardening_referents::is_no_player_referent(&who)
        || matches!(who, PlayerRef::You)
    {
        return false;
    }
    // The cards, read as "for each [thing] revealed this way" counts them.
    let mut counted = prev.clone();
    if !crate::oracle_ext::apply_followup_ext(
        &format!("you draw a card for each {thing} revealed this way"),
        &mut counted,
        b,
    ) && crate::oracle::effects::parse_sentence(
        &format!("you draw a card for each {thing} revealed this way"),
        b,
    )
    .map(|e| {
        counted = Effect::seq(vec![prev.clone(), e]);
    })
    .is_none()
    {
        return false;
    }
    let Effect::Seq(v) = &counted else {
        return false;
    };
    let Some(Effect::Draw {
        n: Value::CountSel(cards),
        ..
    }) = v.last()
    else {
        return false;
    };
    // "4 life", "{2}".
    let cost_text = if cost.starts_with('{') {
        cost.to_string()
    } else {
        format!("pay {cost}")
    };
    let Some(cost) = crate::oracle::keywords::parse_keyword_cost(&cost_text) else {
        return false;
    };
    let each = Effect::ForEach {
        sel: (**cards).clone(),
        var: EACH_REVEALED,
        effect: Box::new(Effect::AsPlayer {
            who,
            effect: Box::new(Effect::PayOptional {
                who: PlayerRef::You,
                cost,
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::Discard {
                    who: PlayerRef::You,
                    n: Value::c(1),
                    random: false,
                    filter: Filter::In(Box::new(Sel::Var(EACH_REVEALED))),
                }),
            }),
        }),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, each]);
    true
}

inventory::submit! { FollowupPattern { name: "followups 4: for each card revealed this way, that player discards that card unless they pay", priority: 25, apply: for_each_revealed_discard_unless } }

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
            ("Enchantment", "At the beginning of combat on each opponent's turn, that player may exile a card from their graveyard. If the player doesn't, creatures they control can't attack you this turn."),
            ("Creature", "At the beginning of combat on each opponent's turn, if you're the monarch, that opponent may pay {X}, where X is the number of cards in their hand. If they don't, they can't attack you this combat."),
            ("Artifact Creature", "When ~ enters, target opponent may have you draw three cards. If the player doesn't, you mill three cards, then ~ deals damage to that player equal to the total mana value of those cards."),
            ("Legendary Artifact", "At the beginning of your end step, put an influence counter on ~ and scry 2. Then target opponent may have you draw a card. If that player doesn't, you mill X cards, where X is the number of influence counters on ~, and that player loses life equal to the total mana value of those cards."),
            ("Artifact", "At the beginning of each player's upkeep, that player taps an untapped artifact, creature, or land they control for each fade counter on ~."),
            ("Enchantment", "At the beginning of each opponent's draw step, that player draws an additional card for each growth counter on ~, then ~ deals damage to the player equal to the number of cards they drew this way."),
            ("Sorcery", "Exile all creatures. Each player creates a 0/0 green and blue Fractal creature token and puts a number of +1/+1 counters on it equal to the total power of creatures they controlled that were exiled this way."),
            ("Sorcery", "Return all creatures to their owners' hands. For each opponent who controlled a creature returned this way, you create two 1/1 blue Faerie creature tokens with flying and \"This token can block only creatures with flying.\""),
            ("Sorcery", "Each opponent exiles a creature with the greatest power among creatures that player controls.\nSpell mastery — If there are two or more instant and/or sorcery cards in your graveyard, ~ deals damage to each opponent equal to the power of the creature they exiled."),
            ("Creature", "When ~ enters, for each opponent, exile up to one target creature that player controls and that player gains life equal to its power."),
            ("Sorcery", "Choose target nonland permanent. Its owner may put it on top of their library. If they do, ~ deals 2 damage to them. If they didn't put the card on top of their library, they put it on the bottom."),
            ("Legendary Creature", "Whenever ~ deals combat damage to a player, that player gains control of target permanent you control. Then each player loses life and discards cards equal to the number of permanents they control but don't own."),
            ("Instant", "Target player reveals their hand. For each blue instant card revealed this way, that player discards that card unless they pay 4 life."),
            ("Creature", "When ~ enters, each opponent may investigate. Each opponent who doesn't loses 1 life. You investigate X times, where X is one plus the number of opponents who investigated this way."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
