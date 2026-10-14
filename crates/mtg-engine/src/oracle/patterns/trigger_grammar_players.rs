//! Player-event trigger conditions (CR 603.2) the compositional parser in `triggers.rs`
//! doesn't cover:
//!
//! * creating tokens: `you create a [token]`, `you create one or more [tokens]` (once per
//!   batch, CR 603.2c), `you create or sacrifice a token`. The player who creates a token
//!   is its owner (CR 111.2).
//! * sacrificing: `you sacrifice ~ or another [object]`, `one or more players sacrifice one
//!   or more [objects]` (look back in time, CR 603.10a).
//! * discarding: `one or more players discard one or more cards`, `you discard a [card] or
//!   a [card]`.
//! * milling (CR 701.17): `[player] mills a [card]` (once for each such card milled, to
//!   whatever public zone it went, CR 701.17c), `[player] mills one or more [cards]` (once
//!   per player milling), `one or more [cards] are milled` (once per batch), `an opponent
//!   discards a card or mills one or more cards`.
//! * life: `you gain or lose life`, `one or more opponents lose life` (once per batch).
//! * losing the game: `an opponent loses the game`, `another player loses the game`.
//! * energy (CR 107.14): `you get one or more {E}` — energy counters put on you, by
//!   whoever put them (CR 122.6); "that much" is how many.
//! * clashing (CR 701.30): `you clash and win`, `you win a clash`.
//! * draws: `you draw your first or second card each turn`, `a player draws their second
//!   card during their turn`; casting: `a player casts their second spell during their
//!   turn` (the Nth of that player's turn, which is their own).
//! * gaining control: `an opponent gains control of a permanent from you` (looks back in
//!   time, CR 603.10d).

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::patterns::trigger_grammar_events::{is_batched, where_events};
use crate::oracle::patterns::triggers::{player_subject, verb};
use crate::oracle::phrases::*;
use crate::types::counters;

type Parsed = (TriggerCond, Sel, PlayerRef);

/// "a [object]" / "an [object]": a singular object phrase.
fn a_phrase(s: &str) -> Option<Filter> {
    let t = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(t)?;
    (!plural && end(tail).is_empty()).then_some(f)
}

/// "one or more [objects]".
fn one_or_more(s: &str) -> Option<Filter> {
    let t = s.strip_prefix("one or more ")?;
    let (f, _, tail) = parse_object_phrase(t)?;
    end(tail).is_empty().then_some(f)
}

fn mentions_token(f: &Filter) -> bool {
    match f {
        Filter::Token => true,
        Filter::And(v) => v.iter().any(mentions_token),
        _ => false,
    }
}

fn batch(trigger: TriggerCond, per: BatchPer) -> TriggerCond {
    TriggerCond::Batched {
        trigger: Box::new(trigger),
        per,
    }
}

fn player_is(rel: PlayerRel) -> Option<Condition> {
    let f = match rel {
        PlayerRel::You => PlayerFilter::You,
        PlayerRel::Opponent => PlayerFilter::Opponent,
        PlayerRel::Any => return None,
        PlayerRel::NotYou => PlayerFilter::NotYou,
        _ => return None,
    };
    Some(Condition::PlayerMatches(PlayerRef::TriggerPlayer, f))
}

/// "[cards] milled": a card moved from its owner's library by milling (CR 701.17a, c),
/// matched as it is in the zone it went to.
fn milled(f: Filter, who: PlayerRel) -> TriggerCond {
    let mut conds = vec![Condition::Custom(
        crate::kw::trigger_event_causes::MILLED.into(),
    )];
    conds.extend(player_is(who));
    TriggerCond::Where {
        trigger: Box::new(TriggerCond::ZoneChange {
            filter: f,
            from: Some(ZoneKind::Library),
            to: None,
        }),
        cond: if conds.len() == 1 {
            conds.pop().unwrap()
        } else {
            Condition::And(conds)
        },
    }
}

fn player_events(r: &str) -> Option<Parsed> {
    let r = end(r);
    let tp = || PlayerRef::TriggerPlayer;
    // "one or more [cards] are milled" (once per batch, even several players milling).
    if let Some(x) = r.strip_suffix(" are milled") {
        let f = one_or_more(x)?;
        // "the nonland cards milled this way": the batch's cards, where they went.
        return Some((
            batch(milled(f, PlayerRel::Any), BatchPer::Batch),
            Sel::TriggerObjects,
            tp(),
        ));
    }
    // "one or more opponents lose life", "one or more players discard one or more cards",
    // "one or more players sacrifice one or more creatures".
    if let Some(x) = r.strip_prefix("one or more opponents ") {
        if x == "lose life" {
            let c = TriggerCond::LosesLife {
                who: PlayerRel::Opponent,
            };
            return Some((batch(c, BatchPer::Batch), Sel::None, tp()));
        }
        return None;
    }
    if let Some(x) = r.strip_prefix("one or more players ") {
        if let Some(o) = x.strip_prefix("discard ") {
            let f = if o == "one or more cards" {
                Filter::Any
            } else {
                one_or_more(o)?
            };
            let c = TriggerCond::Discards {
                who: PlayerRel::Any,
                filter: f,
            };
            return Some((batch(c, BatchPer::Batch), Sel::None, tp()));
        }
        if let Some(o) = x.strip_prefix("sacrifice ") {
            let f = one_or_more(o)?;
            return Some((
                batch(TriggerCond::Sacrificed(f), BatchPer::Batch),
                Sel::None,
                tp(),
            ));
        }
        return None;
    }
    // "a player draws their second card during their turn": the Nth card of their own
    // turn; "a player casts their second spell during their turn".
    if let Some(x) = r.strip_prefix("a player draws their ") {
        let (w, rest) = split_word(x);
        let n = ordinal(w)?;
        if rest != "card during their turn" {
            return None;
        }
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::Draws {
                who: PlayerRel::Any,
            }),
            cond: Condition::And(vec![
                Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(n as i32)),
                Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Active),
            ]),
        };
        return Some((c, Sel::None, tp()));
    }
    if let Some(x) = r.strip_prefix("a player casts their ") {
        let (w, rest) = split_word(x);
        let n = ordinal(w)?;
        if rest != "spell during their turn" {
            return None;
        }
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::NthSpellCast {
                who: PlayerRel::Any,
                n,
            }),
            cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Active),
        };
        return Some((c, Sel::TriggerSpell, tp()));
    }
    let (who, rest) = player_subject(r)?;
    // "an opponent discards a card or mills one or more cards".
    if let Some(t) = rest
        .strip_prefix("discards a card or mills one or more cards")
        .or_else(|| rest.strip_prefix("discard a card or mill one or more cards"))
    {
        if !t.is_empty() {
            return None;
        }
        let c = TriggerCond::AnyOf(vec![
            TriggerCond::Discards {
                who,
                filter: Filter::Any,
            },
            TriggerCond::Mills(who),
        ]);
        return Some((c, Sel::None, tp()));
    }
    // Tokens (CR 111.2: the player who creates a token is its owner).
    if let Some(t) = verb(rest, "create") {
        if who != PlayerRel::You {
            return None;
        }
        let mine = |f: Filter| Filter::and(vec![f, Filter::OwnedBy(PlayerRel::You)]);
        if t == "or sacrifice a token" {
            let c = TriggerCond::AnyOf(vec![
                TriggerCond::TokenCreated(mine(Filter::Token)),
                TriggerCond::YouSacrifice(Filter::Token),
            ]);
            return Some((c, Sel::None, PlayerRef::You));
        }
        if let Some(f) = one_or_more(t) {
            if !mentions_token(&f) {
                return None;
            }
            let c = batch(TriggerCond::TokenCreated(mine(f)), BatchPer::Batch);
            return Some((c, Sel::TriggerObjects, PlayerRef::You));
        }
        let f = a_phrase(t)?;
        if !mentions_token(&f) {
            return None;
        }
        return Some((
            TriggerCond::TokenCreated(mine(f)),
            Sel::TriggerObject,
            PlayerRef::You,
        ));
    }
    // "you sacrifice ~ or another artifact" (looks back in time, CR 603.10a).
    if let Some(t) = verb(rest, "sacrifice") {
        if who != PlayerRel::You {
            return None;
        }
        let x = t.strip_prefix("~ or another ")?;
        let (f, plural, tail) = parse_object_phrase(x)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        let c = TriggerCond::YouSacrifice(Filter::Or(vec![
            Filter::Source,
            Filter::and(vec![f, Filter::Other]),
        ]));
        return Some((c, Sel::TriggerLki, PlayerRef::You));
    }
    // "you discard a Spirit card or a card with disturb".
    if let Some(t) = verb(rest, "discard") {
        let (a, b) = t.split_once(" or ")?;
        let fa = a_phrase(a)?;
        let fb = a_phrase(b)?;
        return Some((
            TriggerCond::Discards {
                who,
                filter: Filter::Or(vec![fa, fb]),
            },
            Sel::TriggerObject,
            tp(),
        ));
    }
    // Milling.
    if let Some(t) = verb(rest, "mill") {
        if let Some(f) = one_or_more(t) {
            return Some((
                batch(milled(f, who), BatchPer::Player),
                Sel::TriggerObjects,
                tp(),
            ));
        }
        let f = a_phrase(t)?;
        // The card in the zone it was milled to (CR 701.17c).
        return Some((milled(f, who), Sel::TriggerObject, tp()));
    }
    // Life.
    if matches!(rest, "gain or lose life" | "gains or loses life") {
        let c = TriggerCond::AnyOf(vec![
            TriggerCond::GainsLife { who },
            TriggerCond::LosesLife { who },
        ]);
        return Some((c, Sel::None, tp()));
    }
    // "an opponent loses the game", "another player loses the game".
    if rest == "loses the game" {
        let cond = player_is(who)?;
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerLoses),
            cond,
        };
        return Some((c, Sel::None, tp()));
    }
    // Energy (CR 107.14): "you get one or more {E}".
    if who == PlayerRel::You && matches!(rest.trim(), "get one or more {e}") {
        let c = TriggerCond::CountersPutBy {
            who: PlayerRel::Any,
            on_objects: None,
            on_players: Some(PlayerFilter::You),
            kind: Some(counters::ENERGY.into()),
            each: false,
        };
        return Some((c, Sel::None, PlayerRef::You));
    }
    // Clashing (CR 701.30): the clash event's amount is 1 when the player won.
    if who == PlayerRel::You && matches!(rest, "clash and win" | "win a clash") {
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAction {
                name: crate::kwa::fateseal_clash::CLASHED.into(),
                who,
            }),
            cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(1)),
        };
        return Some((c, Sel::None, PlayerRef::You));
    }
    // "you draw your first or second card each turn".
    if let Some(x) = rest.strip_prefix("draw your ") {
        let (a, x) = split_word(x);
        let x = x.strip_prefix("or ")?;
        let (b, x) = split_word(x);
        let (a, b) = (ordinal(a)?, ordinal(b)?);
        if x != "card each turn" || a + 1 != b || a != 1 {
            return None;
        }
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::Draws { who }),
            cond: Condition::Compare(Value::EventAmount, Cmp::Le, Value::c(b as i32)),
        };
        return Some((c, Sel::None, tp()));
    }
    // "an opponent gains control of a permanent from you" (CR 603.10d).
    if let Some(x) = rest.strip_prefix("gains control of ") {
        let x = x.strip_suffix(" from you")?;
        let f = a_phrase(x)?;
        let cond = player_is(who)?;
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::LoseControl(f)),
            cond,
        };
        return Some((c, Sel::TriggerObject, tp()));
    }
    None
}

fn ordinal(w: &str) -> Option<u32> {
    Some(match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    })
}

inventory::submit! { TriggerPattern { name: "player events (tokens, sacrifice, mill, life, energy, clash)", priority: 110, parse: player_events } }

/// "[event] and when you sacrifice it" (Carrot Cake): the source's own sacrifice.
fn and_when_you_sacrifice_it(r: &str) -> Option<Parsed> {
    let head = end(r).strip_suffix(" and when you sacrifice it")?;
    if !head.starts_with("~ ") {
        return None;
    }
    let (a, it, p) = crate::oracle::triggers::parse_trigger_condition(&format!("whenever {head}"))?;
    if !matches!(it, Sel::This) {
        return None;
    }
    let c = TriggerCond::AnyOf(vec![a, TriggerCond::YouSacrifice(Filter::Source)]);
    Some((c, Sel::This, p))
}

inventory::submit! { TriggerPattern { name: "[event] and when you sacrifice it", priority: 110, parse: and_when_you_sacrifice_it } }

/// "[event] or another [object] [event]", "[event] or a [object] [event]" where either
/// side is a trigger condition the "either event" pattern doesn't combine (it has several
/// zones, "or another"): "Whenever ~ dies or another artifact you control is put into a
/// graveyard from the battlefield", "Whenever ~ enters or another nontoken Human you
/// control dies", "Whenever a creature dies or a creature card is put into a graveyard
/// from a library".
fn either_events(r: &str) -> Option<Parsed> {
    let r = end(r);
    for sep in [" or another ", " or a ", " or an "] {
        for (i, _) in r.match_indices(sep) {
            let a = &r[..i];
            let b = &r[i + " or ".len()..];
            let Some(pa) =
                crate::oracle::triggers::parse_trigger_condition(&format!("whenever {a}"))
            else {
                continue;
            };
            let Some(pb) =
                crate::oracle::triggers::parse_trigger_condition(&format!("whenever {b}"))
            else {
                continue;
            };
            if is_batched(&pa.0) || is_batched(&pb.0) {
                return None;
            }
            let same = |x: &dyn std::fmt::Debug, y: &dyn std::fmt::Debug| {
                format!("{x:?}") == format!("{y:?}")
            };
            // "~ dies or another artifact ...": "it" is whichever object the event was
            // about, as last known (both leave the battlefield).
            let it = if same(&pa.1, &pb.1) {
                pa.1.clone()
            } else if matches!(pa.1, Sel::This | Sel::TriggerLki)
                && matches!(pb.1, Sel::This | Sel::TriggerLki)
                && leaves(&pa.0)
                && leaves(&pb.0)
            {
                Sel::TriggerLki
            } else {
                Sel::None
            };
            let player = if same(&pa.2, &pb.2) {
                pa.2.clone()
            } else {
                PlayerRef::Iterated
            };
            let mut conds = Vec::new();
            for c in [pa.0, pb.0] {
                match c {
                    TriggerCond::AnyOf(v) => conds.extend(v),
                    c => conds.push(c),
                }
            }
            return Some((TriggerCond::AnyOf(conds), it, player));
        }
    }
    None
}

/// Whether every event of a trigger condition is a permanent leaving the battlefield.
fn leaves(c: &TriggerCond) -> bool {
    match c {
        TriggerCond::Dies(_) | TriggerCond::LeavesBattlefield(_) => true,
        TriggerCond::ZoneChange { from, .. } => *from == Some(ZoneKind::Battlefield),
        TriggerCond::AnyOf(v) => v.iter().all(leaves),
        _ => false,
    }
}

inventory::submit! { TriggerPattern { name: "[event] or another/a [event] (several zones)", priority: 905, parse: either_events } }

/// Event qualifiers that only make sense for the event's own player: "you draw your
/// first card during each of your draw steps" (the card drawn in the draw step's
/// turn-based action, or the first one drawn instead, CR 504.1).
fn first_draw_in_draw_step(r: &str) -> Option<Parsed> {
    let r = end(r);
    let who = match r {
        "you draw your first card during each of your draw steps" => PlayerRel::You,
        _ => return None,
    };
    let c = where_events(
        TriggerCond::Draws { who },
        Condition::Not(Box::new(Condition::Custom(
            crate::kw::draw_step_draws::NOT_FIRST_DRAW_IN_DRAW_STEP.into(),
        ))),
    );
    Some((c, Sel::None, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "you draw your first card during each of your draw steps", priority: 110, parse: first_draw_in_draw_step } }

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Parsed {
        crate::oracle::triggers::parse_trigger_condition(s)
            .unwrap_or_else(|| panic!("failed to parse {s:?}"))
    }

    #[test]
    fn player_events_parse() {
        for s in [
            "whenever you create a token",
            "whenever you create a blood token",
            "whenever you create one or more creature tokens",
            "whenever you create or sacrifice a token",
            "whenever you sacrifice ~ or another artifact",
            "whenever one or more players sacrifice one or more creatures",
            "whenever one or more players discard one or more cards",
            "whenever you discard a spirit card or a card with disturb",
            "whenever a player mills a nonland card",
            "whenever an opponent mills a nonland card",
            "whenever a player mills one or more creature cards",
            "whenever one or more nonland cards are milled",
            "whenever an opponent discards a card or mills one or more cards",
            "whenever you gain or lose life",
            "whenever one or more opponents lose life",
            "whenever another player loses the game",
            "whenever you get one or more {e}",
            "whenever you clash and win",
            "whenever you win a clash",
            "whenever you draw your first or second card each turn",
            "whenever a player draws their second card during their turn",
            "whenever an opponent gains control of a permanent from you",
            "when ~ enters and when you sacrifice it",
            "whenever ~ dies or another artifact you control is put into a graveyard from the battlefield",
            "whenever a creature dies or a creature card is put into a graveyard from a library",
        ] {
            p(s);
        }
    }
}
