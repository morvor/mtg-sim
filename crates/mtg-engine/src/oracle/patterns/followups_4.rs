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
            ("Creature", "When ~ enters, each opponent may investigate. Each opponent who doesn't loses 1 life. You investigate X times, where X is one plus the number of opponents who investigated this way."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
