//! Zone-move grammar: "return"/"put" instructions that move objects between zones, read
//! compositionally (CR 400.3, 400.6, 400.7).
//!
//! ```text
//! move      := [subject] ["may"] verb objects [from] dest [mods] [from]
//!            | [subject] ["may"] verb dest [mods] objects            ("return to your hand all ...")
//! subject   := "" | "you" | "each player" | "each opponent" | "each other player"
//!            | player phrase ("that player", "target opponent", "its controller")
//! verb      := "return" | "put"                                     (third person: "-s")
//! objects   := item (("," | "and" | ", and" | "and/or") item)*
//! item      := referent ("~", "it", "them", "that card", "the exiled cards", targets)
//!            | quantity noun-phrase ["at random"] [qualifiers]
//!            | "one of them" | "N of those cards"
//! quantity  := "a" | N | "X" | "up to" N | "any number of" | "all" | "each" | "another"
//! qualifier := from | "that were put there from the battlefield this turn" | "you own"
//!            | "exiled with ~" | "milled this way" | "of an opponent's choice" ...
//! from      := "from exile" | "from" owner zone ("your graveyard", "a graveyard", "all
//!              graveyards", "their hand", "your hand or graveyard", ...)
//! dest      := ("to" | "onto") "the battlefield" | ("to" | "into") owner's ("hand" | "graveyard")
//!            | "on top of" / "on the bottom of" owner's "library"
//! mods      := "tapped" | "tapped and attacking" | "transformed" | "under [player]'s control"
//!            | "with [counters] on it"
//! ```
//!
//! Every card goes to its owner's hand, library or graveyard whatever the text calls the
//! zone (CR 400.3). The player who puts a permanent onto the battlefield controls it
//! unless the effect says otherwise (CR 110.2a). Cards the instruction chooses (not
//! targets) are chosen as it's performed (CR 608.2c) by the player performing it, or
//! picked at random ("at random"). Objects that move with one instruction move at the same
//! time.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::types::CounterKind;
use smol_str::SmolStr;

/// The candidates a random choice is made among, how many are picked, and the pick.
pub const RANDOM_POOL: Var = vars::USER + 7400;
pub const RANDOM_PICK: Var = vars::USER + 7401;
pub const RANDOM_COUNT: Var = vars::USER + 7402;

fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';'])
}

fn strip_word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let r = s.strip_prefix(p)?;
    word_end(r).then_some(r)
}

/// Who performs the instruction.
#[derive(Clone, Debug)]
struct Subject {
    /// The player performing it, as the instruction refers to them (`Iterated` for each
    /// of several players).
    who: PlayerRef,
    /// "Each player ...": the players who perform it in turn.
    each: Option<PlayerRef>,
    may: bool,
}

impl Subject {
    fn is_you(&self) -> bool {
        matches!(self.who, PlayerRef::You) && self.each.is_none()
    }
}

/// The subject and the rest after the verb ("return"/"put").
fn subject_verb(l: &str, b: &mut Builder) -> Option<(Subject, &'static str, String)> {
    let verbs = [("return", "returns"), ("put", "puts")];
    let imperative = |s: &str| -> Option<(&'static str, String)> {
        for (v, _) in verbs {
            if let Some(r) = s.strip_prefix(v).and_then(|r| r.strip_prefix(' ')) {
                return Some((v, r.to_string()));
            }
        }
        None
    };
    let third = |s: &str| -> Option<(bool, &'static str, String)> {
        let (may, s) = match s.strip_prefix("may ") {
            Some(r) => (true, r),
            None => (false, s),
        };
        if may {
            let (v, r) = imperative(s)?;
            return Some((true, v, r));
        }
        for (v, v3) in verbs {
            if let Some(r) = s.strip_prefix(v3).and_then(|r| r.strip_prefix(' ')) {
                return Some((false, v, r.to_string()));
            }
        }
        None
    };
    let (may, l) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    if let Some((v, r)) = imperative(l) {
        return Some((
            Subject {
                who: PlayerRef::You,
                each: None,
                may,
            },
            v,
            r,
        ));
    }
    if may {
        return None;
    }
    if let Some(r) = l.strip_prefix("you ") {
        let (may, v, r) = third(&format!("may {r}"))
            .filter(|_| r.starts_with("may "))
            .map(|(_, v, r)| (true, v, r))
            .or_else(|| imperative(r).map(|(v, r)| (false, v, r)))?;
        return Some((
            Subject {
                who: PlayerRef::You,
                each: None,
                may,
            },
            v,
            r,
        ));
    }
    for (p, each) in [
        ("each player ", PlayerRef::EachPlayer),
        ("each opponent ", PlayerRef::EachOpponent),
        ("each other player ", PlayerRef::EachOtherPlayer),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            let (may, v, r) = third(r)?;
            return Some((
                Subject {
                    who: PlayerRef::Iterated,
                    each: Some(each),
                    may,
                },
                v,
                r,
            ));
        }
    }
    let saved = (b.targets.len(), b.it_player.clone());
    let Some((who, rest)) = player_ref(l, b) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    let Some((may, v, r)) = third(rest.trim_start()) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    if matches!(who, PlayerRef::You) {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    }
    Some((
        Subject {
            who,
            each: None,
            may,
        },
        v,
        r,
    ))
}

/// How many objects an item takes.
#[derive(Clone, Debug)]
enum Qty {
    All,
    Exactly(Value),
    UpTo(Value),
    AnyNumber,
}

fn quantity(s: &str) -> Option<(Qty, &str)> {
    let s = s.trim_start();
    for p in ["all the ", "all ", "each "] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((Qty::All, r));
        }
    }
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((Qty::AnyNumber, r));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        return Some((Qty::UpTo(n), r));
    }
    // "another creature you control": one, other than the source (the phrase reads it).
    if s.starts_with("another ") {
        return Some((Qty::Exactly(Value::c(1)), s));
    }
    let (n, r) = parse_number(s)?;
    Some((Qty::Exactly(n), r))
}

/// Whether the filter names cards (as opposed to permanents).
fn names_cards(f: &Filter) -> bool {
    match f {
        Filter::Card | Filter::PermanentCard => true,
        Filter::And(v) => v.iter().any(names_cards),
        Filter::Or(v) => !v.is_empty() && v.iter().all(names_cards),
        _ => false,
    }
}

/// Whether the filter limits the objects to a selection ("milled this way") or a zone.
fn located(f: &Filter) -> bool {
    if f.zone().is_some() {
        return true;
    }
    match f {
        Filter::In(_) => true,
        Filter::And(v) => v.iter().any(located),
        Filter::Or(v) => !v.is_empty() && v.iter().all(located),
        _ => false,
    }
}

fn their(b: &Builder, subject: &Subject) -> Option<PlayerRef> {
    use super::oracle_hardening_referents::is_no_player_referent;
    if !matches!(subject.who, PlayerRef::You) {
        return Some(subject.who.clone());
    }
    (!is_no_player_referent(&b.it_player)).then(|| b.it_player.clone())
}

/// "graveyard", "hand", "hand or graveyard", "hand and/or graveyard", "graveyards".
fn zone_words(s: &str) -> Option<(Vec<ZoneKind>, &str)> {
    for (p, z) in [
        (
            "hand or graveyard",
            vec![ZoneKind::Hand, ZoneKind::Graveyard],
        ),
        (
            "hand and/or graveyard",
            vec![ZoneKind::Hand, ZoneKind::Graveyard],
        ),
        (
            "graveyard or hand",
            vec![ZoneKind::Hand, ZoneKind::Graveyard],
        ),
        ("graveyards", vec![ZoneKind::Graveyard]),
        ("graveyard", vec![ZoneKind::Graveyard]),
        ("hands", vec![ZoneKind::Hand]),
        ("hand", vec![ZoneKind::Hand]),
    ] {
        if let Some(r) = strip_word(s, p) {
            return Some((z, r));
        }
    }
    None
}

/// The zone(s) objects come from: "from exile", "from your graveyard", "from a
/// graveyard", "from all graveyards", "from their hand", "from your hand or graveyard",
/// "from an opponent's graveyard", "from target opponent's graveyard", "from your
/// graveyard or from exile". Returns the filter and whether the zone is another player's.
fn from_zone<'a>(
    s: &'a str,
    b: &mut Builder,
    subject: &Subject,
) -> Option<(Filter, bool, &'a str)> {
    use super::value_grammar::owned_by;
    let s = s.trim_start();
    let r = s.strip_prefix("from ").or_else(|| s.strip_prefix("in "))?;
    if let Some(r) = strip_word(r, "exile") {
        return Some((Filter::InZone(ZoneKind::Exile), false, r));
    }
    let (owner, other, r): (Option<Filter>, bool, &str) = if let Some(r) = r
        .strip_prefix("your opponents' ")
        .or_else(|| r.strip_prefix("an opponent's "))
        .or_else(|| r.strip_prefix("opponents' "))
    {
        (Some(Filter::OwnedBy(PlayerRel::Opponent)), true, r)
    } else if let Some(r) = r.strip_prefix("your ") {
        (Some(Filter::OwnedBy(PlayerRel::You)), false, r)
    } else if let Some(r) = r.strip_prefix("their ") {
        let who = their(b, subject)?;
        (Some(owned_by(&who)), !matches!(who, PlayerRef::You), r)
    } else if let Some(r) = r
        .strip_prefix("all ")
        .or_else(|| r.strip_prefix("a "))
        .or_else(|| r.strip_prefix("each player's "))
    {
        if r.starts_with("single ") {
            return None;
        }
        (None, false, r)
    } else if r.starts_with("graveyards") {
        (None, false, r)
    } else if let Some(r) = r.strip_prefix("its owner's ") {
        // A specific object's zone: wherever its owner's is.
        (None, false, r)
    } else if let Some(r) = r.strip_prefix("that player's ") {
        let who = their(b, subject)?;
        (Some(owned_by(&who)), true, r)
    } else if let Some(r) = r.strip_prefix("defending player's ") {
        (Some(Filter::OwnedBy(PlayerRel::Defending)), true, r)
    } else {
        let mut found = None;
        for (p, pf) in [
            ("target opponent's ", PlayerFilter::Opponent),
            ("target player's ", PlayerFilter::Any),
        ] {
            if let Some(r) = r.strip_prefix(p) {
                let it = b.it.clone();
                let text = p.trim_end().trim_end_matches("'s");
                let slot = b.add_target(TargetSpec::player(pf, text), text);
                b.it = it;
                b.it_player = PlayerRef::Target(slot);
                found = Some((Some(Filter::OwnedBy(PlayerRel::Target(slot))), true, r));
                break;
            }
        }
        found?
    };
    let (zones, mut rest) = zone_words(r)?;
    let mut zone = if zones.len() == 1 {
        Filter::InZone(zones[0])
    } else {
        Filter::Or(zones.into_iter().map(Filter::InZone).collect())
    };
    // "from your graveyard or from exile".
    if let Some(r2) = rest.strip_prefix(" or from exile") {
        zone = Filter::Or(vec![zone, Filter::InZone(ZoneKind::Exile)]);
        rest = r2;
    }
    let f = match owner {
        Some(o) => Filter::and(vec![zone, o]),
        None => zone,
    };
    Some((f, other, rest))
}

/// History and choice qualifiers after a card phrase: "that were put there from the
/// battlefield this turn", "milled this way", "of an opponent's choice".
fn history<'a>(s: &'a str, b: &Builder) -> Option<(Filter, &'a str)> {
    use crate::kw::hand_graveyard_actions::PUT_THERE_THIS_TURN;
    let t = s.trim_start();
    for (p, from) in [
        (
            "that was put there from the battlefield this turn",
            "battlefield",
        ),
        (
            "that were put there from the battlefield this turn",
            "battlefield",
        ),
        ("that were put there from anywhere this turn", ""),
        ("that was put there from anywhere this turn", ""),
        ("that were put there this turn", ""),
        ("that was put there this turn", ""),
    ] {
        if let Some(r) = strip_word(t, p) {
            return Some((
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::Custom(SmolStr::new(format!("{PUT_THERE_THIS_TURN}{from}"))),
                ]),
                r,
            ));
        }
    }
    for p in ["that was milled this turn", "that were milled this turn"] {
        if let Some(r) = strip_word(t, p) {
            return Some((
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::Custom(SmolStr::new(MILLED_THIS_TURN)),
                ]),
                r,
            ));
        }
    }
    for p in [
        "that you cycled or discarded this turn",
        "that you discarded this turn",
        "you discarded this turn",
    ] {
        if let Some(r) = strip_word(t, p) {
            return Some((
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::Custom(SmolStr::new(DISCARDED_BY_YOU_THIS_TURN)),
                ]),
                r,
            ));
        }
    }
    // "that were put into your graveyard from the battlefield this turn" (Fell Shepherd).
    for (p, from) in [
        (
            "that were put into your graveyard from the battlefield this turn",
            "battlefield",
        ),
        (
            "that was put into your graveyard from the battlefield this turn",
            "battlefield",
        ),
        ("that were put into your graveyard this turn", ""),
        (
            "that were put into your graveyard from anywhere this turn",
            "",
        ),
    ] {
        if let Some(r) = strip_word(t, p) {
            return Some((
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::OwnedBy(PlayerRel::You),
                    Filter::Custom(SmolStr::new(format!("{PUT_THERE_THIS_TURN}{from}"))),
                ]),
                r,
            ));
        }
    }
    for (p, var) in [
        ("milled this way", vars::IT),
        ("put into a graveyard this way", vars::IT),
        ("put into graveyards this way", vars::IT),
        ("discarded this way", crate::discard_rules::DISCARDED),
        ("destroyed this way", vars::IT),
    ] {
        if let Some(r) = strip_word(t, p) {
            // Only right after the instruction that milled or discarded them.
            if !super::hand_graveyard_grammar::acted(b) && var == vars::IT && !milled_before(b) {
                return None;
            }
            return Some((
                Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::In(Box::new(Sel::Var(var))),
                ]),
                r,
            ));
        }
    }
    None
}

/// Whether an earlier instruction of the text milled cards (whose new objects are "it").
fn milled_before(b: &Builder) -> bool {
    b.sentences > 0 || !matches!(b.it, Sel::This)
}

/// Whether the phrase occurs only inside a quoted ability in the card's text.
fn in_quoted_ability(phrase: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    let key: String = phrase.split(' ').take(4).collect::<Vec<_>>().join(" ");
    let mut any = false;
    for (i, _) in raw.match_indices(&key) {
        any = true;
        let quotes = raw[..i].matches('"').count() + raw[..i].matches('\u{201C}').count();
        if quotes % 2 == 0 {
            return false;
        }
    }
    any
}

/// What an object description of a move names.
#[derive(Clone, Debug)]
enum Kind {
    /// Objects the text already identifies ("~", "it", "the exiled cards").
    Fixed(Sel),
    /// The targets in a slot.
    Target(u8),
    /// Cards or permanents chosen (or all of them) as the instruction is performed.
    Chosen {
        qty: Qty,
        filter: Filter,
        random: bool,
        chooser: PlayerRef,
    },
}

/// One object description of a move and what's known about it.
#[derive(Clone, Debug)]
struct Item {
    kind: Kind,
    /// The objects are in another player's zone ("from an opponent's graveyard").
    others_zone: bool,
}

/// "a creature card at random from your graveyard": the pick (in [`RANDOM_PICK`]).
fn random_pick(filter: Filter, count: Value) -> Effect {
    Effect::seq(vec![
        Effect::Store {
            var: RANDOM_POOL,
            sel: Sel::All(filter),
        },
        Effect::StoreValue {
            var: RANDOM_COUNT,
            value: count,
        },
        Effect::Custom(SmolStr::new(crate::kw::zone_moves::PICK_AT_RANDOM)),
    ])
}

/// A phrase that names its quantity of targets ("target", "up to one target", "two
/// target", "X target", "one, two, or three target", "any number of target").
fn is_target_phrase(s: &str) -> bool {
    let Some(i) = s.find("target ") else {
        return false;
    };
    let head = s[..i].trim();
    if head.is_empty()
        || matches!(
            head,
            "another" | "any number of" | "one, two, or three" | "one or two"
        )
    {
        return true;
    }
    let head = head.strip_prefix("up to ").unwrap_or(head);
    let head = head.strip_suffix(" other").unwrap_or(head);
    match parse_number(head) {
        Some((_, r)) => r.trim().is_empty(),
        None => false,
    }
}

/// "the exiled card(s)": the cards the source's linked ability exiled (CR 607.2a).
fn exiled_cards(s: &str, b: &Builder) -> Option<(Sel, String)> {
    // Only a permanent's abilities are linked (CR 607.1); an ability a quoted text grants
    // another object ("create a token ... with \"When ~ leaves the battlefield, return the
    // exiled card ...\"") is linked to the object that created it, not read here.
    if b.ctx.is_spell() || in_quoted_ability(s) {
        return None;
    }
    for p in [
        "the exiled cards",
        "the exiled card",
        "the cards exiled with ~",
        "the cards exiled with it",
        "all cards exiled with ~",
        "all cards exiled with it",
    ] {
        if let Some(r) = strip_word(s, p) {
            if p.ends_with("it") && !matches!(b.it, Sel::This) {
                return None;
            }
            return Some((Sel::All(linked_exile()), r.to_string()));
        }
    }
    None
}

fn linked_exile() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

/// Whether the text after an object can continue a move.
fn continues(rest: &str) -> bool {
    let t = rest.trim_start();
    t.is_empty()
        || [
            "to ",
            "onto ",
            "into ",
            "from ",
            "on top of",
            "on the bottom of",
            ",",
            "and ",
            "and/or ",
            "under ",
            "tapped",
            "at random",
        ]
        .iter()
        .any(|p| t.starts_with(p))
}

/// Adjectives the shared phrase parser doesn't read: "exiled card" (a card in exile),
/// "face-up exiled card" (CR 406.3).
fn zone_adjectives(s: &str) -> (Vec<Filter>, &str) {
    if let Some(r) = s.strip_prefix("face-up exiled ") {
        return (
            vec![
                Filter::InZone(ZoneKind::Exile),
                Filter::not(Filter::FaceDown),
            ],
            r,
        );
    }
    if let Some(r) = s.strip_prefix("exiled ") {
        return (vec![Filter::InZone(ZoneKind::Exile)], r);
    }
    (vec![], s)
}

/// A described set of objects (see [`described`]).
#[derive(Clone, Debug)]
struct Described {
    filter: Filter,
    /// The zone is another player's ("from an opponent's graveyard").
    others: bool,
    /// "at random".
    random: bool,
    /// "of an opponent's choice".
    opponents_choice: bool,
    rest: String,
}

/// A noun phrase and its qualifiers: "creature card with mana value 2 or less", "card
/// at random from your graveyard", "creature cards in your graveyard that were put there
/// from the battlefield this turn", "land card from your hand or graveyard".
fn described(s: &str, b: &mut Builder, subject: &Subject) -> Option<Described> {
    let (adj, s) = zone_adjectives(s.trim_start());
    // The next item of a list ("artifact card, up to one target land card, and ...")
    // isn't part of this phrase: it's left for the list.
    let (s, next_items) = match next_item_at(s) {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };
    // The shared phrase parser reads some zone phrases ("from your hand") but not others
    // ("from your hand or graveyard"): the zone is read here.
    let cut = s
        .match_indices(" from ")
        .chain(s.match_indices(" in "))
        .map(|(i, _)| i)
        .filter(|&i| {
            let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
            let ok = from_zone(&s[i..], b, subject).is_some();
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            ok
        })
        .min();
    let (head, tail) = match cut {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };
    // "they control", "their graveyard": the player performing the instruction.
    let outer_player = b.it_player.clone();
    if !subject.is_you() {
        b.it_player = subject.who.clone();
    }
    // The reading that understands more of the phrase.
    let parsed = match (
        super::value_grammar::objects(head, b),
        alternatives(head, b),
    ) {
        (Some(a), Some(c)) => Some(if c.1.len() < a.1.len() { c } else { a }),
        (a, c) => a.or(c),
    };
    b.it_player = outer_player;
    let (filter, rest) = parsed?;
    // "outlaw creature cards": the shared parser leaves "cards" after a batch noun.
    let (filter, rest) = match strip_word(rest.trim_start(), "cards")
        .or_else(|| strip_word(rest.trim_start(), "card"))
    {
        Some(r) if !names_cards(&filter) => {
            (Filter::and(vec![filter, Filter::Card]), r.to_string())
        }
        _ => (filter, rest),
    };
    // "cards each with mana value X or less".
    let rest = match rest.trim_start().strip_prefix("each with ") {
        Some(r) => {
            let (f, r2) = super::value_grammar::objects(&format!("card with {r}"), b)?;
            let f = Filter::and(vec![filter.clone(), f]);
            let mut d = finish_described(f, adj, r2, tail, b, subject)?;
            d.rest.push_str(next_items);
            return Some(d);
        }
        None => rest,
    };
    let mut d = finish_described(filter, adj, rest, tail, b, subject)?;
    d.rest.push_str(next_items);
    Some(d)
}

/// Where the next item of a list starts in `s` (at its separator): ", up to one target
/// land card", " and target land", ", and a land card".
fn next_item_at(s: &str) -> Option<usize> {
    let mut best: Option<usize> = None;
    for sep in [", and/or ", ", and ", ", ", " and/or ", " and "] {
        for (i, _) in s.match_indices(sep) {
            let after = &s[i + sep.len()..];
            let starts_item = (is_target_phrase(after)
                && after
                    .find("target ")
                    .is_some_and(|j| after[..j].split(' ').count() <= 6))
                || ["up to ", "a ", "an ", "another ", "all ", "each ", "~ "]
                    .iter()
                    .any(|p| after.starts_with(p));
            if starts_item && best.is_none_or(|b| i < b) {
                best = Some(i);
            }
        }
    }
    best
}

/// "nonland permanent or suspended card", "instant or sorcery card from your graveyard or
/// exiled card with flashback you own": two complete descriptions, each with its own zone.
fn described_alternatives(s: &str, b: &mut Builder, subject: &Subject) -> Option<Described> {
    for (i, _) in s.match_indices(" or ") {
        let (left, right) = (&s[..i], &s[i + " or ".len()..]);
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let l = described(left, b, subject)
            .filter(|d| d.rest.trim().is_empty() && !d.random && d.filter.zone().is_some());
        let r = l.as_ref().and_then(|_| {
            described(right, b, subject)
                .filter(|d| continues(&d.rest) && !d.random && d.filter.zone().is_some())
        });
        match (l, r) {
            (Some(l), Some(r)) if l.filter.zone() != r.filter.zone() => {
                // "spell or nonland permanent an opponent controls": a controller or owner
                // after the second description describes both, unless the first names
                // where it is itself ("instant card from your graveyard or exiled card
                // you own").
                let own_place = [" from ", " in ", "exiled", "suspended"]
                    .iter()
                    .any(|p| left.contains(p));
                let shared: Vec<Filter> = match &r.filter {
                    Filter::And(v) => v
                        .iter()
                        .filter(|f| {
                            matches!(
                                f,
                                Filter::ControlledBy(_)
                                    | Filter::ControlledByPlayer(_)
                                    | Filter::OwnedBy(_)
                                    | Filter::OwnedByPlayer(_)
                            )
                        })
                        .cloned()
                        .collect(),
                    _ => vec![],
                };
                let lf = if own_place || shared.is_empty() {
                    l.filter
                } else {
                    Filter::and(std::iter::once(l.filter).chain(shared).collect())
                };
                return Some(Described {
                    filter: Filter::Or(vec![lf, r.filter]),
                    others: l.others || r.others,
                    random: false,
                    opponents_choice: l.opponents_choice || r.opponents_choice,
                    rest: r.rest,
                });
            }
            _ => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
            }
        }
    }
    None
}

/// "artifact or non-Aura enchantment card", "artifact, enchantment, or legendary card",
/// "Angel, Demon, or Dragon creature card": alternatives (each adjectives and a type or
/// subtype) before the noun and qualifiers they share.
fn alternatives(s: &str, b: &mut Builder) -> Option<(Filter, String)> {
    let mut alts: Vec<Filter> = Vec::new();
    let mut cur: Vec<Filter> = Vec::new();
    let mut rest = s.trim_start();
    loop {
        let (w, r) = split_word(rest);
        let comma = w.ends_with(',');
        let w = w.trim_end_matches(',');
        if w.is_empty() {
            return None;
        }
        // The type or subtype that ends an alternative.
        if let Some(h) = head_noun(w).filter(|h| matches!(h, Filter::Type(_) | Filter::Subtype(_)))
        {
            // A type followed by a noun that narrows it ends the list ("Dragon creature
            // card").
            let next = split_word(r).0.trim_end_matches(',');
            let narrows = !comma && !matches!(next, "or" | "and/or") && head_noun(next).is_some();
            if narrows {
                if alts.is_empty() {
                    return None;
                }
                cur.push(h);
                alts.push(Filter::and(std::mem::take(&mut cur)));
                rest = r;
                break;
            }
            cur.push(h);
            alts.push(Filter::and(std::mem::take(&mut cur)));
            rest = r;
            let t = rest.trim_start();
            if let Some(r2) = t.strip_prefix("or ").or_else(|| t.strip_prefix("and/or ")) {
                rest = r2;
                continue;
            }
            if comma {
                continue;
            }
            break;
        }
        // "artifact, enchantment, or legendary card": the last alternative is adjectives
        // only, before the shared noun.
        if matches!(w, "card" | "cards" | "permanent" | "permanents") {
            if alts.is_empty() || cur.is_empty() {
                return None;
            }
            alts.push(Filter::and(std::mem::take(&mut cur)));
            break;
        }
        // An adjective of the alternative ("non-Aura").
        cur.push(adjective(w)?);
        rest = r;
    }
    if alts.len() < 2 || !cur.is_empty() {
        return None;
    }
    // The shared noun and qualifiers: "card with mana value 3 or less", "creature card".
    let (f, r) = super::value_grammar::objects(rest, b)?;
    Some((Filter::and(vec![Filter::Or(alts), f]), r))
}

fn finish_described(
    filter: Filter,
    adj: Vec<Filter>,
    rest: String,
    tail: &str,
    b: &mut Builder,
    subject: &Subject,
) -> Option<Described> {
    let mut filter = Filter::and(adj.into_iter().chain([filter]).collect());
    let mut rest = format!("{rest}{tail}");
    let mut random = false;
    let mut opponents_choice = false;
    let mut others = owned_by_other(&filter);
    loop {
        let t = rest.trim_start().to_string();
        if let Some(r) = strip_word(&t, "at random") {
            random = true;
            rest = r.to_string();
            continue;
        }
        if filter.zone().is_none() {
            if let Some((f, other, r)) = from_zone(&t, b, subject) {
                filter = Filter::and(vec![filter, f]);
                others |= other;
                rest = r.to_string();
                continue;
            }
        }
        if let Some((f, r)) = history(&t, b) {
            filter = Filter::and(vec![filter, f]);
            rest = r.to_string();
            continue;
        }
        if let Some(r) = strip_word(&t, "of their choice") {
            rest = r.to_string();
            continue;
        }
        if let Some(r) = strip_word(&t, "of an opponent's choice") {
            opponents_choice = true;
            rest = r.to_string();
            continue;
        }
        // Qualifiers after "at random" or a zone ("a card at random exiled with ~").
        if !t.is_empty() && !continues(&t) {
            if let Some((Filter::And(v), r)) =
                super::value_grammar::objects(&format!("card {t}"), b)
            {
                if r.len() < t.len() && matches!(v.first(), Some(Filter::Card)) {
                    filter = Filter::and(
                        vec![filter]
                            .into_iter()
                            .chain(v.into_iter().skip(1))
                            .collect(),
                    );
                    rest = r;
                    continue;
                }
            }
        }
        break;
    }
    Some(Described {
        filter,
        others,
        random,
        opponents_choice,
        rest,
    })
}

/// "target ...", "up to one target ...", "X target ...": the targets, read by the shared
/// target parser or, for descriptions it doesn't read, from the quantity and
/// [`described`].
fn target_item(s: &str, b: &mut Builder, subject: &Subject) -> Option<(Item, String)> {
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    if let Some((Sel::Target(slot), rest)) = object_ref(s, b) {
        if continues(&rest) {
            let mut rest = rest;
            let mut others = false;
            if let Some(TargetKind::Object(f)) =
                b.targets.get(slot as usize).map(|t| t.what.clone())
            {
                others = owned_by_other(&f);
                // "target card with flashback you own from exile".
                if f.zone().is_none() {
                    if let Some((z, other, r)) = from_zone(&rest, b, subject) {
                        b.targets[slot as usize].what = TargetKind::Object(Filter::and(vec![f, z]));
                        others |= other;
                        rest = r.to_string();
                    }
                }
            }
            return Some((
                Item {
                    kind: Kind::Target(slot),
                    others_zone: others,
                },
                rest,
            ));
        }
    }
    b.targets.truncate(saved.0);
    (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    // The quantity.
    let t = s.trim_start();
    let (min, max, r) = if let Some(r) = t.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        (Value::c(0), n, r.trim_start())
    } else if let Some(r) = t.strip_prefix("any number of ") {
        (Value::c(0), Value::c(99), r)
    } else if let Some(r) = t.strip_prefix("one, two, or three ") {
        (Value::c(1), Value::c(3), r)
    } else if let Some(r) = t.strip_prefix("one or two ") {
        (Value::c(1), Value::c(2), r)
    } else if let Some((n, r)) =
        parse_number(t).filter(|(_, r)| r.trim_start().starts_with("target "))
    {
        (n.clone(), n, r.trim_start())
    } else {
        (Value::c(1), Value::c(1), t)
    };
    let (another, r) = match r
        .strip_prefix("another target ")
        .or_else(|| r.strip_prefix("other target "))
    {
        Some(r) => (true, r),
        None => (false, r.strip_prefix("target ")?),
    };
    let d = described(r, b, subject)
        .filter(|d| continues(&d.rest))
        .or_else(|| described_alternatives(r, b, subject));
    let Described {
        filter,
        others,
        random,
        opponents_choice,
        rest,
    } = d?;
    if random || !continues(&rest) {
        return None;
    }
    let text = t[..t.len() - rest.trim_start().len()].trim().to_string();
    // "another target creature card": other than the source, or than earlier targets
    // (see `Builder::add_target`).
    let filter = if another {
        Filter::and(vec![Filter::Other, filter])
    } else {
        filter
    };
    let mut spec = TargetSpec::object(filter, text.clone());
    spec.min = min;
    spec.max = max;
    // CR 601.7: an opponent chooses the target.
    spec.chosen_by_opponent = opponents_choice;
    let slot = b.add_target(spec, &text);
    Some((
        Item {
            kind: Kind::Target(slot),
            others_zone: others,
        },
        rest,
    ))
}

fn item(s: &str, b: &mut Builder, subject: &Subject) -> Option<(Item, String)> {
    let s = s.trim_start();
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let fixed = |sel: Sel| Item {
        kind: Kind::Fixed(sel),
        others_zone: false,
    };
    if let Some((sel, rest)) = exiled_cards(s, b) {
        return Some((fixed(sel), rest));
    }
    // "Choose two target creature cards in your graveyard. ... return the chosen cards":
    // those targets.
    for p in ["the chosen cards", "the chosen card"] {
        if let Some(r) = strip_word(s, p) {
            if let Sel::Target(slot) = b.it {
                return Some((
                    Item {
                        kind: Kind::Target(slot),
                        others_zone: false,
                    },
                    r.to_string(),
                ));
            }
            return None;
        }
    }
    // "all black and all red creature cards": cards of either color.
    if let Some(r) = s.strip_prefix("all ") {
        let (w, r2) = split_word(r);
        if adjective(w).is_some() && head_noun(w).is_none() {
            if let Some(r3) = r2.trim_start().strip_prefix("and all ") {
                return item(&format!("all {w} or {r3}"), b, subject);
            }
        }
    }
    // "one of them", "two of those cards": a choice among the cards the text is about.
    if let Some((n, r)) = parse_number(s) {
        if let Some(r) = r.trim_start().strip_prefix("of ") {
            for p in ["them", "those cards"] {
                if let Some(r2) = strip_word(r, p) {
                    let (sel, _) = super::pronoun_groups::plural_object_ref(p, b)??;
                    return Some((
                        fixed(Sel::Choose {
                            chooser: subject.who.clone(),
                            filter: Filter::In(Box::new(sel)),
                            count: n,
                            up_to: false,
                            store: None,
                        }),
                        r2.to_string(),
                    ));
                }
            }
            return None;
        }
    }
    if is_target_phrase(s) {
        return target_item(s, b, subject);
    }
    // Pronouns and names of objects the text is about.
    if !["all ", "each ", "a ", "an ", "another ", "any ", "up to "]
        .iter()
        .any(|p| s.starts_with(p))
    {
        if let Some((sel, rest)) = object_ref(s, b) {
            if !matches!(sel, Sel::All(_)) && continues(&rest) {
                // "~ from your hand", "~ from exile", "~ from your graveyard or from exile":
                // only from there.
                if let Some((f, _, r)) = from_zone(&rest, b, subject) {
                    // An ability functions in one zone (CR 113.6m): "~ from your
                    // graveyard or from exile" would need two.
                    if matches!(sel, Sel::This)
                        && matches!(f.zone(), None)
                        && matches!(f, Filter::Or(_) | Filter::And(_))
                        && format!("{f:?}").matches("InZone").count() > 1
                    {
                        return None;
                    }
                    // The source may have moved since the ability triggered (an Aura
                    // put into a graveyard): the object it became (CR 400.7).
                    let which = match sel {
                        Sel::This => Filter::Custom(SmolStr::new(
                            crate::kw::hand_graveyard_actions::SOURCE_OR_NEXT,
                        )),
                        sel => Filter::In(Box::new(sel)),
                    };
                    return Some((fixed(Sel::All(Filter::and(vec![which, f]))), r.to_string()));
                }
                return Some((fixed(sel), rest));
            }
        }
        restore(b);
    }
    let (qty, r) = quantity(s)?;
    let Some(d) = described(r, b, subject) else {
        restore(b);
        return None;
    };
    // "a nonland card of an opponent's choice": the controller chooses an opponent, who
    // chooses the card (no target).
    let chooser = if d.opponents_choice {
        if !subject.is_you() {
            restore(b);
            return None;
        }
        PlayerRef::ChosenOpponent
    } else {
        subject.who.clone()
    };
    Some((
        Item {
            kind: Kind::Chosen {
                qty,
                filter: d.filter,
                random: d.random,
                chooser,
            },
            others_zone: d.others,
        },
        d.rest,
    ))
}

fn has_other(f: &Filter) -> bool {
    match f {
        Filter::Other => true,
        Filter::And(v) => v.iter().any(has_other),
        _ => false,
    }
}

/// Marks, in [`Builder::named`], that a move of this grammar happened earlier in the text.
const MOVED: &str = "\u{1}zone move";

fn moved_before(b: &Builder) -> bool {
    b.named.iter().any(|(n, _)| n == MOVED) || matches!(b.it, Sel::Var(vars::IT))
}

/// "Each player returns all ... from their graveyard": every player's cards move at the
/// same time (CR 608.2e), each under its owner's control. `rel`: the players the
/// iteration is over, as a relation (`None` for every player). Returns `None` if the
/// filter refers to the iterated player in a way that can't be rewritten.
fn owners_filter(f: &Filter, rel: &Option<PlayerRel>) -> Option<Filter> {
    let out = match f {
        Filter::OwnedBy(PlayerRel::Iterated) => match rel {
            Some(r) => Filter::OwnedBy(r.clone()),
            None => Filter::Any,
        },
        Filter::And(v) => Filter::and(
            v.iter()
                .map(|x| owners_filter(x, rel))
                .collect::<Option<Vec<_>>>()?,
        ),
        Filter::Or(v) => Filter::Or(
            v.iter()
                .map(|x| owners_filter(x, rel))
                .collect::<Option<Vec<_>>>()?,
        ),
        f => {
            if format!("{f:?}").contains("Iterated") {
                return None;
            }
            f.clone()
        }
    };
    Some(out)
}

/// Whether a filter names objects owned by a player other than the one acting.
fn owned_by_other(f: &Filter) -> bool {
    match f {
        Filter::OwnedBy(PlayerRel::Opponent | PlayerRel::Defending | PlayerRel::Target(_)) => true,
        Filter::And(v) => v.iter().any(owned_by_other),
        _ => false,
    }
}

/// The zone and owner parts of a filter ("from your graveyard").
fn location_parts(f: &Filter) -> Vec<Filter> {
    match f {
        Filter::InZone(_) | Filter::OwnedBy(_) | Filter::OwnedByPlayer(_) => vec![f.clone()],
        Filter::Or(v) if v.iter().all(|x| matches!(x, Filter::InZone(_))) => vec![f.clone()],
        Filter::And(v) => v.iter().flat_map(location_parts).collect(),
        _ => vec![],
    }
}

/// Items joined by "and", ", and", ",", "and/or". A zone named once for the whole list
/// ("up to one target creature card and up to one target land card from your
/// graveyard") applies to each item that names cards without one.
fn items(s: &str, b: &mut Builder, subject: &Subject) -> Option<(Vec<Item>, String)> {
    let (first, mut rest) = item(s, b, subject)?;
    let mut out = vec![first];
    let mut and_or = false;
    loop {
        let t = rest.trim_start().to_string();
        let next = [", and/or ", ", and ", ", ", "and/or ", "and "]
            .iter()
            .find_map(|sep| t.strip_prefix(sep).map(|r| (sep.contains("and/or"), r)));
        let Some((is_and_or, n)) = next else { break };
        and_or |= is_and_or;
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        match item(n, b, subject) {
            Some((i, r)) => {
                out.push(i);
                rest = r;
            }
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
                break;
            }
        }
    }
    // A zone after the whole list.
    let mut shared: Option<(Filter, bool)> = None;
    if let Some((f, other, r)) = from_zone(&rest, b, subject) {
        shared = Some((f, other));
        rest = r.to_string();
    } else if out.len() > 1 {
        // The last item's zone ("..., and up to one target sorcery card from your
        // graveyard"), read by the target parser with it.
        let last = out.last().map(|i| item_filter(i, b)).flatten();
        if let Some(f) = last {
            let loc = location_parts(&f);
            if !loc.is_empty() {
                shared = Some((Filter::and(loc), owned_by_other(&f)));
            }
        }
    }
    if let Some((loc, other)) = shared {
        locate(&mut out, &loc, other, b);
    }
    // "a creature card and/or a land card": either or both, so each is up to that many.
    if and_or {
        for i in out.iter_mut() {
            if let Kind::Chosen { qty, .. } = &mut i.kind {
                if let Qty::Exactly(n) = qty {
                    *qty = Qty::UpTo(n.clone());
                }
            }
        }
    }
    Some((out, rest))
}

/// Applies a zone named once for several items to those that don't name one.
fn locate(items: &mut [Item], loc: &Filter, other: bool, b: &mut Builder) {
    for i in items.iter_mut() {
        let Some(f) = item_filter(i, b) else { continue };
        if located(&f) {
            continue;
        }
        let nf = Filter::and(vec![f, loc.clone()]);
        set_item_filter(i, nf, b);
        i.others_zone |= other;
    }
}

/// Whether every item that names cards says where they are.
fn all_located(items: &[Item], b: &Builder) -> bool {
    items
        .iter()
        .all(|i| item_filter(i, b).is_none_or(|f| !names_cards(&f) || located(&f)))
}

fn item_filter(i: &Item, b: &Builder) -> Option<Filter> {
    match &i.kind {
        Kind::Fixed(_) => None,
        Kind::Target(slot) => match b.targets.get(*slot as usize).map(|t| &t.what) {
            Some(TargetKind::Object(f)) => Some(f.clone()),
            _ => None,
        },
        Kind::Chosen { filter, .. } => Some(filter.clone()),
    }
}

fn set_item_filter(i: &mut Item, f: Filter, b: &mut Builder) {
    match &mut i.kind {
        Kind::Fixed(_) => {}
        Kind::Target(slot) => {
            if let Some(t) = b.targets.get_mut(*slot as usize) {
                t.what = TargetKind::Object(f);
            }
        }
        Kind::Chosen { filter, .. } => *filter = f,
    }
}

/// The selection an item makes, and the effects that must come before the move (a random
/// pick).
fn item_sel(i: &Item, b: &Builder) -> Option<(Sel, Option<Effect>)> {
    Some(match &i.kind {
        Kind::Fixed(s) => (s.clone(), None),
        Kind::Target(slot) => (Sel::Target(*slot), None),
        Kind::Chosen {
            qty,
            filter,
            random,
            chooser,
        } => {
            // "another creature card from your graveyard": other than one an earlier move
            // of the text took ("Return a creature card ... to the battlefield, then
            // return another creature card ... to your hand"). After other instructions
            // (a sacrifice) "another" means something this grammar doesn't track.
            if names_cards(filter) && has_other(filter) && !moved_before(b) {
                return None;
            }
            let filter = filter.clone();
            // "of an opponent's choice": the controller chooses the opponent first.
            if matches!(chooser, PlayerRef::ChosenOpponent) {
                let (sel, pre) = match (qty, random) {
                    (Qty::Exactly(n), false) => (
                        Sel::Choose {
                            chooser: chooser.clone(),
                            filter,
                            count: n.clone(),
                            up_to: false,
                            store: None,
                        },
                        Effect::Choose {
                            who: PlayerRef::You,
                            kind: ChoiceKind::Opponent,
                        },
                    ),
                    _ => return None,
                };
                return Some((sel, Some(pre)));
            }
            match (qty, random) {
                (Qty::All, false) => (Sel::All(filter), None),
                (Qty::Exactly(n), true) => {
                    (Sel::Var(RANDOM_PICK), Some(random_pick(filter, n.clone())))
                }
                (_, true) => return None,
                (Qty::Exactly(n), false) => (
                    Sel::Choose {
                        chooser: chooser.clone(),
                        filter,
                        count: n.clone(),
                        up_to: false,
                        store: None,
                    },
                    None,
                ),
                (Qty::UpTo(n), false) => (
                    Sel::Choose {
                        chooser: chooser.clone(),
                        filter,
                        count: n.clone(),
                        up_to: true,
                        store: None,
                    },
                    None,
                ),
                (Qty::AnyNumber, false) => (
                    Sel::Choose {
                        chooser: chooser.clone(),
                        count: Value::Count(filter.clone()),
                        filter,
                        up_to: true,
                        store: None,
                    },
                    None,
                ),
            }
        }
    })
}

/// The destination zone: "the battlefield", "its owner's hand", "your hand", "their
/// owners' graveyards", "the top of its owner's library".
fn dest_zone(s: &str) -> Option<(Destination, &str)> {
    let s = s.trim_start();
    for p in ["to the battlefield", "onto the battlefield"] {
        if let Some(r) = strip_word(s, p) {
            return Some((Destination::battlefield(), r));
        }
    }
    for p in ["to the top of ", "on top of "] {
        if let Some(r) = s.strip_prefix(p) {
            let (_, r) = owners(r)?;
            let r = strip_word(r, "library").or_else(|| strip_word(r, "libraries"))?;
            return Some((Destination::library_top(), r));
        }
    }
    for p in ["to the bottom of ", "on the bottom of "] {
        if let Some(r) = s.strip_prefix(p) {
            let (_, r) = owners(r)?;
            let r = strip_word(r, "library").or_else(|| strip_word(r, "libraries"))?;
            return Some((Destination::library_bottom(), r));
        }
    }
    let r = s.strip_prefix("to ").or_else(|| s.strip_prefix("into "))?;
    let (_, r) = owners(r)?;
    for (p, z) in [
        ("hands", ZoneKind::Hand),
        ("hand", ZoneKind::Hand),
        ("graveyards", ZoneKind::Graveyard),
        ("graveyard", ZoneKind::Graveyard),
    ] {
        if let Some(r) = strip_word(r, p) {
            return Some((Destination::zone(z), r));
        }
    }
    None
}

/// The owner words before a hand/graveyard/library: every card goes to its owner's
/// (CR 400.3), so "your", "their", "its owner's", "their owners'" all mean that.
fn owners(s: &str) -> Option<(bool, &str)> {
    for p in [
        "its owner's ",
        "their owner's ",
        "their owners' ",
        "his owner's ",
        "her owner's ",
        "its owners' ",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((true, r));
        }
    }
    for p in ["your ", "their ", "his ", "her "] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((false, r));
        }
    }
    None
}

/// "with a finality counter on it", "with two additional +1/+1 counters on it", "with a
/// hexproof counter and an indestructible counter on it".
fn with_counters(s: &str) -> Option<(Vec<(CounterKind, Value)>, &str)> {
    let r = s.trim_start().strip_prefix("with ")?;
    let mut out = Vec::new();
    let mut r = r;
    loop {
        let (n, r2) = parse_number(r)?;
        let r2 = r2.trim_start();
        let r2 = r2.strip_prefix("additional ").unwrap_or(r2);
        let (kind, r2) = crate::oracle::costs::counter_kind(r2)?;
        let r2 = r2.trim_start();
        let r2 = r2
            .strip_prefix("counters")
            .or_else(|| r2.strip_prefix("counter"))?;
        out.push((kind, n));
        if let Some(x) = r2
            .strip_prefix(" and ")
            .or_else(|| r2.strip_prefix(", and "))
            .or_else(|| r2.strip_prefix(", "))
        {
            r = x;
            continue;
        }
        let r2 = r2.strip_prefix(" on ")?;
        for p in ["it", "them", "him", "her", "each of them"] {
            if let Some(x) = strip_word(r2, p) {
                return Some((out, x));
            }
        }
        return None;
    }
}

/// Battlefield modifiers in any order. `owner_of`: the moved objects (whose owners "its
/// owner's control" names); `others_zone`: "under their control" means the owners'.
fn modifiers<'a>(
    mut s: &'a str,
    to: &mut Destination,
    owner_of: &Sel,
    subject: &Subject,
    others_zone: bool,
) -> &'a str {
    loop {
        let t = s.trim_start();
        if let Some(r) = strip_word(t, "tapped and attacking") {
            if to.zone != ZoneKind::Battlefield {
                return s;
            }
            to.tapped = true;
            to.attacking = true;
            s = r;
        } else if let Some(r) = strip_word(t, "tapped") {
            if to.zone != ZoneKind::Battlefield {
                return s;
            }
            to.tapped = true;
            s = r;
        } else if let Some(r) = strip_word(t, "transformed") {
            if to.zone != ZoneKind::Battlefield {
                return s;
            }
            to.transformed = true;
            s = r;
        } else if let Some(r) = strip_word(t, "face down and tapped")
            .map(|r| (r, true))
            .or_else(|| strip_word(t, "face down").map(|r| (r, false)))
            .filter(|_| !super::zz_probe_ps::disabled())
        {
            // "Return it to the battlefield face down": a face-down 2/2 creature with no
            // text, unless the effect lists other characteristics (CR 708.2a, 708.3).
            if to.zone != ZoneKind::Battlefield {
                return s;
            }
            to.face_down = true;
            to.tapped |= r.1;
            s = r.0;
        } else if let Some(r) = strip_word(t, "under your control") {
            to.controller = Some(PlayerRef::You);
            s = r;
        } else if let Some(r) = [
            "under its owner's control",
            "under their owners' control",
            "under their owner's control",
            "under his owner's control",
            "under her owner's control",
        ]
        .iter()
        .find_map(|p| strip_word(t, p))
        {
            to.controller = Some(PlayerRef::OwnerOf(Box::new(owner_of.clone())));
            s = r;
        } else if let Some(r) = strip_word(t, "under their control") {
            if others_zone {
                to.controller = Some(PlayerRef::OwnerOf(Box::new(owner_of.clone())));
            } else if !subject.is_you() {
                to.controller = Some(subject.who.clone());
            } else {
                return s;
            }
            s = r;
        } else if let Some((c, r)) = with_counters(t) {
            to.with_counters.extend(c);
            s = r;
        } else {
            return s;
        }
    }
}

/// "Return a Pirate card from your graveyard to your hand, then do the same for Vampire,
/// Dinosaur, and Merfolk.", "Return all non-Aura enchantment cards from your graveyard to
/// the battlefield, then do the same for Aura cards.": the instruction is repeated for
/// each kind in turn, the kind replacing the words before "card(s)".
fn p_do_the_same_for(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, kinds) = l.split_once(", then do the same for ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    // The kind words in the first instruction: between the quantity and "card(s)".
    let (verb_q, after) = [
        "return a ",
        "return an ",
        "return all ",
        "put a ",
        "put an ",
        "put all ",
    ]
    .iter()
    .find_map(|p| first.strip_prefix(p).map(|r| (*p, r)))?;
    let noun_at = after.find(" cards ").or_else(|| after.find(" card "))?;
    let tail = &after[noun_at..];
    let mut out = vec![parse_move(first, b)?];
    let kinds = kinds.replace(", and ", ", ").replace(" and ", ", ");
    for k in kinds.split(", ") {
        let k = k.trim();
        let k = k
            .strip_suffix(" cards")
            .or_else(|| k.strip_suffix(" card"))
            .unwrap_or(k);
        if k.is_empty() {
            restore(b);
            return None;
        }
        let text = format!("{verb_q}{k}{tail}");
        let Some(e) = parse_move(&text, b) else {
            restore(b);
            return None;
        };
        out.push(e);
    }
    if b.targets.len() != saved.0 {
        restore(b);
        return None;
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "zone-move grammar: [move], then do the same for [kinds]", priority: 970, parse: p_do_the_same_for } }

/// "Choose a card at random that was exiled with ~.", "choose a creature card exiled with
/// ~", "Choose target card exiled with ~.": the chosen card is what "it" and "that card"
/// mean afterwards.
fn p_choose_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let subject = Subject {
        who: PlayerRef::You,
        each: None,
        may: false,
    };
    let r = r.replace(" that was exiled with ", " exiled with ");
    let res = (|| {
        if is_target_phrase(&r) {
            let (it, rest) = target_item(&r, b, &subject)?;
            let Kind::Target(slot) = it.kind else {
                return None;
            };
            if !end(rest.trim()).is_empty() {
                return None;
            }
            // An object target ("choose target opponent" is another pattern's).
            if !matches!(
                b.targets.get(slot as usize).map(|t| &t.what),
                Some(TargetKind::Object(_))
            ) {
                return None;
            }
            b.it = Sel::Target(slot);
            return Some(Effect::Noop);
        }
        let (qty, r2) = quantity(&r)?;
        let Qty::Exactly(n) = qty else { return None };
        let d = described(r2, b, &subject)?;
        // Cards in a zone (choosing permanents is another pattern's).
        if !end(d.rest.trim()).is_empty()
            || !names_cards(&d.filter)
            || !located(&d.filter)
            || d.opponents_choice
        {
            return None;
        }
        // The chosen cards are what the instruction was about. They're added to
        // [`CHOSEN_CARDS`], so that when several players each choose one ("Each opponent
        // chooses a creature card in their graveyard. Put those cards onto the
        // battlefield ...", CR 101.4) every player's choice is kept (see
        // `simultaneous.rs`: a variable an instruction adds to is shared). Only one such
        // choice per ability, so the variable holds nothing else.
        if b.named.iter().any(|(n, _)| n == CHOSE) {
            return None;
        }
        let pick = if d.random {
            Some(random_pick(d.filter.clone(), n.clone()))
        } else {
            None
        };
        let chosen = match pick {
            Some(_) => Sel::Var(RANDOM_PICK),
            None => Sel::Choose {
                chooser: PlayerRef::You,
                filter: d.filter,
                count: n,
                up_to: false,
                store: None,
            },
        };
        let store = Effect::Store {
            var: CHOSEN_CARDS,
            sel: Sel::Union(vec![Sel::Var(CHOSEN_CARDS), chosen]),
        };
        let e = match pick {
            Some(p) => Effect::seq(vec![p, store]),
            None => store,
        };
        b.named.push((CHOSE.to_string(), Sel::Var(CHOSEN_CARDS)));
        b.it = Sel::Var(CHOSEN_CARDS);
        Some(e)
    })();
    if res.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    res
}

inventory::submit! { EffectPattern { name: "zone-move grammar: choose a [card] exiled with ~", priority: 970, parse: p_choose_card } }

/// The cards chosen with [`p_choose_card`] (every choosing player's).
const CHOSEN_CARDS: Var = vars::USER + 7405;
/// Marks, in [`Builder::named`], that [`p_choose_card`] chose cards earlier in the text.
const CHOSE: &str = "\u{1}zone move: chose cards";

/// "Choose a card exiled with ~. You may play that card this turn." (Muse Vessel): a
/// permission to play the chosen card (CR 601.2a, 305.1).
fn f_may_play_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if end(l) != "you may play that card this turn" || !matches!(b.it, Sel::Var(CHOSEN_CARDS)) {
        return false;
    }
    let chose = match &*prev {
        Effect::Store {
            var,
            sel: Sel::Union(v),
        } => *var == CHOSEN_CARDS && v.iter().any(|s| matches!(s, Sel::Choose { .. })),
        _ => false,
    };
    if !chose {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::Var(CHOSEN_CARDS),
            duration: Duration::EndOfTurn,
            free: false,
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "zone-move grammar: choose a card ... you may play that card this turn", priority: 970, apply: f_may_play_chosen } }

fn p_move(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let r = parse_move(l, b);
    if r.is_none() {
        restore(b);
    }
    r
}

fn parse_move(l: &str, b: &mut Builder) -> Option<Effect> {
    let (subject, _verb, r) = subject_verb(l, b)?;
    let r = r.as_str();
    // Inverted order: "return to your hand all ...", "put onto the battlefield under your
    // control all ...".
    let (items, mut to, tail) = if r.starts_with("to ") || r.starts_with("onto ") {
        let (mut to, after) = dest_zone(r)?;
        let after = modifiers(after, &mut to, &Sel::None, &subject, false);
        let (items, tail) = items(after, b, &subject)?;
        (items, to, tail)
    } else {
        let (mut items, rest) = items(r, b, &subject)?;
        // "Return that card under your control": to the battlefield.
        let (to, after) = match dest_zone(&rest) {
            Some(x) => x,
            None if rest.trim_start().starts_with("under ") => {
                (Destination::battlefield(), rest.as_str())
            }
            None => return None,
        };
        // "put an artifact or enchantment card onto the battlefield from their hand".
        let after = match from_zone(after, b, &subject) {
            Some((f, other, r)) => {
                locate(&mut items, &f, other, b);
                r
            }
            None => after,
        };
        (items, to, after.to_string())
    };
    if !all_located(&items, b) {
        return None;
    }
    let mut pre = Vec::new();
    let mut sels = Vec::new();
    for i in &items {
        let (s, p) = item_sel(i, b)?;
        sels.push(s);
        pre.extend(p);
    }
    // One random pick at a time (they share their variables).
    let randoms = items
        .iter()
        .filter(|i| matches!(i.kind, Kind::Chosen { random: true, .. }))
        .count();
    if randoms > 1 {
        return None;
    }
    let sel = if sels.len() == 1 {
        sels[0].clone()
    } else {
        Sel::Union(sels.clone())
    };
    let others = items.iter().any(|i| i.others_zone);
    let tail = modifiers(&tail, &mut to, &sel, &subject, others).to_string();
    // "Return all creatures to their owners' hands except for Merfolk, Krakens, ...".
    let (sel, sels, tail) = match except_for(&tail) {
        Some(ex) => {
            let sels: Vec<Sel> = sels
                .iter()
                .map(|s| match s {
                    Sel::All(f) => Some(Sel::All(Filter::and(vec![
                        f.clone(),
                        Filter::not(ex.clone()),
                    ]))),
                    _ => None,
                })
                .collect::<Option<_>>()?;
            let sel = if sels.len() == 1 {
                sels[0].clone()
            } else {
                Sel::Union(sels.clone())
            };
            (sel, sels, String::new())
        }
        None => (sel, sels, tail),
    };
    // "[... from their hand] onto the battlefield from their hand" (the zone after the
    // destination) is read with the objects; anything else left over isn't understood.
    // "Return all cards exiled with ~ to their owner's hand and you lose that much life":
    // as much life as the number of cards moved.
    let (tail, after) = match tail.trim() {
        "and you lose that much life" => (
            String::new(),
            Some(Effect::LoseLife {
                who: PlayerRef::You,
                n: Value::CountSel(Box::new(Sel::Var(vars::IT))),
            }),
        ),
        _ => (tail, None),
    };
    if !end(tail.trim()).is_empty() {
        return None;
    }
    // "Under its owner's control" in the inverted order: the objects named after it.
    if let Some(PlayerRef::OwnerOf(s)) = &to.controller {
        if matches!(**s, Sel::None) {
            to.controller = Some(PlayerRef::OwnerOf(Box::new(sel.clone())));
        }
    }
    // A zone the objects are already in isn't a move.
    for s in &sels {
        if let Sel::All(f) | Sel::Choose { filter: f, .. } = s {
            if f.zone() == Some(to.zone) && to.zone != ZoneKind::Battlefield {
                return None;
            }
        }
    }
    // The player putting them onto the battlefield controls them (CR 110.2a).
    if to.zone == ZoneKind::Battlefield && to.controller.is_none() && !subject.is_you() {
        to.controller = Some(subject.who.clone());
    }
    // Each player's cards, all of them: one simultaneous move.
    if let Some(e) = each_players_all(&subject, &sels, &pre, &to) {
        b.it = Sel::Var(vars::IT);
        note_moved(b);
        return Some(e);
    }
    pre.push(Effect::Move { what: sel, to });
    pre.extend(after);
    let mut e = Effect::seq(pre);
    if subject.may {
        e = Effect::May {
            who: subject.who.clone(),
            effect: Box::new(e),
        };
    }
    if let Some(each) = subject.each.clone() {
        e = Effect::ForEachPlayer {
            who: each,
            effect: Box::new(e),
        };
    }
    // "It" afterwards: the objects in their new zone (CR 400.7).
    b.it = Sel::Var(vars::IT);
    note_moved(b);
    Some(e)
}

/// "Each player returns all creature cards from their graveyard to the battlefield": all
/// the players' cards move at the same time (CR 608.2e), each player's under their own
/// control.
fn each_players_all(
    subject: &Subject,
    sels: &[Sel],
    pre: &[Effect],
    to: &Destination,
) -> Option<Effect> {
    let each = subject.each.as_ref()?;
    if subject.may || !pre.is_empty() {
        return None;
    }
    let rel = match each {
        PlayerRef::EachPlayer => None,
        PlayerRef::EachOpponent => Some(PlayerRel::Opponent),
        PlayerRef::EachOtherPlayer => Some(PlayerRel::NotYou),
        _ => return None,
    };
    if !matches!(to.controller, None | Some(PlayerRef::Iterated)) {
        return None;
    }
    let mut out = Vec::new();
    for s in sels {
        let Sel::All(f) = s else { return None };
        // Only cards each player owns ("from their graveyard", "they own").
        if !format!("{f:?}").contains("OwnedBy(Iterated)") {
            return None;
        }
        out.push(Sel::All(owners_filter(f, &rel)?));
    }
    let what = if out.len() == 1 {
        out.pop()?
    } else {
        Sel::Union(out)
    };
    let mut to = to.clone();
    if to.zone == ZoneKind::Battlefield {
        to.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
    }
    Some(Effect::Move { what, to })
}

/// "except for Krakens, Leviathans, Octopuses, and Serpents", "except for Giants, Wizards,
/// and lands": the objects of those kinds stay.
fn except_for(s: &str) -> Option<Filter> {
    let r = s.trim_start().strip_prefix("except for ")?;
    let (f, plural, rest) = parse_object_phrase(r)?;
    (plural && end(rest.trim()).is_empty()).then_some(f)
}

/// "[objects] returned this way" after a move of this grammar ("Return any number of
/// Mountains you control to their owner's hand. ~ deals damage to target creature equal to
/// twice the number of Mountains returned this way."): the moved objects of that kind,
/// counted in their new zone (CR 400.7).
pub fn returned_this_way(r: &str, b: &Builder) -> Option<(Value, String)> {
    let i = r.find(" returned this way")?;
    let (noun, rest) = (&r[..i], &r[i + " returned this way".len()..]);
    if !word_end(rest) || !b.named.iter().any(|(n, _)| n == MOVED) {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let f = match f {
        Filter::Permanent | Filter::Card => Filter::In(Box::new(Sel::Var(vars::IT))),
        f => Filter::and(vec![f, Filter::In(Box::new(Sel::Var(vars::IT)))]),
    };
    Some((Value::Count(f), rest.to_string()))
}

/// "Sacrifice them at the beginning of the next end step.", "Return it to your hand at the
/// beginning of the next end step." after a move (whose objects "it" names), with sentences in
/// between that don't change what "it" is ("That creature gains haste."): the moved
/// permanents, stored now (CR 603.7; a permanent that left in the meantime is a new
/// object the delayed trigger can't find, CR 400.7).
fn p_delayed_after_move(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::Var(vars::IT)) || b.sentences == 0 {
        return None;
    }
    let (verb, r, step) = super::damage_removal::delayed_parts(end(l))?;
    let tail = ["it", "them", "that creature", "those creatures"]
        .iter()
        .find_map(|p| strip_word(r, p))?;
    super::damage_removal::delayed_removal(verb, Sel::Var(vars::IT), tail.trim(), step)
}

inventory::submit! { EffectPattern { name: "zone-move grammar: [sacrifice/return] it at the beginning of the next end step (after a move)", priority: 970, parse: p_delayed_after_move } }

/// The cards Living End's first step exiled.
const EXILED_BY_EACH: Var = vars::USER + 7404;

/// "Each player exiles all creature cards from their graveyard, then sacrifices all
/// creatures they control, then puts all cards they exiled this way onto the battlefield."
/// (Living End, Scrap Mastery): no player makes a choice, so each step is performed by all
/// the players at the same time (CR 608.2e), and each player's cards enter under that
/// player's control (CR 110.2a).
fn p_each_player_exile_sacrifice_return(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player exiles all ")?;
    let (cards, r) = r.split_once(" from their graveyard, then sacrifices all ")?;
    let (perms, r) = r.split_once(" they control, then ")?;
    if r != "puts all cards they exiled this way onto the battlefield" {
        return None;
    }
    let (cf, _, t1) = parse_object_phrase(cards)?;
    let (pf, plural, t2) = parse_object_phrase(perms)?;
    if !t1.trim().is_empty()
        || !t2.trim().is_empty()
        || !plural
        || !names_cards(&cf)
        || names_cards(&pf)
    {
        return None;
    }
    let exiled = Sel::Var(EXILED_BY_EACH);
    let mut to = Destination::battlefield();
    to.controller = Some(PlayerRef::OwnerOf(Box::new(exiled.clone())));
    Some(Effect::seq(vec![
        Effect::Exile {
            what: Sel::All(Filter::and(vec![cf, Filter::InZone(ZoneKind::Graveyard)])),
            face_down: false,
            link: false,
        },
        Effect::Store {
            var: EXILED_BY_EACH,
            sel: Sel::Var(vars::IT),
        },
        Effect::SacrificeObjects {
            what: Sel::All(Filter::and(vec![pf, Filter::InZone(ZoneKind::Battlefield)])),
        },
        Effect::Move { what: exiled, to },
    ]))
}

inventory::submit! { EffectPattern { name: "zone-move grammar: each player exiles ..., sacrifices ..., then puts the exiled cards onto the battlefield", priority: 970, parse: p_each_player_exile_sacrifice_return } }

fn note_moved(b: &mut Builder) {
    if !b.named.iter().any(|(n, _)| n == MOVED) {
        b.named.push((MOVED.to_string(), Sel::Var(vars::IT)));
    }
}

/// Where an activated ability whose effect moves its own card functions (CR 113.6m):
/// "Put ~ from exile onto the battlefield" (exile), "Put ~ from your hand onto the
/// battlefield" (the hand), "Return ~ and target land card from your graveyard to the
/// battlefield" (the graveyard). `effect`: the lowercase effect text.
pub fn self_move_zone(effect: &str) -> Option<FunctionZone> {
    for verb in ["return ~", "put ~", "return this card", "put this card"] {
        for (i, _) in effect.match_indices(verb) {
            let after = &effect[i + verb.len()..];
            if after.starts_with(" from exile") {
                return Some(FunctionZone::Exile);
            }
            if after.starts_with(" from your hand") {
                return Some(FunctionZone::Hand);
            }
            if let Some(list) = after.strip_prefix(" and ") {
                // The zone named once for the list ("~ and up to one other target creature
                // card from your graveyard").
                let stop = list
                    .find(" to ")
                    .or_else(|| list.find(" onto "))
                    .unwrap_or(list.len());
                if list[..stop].ends_with(" from your graveyard") {
                    return Some(FunctionZone::Graveyard);
                }
            }
        }
    }
    None
}

/// "Reveal the top card of your library. If it's a land card, put it onto the battlefield.
/// Otherwise, put that card into your hand." (Coiling Oracle): the revealed card that
/// isn't taken goes to the other zone instead of staying on top.
fn f_otherwise_rest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("otherwise, put ") else {
        return false;
    };
    let Some(r) = r
        .strip_prefix("that card ")
        .or_else(|| r.strip_prefix("it "))
    else {
        return false;
    };
    let zone = match r {
        "into your hand" | "into its owner's hand" => ZoneKind::Hand,
        "into your graveyard" | "into its owner's graveyard" => ZoneKind::Graveyard,
        _ => return false,
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        e => Some(e),
    };
    let Some(Effect::Dig {
        n: Value::Const(1),
        take: Value::Const(1),
        take_up_to: false,
        rest_to,
        ..
    }) = last
    else {
        return false;
    };
    if rest_to.zone != ZoneKind::Library || rest_to.position != LibraryPosition::FromTop(0) {
        return false;
    }
    *rest_to = Destination::zone(zone);
    true
}

inventory::submit! { super::FollowupPattern { name: "zone-move grammar: otherwise, put that card into your hand (after a dig)", priority: 150, apply: f_otherwise_rest } }

/// "Each of them enters with an additional -1/-1 counter on it." after a move onto the
/// battlefield: the permanents enter with those counters (CR 122.6, 614.1c).
fn f_enters_with(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = [
        "each of them enters ",
        "it enters ",
        "they enter ",
        "that creature enters ",
        "those creatures enter ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    let Some((c, rest)) = with_counters(r) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let Some(Effect::Move { to, .. }) = last_move_mut(prev) else {
        return false;
    };
    if to.zone != ZoneKind::Battlefield {
        return false;
    }
    to.with_counters.extend(c);
    true
}

inventory::submit! { super::FollowupPattern { name: "zone-move grammar: each of them enters with [counters]", priority: 970, apply: f_enters_with } }

/// The last move of an effect (through sequences, "you may" and conditions).
fn last_move_mut(e: &mut Effect) -> Option<&mut Effect> {
    if matches!(e, Effect::Move { .. }) {
        return Some(e);
    }
    match e {
        Effect::Seq(v) => last_move_mut(v.last_mut()?),
        Effect::May { effect, .. } => last_move_mut(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_move_mut(then),
        _ => None,
    }
}

/// Whether a selection is known to be creatures.
fn creature_sel(what: &Sel, b: &Builder) -> bool {
    fn has_creature(f: &Filter) -> bool {
        match f {
            Filter::Type(crate::types::CardType::Creature) => true,
            Filter::And(v) => v.iter().any(has_creature),
            Filter::Or(v) => !v.is_empty() && v.iter().all(has_creature),
            _ => false,
        }
    }
    match what {
        Sel::Target(slot) => b
            .targets
            .get(*slot as usize)
            .is_some_and(|t| matches!(&t.what, TargetKind::Object(f) if has_creature(f))),
        Sel::All(f) | Sel::Choose { filter: f, .. } => has_creature(f),
        Sel::Union(v) => !v.is_empty() && v.iter().all(|s| creature_sel(s, b)),
        _ => false,
    }
}

/// "It's a 1/1 Spirit creature with flying in addition to its other types.", "Each of them
/// is a 1/1 Spirit in addition to its other types.", "Those creatures are Vampires in
/// addition to their other types.", "It's an artifact in addition to its other types.",
/// "They are 5/5 Elemental creatures in addition to their other types." after a move onto
/// the battlefield: an effect of the move that applies as each permanent enters
/// (CR 611.2e; `Destination::with_mods`).
fn f_enters_in_addition(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    use crate::types::{CardType, Color, ColorSet};
    let l = end(l);
    let Some(r) = [
        "it's ",
        "it is ",
        "that creature is ",
        "each of them is ",
        "they're ",
        "they are ",
        "those creatures are ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    let (x, colors_too) = if let Some(x) = r
        .strip_suffix(" in addition to its other colors and types")
        .or_else(|| r.strip_suffix(" in addition to their other colors and types"))
    {
        (x, true)
    } else if let Some(x) = r
        .strip_suffix(" in addition to its other types")
        .or_else(|| r.strip_suffix(" in addition to their other types"))
    {
        (x, false)
    } else {
        return false;
    };
    let (x, kw) = match x.split_once(" with ") {
        Some((a, k)) => match crate::oracle::effects::keyword_mods(k) {
            Some(m) => (a, m),
            None => return false,
        },
        None => (x, vec![]),
    };
    let x = x
        .strip_prefix("a ")
        .or_else(|| x.strip_prefix("an "))
        .unwrap_or(x);
    let mut pt = None;
    let mut colors = ColorSet::NONE;
    let mut types: Vec<CardType> = Vec::new();
    let mut subtypes: Vec<crate::types::Subtype> = Vec::new();
    for w in x.split(' ') {
        if let Some((p, t)) = w.split_once('/') {
            match (p.parse::<i32>(), t.parse::<i32>()) {
                (Ok(p), Ok(t)) if pt.is_none() => pt = Some((p, t)),
                _ => return false,
            }
        } else if let Some(c) = Color::from_word(w) {
            colors.insert(c);
        } else {
            match head_noun(w) {
                Some(Filter::Type(t)) => types.push(t),
                Some(Filter::Subtype(st)) => subtypes.push(st),
                _ => return false,
            }
        }
    }
    if (colors != ColorSet::NONE) != colors_too || (types.is_empty() && subtypes.is_empty()) {
        return false;
    }
    let Some(Effect::Move { what, to }) = last_move_mut(prev) else {
        return false;
    };
    if to.zone != ZoneKind::Battlefield || !to.with_mods.is_empty() {
        return false;
    }
    // Creature types and a power and toughness only for creatures (CR 205.3d, 208.3).
    let creature = types.contains(&CardType::Creature) || creature_sel(what, b);
    let creature_types = subtypes
        .iter()
        .all(|s| crate::types::subtype_kind(s) == Some(crate::types::SubtypeKind::Creature));
    if (!subtypes.is_empty() && !(creature && creature_types)) || (pt.is_some() && !creature) {
        return false;
    }
    let mut mods = Vec::new();
    if colors_too {
        mods.push(Modification::AddColors(colors));
    }
    if !types.is_empty() {
        mods.push(Modification::AddTypes(types));
    }
    if !subtypes.is_empty() {
        mods.push(Modification::AddSubtypes(subtypes));
    }
    if let Some((p, t)) = pt {
        mods.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
    }
    mods.extend(kw);
    to.with_mods = mods;
    true
}

inventory::submit! { super::FollowupPattern { name: "zone-move grammar: it's a [P/T] [types] in addition to its other types", priority: 970, apply: f_enters_in_addition } }

/// `Modification::Custom` name prefix (layer 6): "has all activated abilities of all
/// [kind] cards exiled with ~" — followed by the card type or subtype word, or nothing
/// for every card. Implemented in `kw/zone_moves.rs`.
pub const ACTIVATED_ABILITIES_OF_EXILED: &str =
    "has all activated abilities of cards exiled with it:";

/// "~ has all activated abilities of all creature cards exiled with it.", "Creatures you
/// control have all activated abilities of all land cards exiled with ~.": the permanents
/// have the abilities of the cards the source's linked abilities exiled that are still in
/// exile (layer 6, CR 613.1f, 607.2a).
fn s_activated_abilities_of_exiled(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l);
    let (who, r) = l
        .split_once(" has all activated abilities of all ")
        .or_else(|| l.split_once(" have all activated abilities of all "))?;
    let r = r
        .strip_suffix(" exiled with ~")
        .or_else(|| r.strip_suffix(" exiled with it"))?;
    let kind = match r {
        "cards" => String::new(),
        _ => {
            let k = r.strip_suffix(" cards")?;
            if crate::types::CardType::from_word(k).is_some() {
                k.to_string()
            } else {
                let title = k[..1].to_uppercase() + &k[1..];
                if !crate::types::is_creature_type(&title) {
                    return None;
                }
                title
            }
        }
    };
    let affected = if who == "~" {
        Filter::Source
    } else {
        let (f, plural, rest) = super::statics::object_phrase(who)?;
        if !plural || !rest.trim().is_empty() {
            return None;
        }
        f
    };
    let st = StaticAbility::new(StaticEffect::Continuous {
        affected,
        mods: vec![Modification::Custom {
            name: SmolStr::new(format!("{ACTIVATED_ABILITIES_OF_EXILED}{kind}")),
            layer: Layer::L6Ability,
        }],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { super::StaticPattern { name: "zone-move grammar: has all activated abilities of cards exiled with ~", priority: 970, parse: s_activated_abilities_of_exiled } }

/// "there are three or more cards exiled with ~", "there are four or more card types
/// among cards exiled with ~" (CR 607.2a).
fn c_exiled_with_count(c: &str) -> Option<Condition> {
    let r = end(c.trim()).strip_prefix("there are ")?;
    let (n, r) = parse_number(r)?;
    n.as_const()?;
    let r = r.trim_start().strip_prefix("or more ")?;
    if !(r.ends_with("exiled with ~") || r.ends_with("exiled with it")) {
        return None;
    }
    let r = r.replace("exiled with it", "exiled with ~");
    let v = super::value_grammar::whole_count(&r, None)?;
    Some(Condition::Compare(v, Cmp::Ge, n))
}

inventory::submit! { super::ConditionPattern { name: "zone-move grammar: there are N or more cards exiled with ~", priority: 970, parse: c_exiled_with_count } }

/// "an opponent has more life than you", "an opponent has more cards in hand than you"
/// (the Pulse cycle's "Then if ..., return ~ to its owner's hand").
fn c_opponent_has_more(c: &str) -> Option<Condition> {
    let f = match end(c.trim()) {
        "an opponent has more life than you" => {
            PlayerFilter::Life(Cmp::Gt, Box::new(Value::LifeTotal(PlayerRef::You)))
        }
        "an opponent has more cards in hand than you" => {
            PlayerFilter::HandSize(Cmp::Gt, Box::new(Value::HandSize(PlayerRef::You)))
        }
        _ => return None,
    };
    Some(Condition::PlayerMatches(PlayerRef::EachOpponent, f))
}

inventory::submit! { super::ConditionPattern { name: "zone-move grammar: an opponent has more life / cards in hand than you", priority: 970, parse: c_opponent_has_more } }

/// `Filter::Custom`: a source that dealt damage this turn ("each creature that dealt
/// damage this turn").
pub const DEALT_DAMAGE_THIS_TURN: &str = "dealt damage this turn (as a source)";

/// `Filter::Custom`: a card milled this turn (CR 701.17), still the object it became.
pub const MILLED_THIS_TURN: &str = "milled this turn";
/// `Filter::Custom`: a card the ability's controller discarded this turn (cycling a card
/// discards it, CR 702.29a), still the object it became.
pub const DISCARDED_BY_YOU_THIS_TURN: &str = "discarded by you this turn";

/// `Filter::Custom`: an object with an odd / even mana value (CR 202.3; 0 is even).
pub const ODD_MANA_VALUE: &str = "odd mana value";
pub const EVEN_MANA_VALUE: &str = "even mana value";

/// "that dealt damage this turn" (as a source), "that isn't a God", "with an odd mana
/// value" after an object noun.
fn f_suffixes<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for (p, name) in [
        ("with an odd mana value", ODD_MANA_VALUE),
        ("with an even mana value", EVEN_MANA_VALUE),
    ] {
        if let Some(r) = strip_word(t, p) {
            return Some((Filter::Custom(SmolStr::new(name)), r));
        }
    }
    if let Some(r) = strip_word(t, "that dealt damage this turn") {
        return Some((Filter::Custom(SmolStr::new(DEALT_DAMAGE_THIS_TURN)), r));
    }

    for p in ["that isn't a ", "that isn't an ", "that aren't "] {
        if let Some(r) = t.strip_prefix(p) {
            let (w, rest) = split_word(r);
            let w = w.trim_end_matches(',');
            let f = head_noun(w)?;
            if !matches!(f, Filter::Type(_) | Filter::Subtype(_)) {
                return None;
            }
            let n = rest.len();
            return Some((Filter::not(f), &r[r.len() - n..]));
        }
    }
    None
}

inventory::submit! { super::FilterSuffixPattern { name: "zone-move grammar: that dealt damage this turn, that isn't a [type]", priority: 970, parse: f_suffixes } }

inventory::submit! { EffectPattern { name: "zone-move grammar: return/put [objects] [from zone] to [zone] [modifiers]", priority: 970, parse: p_move } }

#[cfg(test)]
mod tests {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    /// The compiled abilities of `text` on a card of type `ty`, or None if any of it is
    /// unsupported.
    fn compiled(ty: &str, text: &str) -> Option<String> {
        let tl = TypeLine::parse(ty);
        let ctx = CompileContext {
            card_name: "Test Card",
            full_name: "Test Card",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let r = compile(text, &ctx);
        r.unsupported
            .is_empty()
            .then(|| format!("{:#?}", r.abilities))
    }

    #[test]
    fn zone_moves_are_read_compositionally() {
        for (ty, text) in [
            // Zone-qualified objects.
            ("Artifact", "{T}: Put a card exiled with ~ into its owner's hand."),
            ("Creature", "When ~ dies, put all cards exiled with it onto the battlefield."),
            ("Creature", "When ~ enters, you may put a land card from your hand or graveyard onto the battlefield tapped."),
            ("Sorcery", "Put up to X land cards from your hand and/or graveyard onto the battlefield tapped."),
            ("Sorcery", "Return target card with flashback you own from exile to your hand."),
            ("Sorcery", "Put target face-up exiled card into its owner's graveyard."),
            // History qualifiers.
            ("Sorcery", "Return to your hand all creature cards in your graveyard that were put there from the battlefield this turn."),
            ("Sorcery", "Put onto the battlefield under your control all creature cards in your opponents' graveyards that were put there from the battlefield this turn."),
            ("Sorcery", "Return to your hand all cards in your graveyard that you cycled or discarded this turn."),
            ("Sorcery", "Return each creature that dealt damage this turn to its owner's hand."),
            // Modifiers.
            ("Sorcery", "Return target creature card from your graveyard to the battlefield with two additional +1/+1 counters on it."),
            ("Sorcery", "Return target permanent card from your graveyard to the battlefield with a hexproof counter and an indestructible counter on it."),
            ("Sorcery", "Return target creature card from an opponent's graveyard to the battlefield under their control."),
            ("Sorcery", "Return all creature cards from your graveyard to the battlefield. Each of them is a 1/1 Spirit with flying in addition to its other types."),
            // Lists.
            ("Sorcery", "Return up to one target creature card and up to one target land card from your graveyard to your hand."),
            ("Sorcery", "Return up to one target artifact card, up to one target land card, and up to one target non-Aura enchantment card from your graveyard to the battlefield."),
            ("Sorcery", "Return target creature and target land to their owners' hands."),
            ("Sorcery", "Return target artifact, enchantment, or legendary card from your graveyard to your hand."),
            // Owner-side moves.
            ("Sorcery", "Each player returns all black and all red creature cards from their graveyard to the battlefield."),
            ("Sorcery", "Return all artifacts target player owns to their hand."),
            ("Sorcery", "Return all creatures to their owners' hands except for Krakens, Leviathans, Octopuses, and Serpents."),
            // Bounces and chosen cards.
            ("Creature", "When ~ enters, return another creature you control to its owner's hand."),
            ("Sorcery", "Return two cards at random from your graveyard to your hand."),
            ("Sorcery", "Return a Pirate card from your graveyard to your hand, then do the same for Vampire, Dinosaur, and Merfolk."),
            ("Artifact", "{4}, {T}: Choose a card at random that was exiled with ~. Put that card into its owner's hand."),
            ("Creature", "When ~ enters, return target creature card of an opponent's choice from your graveyard to your hand."),
            ("Creature", "Whenever one or more creature cards are put into your graveyard from your library, put one of them onto the battlefield."),
            ("Creature", "Whenever one or more land cards are put into your graveyard from your library, put them onto the battlefield tapped."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }

    #[test]
    fn unfaithful_zone_moves_are_rejected() {
        for (ty, text) in [
            // A card already in that zone.
            ("Sorcery", "Return a creature card from your graveyard to your graveyard."),
            // Cards with no zone named.
            ("Sorcery", "Return a creature card to your hand."),
            // A spell has no linked abilities to have exiled cards with (CR 607.1).
            ("Sorcery", "Return the exiled cards to their owner's hand."),
            // "Another" relative to something the grammar doesn't track.
            ("Sorcery", "Return another creature card from your graveyard to your hand."),
            // An ability functions in one zone (CR 113.6m).
            ("Creature", "{2}: Return ~ from your graveyard or from exile to the battlefield tapped."),
            // All the cards must come from one graveyard.
            ("Sorcery", "Put two creature cards from a single graveyard onto the battlefield under your control."),
            // Unknown modifier.
            ("Sorcery", "Return target creature card from your graveyard to the battlefield sideways."),
        ] {
            assert!(compiled(ty, text).is_none(), "{text}");
        }
    }
}
