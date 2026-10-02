//! Permissions to play cards (CR 601.2, 601.3, 305.1), as one grammar: WHO may play or
//! cast WHAT, FROM WHERE, FOR HOW LONG, HOW OFTEN, and on WHICH TERMS.
//!
//! * Who: "you", "its owner", "the exiled card's owner".
//! * What: cards the text just moved ("that card", "those cards", "them", "the exiled
//!   card", "cards exiled this way"), some of them ("one of those cards", "up to two of
//!   them", "a spell from among them", "red spells from among those cards"), the card
//!   itself ("~"), or cards with qualities ("instant and sorcery spells", "a creature
//!   spell", "lands", "Zombie spells") in a zone.
//! * From where: "from your graveyard", "from exile", "from the top of your library",
//!   "from your hand", "from among cards (you own) exiled with ~" (CR 607.2a).
//! * For how long: "this turn", "until end of turn", "until the end of your next turn",
//!   "until your next turn", "until your next end step", "for as long as it remains
//!   exiled" (the permission is for that object, CR 400.7), "for as long as you control
//!   ~", "for as long as ~ remains on the battlefield", "until you exile another card with
//!   ~", "during your next turn" — or, without a duration, as the effect resolves
//!   (CR 608.2g).
//! * How often: "once each turn", "once during each of your turns", "a spell from among
//!   them" (one card), "up to two of those cards".
//! * Terms: "without paying its mana cost" (CR 118.9), "by [cost] in addition to paying
//!   its other costs" (CR 601.2b, 601.2f), "as though it had flash" (CR 601.3b), "and you
//!   may spend mana as though it were mana of any color to cast those spells" (CR 609.4b),
//!   "and mana of any type can be spent to cast it" (CR 118.14).
//!
//! The parsed phrase ([`Perm`]) becomes an effect's permission for those cards
//! (`Effect::GrantPlayPermission` with `PlayTerms`, `permissions.rs`), a cast during
//! resolution (`Effect::CastCard`), a resolved effect's permission to play cards with
//! qualities from a zone for a while (`PlayerModification::PlayPermission`), or a static
//! ability (`StaticEffect::PlayPermission`). A phrase the grammar reads only in part isn't
//! understood.

use super::{AbilityPattern, EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::kw::once_each_turn_cast::once_unused;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::types::CardType;

/// Who gets the permission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Who {
    #[default]
    You,
    /// "its owner", "the exiled card's owner", "their owner".
    OwnerOfIt,
}

/// What the permission is for.
#[derive(Clone, Debug, Default)]
pub enum Obj {
    /// The cards an earlier instruction moved ("that card", "those cards", "them"):
    /// `limit` of them ("one of those cards", "a spell from among them").
    Referent { limit: Option<u32> },
    /// The card itself ("~").
    #[default]
    SelfCard,
    /// Cards with qualities ("instant and sorcery spells", "lands", "a creature spell");
    /// `single`: "a creature spell" (one card).
    Class { what: Filter, single: bool },
    /// A target card ("target instant or sorcery card from your graveyard"): the phrase.
    Target { phrase: String },
}

/// Where the cards are played from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum From {
    Graveyard,
    Exile,
    LibraryTop,
    Hand,
    /// "from among cards exiled with ~" (CR 607.2a); `owned`: "cards you own exiled with ~".
    Linked {
        owned: bool,
    },
    /// "from among them", "from among those cards": the cards an earlier instruction moved.
    AmongReferent,
}

/// A parsed permission phrase.
#[derive(Clone, Debug, Default)]
pub struct Perm {
    pub who: Who,
    pub lands: bool,
    pub spells: bool,
    pub obj: Obj,
    pub from: Option<From>,
    pub duration: Option<Duration>,
    pub terms: PlayTerms,
    pub free: bool,
    /// "Once each turn", "once during each of your turns".
    pub once_each_turn: bool,
    /// "During your turn", "during each of your turns".
    pub your_turn: bool,
    /// "You may look at and play that card".
    pub look: bool,
    /// "... if [condition]": a condition on playing the card (or, "if it's an instant or
    /// sorcery spell", a quality of the spell).
    pub cond_text: Option<String>,
    /// "from among cards you own in exile with dream counters on them": which cards in the
    /// zone.
    pub zone_filter: Option<Filter>,
    /// "from your hand or the top of your library", "from your graveyard or from exile":
    /// a second zone.
    pub also_from: Option<From>,
    /// "You may cast any number of spells from among them".
    pub any_number: bool,
    /// The cards are named "the exiled card(s)" (in a static ability, the cards exiled
    /// with the source, CR 607.2a).
    pub exiled_named: bool,
    /// "up to two sorcery spells": how many cards of the class (cast as the effect
    /// resolves).
    pub count: Option<u32>,
}

impl Perm {
    /// The terms with the condition "if [condition]" read: a quality of the spell ("if
    /// it's an instant or sorcery spell", CR 601.3e) or a condition on playing it.
    fn terms_with_condition(&self, ctx: &CompileContext) -> Option<PlayTerms> {
        let mut terms = self.terms.clone();
        let Some(c) = &self.cond_text else {
            return Some(terms);
        };
        if let Some(q) = c
            .strip_prefix("it's ")
            .filter(|q| q.ends_with(" spell") || q.ends_with(" spells"))
        {
            let (f, _, rest) = class_phrase(q)?;
            if !rest.trim().is_empty() {
                return None;
            }
            terms.what = Some(match terms.what.take() {
                Some(w) => Filter::and(vec![w, f]),
                None => f,
            });
            return Some(terms);
        }
        // Another quality of the card ("if it shares a card type with a card exiled with
        // ~") isn't a condition on the game: not understood.
        if ["it ", "its ", "that card ", "that spell ", "the spell "]
            .iter()
            .any(|w| c.starts_with(w))
        {
            return None;
        }
        let cond = crate::oracle::statics::parse_condition(c, ctx)?;
        terms.condition = Some(match terms.condition.take() {
            Some(w) => Condition::And(vec![w, cond]),
            None => cond,
        });
        Some(terms)
    }
}

/// Leading phrases: durations, turn limits, conditions on turns.
fn strip_lead(s: &str, p: &mut Perm) -> Option<String> {
    let mut s = s.to_string();
    loop {
        let before = s.len();
        // "During any turn you attacked with ~, you may play that card": a condition on
        // playing it ("you attacked with ~ this turn").
        // The permission lasts while the card is exiled (CR 400.7).
        if let Some(r) = s.strip_prefix("during any turn ") {
            let (c, rest) = r.split_once(", ")?;
            if p.cond_text.is_some() || !set_duration(p, Duration::Permanent) {
                return None;
            }
            p.cond_text = Some(match c {
                // Its source attacked, even if it has left combat or the battlefield since.
                "you attacked with ~" => "~ attacked this turn".to_string(),
                _ => format!("{c} this turn"),
            });
            s = rest.to_string();
            continue;
        }
        for (lead, f) in LEADS {
            if let Some(r) = s.strip_prefix(lead) {
                if !f(p) {
                    return None;
                }
                s = r.to_string();
                break;
            }
        }
        if s.len() == before {
            return Some(s);
        }
    }
}

type Lead = (&'static str, fn(&mut Perm) -> bool);

fn set_duration(p: &mut Perm, d: Duration) -> bool {
    if p.duration.is_some() {
        return false;
    }
    p.duration = Some(d);
    true
}

const LEADS: &[Lead] = &[
    ("until end of turn, ", |p| {
        set_duration(p, Duration::EndOfTurn)
    }),
    ("until the end of your next turn, ", |p| {
        set_duration(p, Duration::UntilEndOfYourNextTurn)
    }),
    ("until your next turn, ", |p| {
        set_duration(p, Duration::UntilYourNextTurn)
    }),
    ("until your next end step, ", |p| {
        set_duration(p, Duration::UntilYourNextStep(TriggerStep::End))
    }),
    ("until the beginning of your next upkeep, ", |p| {
        set_duration(p, Duration::UntilYourNextStep(TriggerStep::Upkeep))
    }),
    ("for as long as that card remains exiled, ", |p| {
        set_duration(p, Duration::Permanent)
    }),
    ("for as long as it remains exiled, ", |p| {
        set_duration(p, Duration::Permanent)
    }),
    ("for as long as those cards remain exiled, ", |p| {
        set_duration(p, Duration::Permanent)
    }),
    ("for as long as they remain exiled, ", |p| {
        set_duration(p, Duration::Permanent)
    }),
    // Not during the turn the permission is given, then until that turn ends.
    ("during your next turn, ", |p| {
        p.terms.later_turn = true;
        p.your_turn = true;
        set_duration(p, Duration::UntilEndOfYourNextTurn)
    }),
    ("during your turn, ", |p| {
        p.your_turn = true;
        true
    }),
    ("during each of your turns, ", |p| {
        p.your_turn = true;
        true
    }),
    ("once during each of your turns, ", |p| {
        p.your_turn = true;
        p.once_each_turn = true;
        true
    }),
    ("once each turn, ", |p| {
        p.once_each_turn = true;
        true
    }),
    ("during each opponent's end step, ", |p| {
        if p.terms.condition.is_some() {
            return false;
        }
        p.terms.condition = Some(Condition::And(vec![
            Condition::NotYourTurn,
            Condition::Phase(PhaseCond::EndStep),
        ]));
        true
    }),
];

/// "you may", "its owner may", "the exiled card's owner may".
fn strip_subject(s: &str) -> Option<(Who, &str)> {
    for (w, who) in [
        ("you may ", Who::You),
        ("its owner may ", Who::OwnerOfIt),
        ("their owner may ", Who::OwnerOfIt),
        ("the exiled card's owner may ", Who::OwnerOfIt),
    ] {
        if let Some(r) = s.strip_prefix(w) {
            return Some((who, r));
        }
    }
    None
}

/// The phrases naming the cards an earlier instruction moved, with how many of them.
fn referent(s: &str) -> Option<(Option<u32>, &str)> {
    const WHOLE: &[&str] = &[
        "the cards exiled this way",
        "cards exiled this way",
        "those exiled cards",
        "the exiled cards",
        "the exiled card",
        "those cards",
        "that card",
        "them",
        "it",
    ];
    let word_end = |r: &str| r.is_empty() || r.starts_with(' ') || r.starts_with(',');
    for w in WHOLE {
        if let Some(r) = s.strip_prefix(w) {
            if word_end(r) {
                return Some((None, r));
            }
        }
    }
    // "one of those cards", "up to two of them".
    let (n, r) = if let Some(r) = s.strip_prefix("one of ") {
        (1, r)
    } else {
        let r = s.strip_prefix("up to ")?;
        let (n, r) = parse_number(r)?;
        (n.as_const()? as u32, r.trim_start().strip_prefix("of ")?)
    };
    for w in [
        "those cards",
        // "one of those two cards" (Invasion of Alara: the two cards found).
        "those two cards",
        "them",
        "the exiled cards",
        "those exiled cards",
    ] {
        if let Some(r2) = r.strip_prefix(w) {
            if word_end(r2) {
                return Some((Some(n), r2));
            }
        }
    }
    None
}

/// "from among [referent]".
fn among_referent(s: &str) -> Option<&str> {
    let r = s.strip_prefix(" from among ")?;
    for w in [
        "the cards exiled this way",
        "cards exiled this way",
        "those exiled cards",
        "the exiled cards",
        "those cards",
        "them",
    ] {
        if let Some(r2) = r.strip_prefix(w) {
            if r2.is_empty() || r2.starts_with(' ') || r2.starts_with(',') {
                return Some(r2);
            }
        }
    }
    None
}

/// A class of cards as a phrase: "instant and sorcery spells", "a creature spell with mana
/// value 3 or less", "lands", "historic lands", "cards", "Forests". Returns the filter,
/// whether it names one card ("a creature spell"), whether it names spells (rather than
/// cards or lands), and the rest.
fn class_phrase(s: &str) -> Option<(Filter, bool, &str)> {
    let (single, body) = match s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        Some(r) => (true, r),
        None => (false, s),
    };
    // The phrase ends where the zone (or a qualifier the object phrase doesn't take)
    // begins.
    let cut = [
        " from ",
        " without paying",
        " this turn",
        " until ",
        " for as long as ",
        " as though ",
        " by ",
        ", and ",
        " and you may ",
        " and mana of ",
        " exiled with ",
        " you own exiled with ",
        " you own in exile",
        " in exile",
    ]
    .iter()
    .filter_map(|c| body.find(c))
    .min()
    .unwrap_or(body.len());
    let (head, rest) = body.split_at(cut);
    let words: Vec<&str> = head.split(' ').collect();
    let spell_word = words.iter().position(|w| *w == "spells" || *w == "spell")?;
    let card_phrase: Vec<&str> = words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if i == spell_word {
                if *w == "spells" {
                    "cards"
                } else {
                    "card"
                }
            } else {
                w
            }
        })
        .collect();
    let card_phrase = card_phrase.join(" ");
    let f = object_filter(&card_phrase)?;
    Some((f, single, rest))
}

/// A card phrase ("red cards", "creature card with mana value 3 or less") as a filter,
/// with nothing left over.
fn object_filter(s: &str) -> Option<Filter> {
    if s == "cards" || s == "card" {
        return Some(Filter::Card);
    }
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(f)
}

/// "lands", "a land", "historic lands", "Forests", "Desert lands".
fn land_phrase(s: &str) -> Option<(Filter, bool, &str)> {
    let (single, body) = match s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        Some(r) => (true, r),
        None => (false, s),
    };
    let cut = [" from ", " and cast ", " or cast ", " this turn", " until "]
        .iter()
        .filter_map(|c| body.find(c))
        .min()
        .unwrap_or(body.len());
    let (head, rest) = body.split_at(cut);
    let f = object_filter(head)?;
    if !mentions_land(&f) {
        return None;
    }
    Some((f, single, rest))
}

fn mentions_land(f: &Filter) -> bool {
    match f {
        Filter::Type(CardType::Land) => true,
        Filter::Subtype(s) => crate::types::land_types().iter().any(|t| t == s.as_str()),
        Filter::And(v) => v.iter().any(mentions_land),
        _ => false,
    }
}

/// "cards exiled with ~" as an object ("you may play cards exiled with ~", "you may cast
/// an instant or sorcery card exiled with ~").
fn linked_cards(s: &str) -> Option<(Filter, bool, bool, &str)> {
    let (single, body) = match s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (head, owned, rest) = if let Some((h, r)) = body.split_once(" you own exiled with ~") {
        (h, true, r)
    } else {
        let (h, r) = body.split_once(" exiled with ~")?;
        (h, false, r)
    };
    let f = object_filter(head)?;
    Some((f, single, owned, rest))
}

/// The zone after the object.
fn strip_from<'a>(s: &'a str, p: &mut Perm) -> Option<(From, &'a str)> {
    if let Some(r) = among_referent(s) {
        return Some((From::AmongReferent, r));
    }
    // "from among cards you own in exile with dream counters on them", "from among cards
    // in exile with page counters on them".
    if let Some((d, rest)) = s
        .strip_prefix(" from among cards ")
        .and_then(|r| r.split_once(" on them"))
        .filter(|(d, _)| d.contains(" in exile ") || d.starts_with("in exile "))
    {
        let d = d.replacen("in exile ", "", 1);
        let desc = format!("cards {} on them", d.trim());
        let f = object_filter(&desc)?;
        p.zone_filter = Some(f);
        return Some((From::Exile, rest));
    }
    for (p, z) in [
        (" from your graveyard", From::Graveyard),
        (" from exile", From::Exile),
        (" from the top of your library", From::LibraryTop),
        (" from your hand", From::Hand),
        (
            " from among cards you own exiled with ~",
            From::Linked { owned: true },
        ),
        (
            " from among cards exiled with ~",
            From::Linked { owned: false },
        ),
        (
            " from among the cards exiled with ~",
            From::Linked { owned: false },
        ),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
                return Some((z, r));
            }
        }
    }
    None
}

/// "by discarding a card", "by foraging", "by paying 2 life and exiling a card from your
/// graveyard": the cost, in the imperative.
fn gerund_cost(s: &str) -> Option<Cost> {
    let mut parts = Vec::new();
    for part in s.split(" and ") {
        let part = part.trim();
        let (verb, rest) = part.split_once(' ').unwrap_or((part, ""));
        let imp = match verb {
            "paying" if rest.starts_with('{') => {
                parts.push(rest.to_string());
                continue;
            }
            "paying" => "pay",
            "exiling" => "exile",
            "discarding" => "discard",
            "sacrificing" => "sacrifice",
            "removing" => "remove",
            "tapping" => "tap",
            "returning" => "return",
            "foraging" => "forage",
            "collecting" => "collect",
            "milling" => "mill",
            "revealing" => "reveal",
            _ => return None,
        };
        parts.push(if rest.is_empty() {
            imp.to_string()
        } else {
            format!("{imp} {rest}")
        });
    }
    let (cost, loyalty) = crate::oracle::costs::parse_cost(&parts.join(", "))?;
    (!loyalty).then_some(cost)
}

/// Trailing qualifiers, in any order: durations, "without paying its mana cost", "as
/// though it had flash", "by [cost] in addition to paying its other costs".
fn strip_tails(mut s: &str, p: &mut Perm) -> Option<()> {
    loop {
        let t = s.trim_end();
        if t.is_empty() {
            return Some(());
        }
        let mut done = false;
        // "you may play cards exiled with ~ as long as you attacked this turn": the
        // condition's own "this turn", not the permission's duration.
        let cond_tail = t
            .rsplit_once(" if ")
            .or_else(|| t.rsplit_once(" as long as "))
            .filter(|(h, c)| !h.ends_with(" for") && c.ends_with(" this turn"));
        if let Some((head, cond)) = cond_tail {
            if p.cond_text.is_some() {
                return None;
            }
            p.cond_text = Some(cond.to_string());
            s = head;
            continue;
        }
        for (suffix, f) in TAILS {
            if let Some(r) = t.strip_suffix(suffix) {
                if !f(p) {
                    return None;
                }
                s = r;
                done = true;
                break;
            }
        }
        if done {
            continue;
        }
        // "by [cost] in addition to paying its other costs"
        for tail in [
            " in addition to paying its other costs",
            " in addition to paying their other costs",
        ] {
            if let Some(r) = t.strip_suffix(tail) {
                let (head, cost) = r.rsplit_once(" by ")?;
                if p.terms.extra_cost.is_some() {
                    return None;
                }
                p.terms.extra_cost = Some(gerund_cost(cost)?);
                s = head;
                done = true;
                break;
            }
        }
        // "you may play it if you control a Kavu", "you may cast ~ as though it had flash
        // if you control a Human", "you may cast ~ from exile as long as you control a ~
        // planeswalker": a condition on playing it.
        if let Some((head, cond)) = t
            .rsplit_once(" if ")
            .or_else(|| t.rsplit_once(" as long as "))
            .filter(|(h, _)| !h.ends_with(" for"))
        {
            if p.cond_text.is_some() || cond.is_empty() {
                return None;
            }
            p.cond_text = Some(cond.to_string());
            s = head;
            continue;
        }
        if !done {
            return if t.is_empty() { Some(()) } else { None };
        }
    }
}

type Tail = (&'static str, fn(&mut Perm) -> bool);

const TAILS: &[Tail] = &[
    (" this turn", |p| set_duration(p, Duration::EndOfTurn)),
    (" until end of turn", |p| {
        set_duration(p, Duration::EndOfTurn)
    }),
    (" until the end of your next turn", |p| {
        set_duration(p, Duration::UntilEndOfYourNextTurn)
    }),
    (" until your next turn", |p| {
        set_duration(p, Duration::UntilYourNextTurn)
    }),
    (" until your next end step", |p| {
        set_duration(p, Duration::UntilYourNextStep(TriggerStep::End))
    }),
    (" for as long as it remains exiled", |p| {
        set_duration(p, Duration::Permanent)
    }),
    (" for as long as they remain exiled", |p| {
        set_duration(p, Duration::Permanent)
    }),
    (" for as long as that card remains exiled", |p| {
        set_duration(p, Duration::Permanent)
    }),
    (" for as long as you control ~", |p| {
        set_duration(p, Duration::WhileYouControlSource)
    }),
    (" for as long as ~ remains on the battlefield", |p| {
        set_duration(p, Duration::WhileSourceOnBattlefield)
    }),
    // The permission is for that object, until the source gives another such one.
    (" until you exile another card with ~", |p| {
        p.terms.until_another = true;
        set_duration(p, Duration::Permanent)
    }),
    (" without paying its mana cost", |p| {
        let was = p.free;
        p.free = true;
        !was
    }),
    (" without paying their mana costs", |p| {
        let was = p.free;
        p.free = true;
        !was
    }),
    (" as though it had flash", |p| {
        let was = p.terms.flash;
        p.terms.flash = true;
        !was
    }),
    (" as though they had flash", |p| {
        let was = p.terms.flash;
        p.terms.flash = true;
        !was
    }),
    (" and as though it had flash", |p| {
        let was = p.terms.flash;
        p.terms.flash = true;
        !was
    }),
];

/// ", and you may spend mana as though it were mana of any color to cast those spells",
/// ", and mana of any type can be spent to cast it" at the end.
fn strip_spend(s: &str, p: &mut Perm) -> String {
    for lead in [", and ", " and "] {
        for (clause, any_type) in [
            (
                "you may spend mana as though it were mana of any color to cast ",
                false,
            ),
            ("mana of any type can be spent to cast ", true),
            (
                "you may spend mana as though it were mana of any type to cast ",
                true,
            ),
        ] {
            for obj in ["it", "that spell", "those spells", "them", "that card"] {
                let suffix = format!("{lead}{clause}{obj}");
                if let Some(r) = s.strip_suffix(&suffix) {
                    if any_type {
                        p.terms.spend_any_type = true;
                    } else {
                        p.terms.spend_as_any_color = true;
                    }
                    return r.to_string();
                }
            }
        }
    }
    // "..., and you may cast them as though they had flash" (Azula, Cunning Usurper).
    for suffix in [
        " and you may cast them as though they had flash",
        ", and you may cast them as though they had flash",
    ] {
        if let Some(r) = s.strip_suffix(suffix) {
            p.terms.flash = true;
            return r.to_string();
        }
    }
    s.to_string()
}

/// Parses a permission phrase (lowercase, without the final period).
pub fn parse(l: &str) -> Option<Perm> {
    let mut p = Perm::default();
    let s = strip_lead(end(l).trim(), &mut p)?;
    // "Cast any number of red instant and/or sorcery cards from your graveyard without
    // paying their mana costs.": an instruction (the player chooses which, if any).
    let (who, s) = match s.strip_prefix("cast any number of ") {
        Some(_) => (Who::You, s.as_str()),
        None => strip_subject(&s)?,
    };
    p.who = who;
    let s = strip_spend(s, &mut p);
    let mut r: &str = &s;
    // The verb, and what it takes.
    if let Some(x) = r.strip_prefix("look at and play ") {
        p.look = true;
        p.lands = true;
        p.spells = true;
        r = x;
    } else if let Some(x) = r.strip_prefix("play ") {
        p.lands = true;
        p.spells = true;
        r = x;
    } else if let Some(x) = r.strip_prefix("cast ") {
        p.spells = true;
        r = x;
    } else {
        return None;
    }
    if let Some(x) = r.strip_prefix("any number of ") {
        p.any_number = true;
        r = x;
    }
    let rest: &str;
    if let Some(x) = r.strip_prefix("the top card of your library") {
        // "you may play the top card of your library" (The Lunar Whale).
        p.obj = Obj::Class {
            what: Filter::Card,
            single: false,
        };
        p.from = Some(From::LibraryTop);
        rest = x;
    } else if p.any_number && p.spells && !p.lands && !r.contains(" spell") {
        // "cast any number of red instant and/or sorcery cards from your graveyard",
        // "cast any number of cards exiled with ~".
        if let Some((f, single, owned, x)) = linked_cards(r) {
            p.obj = Obj::Class { what: f, single };
            p.from = Some(From::Linked { owned });
            rest = x;
        } else {
            let (head, _) = r.split_once(" from ")?;
            let f = object_filter(head)?;
            p.obj = Obj::Class {
                what: Filter::and(vec![Filter::Not(Box::new(Filter::Type(CardType::Land))), f]),
                single: false,
            };
            rest = &r[head.len()..];
        }
    } else if let Some((limit, x)) = referent(r) {
        p.exiled_named = r.starts_with("the exiled card");
        p.obj = Obj::Referent { limit };
        rest = x;
    } else if let Some((_, x)) = r
        .starts_with("target ")
        .then(|| crate::oracle::phrases::parse_target(r))
        .flatten()
    {
        // "cast target instant or sorcery card from your graveyard this turn".
        p.obj = Obj::Target {
            phrase: r[..r.len() - x.len()].trim().to_string(),
        };
        rest = x;
    } else if let Some(x) = r
        .strip_prefix("~")
        .filter(|x| x.is_empty() || x.starts_with(' '))
    {
        p.obj = Obj::SelfCard;
        rest = x;
    } else if p.lands && p.spells && {
        let head = r.split(" from ").next().unwrap_or(r);
        head.contains(" and cast ") || head.contains(" or cast ")
    } {
        // "play lands and cast [spells] ...", "play historic lands and cast historic
        // spells ...", "play a land or cast a spell ...".
        let (lf, lsingle, x) = land_phrase(r)?;
        let (or, x) = match x.strip_prefix(" and cast ") {
            Some(x) => (false, x),
            None => (true, x.strip_prefix(" or cast ")?),
        };
        let (sf, ssingle, x) = class_phrase(x)?;
        if lsingle != ssingle || lsingle != or {
            return None;
        }
        // "from among [cards]": the qualities of the lands and the spells.
        let land = Filter::and(vec![lf, Filter::Type(CardType::Land)]);
        let spell = Filter::and(vec![
            Filter::Not(Box::new(Filter::Type(CardType::Land))),
            sf,
        ]);
        p.obj = Obj::Class {
            what: Filter::Or(vec![land, spell]),
            single: lsingle,
        };
        rest = x;
    } else if let Some((f, single, owned, x)) = linked_cards(r) {
        p.obj = Obj::Class { what: f, single };
        p.from = Some(From::Linked { owned });
        rest = x;
    } else if p.spells && !p.lands {
        // "up to two sorcery spells with mana value 3 or less from among them".
        let r = match r.strip_prefix("up to ").and_then(parse_number) {
            Some((n, x)) => {
                p.count = Some(n.as_const()? as u32);
                x.trim_start()
            }
            None => r,
        };
        let (f, single, x) = class_phrase(r)?;
        if single && p.count.is_some() {
            return None;
        }
        p.obj = Obj::Class {
            what: Filter::and(vec![Filter::Not(Box::new(Filter::Type(CardType::Land))), f]),
            single,
        };
        rest = x;
    } else if let Some((f, single, x)) = land_phrase(r) {
        // "play lands from your graveyard", "play Forests from your graveyard".
        p.spells = false;
        p.obj = Obj::Class { what: f, single };
        rest = x;
    } else {
        // "play cards from ...", "play a card from ...".
        let (single, body) = match r.strip_prefix("a ") {
            Some(b) => (true, b),
            None => (false, r),
        };
        let (head, _) = body.split_once(" from ")?;
        let f = object_filter(head)?;
        p.obj = Obj::Class { what: f, single };
        rest = &body[head.len()..];
    }
    let mut rest = rest;
    if p.from.is_none() {
        if let Some((z, x)) = strip_from(rest, &mut p) {
            p.from = Some(z);
            rest = x;
            // "from your graveyard or from exile", "from your hand or the top of your
            // library".
            for (w, z2) in [
                (" or from exile", From::Exile),
                (" or from your graveyard", From::Graveyard),
                (" or the top of your library", From::LibraryTop),
                (" or from the top of your library", From::LibraryTop),
            ] {
                if let Some(x2) = rest.strip_prefix(w) {
                    p.also_from = Some(z2);
                    rest = x2;
                    break;
                }
            }
        }
    } else if let Some(x) = rest.strip_prefix(" from exile") {
        rest = x;
    }
    // "You may play that card from exile this turn" (the cards are in exile).
    if matches!(p.obj, Obj::Referent { .. }) && p.from.is_none() {
        if let Some(x) = rest.strip_prefix(" from exile") {
            rest = x;
        }
    }
    strip_tails(rest, &mut p)?;
    Some(p)
}

/// How the text's earlier instructions left the cards "that card" refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// The last instruction that set "it" moved cards to this zone.
    Cards(ZoneKind),
    /// No instruction here sets "it".
    Unchanged,
    /// Something else.
    Other,
}

/// Whether the effect leaves the cards it moved in `vars::IT` (and where), as the last
/// instruction that sets it.
pub fn moved(e: &Effect) -> Moved {
    match e {
        Effect::Exile { face_down, .. } => {
            let _ = face_down;
            Moved::Cards(ZoneKind::Exile)
        }
        Effect::Mill { .. } => Moved::Cards(ZoneKind::Graveyard),
        // "Exile another target creature or artifact until ~ leaves the battlefield."
        Effect::ExileUntil { .. } => Moved::Cards(ZoneKind::Exile),
        Effect::Move { to, .. } => match to.zone {
            ZoneKind::Exile | ZoneKind::Graveyard | ZoneKind::Hand => Moved::Cards(to.zone),
            _ => Moved::Other,
        },
        // The cards found, moved (and the shuffle after doesn't change "it").
        Effect::Search { to, .. } => match to.zone {
            ZoneKind::Exile | ZoneKind::Graveyard => Moved::Cards(to.zone),
            _ => Moved::Other,
        },
        // "Exile cards from the top of your library until you exile a nonland card": the
        // card found.
        Effect::RevealUntil { found_to, .. } => match found_to.zone {
            ZoneKind::Exile | ZoneKind::Graveyard => Moved::Cards(found_to.zone),
            _ => Moved::Other,
        },
        Effect::Discard { .. } => Moved::Cards(ZoneKind::Graveyard),
        // "Look at the top four cards of your library." (nothing taken yet): the cards
        // looked at, still in the library.
        Effect::Dig {
            take: Value::Const(0),
            rest_to,
            ..
        } if super::card_flow_dig::is_in_place(rest_to) => Moved::Cards(ZoneKind::Library),
        // "Choose one of them" (`card_flow_choose_one_of_them`): the chosen card is
        // stored as "it".
        Effect::ForEach {
            sel:
                Sel::Choose {
                    store: Some(v),
                    filter,
                    ..
                },
            effect,
            ..
        } if *v == vars::IT && matches!(**effect, Effect::Noop) => match filter {
            Filter::In(s) if matches!(**s, Sel::Var(x) if x == vars::IT) => Moved::Unchanged,
            _ => Moved::Other,
        },
        Effect::Seq(v) => {
            for x in v.iter().rev() {
                match moved(x) {
                    Moved::Unchanged => continue,
                    m => return m,
                }
            }
            Moved::Unchanged
        }
        Effect::May { effect, .. } => moved(effect),
        Effect::If {
            then, otherwise, ..
        } => match (moved(then), moved(otherwise)) {
            (a, Moved::Unchanged) => a,
            (Moved::Unchanged, b) => b,
            (a, b) if a == b => a,
            _ => Moved::Other,
        },
        Effect::Noop
        | Effect::Modify { .. }
        | Effect::Shuffle { .. }
        | Effect::GainLife { .. }
        | Effect::LoseLife { .. }
        | Effect::DealDamage { .. }
        | Effect::AddMana { .. }
        | Effect::Draw { .. }
        | Effect::Scry { .. }
        | Effect::Surveil { .. }
        | Effect::AddPlayerCounters { .. }
        | Effect::CreateToken { .. }
        | Effect::GrantPlayPermission { .. }
        | Effect::WithPlayTerms { .. }
        | Effect::StoreValue { .. }
        | Effect::Store { .. } => Moved::Unchanged,
        // "You may look at those cards for as long as they remain exiled."
        Effect::Custom(n) if n.as_str() == crate::zones::MAY_LOOK_AT_EXILED => Moved::Unchanged,
        _ => Moved::Other,
    }
}

/// The zone the cards "that card" refers to are in, if the text's earlier instructions
/// moved them: `prev` (the previous sentence's effect, if known) and what "it" means.
fn referent_zone(prev: Option<&Effect>, b: &Builder) -> Option<ZoneKind> {
    let it_is_moved = matches!(&b.it, Sel::Var(v) if *v == vars::IT);
    match prev.map(moved) {
        Some(Moved::Cards(z)) => Some(z),
        // The cards an earlier sentence moved, which "it" still means.
        Some(Moved::Unchanged) | None if it_is_moved => Some(ZoneKind::Exile),
        _ => None,
    }
}

/// Whether a filter or cost mentions X (the X of the ability, which a permission lasting
/// beyond its resolution couldn't know).
fn mentions_x<T: serde::Serialize>(t: &T) -> bool {
    fn walk(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::String(s) => s == "X",
            serde_json::Value::Object(m) => m.values().any(walk),
            serde_json::Value::Array(a) => a.iter().any(walk),
            _ => false,
        }
    }
    serde_json::to_value(t).is_ok_and(|v| walk(&v))
}

/// The permission as an effect: for the cards an earlier instruction moved to `zone`
/// (`None`: no such cards), or for cards with qualities in a zone for a while.
pub fn to_effect(p: &Perm, zone: Option<ZoneKind>, ctx: &CompileContext) -> Option<Effect> {
    // A second zone is a static permission's; "any number of" is chosen as the effect
    // resolves.
    if p.once_each_turn
        || p.also_from.is_some()
        || ((p.any_number || p.count.is_some()) && p.duration.is_some())
    {
        return None;
    }
    let it = Sel::Var(vars::IT);
    let who = match p.who {
        Who::You => PlayerRef::You,
        Who::OwnerOfIt => PlayerRef::OwnerOf(Box::new(it.clone())),
    };
    let mut terms = p.terms_with_condition(ctx)?;
    // "Until your next turn, you may cast sorcery spells as though they had flash."
    if let (
        Obj::Class {
            what,
            single: false,
        },
        None,
        Some(d),
    ) = (&p.obj, p.from, &p.duration)
    {
        if terms.flash && flash_only(p, &terms) && p.who == Who::You {
            if !matches!(
                d,
                Duration::EndOfTurn
                    | Duration::UntilYourNextTurn
                    | Duration::UntilEndOfYourNextTurn
            ) || mentions_x(what)
            {
                return None;
            }
            return Some(Effect::AddPlayerEffect {
                who: PlayerRef::You,
                effect: PlayerModification::FlashPermission(what.clone()),
                duration: d.clone(),
            });
        }
    }
    if p.your_turn {
        terms.condition = Some(match terms.condition.take() {
            Some(c) => Condition::And(vec![Condition::YourTurn, c]),
            None => Condition::YourTurn,
        });
    }
    match &p.obj {
        Obj::Referent { limit } => {
            let zone = zone?;
            if p.from.is_some_and(|f| f != From::Exile) {
                return None;
            }
            referent_effect(p, who, it, zone, *limit, None, terms)
        }
        Obj::Class { what, single } if p.from == Some(From::AmongReferent) => {
            let zone = zone?;
            let limit = if *single { Some(1) } else { p.count };
            referent_effect(p, who, it, zone, limit, Some(what.clone()), terms)
        }
        Obj::Class { what, single } => {
            if p.who != Who::You || p.look {
                return None;
            }
            let Some(duration) = p.duration.clone() else {
                return class_cast_now(p, what.clone(), *single, &terms);
            };
            if !matches!(
                duration,
                Duration::EndOfTurn
                    | Duration::UntilEndOfYourNextTurn
                    | Duration::UntilYourNextTurn
                    | Duration::UntilYourNextStep(_)
            ) || terms.until_another
                || terms.later_turn
            {
                return None;
            }
            // One card: a single use, this turn.
            if *single {
                if !matches!(duration, Duration::EndOfTurn) {
                    return None;
                }
                terms.limit = Some(1);
            }
            let pp = class_permission(p, what.clone(), terms)?;
            if mentions_x(&pp) {
                return None;
            }
            Some(Effect::AddPlayerEffect {
                who: PlayerRef::You,
                effect: PlayerModification::PlayPermission(pp),
                duration,
            })
        }
        Obj::SelfCard | Obj::Target { .. } => None,
    }
}

/// Whether the phrase only lets spells be cast as though they had flash ("You may cast
/// noncreature spells as though they had flash", CR 601.3b): no zone, cost or other term.
fn flash_only(p: &Perm, terms: &PlayTerms) -> bool {
    p.spells
        && !p.lands
        && !p.free
        && !p.look
        && p.from.is_none()
        && terms.extra_cost.is_none()
        && !terms.spend_any_type
        && !terms.spend_as_any_color
        && terms.what.is_none()
        && terms.limit.is_none()
        && !terms.until_another
        && !terms.later_turn
}

/// "You may cast a spell with mana value X from among cards exiled with ~ without paying
/// its mana cost": a spell cast as the effect resolves (CR 608.2g), chosen among the
/// cards with those qualities as the spell each would become (CR 601.3e).
fn class_cast_now(p: &Perm, what: Filter, single: bool, terms: &PlayTerms) -> Option<Effect> {
    if p.lands
        || terms.flash
        || terms.extra_cost.is_some()
        || terms.spend_any_type
        || terms.spend_as_any_color
        || terms.condition.is_some()
    {
        return None;
    }
    let zone = match p.from? {
        From::Graveyard => Filter::and(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        From::Hand => Filter::and(vec![
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        From::Linked { owned } => {
            let mut v = vec![
                Filter::In(Box::new(Sel::Linked)),
                Filter::InZone(ZoneKind::Exile),
            ];
            if owned {
                v.push(Filter::OwnedBy(PlayerRel::You));
            }
            Filter::and(v)
        }
        // "from among cards you own in exile with dream counters on them".
        From::Exile => Filter::and(vec![
            Filter::InZone(ZoneKind::Exile),
            p.zone_filter.clone()?,
        ]),
        _ => return None,
    };
    if p.from != Some(From::Exile) && p.zone_filter.is_some() {
        return None;
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![zone, what]),
            count: Value::c(if single {
                1
            } else {
                p.count.unwrap_or(99) as i32
            }),
            up_to: true,
            store: None,
        },
        free: p.free,
        optional: false,
    })
}

/// The cards a spell is chosen among as the effect resolves ("you may cast a spell from
/// among them"), remembered before any is cast.
const AMONG: Var = vars::USER + 4601;

/// Whether the effect remembers the cards it chooses among in [`AMONG`].
fn stores_among(e: &Effect) -> bool {
    match e {
        Effect::Store { var, .. } => *var == AMONG,
        Effect::Seq(v) => v.iter().any(stores_among),
        _ => false,
    }
}

/// A permission for (some of) the cards an earlier instruction moved to `zone`.
fn referent_effect(
    p: &Perm,
    who: PlayerRef,
    it: Sel,
    zone: ZoneKind,
    limit: Option<u32>,
    quality: Option<Filter>,
    mut terms: PlayTerms,
) -> Option<Effect> {
    let cast_only = !p.lands;
    // Cards looked at in a library can be cast only as the effect resolves.
    if zone == ZoneKind::Library && p.duration.is_some() {
        return None;
    }
    match &p.duration {
        // As the effect resolves (CR 608.2g).
        None => {
            if p.look
                || terms.flash
                || terms.extra_cost.is_some()
                || terms.spend_any_type
                || terms.spend_as_any_color
                || terms.condition.is_some()
                || p.who != Who::You
            {
                return None;
            }
            let whole = limit.is_none() && quality.is_none();
            // The cards are remembered (for "put the rest ...", once some are cast).
            let among = if whole { it.clone() } else { Sel::Var(AMONG) };
            let mut parts = vec![Filter::In(Box::new(among)), Filter::InZone(zone)];
            if cast_only {
                parts.push(Filter::Not(Box::new(Filter::Type(CardType::Land))));
            }
            if let Some(q) = quality {
                parts.push(q);
            }
            // "You may cast it": each of them (a land card can't be cast, CR 305.9).
            let what = if whole {
                it.clone()
            } else {
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::and(parts),
                    count: Value::c(limit.unwrap_or(99) as i32),
                    up_to: true,
                    store: None,
                }
            };
            let chooses = matches!(what, Sel::Choose { .. });
            let cast = if cast_only {
                Effect::CastCard {
                    who,
                    what,
                    free: p.free,
                    optional: !chooses,
                }
            } else {
                Effect::PlayCard {
                    who,
                    what,
                    free: p.free,
                    optional: !chooses,
                }
            };
            Some(if whole {
                cast
            } else {
                Effect::seq(vec![
                    Effect::Store {
                        var: AMONG,
                        sel: it,
                    },
                    cast,
                ])
            })
        }
        Some(d) => {
            terms.spells_only = cast_only;
            terms.limit = limit;
            if let Some(q) = quality {
                terms.what = Some(q);
            }
            if mentions_x(&terms) {
                return None;
            }
            let grant = Effect::WithPlayTerms {
                terms,
                effect: Box::new(Effect::GrantPlayPermission {
                    who,
                    what: it,
                    duration: d.clone(),
                    free: p.free,
                }),
            };
            if p.look {
                // "You may look at and play that card ...": only the player who gets the
                // permission looks, and only while it's exiled (CR 406.3b).
                if p.who != Who::You || !matches!(d, Duration::Permanent) {
                    return None;
                }
                return Some(Effect::seq(vec![
                    Effect::Custom(crate::zones::MAY_LOOK_AT_EXILED.into()),
                    grant,
                ]));
            }
            Some(grant)
        }
    }
}

/// A permission to play cards with the qualities `what` from the phrase's zone.
fn class_permission(p: &Perm, what: Filter, terms: PlayTerms) -> Option<PlayPermission> {
    let from = p.from?;
    class_permission_from(p, from, what, terms)
}

/// A permission to play cards with the qualities `what` from the zone `from`.
fn class_permission_from(
    p: &Perm,
    from: From,
    what: Filter,
    terms: PlayTerms,
) -> Option<PlayPermission> {
    // "from among cards you own in exile with croak counters on them".
    let what = match (&p.zone_filter, from) {
        (Some(f), From::Exile) => Filter::and(vec![f.clone(), what]),
        (Some(_), _) => return None,
        (None, _) => what,
    };
    let (zone, top_only, what) = match from {
        From::Graveyard => (ZoneKind::Graveyard, false, what),
        From::Exile => (ZoneKind::Exile, false, what),
        From::LibraryTop => (ZoneKind::Library, true, what),
        From::Hand => (ZoneKind::Hand, false, what),
        From::Linked { owned } => {
            let mut v = vec![
                Filter::In(Box::new(Sel::Linked)),
                Filter::InZone(ZoneKind::Exile),
                what,
            ];
            if owned {
                v.push(Filter::OwnedBy(PlayerRel::You));
            }
            (ZoneKind::Exile, false, Filter::and(v))
        }
        From::AmongReferent => return None,
    };
    // Spells cast from the hand "without paying their mana costs" are offered that cost
    // by `casting.rs`; nothing else changes how they're cast from there.
    if zone == ZoneKind::Hand && (!p.free || terms.flash || terms.extra_cost.is_some()) {
        return None;
    }
    if !p.spells && (p.free || terms.flash || terms.extra_cost.is_some()) {
        return None;
    }
    Some(PlayPermission {
        who: PlayerRel::You,
        zone,
        top_only,
        what,
        lands: p.lands,
        spells: p.spells,
        cost: p.free.then(Cost::free),
        flash: false,
        terms,
    })
}

/// The permission as static abilities (`text` is the ability's text).
pub fn to_statics(p: &Perm, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if p.who != Who::You || p.duration.is_some() || p.look || p.any_number || p.count.is_some() {
        return None;
    }
    // "You may cast ~ as though it had flash by tapping three untapped creatures you
    // control with flying in addition to paying its other costs." (CR 601.3c): flash for
    // an additional cost, as "... if you pay {2} more to cast it".
    if let (Obj::SelfCard, None, true, Some(extra)) =
        (&p.obj, p.from, p.terms.flash, p.terms.extra_cost.as_ref())
    {
        let mut rest = p.terms.clone();
        rest.flash = false;
        rest.extra_cost = None;
        let plain = format!("{:?}", PlayTerms::default()) == format!("{:?}", rest);
        if !plain || p.free || p.cond_text.is_some() || p.your_turn || p.once_each_turn {
            return None;
        }
        let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            who: PlayerRel::You,
            change: CostChange::FlashForAdditionalCost(extra.clone()),
        }));
        s.zone = FunctionZone::Anywhere;
        return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
    }
    let mut terms = p.terms_with_condition(ctx)?;
    if terms.limit.is_some() || terms.until_another || terms.later_turn {
        return None;
    }
    // A quality the spell must have ("... if it's an instant or sorcery spell") is part of
    // what the static permission is for (its `what`, judged as the spell, CR 601.3e).
    let quality = terms.what.take();
    // A condition on playing the card ("... if you control a Human") is the static
    // ability's condition.
    let play_condition = terms.condition.take();
    let mut conds = Vec::new();
    if p.your_turn {
        conds.push(Condition::YourTurn);
    }
    if p.once_each_turn {
        conds.push(once_unused(""));
    }
    conds.extend(play_condition);
    let condition = match conds.len() {
        0 => None,
        1 => conds.pop(),
        _ => Some(Condition::And(conds)),
    };
    // "You may cast ~ as though it had flash [if ...]", "You may cast noncreature spells
    // as though they had flash" (CR 601.3b): a timing permission. The card's own functions
    // wherever it could be cast from (CR 601.3d).
    if terms.flash && quality.is_none() && flash_only(p, &terms) && !p.once_each_turn {
        let (what, zone) = match &p.obj {
            Obj::SelfCard => (Filter::Source, FunctionZone::Anywhere),
            Obj::Class {
                what,
                single: false,
            } => (what.clone(), FunctionZone::Battlefield),
            _ => return None,
        };
        if mentions_x(&what) || terms.condition.is_some() {
            return None;
        }
        let mut s = StaticAbility::new(StaticEffect::FlashPermission {
            who: PlayerRel::You,
            what,
        });
        s.zone = zone;
        s.condition = condition;
        return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
    }
    let (pp, zone) = match &p.obj {
        Obj::SelfCard => {
            let (zone, fz) = match p.from? {
                From::Graveyard => (ZoneKind::Graveyard, FunctionZone::Graveyard),
                From::Exile => (ZoneKind::Exile, FunctionZone::Exile),
                From::LibraryTop => (ZoneKind::Library, FunctionZone::Library),
                _ => return None,
            };
            let _ = ctx;
            (
                PlayPermission {
                    who: PlayerRel::You,
                    zone,
                    top_only: zone == ZoneKind::Library,
                    what: Filter::Source,
                    lands: p.lands,
                    spells: p.spells,
                    cost: p.free.then(Cost::free),
                    flash: false,
                    terms: terms.clone(),
                },
                fz,
            )
        }
        Obj::Class { what, single } => {
            if *single && !p.once_each_turn {
                return None;
            }
            (
                class_permission(p, what.clone(), terms.clone())?,
                FunctionZone::Battlefield,
            )
        }
        // "As long as there are seven or more cards in your graveyard, you may cast the
        // exiled card" (Null Summoner): the card exiled with ~ (CR 607.2a).
        Obj::Referent { limit: None } if p.exiled_named && p.from.is_none() => {
            let what = if p.lands {
                Filter::Card
            } else {
                Filter::and(vec![
                    Filter::Not(Box::new(Filter::Type(CardType::Land))),
                    Filter::Card,
                ])
            };
            (
                class_permission_from(p, From::Linked { owned: false }, what, terms.clone())?,
                FunctionZone::Battlefield,
            )
        }
        Obj::Referent { .. } | Obj::Target { .. } => return None,
    };
    let mut pp = pp;
    if let Some(q) = quality {
        pp.what = Filter::and(vec![pp.what, q]);
    }
    if mentions_x(&pp) {
        return None;
    }
    let mut perms = vec![pp];
    // "from your hand or the top of your library", "from your graveyard or from exile":
    // a permission for each zone (sharing a once-each-turn use, see `once_each_turn_cast`).
    if let Some(z2) = p.also_from {
        let second = match &p.obj {
            Obj::SelfCard => {
                let mut pp2 = perms[0].clone();
                pp2.zone = match z2 {
                    From::Graveyard => ZoneKind::Graveyard,
                    From::Exile => ZoneKind::Exile,
                    _ => return None,
                };
                pp2
            }
            Obj::Class { what, .. } => class_permission_from(p, z2, what.clone(), terms)?,
            _ => return None,
        };
        perms.push(second);
    }
    let fzone = |pp: &PlayPermission| match (&p.obj, pp.zone) {
        (Obj::SelfCard, ZoneKind::Graveyard) => FunctionZone::Graveyard,
        (Obj::SelfCard, ZoneKind::Exile) => FunctionZone::Exile,
        (Obj::SelfCard, ZoneKind::Library) => FunctionZone::Library,
        _ => zone,
    };
    Some(
        perms
            .into_iter()
            .map(|pp| {
                let mut s = StaticAbility::new(StaticEffect::PlayPermission(pp.clone()));
                s.zone = fzone(&pp);
                s.condition = condition.clone();
                AbilityDef::new(AbilityKind::Static(s), text)
            })
            .collect(),
    )
}

/// A permission sentence after the instruction that moved the cards it's about ("Exile
/// the top card of your library. You may play that card this turn."), with an optional
/// leading condition ("If you do, ...", "If it's red, ...").
fn followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l).trim();
    let (cond, rest) = if let Some(r) = l.strip_prefix("if you do, ") {
        (Some(Condition::PrevHappened), r)
    } else if let Some(r) = l.strip_prefix("if you don't, ") {
        (Some(Condition::Not(Box::new(Condition::PrevHappened))), r)
    } else {
        (None, l)
    };
    let Some(p) = parse(rest) else {
        return false;
    };
    let zone = referent_zone(Some(prev), b);
    if !matches!(p.obj, Obj::Referent { .. }) && p.from != Some(From::AmongReferent) {
        return false;
    }
    let Some(e) = to_effect(&p, zone, b.ctx) else {
        return false;
    };
    // "Put the rest on the bottom of your library in a random order.": the cards chosen
    // among that are still where they were (see `r406_exile_until`).
    if let (Some(z), true) = (zone, stores_among(&e)) {
        b.named.push((
            super::r406_exile_until::THE_REST.into(),
            Sel::All(Filter::and(vec![
                Filter::In(Box::new(Sel::Var(AMONG))),
                Filter::InZone(z),
            ])),
        ));
    }
    let e = match cond {
        Some(c) => Effect::If {
            cond: c,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
        None => e,
    };
    *prev = Effect::seq(vec![std::mem::take(prev), e]);
    true
}

/// A permission as an instruction of its own: for cards with qualities in a zone for a
/// while ("You may cast Zombie spells from your graveyard this turn."), or for the cards
/// "it" means after an earlier instruction moved them.
pub fn effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let p = parse(l)?;
    if matches!(p.obj, Obj::Target { .. }) {
        return target_effect(&p, b);
    }
    // "At the beginning of your end step, if you gained life this turn, you may cast ~
    // from your graveyard.": a triggered ability that functions in the graveyard (see
    // `triggers::trigger_zone`) casts the card as it resolves (CR 608.2g).
    if matches!(p.obj, Obj::SelfCard) {
        let plain = format!("{:?}", PlayTerms::default()) == format!("{:?}", p.terms);
        if !b.in_trigger
            || b.ctx.is_spell()
            || p.from != Some(From::Graveyard)
            || p.also_from.is_some()
            || p.duration.is_some()
            || p.who != Who::You
            || p.lands
            || p.look
            || p.cond_text.is_some()
            || p.your_turn
            || p.once_each_turn
            || !plain
        {
            return None;
        }
        return Some(Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::This,
            free: p.free,
            optional: true,
        });
    }
    let zone = referent_zone(None, b);
    to_effect(&p, zone, b.ctx)
}

/// "You may cast target instant or sorcery card from your graveyard this turn": a
/// permission for that card (it stays the same object while it's there, CR 400.7), or,
/// without a duration, a spell cast as the effect resolves (CR 608.2g).
fn target_effect(p: &Perm, b: &mut Builder) -> Option<Effect> {
    let Obj::Target { phrase } = &p.obj else {
        return None;
    };
    if p.who != Who::You || p.look || p.once_each_turn || p.your_turn || p.from.is_some() {
        return None;
    }
    let mut terms = p.terms_with_condition(b.ctx)?;
    let first = b.targets.len();
    let (sel, rest) = crate::oracle::effects::object_ref(phrase, b)?;
    // A card target in a graveyard or exile, not a permanent or a spell.
    let card_target = matches!(sel, Sel::Target(s) if s as usize == first)
        && b.targets.len() == first + 1
        && matches!(&b.targets[first].what, TargetKind::Object(f)
            if matches!(f.zone(), Some(ZoneKind::Graveyard | ZoneKind::Exile)));
    if !rest.trim().is_empty() || !card_target {
        return None;
    }
    let cast_only = !p.lands;
    match &p.duration {
        None => {
            if terms.flash
                || terms.extra_cost.is_some()
                || terms.spend_as_any_color
                || terms.condition.is_some()
                || terms.exile_instead
            {
                return None;
            }
            let cast = if cast_only {
                Effect::CastCard {
                    who: PlayerRef::You,
                    what: sel.clone(),
                    free: p.free,
                    optional: true,
                }
            } else {
                Effect::PlayCard {
                    who: PlayerRef::You,
                    what: sel.clone(),
                    free: p.free,
                    optional: true,
                }
            };
            // "..., and mana of any type can be spent to cast that spell" (CR 118.14).
            Some(if terms.spend_any_type {
                Effect::seq(vec![
                    Effect::SpendAnyTypeMana {
                        who: PlayerRef::You,
                        what: sel,
                        duration: Duration::EndOfTurn,
                    },
                    cast,
                ])
            } else {
                cast
            })
        }
        Some(d) => {
            if !matches!(
                d,
                Duration::EndOfTurn
                    | Duration::UntilEndOfYourNextTurn
                    | Duration::UntilYourNextTurn
            ) || terms.until_another
                || terms.later_turn
            {
                return None;
            }
            terms.spells_only = cast_only;
            Some(Effect::WithPlayTerms {
                terms,
                effect: Box::new(Effect::GrantPlayPermission {
                    who: PlayerRef::You,
                    what: sel,
                    duration: d.clone(),
                    free: p.free,
                }),
            })
        }
    }
}

/// The terms of the last permission the effect gives, if it ends by giving one.
fn last_terms(e: &mut Effect) -> Option<&mut PlayTerms> {
    match e {
        Effect::WithPlayTerms { terms, .. } => Some(terms),
        Effect::AddPlayerEffect {
            effect: PlayerModification::PlayPermission(pp),
            ..
        } => Some(&mut pp.terms),
        Effect::Seq(v) => v.last_mut().and_then(last_terms),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_terms(then),
        _ => None,
    }
}

/// Whether the effect ends by casting a card as it resolves.
fn ends_casting(e: &Effect) -> bool {
    match e {
        Effect::CastCard { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_casting),
        _ => false,
    }
}

/// "If that spell would be put into your graveyard, exile it instead." (and "If a spell
/// cast this way would be put into a graveyard, ...") after a permission to cast cards:
/// a replacement effect for each spell cast with it (CR 614.1a); after a spell cast as
/// the effect resolves, for that spell (see `spell_cast_this_way_exiled`).
fn exile_instead(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l).trim();
    // "If an instant or sorcery spell cast this way would be put into your graveyard,
    // exile it instead." (Bilbo, Thief in the Night): only those spells.
    let mut quality = None;
    let r = if let Some(r) = [
        "if that spell would be put into ",
        "if a spell cast this way would be put into ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p))
    {
        r
    } else {
        let Some((q, r)) = l
            .strip_prefix("if an ")
            .or_else(|| l.strip_prefix("if a "))
            .and_then(|r| r.split_once(" cast this way would be put into "))
        else {
            return false;
        };
        let Some((f, _, rest)) = class_phrase(q) else {
            return false;
        };
        if !rest.trim().is_empty() {
            return false;
        }
        quality = Some(f);
        r
    };
    if !matches!(
        r,
        "your graveyard, exile it instead" | "a graveyard, exile it instead"
    ) {
        return false;
    }
    // (A permission to play lands as well: only its spells are cast.)
    if quality.is_none() {
        if let Some(terms) = last_terms(prev) {
            terms.exile_instead = true;
            return true;
        }
    }
    if ends_casting(prev) {
        let mut filter = Filter::In(Box::new(Sel::Var(vars::IT)));
        if let Some(q) = quality {
            filter = Filter::and(vec![filter, crate::casting::as_spell_filter(&q)]);
        }
        let replacement = Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::ZoneChange {
                    filter,
                    from: None,
                    to: Some(ZoneKind::Graveyard),
                },
                action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::Permanent,
            uses: None,
        };
        *prev = Effect::seq(vec![std::mem::take(prev), replacement]);
        return true;
    }
    false
}

/// Second sentences that modify a static permission: "You can't cast more than one spell
/// this way each turn.", "If a spell cast this way would be put into your graveyard,
/// exile it instead.", "If you cast a spell this way, you may spend mana as though it
/// were mana of any color to cast it."
fn static_rider(s: &str, p: &mut Perm) -> bool {
    match end(s).trim() {
        "you can't cast more than one spell this way each turn" if p.spells && !p.lands => {
            p.once_each_turn = true;
        }
        "if a spell cast this way would be put into your graveyard, exile it instead"
        | "if a spell cast this way would be put into a graveyard, exile it instead"
            if p.spells =>
        {
            p.terms.exile_instead = true;
        }
        "mana of any type can be spent to cast those spells"
        | "mana of any type can be spent to cast them"
            if p.spells =>
        {
            p.terms.spend_any_type = true;
        }
        "if you cast a spell this way, you may spend mana as though it were mana of any color to cast it"
        | "if you cast a spell this way, you may spend mana as though it were mana of any color to cast that spell" => {
            p.terms.spend_as_any_color = true;
        }
        _ => return false,
    }
    true
}

fn statics(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l).trim();
    // "You may look at cards exiled with ~, and you may play lands and cast spells from
    // among those cards." (Kheru Mind-Eater): looking at the face-down cards, and a
    // permission to play them.
    for lead in [
        "you may look at cards exiled with ~, and ",
        "you may look at the cards exiled with ~, and ",
    ] {
        let Some(r) = l.strip_prefix(lead) else {
            continue;
        };
        let r = r.strip_suffix(" from among those cards")?;
        let r = r.strip_prefix("you may ").unwrap_or(r);
        let p = parse(&format!("you may {r} from among cards exiled with ~"))?;
        let mut v = to_statics(&p, text, ctx)?;
        v.push(AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
                crate::kw::look_at_exiled_with::MAY_LOOK_AT_LINKED.into(),
            ))),
            text,
        ));
        return Some(v);
    }
    // "[permission]. [rider]."
    if let Some((first, second)) = l.split_once(". ") {
        let mut p = parse(first)?;
        if !static_rider(second, &mut p) {
            return None;
        }
        return to_statics(&p, text, ctx);
    }
    // "If you control a creature with power 4 or greater, you may cast ~ as though it had
    // flash.": a condition on casting it.
    if let Some(r) = l.strip_prefix("if ") {
        let (c, rest) = r.split_once(", ")?;
        let mut p = parse(rest)?;
        if p.cond_text.is_some() {
            return None;
        }
        p.cond_text = Some(c.to_string());
        return to_statics(&p, text, ctx);
    }
    // "During your turn, if an opponent lost life this turn, you may play lands and cast
    // spells from among cards exiled with ~.", "During your turn, as long as you've
    // sacrificed a nontoken permanent this turn, you may play cards exiled with ~."
    for lead in ["during your turn, ", "during each of your turns, "] {
        let Some(r) = l.strip_prefix(lead) else {
            continue;
        };
        let Some(r) = r
            .strip_prefix("if ")
            .or_else(|| r.strip_prefix("as long as "))
        else {
            continue;
        };
        let (c, rest) = r.split_once(", ")?;
        let mut p = parse(&format!("{lead}{rest}"))?;
        if p.cond_text.is_some() {
            return None;
        }
        p.cond_text = Some(c.to_string());
        return to_statics(&p, text, ctx);
    }
    let p = parse(l)?;
    to_statics(&p, text, ctx)
}

/// "You may cast ~ from your graveyard." on an instant or sorcery (which the static
/// patterns don't see): a static ability that functions in that zone (CR 113.6).
fn spell_self_permission(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(lower.trim());
    if l.contains(". ") {
        return None;
    }
    // "If there are two or more instant and/or sorcery cards in your graveyard, you may
    // cast ~ as though it had flash." (a condition on casting it).
    let (cond, l) = match l.strip_prefix("if ").and_then(|r| r.split_once(", ")) {
        Some((c, rest)) => (Some(c), rest),
        None => (None, l),
    };
    let mut p = parse(l)?;
    if !matches!(p.obj, Obj::SelfCard) {
        return None;
    }
    if let Some(c) = cond {
        if p.cond_text.is_some() {
            return None;
        }
        p.cond_text = Some(c.to_string());
    }
    to_statics(&p, block.trim(), ctx)
}

/// "The next creature spell you cast this turn can be cast as though it had flash.", "The
/// next creature card you play this turn can be played as though it had flash." (CR
/// 601.3b): a waiting effect for the next such spell its controller casts this turn
/// (CR 611.2f), which lets that spell be cast any time they could cast an instant; casting
/// the next such spell, whenever, uses it up.
fn next_spell_flash(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).trim().strip_prefix("the next ")?;
    // "[quality] spell you cast", or "[quality] card you play" (a nonland card played is
    // cast, CR 305.9).
    let quality = if r == "spell you cast this turn can be cast as though it had flash" {
        ""
    } else if let Some(s) =
        r.strip_suffix(" spell you cast this turn can be cast as though it had flash")
    {
        s
    } else {
        let s = r.strip_suffix(" card you play this turn can be played as though it had flash")?;
        if s.is_empty() {
            return None;
        }
        s
    };
    let filter = if quality == "spell" || quality.is_empty() {
        Filter::Spell
    } else {
        let f = object_filter(&format!("{quality} card"))?;
        if mentions_land(&f) {
            return None;
        }
        Filter::and(vec![f, Filter::Spell])
    };
    if mentions_x(&filter) {
        return None;
    }
    Some(Effect::NextSpell {
        filter,
        mods: vec![Modification::AddKeyword(
            crate::keywords::Keyword::new(crate::keywords::KeywordKind::Flash).text("flash"),
        )],
        expires: Duration::EndOfTurn,
    })
}

/// "Put the exiled cards not cast this way on the bottom of your library in a random
/// order." (Collected Conjuring), after "You may cast up to two sorcery spells ... from
/// among them": the cards chosen among that are still in exile (see [`followup`]).
fn put_those_not_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = [
        "put the exiled cards not cast this way ",
        "put the exiled cards that weren't cast this way ",
        "put the cards not cast this way ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p))?;
    if r != "on the bottom of your library in a random order" {
        return None;
    }
    let (_, rest) = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == super::r406_exile_until::THE_REST)?;
    let mut to = Destination::zone(ZoneKind::Library);
    to.position = LibraryPosition::BottomRandom;
    Some(Effect::Move {
        what: rest.clone(),
        to,
    })
}

inventory::submit! { EffectPattern { name: "permission grammar: the next spell you cast this turn can be cast as though it had flash", priority: 120, parse: next_spell_flash } }
inventory::submit! { EffectPattern { name: "permission grammar: put the exiled cards not cast this way on the bottom", priority: 80, parse: put_those_not_cast } }
inventory::submit! { FollowupPattern { name: "permission grammar: you may play the cards an earlier instruction moved", priority: 120, apply: followup } }
inventory::submit! { EffectPattern { name: "permission grammar: a permission to play cards", priority: 450, parse: effect } }
inventory::submit! { StaticPattern { name: "permission grammar: a static permission to play cards", priority: 120, parse: statics } }
inventory::submit! { FollowupPattern { name: "permission grammar: if that spell would be put into your graveyard, exile it instead", priority: 95, apply: exile_instead } }
inventory::submit! { AbilityPattern { name: "permission grammar: an instant's or sorcery's permission to cast itself", priority: 120, parse: spell_self_permission } }

#[cfg(test)]
mod tests {
    use super::*;

    fn perm(s: &str) -> Perm {
        parse(s).unwrap_or_else(|| panic!("not parsed: {s}"))
    }

    #[test]
    fn referents_durations_and_terms() {
        let p = perm("you may play that card this turn");
        assert!(matches!(p.obj, Obj::Referent { limit: None }));
        assert!(matches!(p.duration, Some(Duration::EndOfTurn)));
        assert!(p.lands && p.spells);
        let p = perm("until the end of your next turn, you may play that card, and you may spend mana as though it were mana of any color to cast it");
        assert!(matches!(p.duration, Some(Duration::UntilEndOfYourNextTurn)));
        assert!(p.terms.spend_as_any_color);
        let p = perm("you may cast that card for as long as it remains exiled, and mana of any type can be spent to cast that spell");
        assert!(!p.lands && p.terms.spend_any_type);
        assert!(matches!(p.duration, Some(Duration::Permanent)));
        let p = perm("you may play it until you exile another card with ~");
        assert!(p.terms.until_another);
        let p = perm("you may play that card for as long as you control ~");
        assert!(matches!(p.duration, Some(Duration::WhileYouControlSource)));
        let p = perm("you may play up to two of those cards until the end of your next turn");
        assert!(matches!(p.obj, Obj::Referent { limit: Some(2) }));
        let p = perm("during your next turn, you may play that card");
        assert!(p.terms.later_turn && p.your_turn);
        let p = perm("you may play them this turn without paying their mana costs");
        assert!(p.free);
    }

    #[test]
    fn spells_from_among_referents() {
        let p = perm("you may cast red spells from among them this turn");
        assert_eq!(p.from, Some(From::AmongReferent));
        assert!(matches!(p.obj, Obj::Class { single: false, .. }));
        let p = perm("until the end of your next turn, you may cast an instant or sorcery spell from among those exiled cards");
        assert!(matches!(p.obj, Obj::Class { single: true, .. }));
        let p = perm("you may cast a spell from among those cards");
        assert!(p.duration.is_none());
    }

    #[test]
    fn classes_zones_and_costs() {
        let p = perm("you may cast zombie spells from your graveyard this turn");
        assert_eq!(p.from, Some(From::Graveyard));
        let p = perm("you may play lands and cast insect spells from your graveyard");
        assert!(p.lands && p.spells);
        let p = perm("you may cast ~ from your graveyard by discarding two cards in addition to paying its other costs");
        assert!(p.terms.extra_cost.is_some());
        let p = perm("until end of turn, you may cast creature spells from your graveyard by foraging in addition to paying their other costs");
        assert!(p.terms.extra_cost.is_some());
        let p = perm("you may play lands and cast spells from among cards exiled with ~");
        assert_eq!(p.from, Some(From::Linked { owned: false }));
        let p = perm("during your turn, you may play cards exiled with ~");
        assert!(p.your_turn);
        assert_eq!(p.from, Some(From::Linked { owned: false }));
        let p =
            perm("you may cast dinosaur creature spells from among cards you own exiled with ~");
        assert_eq!(p.from, Some(From::Linked { owned: true }));
        let p = perm(
            "you may play historic lands and cast historic spells from the top of your library",
        );
        assert_eq!(p.from, Some(From::LibraryTop));
        let p = perm(
            "once each turn, you may cast an instant or sorcery spell from the top of your library",
        );
        assert!(p.once_each_turn);
        assert!(parse("you may play forests from your graveyard").is_some());
    }
}
