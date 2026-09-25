//! Trigger conditions (CR 603): a compositional parser for "When/Whenever/At …" phrases
//! that the built-in forms in `oracle/triggers.rs` don't cover.
//!
//! Grammar (lowercase, self references already normalized to `~`):
//!
//! * `[cond] and whenever [cond]`, `[cond] and at the beginning of …` — several trigger
//!   conditions on one ability ([`TriggerCond::AnyOf`]).
//! * `at the beginning of [whose] [step]`, `at end of combat [on your turn]`.
//! * State triggers (CR 603.8): `you control no [objects]`, `there are no [objects] on the
//!   battlefield`, `you have no cards in hand`.
//! * `[player] [verb]`: `you gain life [for the first time each turn]`, `an opponent casts
//!   a noncreature spell`, `you cast your second spell each turn`, `you sacrifice a Food`,
//!   `you attack with two or more creatures`, …
//! * `[subject] [verb] [or verb …]` where the subject is `~`, `~ or another [object]`,
//!   `a/an/another [object]`, `one or more [objects]`, or `enchanted/equipped creature`,
//!   and verbs include enters, dies, leaves the battlefield, attacks [alone], blocks [a
//!   creature], becomes blocked [by a creature], deals [combat] damage [to …], is dealt
//!   damage, becomes tapped/untapped, is turned face up, becomes the target of …
//!
//! Each parse also decides what "it"/"that creature" and "that player" refer to in the
//! effect, depending on the trigger event:
//!
//! | event | "it" | "that player" |
//! |---|---|---|
//! | enters | the permanent (`~` itself for self triggers) | its controller |
//! | dies / leaves | last known information; actions find the card (CR 400.7e) | its controller |
//! | attacks | the attacker | the attacked player |
//! | blocks a creature / becomes blocked by a creature | the other creature (CR 509.3b/d) | its controller |
//! | deals damage to a player | the source (`~` for self triggers) | the damaged player |
//! | deals damage to a creature (self) | the damaged creature | its controller |
//! | casts a spell | the spell | the caster |
//! | player events (gain life, draw, …) | — | the player |
//! | beginning of a step | `~` | the active player |
//!
//! When several conditions have different referents, "it"/"that player" are marked
//! unknown (`Sel::None` / `PlayerRef::Iterated`) and bodies using them are rejected.

use crate::ability::*;
use crate::oracle::patterns::TriggerPattern;
use crate::oracle::phrases::*;
use crate::types::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

inventory::submit! {
    TriggerPattern { name: "compositional trigger conditions", priority: 100, parse: parse_trigger }
}

/// Entry point: `r` is the text after "when"/"whenever", or the whole text for "at ...".
pub fn parse_trigger(r: &str) -> Option<Parsed> {
    let r = r.trim();
    if let Some(v) = parse_conjunction(r) {
        return Some(v);
    }
    parse_single(r)
}

fn parse_single(r: &str) -> Option<Parsed> {
    if r.starts_with("at ") {
        return parse_at(r);
    }
    if r == "day becomes night or night becomes day" {
        return Some((
            TriggerCond::DayNightChanges,
            Sel::None,
            PlayerRef::ActivePlayer,
        ));
    }
    // "deals noncombat damage", "is dealt noncombat damage": the damage trigger, restricted
    // to damage that isn't combat damage (CR 510.2 vs 120.2).
    if r.contains(" noncombat damage") {
        let (c, it, p) = parse_single(&r.replacen(" noncombat damage", " damage", 1))?;
        return Some((noncombat(c)?, it, p));
    }
    parse_state(r)
        .or_else(|| parse_counters_put(r))
        .or_else(|| parse_player_trigger(r))
        .or_else(|| parse_object_trigger(r))
}

/// Restricts the damage events of a trigger condition to noncombat damage.
fn noncombat(c: TriggerCond) -> Option<TriggerCond> {
    Some(match c {
        TriggerCond::Batched { trigger, per } => TriggerCond::Batched {
            trigger: Box::new(noncombat(*trigger)?),
            per,
        },
        TriggerCond::Where { trigger, cond } => TriggerCond::Where {
            trigger: Box::new(noncombat(*trigger)?),
            cond,
        },
        TriggerCond::FirstTimeEachTurn(t) => {
            TriggerCond::FirstTimeEachTurn(Box::new(noncombat(*t)?))
        }
        TriggerCond::AnyOf(v) => {
            TriggerCond::AnyOf(v.into_iter().map(noncombat).collect::<Option<Vec<_>>>()?)
        }
        c @ (TriggerCond::DealsDamage {
            combat_only: false, ..
        }
        | TriggerCond::IsDealtDamage {
            combat_only: false, ..
        }
        | TriggerCond::PlayerDealtDamage {
            combat_only: false, ..
        }) => TriggerCond::Noncombat(Box::new(c)),
        _ => return None,
    })
}

/// "one or more +1/+1 counters are put on [object]", "a +1/+1 counter is put on [object]".
/// Each such event is one put action on one object.
fn parse_counters_put(r: &str) -> Option<Parsed> {
    let (x, plural) = if let Some(x) = r.strip_prefix("one or more ") {
        (x, true)
    } else {
        (
            r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?,
            false,
        )
    };
    let (kind, rest) = crate::oracle::costs::counter_kind(x)?;
    let rest = if plural {
        rest.strip_prefix("counters are put on ")?
    } else {
        rest.strip_prefix("counter is put on ")?
    };
    let subj = parse_subject(rest)?;
    if subj.one_or_more {
        return None;
    }
    let it = if subj.self_only {
        Sel::This
    } else {
        Sel::TriggerObject
    };
    Some((
        TriggerCond::CountersPut {
            filter: subj.filter,
            kind: Some(kind),
        },
        it,
        PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
    ))
}

/// Merges the referents of several alternative trigger conditions.
fn merge(parts: Vec<Parsed>) -> Parsed {
    if parts.len() == 1 {
        return parts.into_iter().next().unwrap();
    }
    let same_it = parts
        .windows(2)
        .all(|w| format!("{:?}", w[0].1) == format!("{:?}", w[1].1));
    let same_player = parts
        .windows(2)
        .all(|w| format!("{:?}", w[0].2) == format!("{:?}", w[1].2));
    let it = if same_it {
        parts[0].1.clone()
    } else {
        Sel::None
    };
    let player = if same_player {
        parts[0].2.clone()
    } else {
        PlayerRef::Iterated
    };
    let conds: Vec<TriggerCond> = parts.into_iter().map(|p| p.0).collect();
    (TriggerCond::AnyOf(conds), it, player)
}

// ---------------------------------------------------------------------------
// "[cond] and whenever [cond]" (CR 603.1b: several trigger conditions)
// ---------------------------------------------------------------------------

fn parse_conjunction(r: &str) -> Option<Parsed> {
    const SEPS: [&str; 3] = [" and whenever ", " and when ", " and at the beginning of "];
    let (i, sep) = SEPS
        .iter()
        .filter_map(|s| r.find(s).map(|i| (i, *s)))
        .min_by_key(|(i, _)| *i)?;
    let first = &r[..i];
    let rest = &r[i + " and ".len()..];
    let _ = sep;
    // The first part had its "when"/"whenever" stripped; re-add one for the full parser.
    let first_full = if first.starts_with("at ") {
        first.to_string()
    } else {
        format!("whenever {first}")
    };
    let a = crate::oracle::triggers::parse_trigger_condition(&first_full)?;
    let b = crate::oracle::triggers::parse_trigger_condition(rest)?;
    if is_batch(&a.0) || is_batch(&b.0) {
        return None;
    }
    let mut parts = Vec::new();
    for p in [a, b] {
        match p {
            (TriggerCond::AnyOf(v), it, pl) => {
                for c in v {
                    parts.push((c, it.clone(), pl.clone()));
                }
            }
            other => parts.push(other),
        }
    }
    Some(merge(parts))
}

fn is_batch(c: &TriggerCond) -> bool {
    matches!(c, TriggerCond::Batched { .. })
}

// ---------------------------------------------------------------------------
// "At the beginning of ..." (CR 603.2b)
// ---------------------------------------------------------------------------

/// Whose turn a beginning-of-step trigger is restricted to.
enum Whose {
    Rel(PlayerRel),
    /// The active player must be this player (e.g. "enchanted player's upkeep", "the
    /// upkeep of enchanted creature's controller").
    Player(PlayerRef),
}

fn step_word(s: &str) -> Option<(TriggerStep, &str)> {
    let s = s.trim_start();
    for (p, st) in [
        ("upkeeps", TriggerStep::Upkeep),
        ("upkeep", TriggerStep::Upkeep),
        ("draw steps", TriggerStep::Draw),
        ("draw step", TriggerStep::Draw),
        ("end steps", TriggerStep::End),
        ("end step", TriggerStep::End),
        ("precombat main phases", TriggerStep::PrecombatMain),
        ("precombat main phase", TriggerStep::PrecombatMain),
        ("first main phases", TriggerStep::PrecombatMain),
        ("first main phase", TriggerStep::PrecombatMain),
        ("postcombat main phases", TriggerStep::PostcombatMain),
        ("postcombat main phase", TriggerStep::PostcombatMain),
        ("second main phases", TriggerStep::PostcombatMain),
        ("second main phase", TriggerStep::PostcombatMain),
        ("combat", TriggerStep::BeginningOfCombat),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with(' ') {
                return Some((st, r));
            }
        }
    }
    None
}

/// "your", "each", "each player's", "each opponent's", "enchanted player's", "the
/// monarch's", "the" → whose turn.
fn possessive(s: &str) -> Option<(Whose, &str)> {
    let s = s.trim_start();
    for (p, w) in [
        ("each of your ", Whose::Rel(PlayerRel::You)),
        ("your ", Whose::Rel(PlayerRel::You)),
        ("each player's ", Whose::Rel(PlayerRel::Any)),
        ("each opponent's ", Whose::Rel(PlayerRel::Opponent)),
        ("each other player's ", Whose::Rel(PlayerRel::NotYou)),
        ("each ", Whose::Rel(PlayerRel::Any)),
        ("the ", Whose::Rel(PlayerRel::Any)),
        (
            "enchanted player's ",
            Whose::Player(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))),
        ),
        (
            "enchanted opponent's ",
            Whose::Player(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))),
        ),
        ("the monarch's ", Whose::Player(PlayerRef::Monarch)),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((w, r));
        }
    }
    None
}

/// "enchanted creature's controller", "enchanted permanent's controller", ...
fn attached_controller(s: &str) -> Option<&str> {
    for p in [
        "enchanted creature's controller",
        "enchanted permanent's controller",
        "enchanted land's controller",
        "enchanted artifact's controller",
        "enchanted enchantment's controller",
        "equipped creature's controller",
    ] {
        if let Some(r) = s.trim_start().strip_prefix(p) {
            return Some(r);
        }
    }
    None
}

fn beginning_of(step: TriggerStep, whose: Whose) -> Parsed {
    let cond = match whose {
        Whose::Rel(rel) => TriggerCond::BeginningOf { step, whose: rel },
        Whose::Player(p) => TriggerCond::Where {
            trigger: Box::new(TriggerCond::BeginningOf {
                step,
                whose: PlayerRel::Any,
            }),
            cond: Condition::PlayerMatches(p, PlayerFilter::Active),
        },
    };
    (cond, Sel::This, PlayerRef::ActivePlayer)
}

fn parse_at(r: &str) -> Option<Parsed> {
    // "at end of combat [on your turn]", "at the end of combat on your turn"
    for p in ["at end of combat", "at the end of combat"] {
        if let Some(rest) = r.strip_prefix(p) {
            let whose = match rest.trim() {
                "" => PlayerRel::Any,
                "on your turn" => PlayerRel::You,
                _ => return None,
            };
            return Some(beginning_of(TriggerStep::EndOfCombat, Whose::Rel(whose)));
        }
    }
    let x = r.strip_prefix("at the beginning of ")?;
    // "the next ..." and "your next ..." are delayed-trigger phrasing; not handled here.
    if x.contains("next") {
        return None;
    }
    // "combat on [whose] turn"
    if let Some(w) = x.strip_prefix("combat on ") {
        let w = w.strip_suffix(" turn")?;
        let whose = match w {
            "your" => Whose::Rel(PlayerRel::You),
            "each player's" => Whose::Rel(PlayerRel::Any),
            "each opponent's" => Whose::Rel(PlayerRel::Opponent),
            "each other player's" => Whose::Rel(PlayerRel::NotYou),
            "enchanted player's" | "enchanted opponent's" => {
                Whose::Player(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo)))
            }
            _ => return None,
        };
        return Some(beginning_of(TriggerStep::BeginningOfCombat, whose));
    }
    // "each combat" / "each end step" etc. (possessive + step)
    if let Some((whose, rest)) = possessive(x) {
        if let Some((step, tail)) = step_word(rest) {
            let tail = tail.trim();
            if tail.is_empty() {
                // "the combat" isn't a thing; "each combat" is handled by the core.
                return Some(beginning_of(step, whose));
            }
            // "the upkeep of enchanted creature's controller"
            if let Some(t) = tail.strip_prefix("of ") {
                if let Some(rest) = attached_controller(t) {
                    if rest.trim().is_empty() {
                        // "that creature" is the enchanted creature.
                        let (c, _, p) = beginning_of(
                            step,
                            Whose::Player(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))),
                        );
                        return Some((c, Sel::AttachedTo, p));
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// State triggers (CR 603.8)
// ---------------------------------------------------------------------------

fn parse_state(r: &str) -> Option<Parsed> {
    let cond = if let Some(x) = r.strip_prefix("you control no ") {
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        Condition::Not(Box::new(Condition::Exists(f.you_control())))
    } else if let Some(x) = r.strip_prefix("there are no ") {
        let x = x.strip_suffix(" on the battlefield")?;
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        Condition::Not(Box::new(Condition::Exists(f)))
    } else if r == "you have no cards in hand" {
        Condition::Compare(Value::HandSize(PlayerRef::You), Cmp::Eq, Value::c(0))
    } else {
        return None;
    };
    Some((TriggerCond::State(cond), Sel::This, PlayerRef::You))
}

// ---------------------------------------------------------------------------
// Player triggers: "[player] [verb]"
// ---------------------------------------------------------------------------

pub(crate) fn player_subject(r: &str) -> Option<(PlayerRel, &str)> {
    for (p, rel) in [
        ("you ", PlayerRel::You),
        ("an opponent ", PlayerRel::Opponent),
        ("a player ", PlayerRel::Any),
        ("another player ", PlayerRel::NotYou),
        ("one of your opponents ", PlayerRel::Opponent),
    ] {
        if let Some(x) = r.strip_prefix(p) {
            return Some((rel, x));
        }
    }
    None
}

/// Strips a verb in either person ("gain"/"gains").
pub(crate) fn verb<'a>(s: &'a str, base: &str) -> Option<&'a str> {
    let s = s.trim_start();
    for form in [
        format!("{base}s "),
        format!("{base}es "),
        format!("{base} "),
    ] {
        if let Some(r) = s.strip_prefix(form.as_str()) {
            return Some(r);
        }
    }
    for form in [format!("{base}s"), format!("{base}es"), base.to_string()] {
        if s == form {
            return Some("");
        }
    }
    None
}

/// "for the first time each turn" / "during your turn" / "during an opponent's turn"
/// qualifiers on player events.
fn qualify(cond: TriggerCond, tail: &str) -> Option<TriggerCond> {
    let t = end(tail);
    Some(match t {
        "" => cond,
        "for the first time each turn" => TriggerCond::FirstTimeEachTurn(Box::new(cond)),
        "during your turn" => TriggerCond::Where {
            trigger: Box::new(cond),
            cond: Condition::YourTurn,
        },
        "during an opponent's turn" | "during each opponent's turn" => TriggerCond::Where {
            trigger: Box::new(cond),
            cond: Condition::NotYourTurn,
        },
        _ => return None,
    })
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

fn parse_player_trigger(r: &str) -> Option<Parsed> {
    // "whenever you're dealt damage": damage from several sources at once is one event.
    for (p, who) in [
        ("you're dealt damage", PlayerRel::You),
        ("you are dealt damage", PlayerRel::You),
        ("an opponent is dealt damage", PlayerRel::Opponent),
        ("a player is dealt damage", PlayerRel::Any),
    ] {
        if r == p {
            let c = TriggerCond::Batched {
                trigger: Box::new(TriggerCond::PlayerDealtDamage {
                    who,
                    combat_only: false,
                }),
                per: BatchPer::Player,
            };
            return Some((c, Sel::None, PlayerRef::TriggerPlayer));
        }
    }
    let (who, rest) = player_subject(r)?;
    let tp = || PlayerRef::TriggerPlayer;
    // Life.
    if let Some(t) = verb(rest, "gain").and_then(|x| x.strip_prefix("life")) {
        let c = qualify(TriggerCond::GainsLife { who }, t)?;
        return Some((c, Sel::None, tp()));
    }
    if let Some(t) = verb(rest, "lose").and_then(|x| x.strip_prefix("life")) {
        let c = qualify(TriggerCond::LosesLife { who }, t)?;
        return Some((c, Sel::None, tp()));
    }
    // Draws: "draw a card", "draw your second card each turn".
    if let Some(t) = verb(rest, "draw") {
        if let Some(q) = t.strip_prefix("a card") {
            let c = qualify(TriggerCond::Draws { who }, q)?;
            return Some((c, Sel::None, tp()));
        }
        let t = t
            .strip_prefix("your ")
            .or_else(|| t.strip_prefix("their "))?;
        let (w, t) = split_word(t);
        let n = ordinal(w)?;
        if !matches!(end(t), "card each turn" | "card in a turn") {
            return None;
        }
        // Event amount of a draw is its 1-based count this turn.
        let c = TriggerCond::Where {
            trigger: Box::new(TriggerCond::Draws { who }),
            cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(n as i32)),
        };
        return Some((c, Sel::None, tp()));
    }
    // "cast or copy an instant or sorcery spell" (CR 707.10: a copy isn't cast).
    if let Some(t) = rest
        .strip_prefix("cast or copy ")
        .or_else(|| rest.strip_prefix("casts or copies "))
    {
        let x = end(t);
        let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an "))?;
        let (filter, cond) = parse_spell_phrase(x)?;
        if cond.is_some() {
            return None;
        }
        let c = TriggerCond::AnyOf(vec![
            TriggerCond::CastSpell {
                who,
                filter: filter.clone(),
            },
            TriggerCond::SpellCopied { who, filter },
        ]);
        return Some((c, Sel::TriggerSpell, tp()));
    }
    // Casting spells.
    if let Some(t) = verb(rest, "cast") {
        return parse_cast(who, t);
    }
    // Cycling (CR 702.29c-d): "cycle or discard" triggers once for a cycled card.
    if let Some(t) = rest
        .strip_prefix("cycle or discard ")
        .or_else(|| rest.strip_prefix("cycles or discards "))
    {
        let filter = match end(t) {
            "a card" => Filter::Any,
            "another card" => Filter::Other,
            _ => return None,
        };
        return Some((
            TriggerCond::Discards { who, filter },
            Sel::TriggerObject,
            tp(),
        ));
    }
    if let Some(t) = verb(rest, "cycle") {
        let (filter, it) = match end(t) {
            "~" => (Filter::Source, Sel::This),
            "a card" => (Filter::Any, Sel::TriggerObject),
            "another card" => (Filter::Other, Sel::TriggerObject),
            _ => return None,
        };
        return Some((TriggerCond::Cycled { who, filter }, it, tp()));
    }
    // Discarding.
    if let Some(t) = verb(rest, "discard") {
        let t = end(t);
        if let Some(x) = t.strip_prefix("one or more ") {
            let (f, plural, tail) = parse_object_phrase(x)?;
            if !plural || !end(tail).is_empty() {
                return None;
            }
            return Some((
                TriggerCond::Batched {
                    trigger: Box::new(TriggerCond::Discards { who, filter: f }),
                    per: BatchPer::Player,
                },
                Sel::None,
                tp(),
            ));
        }
        let t = t.strip_prefix("a ").or_else(|| t.strip_prefix("an "))?;
        let (f, _, tail) = parse_object_phrase(t)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::Discards { who, filter: f },
            Sel::TriggerObject,
            tp(),
        ));
    }
    // Sacrificing.
    if let Some(t) = verb(rest, "sacrifice") {
        let t = end(t);
        if t == "~" && who == PlayerRel::You {
            // Looks back in time (CR 603.10a); actions find the card (CR 400.7e).
            return Some((TriggerCond::YouSacrifice(Filter::Source), Sel::This, tp()));
        }
        let (one_or_more, t) = match t.strip_prefix("one or more ") {
            Some(x) => (true, x),
            None => (
                false,
                t.strip_prefix("a ")
                    .or_else(|| t.strip_prefix("an "))
                    .unwrap_or(t),
            ),
        };
        let (f, plural, tail) = parse_object_phrase(t)?;
        if !end(tail).is_empty() || plural != one_or_more {
            return None;
        }
        // The sacrificed object is last known information (CR 603.10a).
        let base = if who == PlayerRel::You {
            TriggerCond::YouSacrifice(f)
        } else {
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::Sacrificed(f)),
                cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, rel_filter(who)?),
            }
        };
        if one_or_more {
            return Some((
                TriggerCond::Batched {
                    trigger: Box::new(base),
                    per: BatchPer::Batch,
                },
                Sel::None,
                tp(),
            ));
        }
        return Some((base, Sel::TriggerLki, tp()));
    }
    // Attacking: "you attack", "you attack with two or more creatures".
    if let Some(t) = verb(rest, "attack") {
        let t = end(t);
        let base = TriggerCond::PlayerAttacks(who);
        if t.is_empty() {
            return Some((base, Sel::None, tp()));
        }
        let x = t.strip_prefix("with ")?;
        let (n, x) = parse_number(x)?;
        let n = n.as_const()?;
        let x = end(x);
        if x != "or more creatures" {
            return None;
        }
        return Some((
            TriggerCond::Where {
                trigger: Box::new(base),
                cond: Condition::Compare(Value::EventAmount, Cmp::Ge, Value::c(n)),
            },
            Sel::None,
            tp(),
        ));
    }
    // Misc player events.
    let t = end(rest);
    let action = |name: &str| TriggerCond::PlayerAction {
        name: name.into(),
        who,
    };
    let simple = match t {
        "commit a crime" | "commits a crime" => TriggerCond::CommitCrime(who),
        "search your library" | "searches their library" => TriggerCond::Searched(who),
        "roll a die" | "rolls a die" => TriggerCond::RollDie(who),
        "flip a coin" | "flips a coin" => TriggerCond::FlipCoin(who),
        "win a coin flip" | "wins a coin flip" => TriggerCond::Where {
            trigger: Box::new(TriggerCond::FlipCoin(who)),
            cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(1)),
        },
        "play a land" | "plays a land" => TriggerCond::LandPlayed {
            who,
            filter: Filter::Any,
        },
        "loses the game" if who == PlayerRel::Any => TriggerCond::PlayerLoses,
        "scry" | "scries" => action("scry"),
        "surveil" | "surveils" => action("surveil"),
        "scry or surveil" | "scries or surveils" => {
            TriggerCond::AnyOf(vec![action("scry"), action("surveil")])
        }
        "proliferate" | "proliferates" => action("proliferate"),
        "roll one or more dice" | "rolls one or more dice" => TriggerCond::Batched {
            trigger: Box::new(TriggerCond::RollDie(who)),
            per: BatchPer::Batch,
        },
        _ => {
            // "roll a 1": the result of a die roll.
            let x = t
                .strip_prefix("roll a ")
                .or_else(|| t.strip_prefix("rolls a "))?;
            let n: i32 = x.parse().ok()?;
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::RollDie(who)),
                cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(n)),
            }
        }
    };
    Some((simple, Sel::None, tp()))
}

fn rel_filter(rel: PlayerRel) -> Option<PlayerFilter> {
    Some(match rel {
        PlayerRel::You => PlayerFilter::You,
        PlayerRel::Opponent => PlayerFilter::Opponent,
        PlayerRel::Any => PlayerFilter::Any,
        PlayerRel::NotYou => PlayerFilter::NotYou,
        _ => return None,
    })
}

/// "a [spell phrase]", "your second spell each turn", "their first spell each turn".
fn parse_cast(who: PlayerRel, t: &str) -> Option<Parsed> {
    let t = end(t);
    let spell = || Sel::TriggerSpell;
    let tp = || PlayerRef::TriggerPlayer;
    if let Some(x) = t.strip_prefix("your ").or_else(|| t.strip_prefix("their ")) {
        // "your second spell each turn", "your first spell during each opponent's turn"
        let (w, x) = split_word(x);
        let n = ordinal(w)?;
        // "your first noncreature spell each turn": the first such spell this turn.
        if let Some(phrase) = x.strip_suffix(" each turn") {
            if phrase != "spell" && n == 1 {
                let (filter, cond) = parse_spell_phrase(phrase)?;
                if cond.is_some() {
                    return None;
                }
                let c = TriggerCond::FirstTimeEachTurn(Box::new(TriggerCond::CastSpell {
                    who,
                    filter,
                }));
                return Some((c, spell(), tp()));
            }
        }
        let base = TriggerCond::NthSpellCast { who, n };
        let c = match x {
            "spell each turn" => base,
            "spell during each opponent's turn" | "spell during an opponent's turn" => {
                TriggerCond::Where {
                    trigger: Box::new(base),
                    cond: Condition::NotYourTurn,
                }
            }
            "spell during your turn" | "spell each turn during your turn" => TriggerCond::Where {
                trigger: Box::new(base),
                cond: Condition::YourTurn,
            },
            _ => return None,
        };
        return Some((c, spell(), tp()));
    }
    if t == "~" {
        return Some((
            TriggerCond::CastSpell {
                who,
                filter: Filter::Source,
            },
            Sel::This,
            tp(),
        ));
    }
    let x = t.strip_prefix("a ").or_else(|| t.strip_prefix("an "))?;
    // Older wording: "whenever you cast an instant", "a creature" (a spell being cast).
    let spellified;
    let x = if parse_spell_phrase(x).is_none() {
        match parse_object_phrase(x) {
            Some((f, false, tail)) if end(tail).is_empty() && is_card_type_filter(&f) => {
                spellified = format!("{x} spell");
                spellified.as_str()
            }
            _ => x,
        }
    } else {
        x
    };
    let (filter, cond) = parse_spell_phrase(x)?;
    let base = TriggerCond::CastSpell { who, filter };
    let c = match cond {
        Some(cond) => TriggerCond::Where {
            trigger: Box::new(base),
            cond,
        },
        None => base,
    };
    Some((c, spell(), tp()))
}

/// "[adjectives] spell [with …] [that targets …] [from …] [during …]" → (filter on the
/// spell, condition on the moment of casting).
pub fn parse_spell_phrase(x: &str) -> Option<(Filter, Option<Condition>)> {
    let mut parts = vec![];
    let x = match x.strip_prefix("kicked ") {
        Some(r) => {
            parts.push(Filter::CastWithCost("kicker".into()));
            r
        }
        None => x,
    };
    let (f, _, mut rest) = parse_object_phrase(x)?;
    if !mentions_spell(&f) {
        return None;
    }
    parts.push(f);
    let mut cond = None;
    loop {
        let t = rest.trim_start();
        if t.is_empty() {
            break;
        }
        if let Some(r) = t.strip_prefix("that targets ") {
            let (target, tail) = if let Some(r2) = r.strip_prefix('~') {
                (Filter::Source, r2)
            } else {
                let r2 = r
                    .strip_prefix("a ")
                    .or_else(|| r.strip_prefix("an "))
                    .or_else(|| r.strip_prefix("one or more "))
                    .unwrap_or(r);
                let (g, _, tail) = parse_object_phrase(r2)?;
                (g, tail)
            };
            parts.push(Filter::Targets(Box::new(target)));
            rest = tail;
            continue;
        }
        if let Some(r) = t.strip_prefix("from anywhere other than your hand") {
            parts.push(Filter::not(Filter::CastFrom(ZoneKind::Hand)));
            rest = r;
            continue;
        }
        if let Some(r) = t.strip_prefix("from exile") {
            parts.push(Filter::CastFrom(ZoneKind::Exile));
            rest = r;
            continue;
        }
        if let Some(r) = t
            .strip_prefix("from your graveyard")
            .or_else(|| t.strip_prefix("from a graveyard"))
        {
            parts.push(Filter::CastFrom(ZoneKind::Graveyard));
            rest = r;
            continue;
        }
        if let Some(r) = t.strip_prefix("with {x} in its mana cost") {
            parts.push(Filter::HasX);
            rest = r;
            continue;
        }
        if let Some(r) = t.strip_prefix("you don't own") {
            parts.push(Filter::OwnedBy(PlayerRel::NotYou));
            rest = r;
            continue;
        }
        if let Some(r) = t
            .strip_prefix("during an opponent's turn")
            .or_else(|| t.strip_prefix("during each opponent's turn"))
        {
            cond = Some(Condition::NotYourTurn);
            rest = r;
            continue;
        }
        if let Some(r) = t.strip_prefix("during your turn") {
            cond = Some(Condition::YourTurn);
            rest = r;
            continue;
        }
        return None;
    }
    Some((Filter::and(parts), cond))
}

/// A filter made only of card types ("instant", "instant or sorcery", "artifact").
fn is_card_type_filter(f: &Filter) -> bool {
    match f {
        Filter::Type(_) => true,
        Filter::Or(v) | Filter::And(v) => v.iter().all(is_card_type_filter),
        _ => false,
    }
}

fn mentions_spell(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(mentions_spell),
        Filter::Or(v) => v.iter().any(mentions_spell),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Object triggers: "[subject] [verb] [or verb …]"
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Subject {
    pub(crate) filter: Filter,
    /// The subject is exactly this object ("~").
    pub(crate) self_only: bool,
    /// "one or more [objects]": batch trigger.
    pub(crate) one_or_more: bool,
}

pub(crate) fn parse_subject(s: &str) -> Option<Subject> {
    let s = s.trim();
    let mk = |filter, self_only, one_or_more| {
        Some(Subject {
            filter,
            self_only,
            one_or_more,
        })
    };
    if s == "~" {
        return mk(Filter::Source, true, false);
    }
    // "a creature dealt damage by ~ this turn", "... by equipped creature this turn"
    if let Some((head, by)) = s.split_once(" dealt damage by ") {
        let source = match by.strip_suffix(" this turn")? {
            "~" => Sel::This,
            "equipped creature" | "enchanted creature" => Sel::AttachedTo,
            _ => return None,
        };
        let mut subj = parse_subject(head)?;
        if subj.self_only {
            return None;
        }
        subj.filter = Filter::and(vec![
            subj.filter,
            Filter::DealtDamageThisTurnBy(Box::new(source)),
        ]);
        return Some(subj);
    }
    // "a source", "a source you control", "a source an opponent controls" (CR 120.2: any
    // object can be a source of damage).
    for (p, f) in [
        ("a source", Filter::Any),
        ("a source you control", Filter::ControlledBy(PlayerRel::You)),
        (
            "a source an opponent controls",
            Filter::ControlledBy(PlayerRel::Opponent),
        ),
    ] {
        if s == p {
            return mk(f, false, false);
        }
    }
    if s == "~ or enchanted creature" || s == "~ or equipped creature" {
        return mk(
            Filter::Or(vec![Filter::Source, Filter::AttachedToSource]),
            false,
            false,
        );
    }
    if s == "your commander" {
        return mk(
            Filter::and(vec![Filter::Commander, Filter::OwnedBy(PlayerRel::You)]),
            false,
            false,
        );
    }
    if let Some(r) = s.strip_prefix("~ or another ") {
        let (f, plural, tail) = parse_object_phrase(r)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        return mk(
            Filter::Or(vec![Filter::Source, Filter::and(vec![f, Filter::Other])]),
            false,
            false,
        );
    }
    for p in [
        "enchanted creature",
        "equipped creature",
        "enchanted permanent",
        "enchanted land",
        "enchanted artifact",
        "enchanted enchantment",
        "enchanted planeswalker",
        "fortified land",
    ] {
        if s == p {
            return mk(Filter::AttachedToSource, false, false);
        }
    }
    if let Some(r) = s.strip_prefix("one or more ") {
        let (f, plural, tail) = parse_object_phrase(r)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        return mk(f, false, true);
    }
    let r = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    // Bare nouns need an article ("a creature"); "another creature" is fine.
    if r == s && !s.starts_with("another ") {
        return None;
    }
    mk(f, false, false)
}

fn parse_object_trigger(r: &str) -> Option<Parsed> {
    // Try every split point between subject and verb phrase.
    for (i, ch) in r.char_indices() {
        if ch != ' ' {
            continue;
        }
        let (subj_s, verb_s) = (&r[..i], &r[i + 1..]);
        let Some(subj) = parse_subject(subj_s) else {
            continue;
        };
        if let Some(p) = parse_verbs(verb_s, &subj) {
            return Some(p);
        }
    }
    None
}

/// "[verb] [or verb …]" for a subject.
fn parse_verbs(s: &str, subj: &Subject) -> Option<Parsed> {
    let mut alts: Vec<Parsed> = Vec::new();
    let mut s = s.trim();
    loop {
        let (p, rest) = parse_verb(s, subj)?;
        alts.push(p);
        let rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        s = rest.strip_prefix("or ")?;
    }
    if subj.one_or_more {
        if alts.len() != 1 {
            return None;
        }
        return Some(alts.pop().unwrap());
    }
    Some(merge(alts))
}

/// Tries each verb phrase at the start of `s`; returns the parse and the remaining text.
fn parse_verb<'a>(s: &'a str, subj: &Subject) -> Option<(Parsed, &'a str)> {
    let f = subj.filter.clone();
    let so = subj.self_only;
    let this_or = |other: Sel| if so { Sel::This } else { other };
    let ctl_of = |sel: Sel| PlayerRef::ControllerOf(Box::new(sel));
    let batch = |cond: TriggerCond, per_player: bool, player: PlayerRef| {
        (
            TriggerCond::Batched {
                trigger: Box::new(cond),
                per: if per_player {
                    BatchPer::Player
                } else {
                    BatchPer::Batch
                },
            },
            Sel::None,
            player,
        )
    };
    // Helper: phrase alternatives; each is (prefix, handler).
    let starts = |p: &str| -> Option<&'a str> {
        let r = s.strip_prefix(p)?;
        if r.is_empty() || r.starts_with(' ') || p.ends_with(' ') {
            Some(r)
        } else {
            None
        }
    };

    // --- enters -----------------------------------------------------------------------
    for p in [
        "enters the battlefield under your control",
        "enter the battlefield under your control",
        "enters under your control",
        "enter under your control",
        "enters the battlefield",
        "enter the battlefield",
        "enters",
        "enter",
    ] {
        if let Some(r) = starts(p) {
            let f = if p.contains("under your control") {
                f.clone().you_control()
            } else {
                f.clone()
            };
            let cond = TriggerCond::EntersBattlefield(f);
            if subj.one_or_more {
                return Some((batch(cond, false, ctl_of(Sel::TriggerObject)), r));
            }
            return Some((
                (
                    cond,
                    this_or(Sel::TriggerObject),
                    ctl_of(Sel::TriggerObject),
                ),
                r,
            ));
        }
    }
    // --- leaves the battlefield / dies / put into a graveyard ---------------------------
    let zone_change = |cond: TriggerCond, r: &'a str| {
        if subj.one_or_more {
            return Some((batch(cond, false, ctl_of(Sel::TriggerLki)), r));
        }
        // "It" is the object that left: last known information for its characteristics,
        // and the new object for actions (CR 400.7e; see `Game::resolve_sel`).
        Some(((cond, this_or(Sel::TriggerLki), ctl_of(Sel::TriggerLki)), r))
    };
    for p in ["dies", "die"] {
        if let Some(r) = starts(p) {
            return zone_change(TriggerCond::Dies(f.clone()), r);
        }
    }
    for p in ["leaves the battlefield", "leave the battlefield"] {
        if let Some(r) = starts(p) {
            return zone_change(TriggerCond::LeavesBattlefield(f.clone()), r);
        }
    }
    for (p, owner) in [
        ("is put into a graveyard from the battlefield", None),
        ("are put into a graveyard from the battlefield", None),
        (
            "is put into your graveyard from the battlefield",
            Some(PlayerRel::You),
        ),
        (
            "are put into your graveyard from the battlefield",
            Some(PlayerRel::You),
        ),
        (
            "is put into an opponent's graveyard from the battlefield",
            Some(PlayerRel::Opponent),
        ),
    ] {
        if let Some(r) = starts(p) {
            let f = match owner {
                Some(o) => Filter::and(vec![f.clone(), Filter::OwnedBy(o)]),
                None => f.clone(),
            };
            // CR 700.4: "dies" means "is put into a graveyard from the battlefield".
            return zone_change(TriggerCond::Dies(f), r);
        }
    }
    // "When enchanted artifact is put into a graveyard": it's a permanent, so from the
    // battlefield.
    if let Some(r) = starts("is put into a graveyard") {
        if matches!(f, Filter::AttachedToSource) {
            return zone_change(TriggerCond::Dies(f.clone()), r);
        }
    }
    for p in [
        "is put into exile from the battlefield",
        "are put into exile from the battlefield",
    ] {
        if let Some(r) = starts(p) {
            return zone_change(
                TriggerCond::ZoneChange {
                    filter: f.clone(),
                    from: Some(ZoneKind::Battlefield),
                    to: Some(ZoneKind::Exile),
                },
                r,
            );
        }
    }
    // "is put into a graveyard from anywhere", "is put into your graveyard from anywhere"
    for (p, owner) in [
        ("is put into a graveyard from anywhere", None),
        ("are put into a graveyard from anywhere", None),
        (
            "is put into your graveyard from anywhere",
            Some(PlayerRel::You),
        ),
        (
            "are put into your graveyard from anywhere",
            Some(PlayerRel::You),
        ),
        (
            "is put into an opponent's graveyard from anywhere",
            Some(PlayerRel::Opponent),
        ),
    ] {
        if let Some(r) = starts(p) {
            let f = match owner {
                Some(o) => Filter::and(vec![f.clone(), Filter::OwnedBy(o)]),
                None => f.clone(),
            };
            let cond = TriggerCond::ZoneChange {
                filter: f,
                from: None,
                to: Some(ZoneKind::Graveyard),
            };
            if subj.one_or_more {
                return Some((
                    batch(
                        cond,
                        false,
                        PlayerRef::OwnerOf(Box::new(Sel::TriggerObject)),
                    ),
                    r,
                ));
            }
            // The card in the graveyard.
            return Some((
                (
                    cond,
                    Sel::TriggerObject,
                    PlayerRef::OwnerOf(Box::new(Sel::TriggerObject)),
                ),
                r,
            ));
        }
    }
    for p in [
        "is put into your graveyard from your library",
        "are put into your graveyard from your library",
    ] {
        if let Some(r) = starts(p) {
            let cond = TriggerCond::ZoneChange {
                filter: Filter::and(vec![f.clone(), Filter::OwnedBy(PlayerRel::You)]),
                from: Some(ZoneKind::Library),
                to: Some(ZoneKind::Graveyard),
            };
            if subj.one_or_more {
                return Some((batch(cond, false, PlayerRef::You), r));
            }
            return Some(((cond, Sel::TriggerObject, PlayerRef::You), r));
        }
    }
    // "one or more cards leave your graveyard" (look back in time, CR 603.10a).
    for p in ["leaves your graveyard", "leave your graveyard"] {
        if let Some(r) = starts(p) {
            let cond = TriggerCond::ZoneChange {
                filter: Filter::and(vec![f.clone(), Filter::OwnedBy(PlayerRel::You)]),
                from: Some(ZoneKind::Graveyard),
                to: None,
            };
            if subj.one_or_more {
                return Some((batch(cond, false, PlayerRef::You), r));
            }
            return Some(((cond, Sel::TriggerObject, PlayerRef::You), r));
        }
    }
    // --- combat ---------------------------------------------------------------------
    // "~ and at least two other creatures attack"
    if let Some(r) = starts("and at least ") {
        if !so {
            return None;
        }
        let (n, r) = parse_number(r)?;
        let n = n.as_const()?;
        let r = r
            .trim_start()
            .strip_prefix("other creatures attack")
            .or_else(|| r.trim_start().strip_prefix("other creature attack"))?;
        let cond = TriggerCond::Where {
            trigger: Box::new(TriggerCond::Attacks(f)),
            cond: Condition::Compare(Value::EventAmount, Cmp::Ge, Value::c(n + 1)),
        };
        return Some(((cond, Sel::This, PlayerRef::TriggerPlayer), r));
    }
    if let Some(r) = starts("attacks and isn't blocked") {
        if subj.one_or_more {
            return None;
        }
        return Some((
            (
                TriggerCond::AttacksUnblocked(f),
                this_or(Sel::TriggerObject),
                PlayerRef::DefendingPlayer,
            ),
            r,
        ));
    }
    for p in ["attacks", "attack"] {
        if let Some(r) = starts(p) {
            let mut cond = TriggerCond::Attacks(f.clone());
            let mut r = r;
            let t = r.trim_start();
            if let Some(x) = t.strip_prefix("alone") {
                // CR 506.5: a creature attacks alone if it's the only creature declared as
                // an attacker.
                cond = TriggerCond::Where {
                    trigger: Box::new(cond),
                    cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(1)),
                };
                r = x;
            } else if let Some(x) = t.strip_prefix("you or a planeswalker you control") {
                cond = TriggerCond::Where {
                    trigger: Box::new(cond),
                    cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::You),
                };
                r = x;
            } else if let Some(x) = t.strip_prefix("you") {
                if x.is_empty() || x.starts_with(' ') {
                    cond = TriggerCond::Where {
                        trigger: Box::new(cond),
                        cond: Condition::And(vec![
                            Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::You),
                            Condition::Not(Box::new(Condition::SelNonEmpty(
                                Sel::TriggerOtherObject,
                            ))),
                        ]),
                    };
                    r = x;
                }
            } else if let Some((x, who)) = [
                ("one of your opponents", PlayerFilter::Opponent),
                ("an opponent", PlayerFilter::Opponent),
                ("a player", PlayerFilter::Any),
                (
                    "enchanted player",
                    PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo)))),
                ),
            ]
            .into_iter()
            .find_map(|(p, who)| {
                let x = t.strip_prefix(p)?;
                (x.is_empty() || x.starts_with(' ')).then_some((x, who))
            }) {
                // CR 508.3a: attacking that player (not a planeswalker or battle).
                cond = TriggerCond::Where {
                    trigger: Box::new(cond),
                    cond: Condition::And(vec![
                        Condition::PlayerMatches(PlayerRef::TriggerPlayer, who),
                        Condition::Not(Box::new(Condition::SelNonEmpty(Sel::TriggerOtherObject))),
                    ]),
                };
                r = x;
            } else if let Some(x) = t.strip_prefix("for the first time each turn") {
                cond = TriggerCond::FirstTimeEachTurn(Box::new(cond));
                r = x;
            } else if let Some(x) = t.strip_prefix("while you control ") {
                let x2 = x
                    .strip_prefix("a ")
                    .or_else(|| x.strip_prefix("an "))
                    .unwrap_or(x);
                let (g, plural, tail) = parse_object_phrase(x2)?;
                if plural {
                    return None;
                }
                cond = TriggerCond::Where {
                    trigger: Box::new(cond),
                    cond: Condition::Exists(g.you_control()),
                };
                r = tail;
            }
            if subj.one_or_more {
                return Some((batch(cond, false, PlayerRef::TriggerPlayer), r));
            }
            return Some((
                (cond, this_or(Sel::TriggerObject), PlayerRef::TriggerPlayer),
                r,
            ));
        }
    }
    // "blocks or becomes blocked by a creature", "blocks or becomes blocked"
    if let Some((g, r)) = starts("blocks or becomes blocked by ").and_then(article_phrase) {
        if !so {
            return None;
        }
        return Some((
            (
                TriggerCond::AnyOf(vec![
                    TriggerCond::BlocksCreature {
                        blocker: f.clone(),
                        attacker: g.clone(),
                    },
                    TriggerCond::BlockedByCreature {
                        attacker: f,
                        blocker: g,
                    },
                ]),
                Sel::TriggerOtherObject,
                ctl_of(Sel::TriggerOtherObject),
            ),
            r,
        ));
    }
    if let Some(r) = starts("blocks or becomes blocked") {
        if subj.one_or_more {
            return None;
        }
        return Some((
            (
                TriggerCond::BlocksOrBecomesBlocked(f),
                this_or(Sel::TriggerObject),
                PlayerRef::You,
            ),
            r,
        ));
    }
    if let Some((g, r)) = starts("blocks ").and_then(article_phrase) {
        // "~ blocks a creature [with flying]": once per blocked attacker (CR 509.3b).
        if !so {
            return None;
        }
        return Some((
            (
                TriggerCond::BlocksCreature {
                    blocker: f,
                    attacker: g,
                },
                Sel::TriggerOtherObject,
                ctl_of(Sel::TriggerOtherObject),
            ),
            r,
        ));
    }
    for p in ["blocks", "block"] {
        if let Some(r) = starts(p) {
            // CR 509.3a: once per combat for each blocking creature.
            if subj.one_or_more {
                return None;
            }
            return Some((
                (
                    TriggerCond::Blocks(f),
                    this_or(Sel::TriggerObject),
                    PlayerRef::You,
                ),
                r,
            ));
        }
    }
    if let Some((g, r)) = starts("becomes blocked by ").and_then(article_phrase) {
        // CR 509.3d: once for each creature blocking it.
        if !so {
            return None;
        }
        return Some((
            (
                TriggerCond::BlockedByCreature {
                    attacker: f,
                    blocker: g,
                },
                Sel::TriggerOtherObject,
                ctl_of(Sel::TriggerOtherObject),
            ),
            r,
        ));
    }
    for p in ["becomes blocked", "become blocked"] {
        if let Some(r) = starts(p) {
            // CR 509.3c: once per combat, however many creatures block it.
            let cond = TriggerCond::BecomesBlocked(f);
            if subj.one_or_more {
                return Some((batch(cond, false, PlayerRef::DefendingPlayer), r));
            }
            return Some((
                (
                    cond,
                    this_or(Sel::TriggerObject),
                    PlayerRef::DefendingPlayer,
                ),
                r,
            ));
        }
    }
    // --- damage ---------------------------------------------------------------------
    if let Some(p) = parse_damage_verb(s, subj) {
        return Some(p);
    }
    for (p, combat_only) in [("is dealt combat damage", true), ("is dealt damage", false)] {
        if let Some(r) = starts(p) {
            if subj.one_or_more {
                return None;
            }
            // Damage dealt to it by several sources at once is one event: the ability
            // triggers once, and "that much" is the total (Boros Reckoner ruling).
            return Some((
                (
                    TriggerCond::Batched {
                        trigger: Box::new(TriggerCond::IsDealtDamage {
                            filter: f,
                            combat_only,
                        }),
                        per: BatchPer::Object,
                    },
                    this_or(Sel::TriggerObject),
                    ctl_of(Sel::TriggerObject),
                ),
                r,
            ));
        }
    }
    // --- status changes ---------------------------------------------------------------
    let simple: [(&str, fn(Filter) -> TriggerCond); 7] = [
        ("becomes tapped", TriggerCond::BecomesTapped),
        ("become tapped", TriggerCond::BecomesTapped),
        ("becomes untapped", TriggerCond::BecomesUntapped),
        ("become untapped", TriggerCond::BecomesUntapped),
        ("is turned face up", TriggerCond::TurnedFaceUp),
        ("transforms into ~", TriggerCond::Transforms),
        ("transforms", TriggerCond::Transforms),
    ];
    for (p, mk) in simple {
        if let Some(r) = starts(p) {
            let cond = mk(f.clone());
            if subj.one_or_more {
                return Some((batch(cond, false, ctl_of(Sel::TriggerObject)), r));
            }
            return Some((
                (
                    cond,
                    this_or(Sel::TriggerObject),
                    ctl_of(Sel::TriggerObject),
                ),
                r,
            ));
        }
    }
    // --- targeting ------------------------------------------------------------------
    if let Some(r) = starts("becomes the target of ") {
        if subj.one_or_more {
            return None;
        }
        let (by, only_spells, mut r) = if let Some(x) = r.strip_prefix("a spell or ability") {
            let (by, x) = controller_suffix(x);
            (by, false, x)
        } else if let Some(x) = r.strip_prefix("a spell") {
            let (by, x) = controller_suffix(x);
            (by, true, x)
        } else {
            return None;
        };
        let mut cond = TriggerCond::BecomesTarget { filter: f, by };
        if only_spells {
            cond = TriggerCond::Where {
                trigger: Box::new(cond),
                cond: Condition::SelMatches(Sel::TriggerSpell, Filter::Spell),
            };
        }
        if let Some(x) = r.trim_start().strip_prefix("for the first time each turn") {
            cond = TriggerCond::FirstTimeEachTurn(Box::new(cond));
            r = x;
        }
        return Some((
            (cond, this_or(Sel::TriggerObject), PlayerRef::TriggerPlayer),
            r,
        ));
    }
    None
}

/// "[a spell or ability] an opponent controls" / "you control".
fn controller_suffix(s: &str) -> (PlayerRel, &str) {
    let t = s.trim_start();
    if let Some(r) = t.strip_prefix("an opponent controls") {
        return (PlayerRel::Opponent, r);
    }
    if let Some(r) = t.strip_prefix("you control") {
        return (PlayerRel::You, r);
    }
    (PlayerRel::Any, s)
}

/// "a creature [with flying]", "a non-Wall creature", "an artifact creature" after
/// "blocks"/"becomes blocked by": the filter and the remaining text.
fn article_phrase(r: &str) -> Option<(Filter, &str)> {
    let t = r.trim_start();
    let t = t.strip_prefix("a ").or_else(|| t.strip_prefix("an "))?;
    let (g, plural, tail) = parse_object_phrase(t)?;
    if plural {
        return None;
    }
    Some((g, tail))
}

/// "deals [combat] damage [to a player | to an opponent | to you | to a creature | to a
/// player or planeswalker]".
fn parse_damage_verb<'a>(s: &'a str, subj: &Subject) -> Option<(Parsed, &'a str)> {
    let (combat, r) = [
        ("deals combat damage", true),
        ("deal combat damage", true),
        ("deals damage", false),
        ("deal damage", false),
    ]
    .iter()
    .find_map(|(p, c)| s.strip_prefix(p).map(|r| (*c, r)))?;
    let t = r.trim_start();
    let recipients: [(&str, DamageRecipient, bool); 10] = [
        (
            "to one of your opponents",
            DamageRecipient::Player(PlayerRel::Opponent),
            true,
        ),
        ("to ~", DamageRecipient::Object(Filter::Source), false),
        (
            "to a player or planeswalker",
            DamageRecipient::PlayerOrPlaneswalker(PlayerRel::Any),
            true,
        ),
        (
            "to an opponent or planeswalker",
            DamageRecipient::PlayerOrPlaneswalker(PlayerRel::Opponent),
            true,
        ),
        ("to a player", DamageRecipient::Player(PlayerRel::Any), true),
        (
            "to an opponent",
            DamageRecipient::Player(PlayerRel::Opponent),
            true,
        ),
        ("to you", DamageRecipient::Player(PlayerRel::You), true),
        (
            "to a creature",
            DamageRecipient::Object(Filter::creature()),
            false,
        ),
        (
            "to a planeswalker",
            DamageRecipient::Object(Filter::Type(CardType::Planeswalker)),
            false,
        ),
        (
            "to another creature",
            DamageRecipient::Object(Filter::and(vec![Filter::creature(), Filter::Other])),
            false,
        ),
    ];
    if let Some(rest) = t.strip_prefix("to a player or battle") {
        if subj.one_or_more {
            return None;
        }
        let mk = |to| TriggerCond::DealsDamage {
            source: subj.filter.clone(),
            to,
            combat_only: combat,
        };
        let c = TriggerCond::AnyOf(vec![
            mk(DamageRecipient::Player(PlayerRel::Any)),
            mk(DamageRecipient::Object(Filter::Type(CardType::Battle))),
        ]);
        let it = if subj.self_only {
            Sel::This
        } else {
            Sel::TriggerOtherObject
        };
        return Some(((c, it, PlayerRef::TriggerPlayer), rest));
    }
    let (to, to_player, rest) = if t.is_empty() || !t.starts_with("to ") {
        (DamageRecipient::Any, false, r)
    } else {
        let (p, to, pl) = recipients
            .iter()
            .find(|(p, _, _)| {
                t.strip_prefix(p)
                    .is_some_and(|x| x.is_empty() || x.starts_with(' '))
            })?
            .clone();
        (to, pl, &t[p.len()..])
    };
    let cond = TriggerCond::DealsDamage {
        source: subj.filter.clone(),
        to: to.clone(),
        combat_only: combat,
    };
    if subj.one_or_more {
        // "one or more creatures you control deal combat damage to a player": once for each
        // player dealt damage (CR 603.2c).
        if !to_player {
            return None;
        }
        return Some((
            (
                TriggerCond::Batched {
                    trigger: Box::new(cond),
                    per: BatchPer::Player,
                },
                Sel::None,
                PlayerRef::TriggerPlayer,
            ),
            rest,
        ));
    }
    if matches!(to, DamageRecipient::Any) {
        // "~ deals damage" / "equipped creature deals combat damage": damage a source deals
        // to several recipients at once is one event, so this triggers once per source,
        // and "that much" is the total dealt.
        return Some((
            (
                TriggerCond::Batched {
                    trigger: Box::new(cond),
                    per: BatchPer::Other,
                },
                if subj.self_only {
                    Sel::This
                } else {
                    Sel::TriggerOtherObject
                },
                PlayerRef::Iterated,
            ),
            rest,
        ));
    }
    let parsed = match (&to, subj.self_only) {
        // "~ deals damage to a creature, destroy that creature"
        (DamageRecipient::Object(_), true) => (
            cond,
            Sel::TriggerObject,
            PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
        ),
        (DamageRecipient::Object(_), false) => (cond, Sel::None, PlayerRef::Iterated),
        (_, true) => (cond, Sel::This, PlayerRef::TriggerPlayer),
        (_, false) => (cond, Sel::TriggerOtherObject, PlayerRef::TriggerPlayer),
    };
    Some((parsed, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Parsed {
        crate::oracle::triggers::parse_trigger_condition(s)
            .unwrap_or_else(|| panic!("failed to parse {s:?}"))
    }

    #[test]
    fn conditions_parse() {
        for s in [
            "whenever you cast a noncreature spell",
            "whenever you cast a spell that targets ~",
            "whenever you cast your second spell each turn",
            "whenever you draw your second card each turn",
            "whenever ~ enters or attacks",
            "when ~ enters or dies",
            "whenever ~ or another creature you control enters",
            "when ~ is turned face up",
            "at the beginning of the end step",
            "at the beginning of the upkeep of enchanted creature's controller",
            "at the beginning of enchanted player's upkeep",
            "at end of combat",
            "when you control no islands",
            "whenever one or more creatures you control deal combat damage to a player",
            "whenever a creature you control attacks alone",
            "whenever ~ becomes blocked by a creature",
            "whenever ~ blocks a creature with flying",
            "whenever you gain life for the first time each turn",
            "whenever you sacrifice another permanent",
            "whenever you attack with two or more creatures",
            "when you cast ~",
            "whenever one or more cards leave your graveyard",
            "at the beginning of your upkeep and whenever you cast a green spell",
            "whenever a creature you control with deathtouch deals combat damage to a player",
            "whenever a creature with a -1/-1 counter on it dies",
            "whenever ~ attacks for the first time each turn",
            "whenever a creature deals combat damage to one of your opponents",
            "whenever you're dealt damage",
            "whenever a source you control deals damage to you",
            "whenever you cast an instant",
            "when you draw your third card in a turn",
            "whenever a creature dealt damage by ~ this turn dies",
            "whenever ~ becomes blocked by a non-wall creature",
        ] {
            p(s);
        }
    }

    #[test]
    fn one_or_more_is_batched() {
        let (c, _, _) =
            p("whenever one or more creatures you control deal combat damage to a player");
        assert!(matches!(
            c,
            TriggerCond::Batched {
                per: BatchPer::Player,
                ..
            }
        ));
        let (c, _, _) = p("whenever one or more tokens you control enter");
        assert!(matches!(
            c,
            TriggerCond::Batched {
                per: BatchPer::Batch,
                ..
            }
        ));
    }
}
