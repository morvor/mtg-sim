//! Conditions about the game state that don't refer back to anything (no "it", "that
//! player"): read wherever conditions are ("if ...", "as long as ...", "unless ...").
//!
//! * Players compared with players, quantified: "an opponent has more cards in hand than
//!   you", "you have less life than an opponent", "you control fewer creatures than an
//!   opponent", "a player has more life than each other player", "a player controls more
//!   Wizards than each other player", "each player has 10 or less life", "a player controls
//!   no creatures", "a player has one or fewer cards in hand", "defending player has more
//!   cards in hand than you", "no opponent has more life than you", "an opponent was dealt
//!   damage this turn", "you're the defending player", "there is no monarch".
//! * Your hand, library and life: "you have a card in hand", "you have exactly thirteen
//!   cards in your hand", "you have 200 or more cards in your library", "there are no cards
//!   in your library", "the top card of your library is a creature card", "your life total
//!   is 5 or less".
//! * Attacks: "a creature is attacking you", "a white creature is attacking", "an artifact
//!   creature you control is attacking".
//! * Groups: "creatures you control have total toughness 10 or greater", "there are three
//!   or more different kinds of counters among creatures you control", "there are ten or
//!   more creature cards total in all graveyards", "all lands on the battlefield are
//!   Islands", "all nonland permanents you control are white", "you control a permanent of
//!   each color", "you control a land of each basic land type and a creature of each
//!   color", "another creature is on the battlefield", "your team controls another
//!   Warrior" (CR 102.4).
//! * The source: "~ is in the command zone", "~ is the only creature card in your
//!   graveyard", cards "exiled with ~" ("there are four or more creature cards exiled with
//!   ~", "three or more cards have been exiled with ~", "a card is exiled with ~").
//!
//! Each is checked as it's read: when the ability resolves, or continuously for a static
//! ability.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::patterns::statics_conditions::{amount_cmp, color_or_phrase};
use crate::oracle::phrases::*;
use crate::types::*;

/// An object phrase, with "you don't own" / "you own" after it ("you control three or
/// more permanents you don't own").
fn noun(r: &str) -> Option<Filter> {
    if let Some(x) = r.strip_suffix(" you don't own") {
        return Some(Filter::and(vec![
            color_or_phrase(x)?,
            Filter::not(Filter::OwnedBy(PlayerRel::You)),
        ]));
    }
    if let Some(x) = r.strip_suffix(" you own") {
        return Some(Filter::and(vec![
            color_or_phrase(x)?,
            Filter::OwnedBy(PlayerRel::You),
        ]));
    }
    color_or_phrase(r)
}

/// How a player phrase quantifies over players.
#[derive(Clone)]
enum Quant {
    /// "a player", "an opponent": some player matches.
    Some(PlayerRef),
    /// "each player", "each opponent": all of them match.
    Each(PlayerRef),
    /// "no player", "no opponent".
    No(PlayerRef),
    /// One player: "you", "defending player".
    One(PlayerRef),
}

fn quantifier(c: &str) -> Option<(Quant, bool, &str)> {
    // (quantifier, subject is "you" (base-form verbs), rest)
    for (p, q, you) in [
        ("a player ", Quant::Some(PlayerRef::EachPlayer), false),
        ("an opponent ", Quant::Some(PlayerRef::EachOpponent), false),
        ("each player ", Quant::Each(PlayerRef::EachPlayer), false),
        ("each opponent ", Quant::Each(PlayerRef::EachOpponent), false),
        ("no player ", Quant::No(PlayerRef::EachPlayer), false),
        ("no opponent ", Quant::No(PlayerRef::EachOpponent), false),
        ("defending player ", Quant::One(PlayerRef::DefendingPlayer), false),
        ("you ", Quant::One(PlayerRef::You), true),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            return Some((q, you, r));
        }
    }
    None
}

/// A number a player has, for comparisons between players.
#[derive(Clone)]
enum Stat {
    Life,
    Hand,
    Controls(Filter),
}

impl Stat {
    fn filter(&self, cmp: Cmp, v: Value) -> PlayerFilter {
        match self {
            Stat::Life => PlayerFilter::Life(cmp, Box::new(v)),
            Stat::Hand => PlayerFilter::HandSize(cmp, Box::new(v)),
            Stat::Controls(f) => PlayerFilter::Controls(Box::new(f.clone()), cmp, Box::new(v)),
        }
    }
    /// The number for player `who`.
    fn of(&self, who: PlayerRef) -> Value {
        match self {
            Stat::Life => Value::LifeTotal(who),
            Stat::Hand => Value::HandSize(who),
            Stat::Controls(f) => {
                let rel = match who {
                    PlayerRef::You => PlayerRel::You,
                    _ => PlayerRel::Iterated,
                };
                Value::Count(Filter::and(vec![f.clone(), Filter::ControlledBy(rel)]))
            }
        }
    }
}

/// "more life than [other]", "more cards in hand than [other]", "fewer creatures than
/// [other]" (after "has"/"have"/"controls"/"control"): the stat, the comparison, and the
/// other player phrase.
fn comparison(r: &str, controls: bool) -> Option<(Stat, Cmp, &str)> {
    let (cmp, r) = if let Some(x) = r.strip_prefix("more ") {
        (Cmp::Gt, x)
    } else if let Some(x) = r
        .strip_prefix("fewer ")
        .or_else(|| r.strip_prefix("less "))
    {
        (Cmp::Lt, x)
    } else {
        return None;
    };
    let (what, other) = r.split_once(" than ")?;
    let stat = if controls {
        Stat::Controls(color_or_phrase(what)?)
    } else {
        match what {
            "life" => Stat::Life,
            "cards in hand" | "cards in their hand" | "cards in your hand" => Stat::Hand,
            _ => return None,
        }
    };
    Some((stat, cmp, other))
}

/// The player filter for "[stat] [cmp] [other]" where `other` is one player ("you"), and
/// the quantifier over the others ("an opponent": `Some`).
fn compare_with(stat: &Stat, cmp: Cmp, other: &str) -> Option<PlayerFilter> {
    match other {
        "you" => Some(stat.filter(cmp, stat.of(PlayerRef::You))),
        _ => None,
    }
}

fn flip(c: Cmp) -> Cmp {
    match c {
        Cmp::Gt => Cmp::Lt,
        Cmp::Lt => Cmp::Gt,
        Cmp::Ge => Cmp::Le,
        Cmp::Le => Cmp::Ge,
        c => c,
    }
}

/// What a player has or did, worded after the subject (`you`: base-form verbs).
fn player_state(r: &str, you: bool) -> Option<PlayerFilter> {
    let (has, controls, is) = if you {
        ("have ", "control ", "are ")
    } else {
        ("has ", "controls ", "is ")
    };
    let r = end(r);
    match r {
        "was dealt damage this turn" | "has been dealt damage this turn" if !you => {
            return Some(PlayerFilter::DealtDamageThisTurn)
        }
        "were dealt damage this turn" | "have been dealt damage this turn" if you => {
            return Some(PlayerFilter::DealtDamageThisTurn)
        }
        _ => {}
    }
    if let Some(x) = r.strip_prefix(is) {
        return match x {
            "the monarch" => Some(PlayerFilter::Monarch),
            "poisoned" => Some(PlayerFilter::Poisoned),
            _ => None,
        };
    }
    if let Some(x) = r.strip_prefix(has) {
        if let Some((stat, cmp, other)) = comparison(x, false) {
            return compare_with(&stat, cmp, other);
        }
        let x = x.strip_prefix("a card in ").map(|h| format!("1 or more cards in {h}"));
        let x = x.as_deref().unwrap_or(&r[has.len()..]);
        let (cmp, n, tail) = amount_cmp(x)?;
        return match end(tail) {
            "life" => Some(PlayerFilter::Life(cmp, Box::new(n))),
            "cards in hand" | "card in hand" | "cards in their hand" | "cards in your hand"
            | "card in your hand" => Some(PlayerFilter::HandSize(cmp, Box::new(n))),
            _ => None,
        };
    }
    if let Some(x) = r.strip_prefix(controls) {
        if let Some((stat, cmp, other)) = comparison(x, true) {
            return compare_with(&stat, cmp, other);
        }
        if let Some(y) = x.strip_prefix("no ") {
            return Some(PlayerFilter::Controls(
                Box::new(color_or_phrase(y)?),
                Cmp::Eq,
                Box::new(Value::c(0)),
            ));
        }
        let (cmp, n, rest) = amount_cmp(x)?;
        return Some(PlayerFilter::Controls(
            Box::new(noun(rest)?),
            cmp,
            Box::new(n),
        ));
    }
    None
}

fn quantified(q: Quant, f: PlayerFilter) -> Condition {
    match q {
        Quant::Some(r) | Quant::One(r) => Condition::PlayerMatches(r, f),
        Quant::Each(r) => Condition::Not(Box::new(Condition::PlayerMatches(
            r,
            PlayerFilter::Not(Box::new(f)),
        ))),
        Quant::No(r) => Condition::Not(Box::new(Condition::PlayerMatches(r, f))),
    }
}

/// "a player has more life than each other player": one player has more than every other
/// (exactly one player has the most).
fn more_than_each_other(stat: &Stat) -> Condition {
    let most = Value::OverPlayers(
        AggOp::Max,
        PlayerFilter::Any,
        Box::new(stat.of(PlayerRef::Iterated)),
    );
    Condition::Compare(
        Value::CountPlayers(stat.filter(Cmp::Eq, most)),
        Cmp::Eq,
        Value::c(1),
    )
}

/// Conditions with a player subject.
fn player_condition(c: &str) -> Option<Condition> {
    if c == "there is no monarch" {
        return Some(Condition::Not(Box::new(Condition::PlayerMatches(
            PlayerRef::EachPlayer,
            PlayerFilter::Monarch,
        ))));
    }
    if c == "you're the defending player" {
        return Some(Condition::PlayerMatches(
            PlayerRef::You,
            PlayerFilter::Defending,
        ));
    }
    let (q, you, r) = quantifier(c)?;
    // "a player has more life than each other player", "a player controls more Wizards
    // than each other player".
    if matches!(&q, Quant::Some(PlayerRef::EachPlayer)) {
        for (verb, controls) in [("has ", false), ("controls ", true)] {
            if let Some((stat, Cmp::Gt, "each other player")) =
                r.strip_prefix(verb).and_then(|x| comparison(x, controls))
            {
                return Some(more_than_each_other(&stat));
            }
        }
    }
    // "you have less life than an opponent", "you control fewer creatures than an
    // opponent": an opponent has more.
    if matches!(&q, Quant::One(PlayerRef::You)) {
        for (verb, controls) in [("have ", false), ("control ", true)] {
            if let Some((stat, cmp, "an opponent")) =
                r.strip_prefix(verb).and_then(|x| comparison(x, controls))
            {
                return Some(Condition::PlayerMatches(
                    PlayerRef::EachOpponent,
                    stat.filter(flip(cmp), stat.of(PlayerRef::You)),
                ));
            }
        }
    }
    let f = player_state(r, you)?;
    Some(quantified(q, f))
}

/// "your life total is 5 or less", "... is less than 7", "your library", "your hand", the
/// top card of your library.
fn your_zones_condition(c: &str) -> Option<Condition> {
    if let Some(r) = c.strip_prefix("your life total is ") {
        let (cmp, n, tail) = amount_cmp(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Compare(Value::LifeTotal(PlayerRef::You), cmp, n));
    }
    if c == "there are no cards in your library" {
        return Some(Condition::Compare(
            Value::LibrarySize(PlayerRef::You),
            Cmp::Eq,
            Value::c(0),
        ));
    }
    if let Some(r) = c.strip_prefix("you have ") {
        let r = r.strip_prefix("a card ").map(|t| format!("1 or more cards {t}"));
        let r = r.as_deref().unwrap_or(&c["you have ".len()..]);
        let (cmp, n, tail) = amount_cmp(r)?;
        let v = match end(tail) {
            "cards in your library" | "card in your library" => {
                Value::LibrarySize(PlayerRef::You)
            }
            "cards in hand" | "card in hand" | "cards in your hand" | "card in your hand" => {
                Value::HandSize(PlayerRef::You)
            }
            _ => return None,
        };
        return Some(Condition::Compare(v, cmp, n));
    }
    // "the top card of your library is a creature card", "... is black": a card there
    // that is.
    if let Some(r) = c.strip_prefix("the top card of your library is ") {
        let f = match Color::from_word(r) {
            Some(col) => Filter::Color(col),
            None => {
                let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
                color_or_phrase(r)?
            }
        };
        let top = Sel::TopOfLibrary(PlayerRef::You, Value::c(1));
        return Some(Condition::SelMatches(top, f));
    }
    None
}

/// "a creature is attacking you", "a white creature is attacking", "an artifact creature
/// you control is attacking".
fn attacking_condition(c: &str) -> Option<Condition> {
    let (r, attacking) = if let Some(r) = c.strip_suffix(" is attacking you") {
        (
            r,
            Filter::and(vec![
                Filter::Attacking,
                Filter::AttackingPlayer(PlayerRel::You),
            ]),
        )
    } else if let Some(r) = c.strip_suffix(" is attacking") {
        (r, Filter::Attacking)
    } else {
        return None;
    };
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let f = color_or_phrase(r)?;
    Some(Condition::Exists(Filter::and(vec![f, attacking])))
}

/// Cards exiled with ~ (CR 607.2a): "[N or more] [cards] [have been] exiled with ~".
fn exiled_with_source(f: Filter) -> Filter {
    Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Exile),
        Filter::In(Box::new(Sel::Linked)),
    ])
}

/// Groups of objects: totals, kinds of counters, "all ... are ...", "each color".
fn group_condition(c: &str) -> Option<Condition> {
    // "creatures you control have total toughness 10 or greater"
    for (p, stat) in [
        ("creatures you control have total toughness ", Stat_::Toughness),
        ("creatures you control have total power ", Stat_::Power),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            let (cmp, n, tail) = amount_cmp(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            let creatures = Sel::All(Filter::Type(CardType::Creature).you_control());
            let stat = match stat {
                Stat_::Toughness => crate::ability::Stat::Toughness,
                Stat_::Power => crate::ability::Stat::Power,
            };
            return Some(Condition::Compare(
                Value::Aggregate(AggOp::Sum, stat, Box::new(creatures)),
                cmp,
                n,
            ));
        }
    }
    if let Some(r) = c.strip_prefix("there are ") {
        // "there are three or more different kinds of counters among creatures you control"
        let (cmp, n, tail) = amount_cmp(r)?;
        if let Some(among) = tail.strip_prefix("different kinds of counters among ") {
            let f = color_or_phrase(among)?;
            return Some(Condition::Compare(
                Value::DistinctAmong(Among::CounterKinds, Box::new(Sel::All(f))),
                cmp,
                n,
            ));
        }
        // "there are ten or more creature cards total in all graveyards"
        if let Some(what) = tail.strip_suffix(" total in all graveyards") {
            let (f, _, t) = parse_object_phrase(what)?;
            if !end(t).is_empty() {
                return None;
            }
            return Some(Condition::Compare(
                Value::Count(Filter::and(vec![f, Filter::InZone(ZoneKind::Graveyard)])),
                cmp,
                n,
            ));
        }
        // "there are four or more creature cards exiled with ~"
        if let Some(what) = tail.strip_suffix(" exiled with ~") {
            let (f, _, t) = parse_object_phrase(what)?;
            if !end(t).is_empty() {
                return None;
            }
            return Some(Condition::Compare(
                Value::Count(exiled_with_source(f)),
                cmp,
                n,
            ));
        }
        return None;
    }
    // "three or more cards have been exiled with ~", "a card is exiled with ~"
    for suffix in [" have been exiled with ~", " are exiled with ~", " is exiled with ~"] {
        if let Some(r) = c.strip_suffix(suffix) {
            let (n, what) = parse_number(r)?;
            let (cmp, what) = match strip(what, "or more") {
                Some(w) => (Cmp::Ge, w),
                None if n.as_const() == Some(1) => (Cmp::Ge, what),
                None => return None,
            };
            let (f, _, t) = parse_object_phrase(what)?;
            if !end(t).is_empty() || matches!(n, Value::X) {
                return None;
            }
            return Some(Condition::Compare(
                Value::Count(exiled_with_source(f)),
                cmp,
                n,
            ));
        }
    }
    // "all lands on the battlefield are Islands", "all nonland permanents you control are
    // white": none of them isn't.
    if let Some(r) = c.strip_prefix("all ") {
        let (subj, pred) = r.split_once(" are ")?;
        let subj = subj.strip_suffix(" on the battlefield").unwrap_or(subj);
        let (f, plural, t) = parse_object_phrase(subj)?;
        if !plural || !end(t).is_empty() {
            return None;
        }
        let p = match Color::from_word(pred) {
            Some(col) => Filter::Color(col),
            None => {
                let (p, plural, t) = parse_object_phrase(pred)?;
                if !plural || !end(t).is_empty() {
                    return None;
                }
                p
            }
        };
        return Some(Condition::Not(Box::new(Condition::Exists(Filter::and(
            vec![f, Filter::not(p)],
        )))));
    }
    // "another creature is on the battlefield", "there's another creature on the
    // battlefield".
    if let Some(r) = c
        .strip_prefix("another ")
        .and_then(|r| r.strip_suffix(" is on the battlefield"))
        .or_else(|| {
            c.strip_prefix("there's another ")
                .and_then(|r| r.strip_suffix(" on the battlefield"))
        })
    {
        let f = color_or_phrase(r)?;
        return Some(Condition::Exists(Filter::and(vec![
            f,
            Filter::Other,
            Filter::InZone(ZoneKind::Battlefield),
        ])));
    }
    // "your team controls another Warrior" (CR 102.4: you and/or your teammates).
    if let Some(r) = c.strip_prefix("your team controls ") {
        let (other, r) = match r.strip_prefix("another ") {
            Some(x) => (true, x),
            None => (false, r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?),
        };
        let mut v = vec![
            color_or_phrase(r)?,
            Filter::Or(vec![
                Filter::ControlledBy(PlayerRel::You),
                Filter::ControlledBy(PlayerRel::Teammate),
            ]),
        ];
        if other {
            v.push(Filter::Other);
        }
        return Some(Condition::Exists(Filter::and(v)));
    }
    each_kind_condition(c)
}

/// Which number [`group_condition`] totals.
enum Stat_ {
    Toughness,
    Power,
}

/// "you control a permanent of each color", "you control a land of each basic land type
/// and a creature of each color": one of each (an object may count for several).
fn each_kind_condition(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("you control ")?;
    let mut parts = Vec::new();
    for part in r.split(" and ") {
        let part = part.strip_prefix("a ").or_else(|| part.strip_prefix("an "))?;
        let (noun, kind) = part.split_once(" of each ")?;
        let f = color_or_phrase(noun)?.you_control();
        match kind {
            "color" => {
                for col in Color::ALL {
                    parts.push(Condition::Exists(Filter::and(vec![
                        f.clone(),
                        Filter::Color(col),
                    ])));
                }
            }
            "basic land type" => {
                for t in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
                    parts.push(Condition::Exists(Filter::and(vec![
                        f.clone(),
                        Filter::Subtype(Subtype::from(t)),
                    ])));
                }
            }
            _ => return None,
        }
    }
    Some(Condition::And(parts))
}

/// "~ is in the command zone", "~ is the only creature card in your graveyard".
fn source_condition(c: &str) -> Option<Condition> {
    match c {
        "~ is in the command zone" => {
            return Some(Condition::SelMatches(
                Sel::This,
                Filter::InZone(ZoneKind::Command),
            ))
        }
        "~ is the only creature card in your graveyard" => {
            let gy = Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ]);
            return Some(Condition::And(vec![
                Condition::SelMatches(Sel::This, gy.clone()),
                Condition::Compare(
                    Value::Count(Filter::and(vec![
                        Filter::Type(CardType::Creature),
                        Filter::Card,
                        gy,
                    ])),
                    Cmp::Eq,
                    Value::c(1),
                ),
            ]));
        }
        _ => {}
    }
    None
}

/// The zone a triggered ability functions from when its intervening "if" clause requires
/// its source to be there: "At the beginning of your upkeep, if ~ is in the command zone,
/// ..." (CR 113.6b: an ability that states which zones it functions in).
pub(crate) fn required_source_zone(c: &Condition) -> Option<FunctionZone> {
    match c {
        Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Command)) => {
            Some(FunctionZone::Command)
        }
        Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Exile)) => {
            Some(FunctionZone::Exile)
        }
        Condition::And(v) => v.iter().find_map(required_source_zone),
        _ => None,
    }
}

fn state_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    player_condition(c)
        .or_else(|| your_zones_condition(c))
        .or_else(|| attacking_condition(c))
        .or_else(|| group_condition(c))
        .or_else(|| source_condition(c))
}

inventory::submit! { ConditionPattern { name: "state conditions: players, zones, attacks, groups, the source", priority: 400, parse: state_condition } }

/// "the player with the most life", "the player who has the most cards in hand", "the
/// player who controls the most Wizards": each player with the greatest number (the one
/// player an intervening "if a player has more ... than each other player" makes it).
fn most_player(s: &str) -> Option<(PlayerRef, &str)> {
    let (stat, rest) = if let Some(r) = s.strip_prefix("the player with the most life ") {
        (Stat::Life, r)
    } else if let Some(r) = s.strip_prefix("the player who has the most cards in hand ") {
        (Stat::Hand, r)
    } else if let Some(r) = s.strip_prefix("the player who controls the most ") {
        let (noun, rest) = r.split_once(' ')?;
        (Stat::Controls(color_or_phrase(noun)?), rest)
    } else {
        return None;
    };
    let most = Value::OverPlayers(
        AggOp::Max,
        PlayerFilter::Any,
        Box::new(stat.of(PlayerRef::Iterated)),
    );
    Some((PlayerRef::Each(stat.filter(Cmp::Eq, most)), rest))
}

/// "[the player with the most life] gains control of ~": that player performs the
/// instruction ("you" in it is that player).
fn most_player_does(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let (who, rest) = most_player(end(l))?;
    let (verb, tail) = split_word(rest);
    let verb = match verb {
        "gains" => "gain",
        _ => return None,
    };
    let e = crate::oracle::effects::parse_clause(&format!("{verb} {tail}"), b)?;
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(e),
    })
}

inventory::submit! { super::EffectPattern { name: "state conditions: the player with the most [stat] [does]", priority: 400, parse: most_player_does } }
