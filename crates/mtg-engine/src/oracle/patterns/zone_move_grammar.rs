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
//! targets) are chosen as it's performed (CR 608.2c) by the player performing it; "at
//! random" picks them at random (CR 701.9b-like random selection). Objects that move with
//! one instruction move at the same time (CR 608.2e).

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
        ("hand or graveyard", vec![ZoneKind::Hand, ZoneKind::Graveyard]),
        ("hand and/or graveyard", vec![ZoneKind::Hand, ZoneKind::Graveyard]),
        ("graveyard or hand", vec![ZoneKind::Hand, ZoneKind::Graveyard]),
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
fn from_zone<'a>(s: &'a str, b: &mut Builder, subject: &Subject) -> Option<(Filter, bool, &'a str)> {
    use super::value_grammar::owned_by;
    let s = s.trim_start();
    let r = s.strip_prefix("from ")?;
    if let Some(r) = strip_word(r, "exile") {
        return Some((Filter::InZone(ZoneKind::Exile), false, r));
    }
    let (owner, other, r): (Option<Filter>, bool, &str) = if let Some(r) = r.strip_prefix("your ") {
        (Some(Filter::OwnedBy(PlayerRel::You)), false, r)
    } else if let Some(r) = r.strip_prefix("their ") {
        let who = their(b, subject)?;
        (Some(owned_by(&who)), !matches!(who, PlayerRef::You), r)
    } else if let Some(r) = r
        .strip_prefix("an opponent's ")
        .or_else(|| r.strip_prefix("your opponents' "))
        .or_else(|| r.strip_prefix("opponents' "))
    {
        (Some(Filter::OwnedBy(PlayerRel::Opponent)), true, r)
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
        ("that was put there from the battlefield this turn", "battlefield"),
        ("that were put there from the battlefield this turn", "battlefield"),
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
    for (p, var) in [
        ("milled this way", vars::IT),
        ("put into a graveyard this way", vars::IT),
        ("put into graveyards this way", vars::IT),
        ("discarded this way", crate::discard_rules::DISCARDED),
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
    if head.is_empty() || matches!(head, "another" | "any number of" | "one, two, or three") {
        return true;
    }
    let head = head.strip_prefix("up to ").unwrap_or(head);
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
            "to ", "onto ", "into ", "from ", "on top of", "on the bottom of", ",", "and ",
            "and/or ", "under ", "tapped", "at random",
        ]
        .iter()
        .any(|p| t.starts_with(p))
}

/// Adjectives the shared phrase parser doesn't read: "exiled card" (a card in exile),
/// "face-up exiled card" (CR 406.3).
fn zone_adjectives(s: &str) -> (Vec<Filter>, &str) {
    if let Some(r) = s.strip_prefix("face-up exiled ") {
        return (
            vec![Filter::InZone(ZoneKind::Exile), Filter::not(Filter::FaceDown)],
            r,
        );
    }
    if let Some(r) = s.strip_prefix("exiled ") {
        return (vec![Filter::InZone(ZoneKind::Exile)], r);
    }
    (vec![], s)
}

/// A noun phrase and its qualifiers: "creature card with mana value 2 or less", "card
/// at random from your graveyard", "creature cards in your graveyard that were put there
/// from the battlefield this turn", "land card from your hand or graveyard". Returns the
/// filter, whether the zone is another player's, whether it's "at random", and the rest.
fn described(
    s: &str,
    b: &mut Builder,
    subject: &Subject,
) -> Option<(Filter, bool, bool, String)> {
    let (adj, s) = zone_adjectives(s.trim_start());
    // The shared phrase parser reads some zone phrases ("from your hand") but not others
    // ("from your hand or graveyard"): the zone is read here.
    let cut = s
        .match_indices(" from ")
        .map(|(i, _)| i)
        .find(|&i| {
            let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
            let ok = from_zone(&s[i..], b, subject).is_some();
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            ok
        });
    let (head, tail) = match cut {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };
    // "they control", "their graveyard": the player performing the instruction.
    let outer_player = b.it_player.clone();
    if !subject.is_you() {
        b.it_player = subject.who.clone();
    }
    let parsed = super::value_grammar::objects(head, b);
    b.it_player = outer_player;
    let (filter, rest) = parsed?;
    let mut filter = Filter::and(adj.into_iter().chain([filter]).collect());
    let mut rest = format!("{rest}{tail}");
    let mut random = false;
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
        break;
    }
    Some((filter, others, random, rest))
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
            if let Some(TargetKind::Object(f)) = b.targets.get(slot as usize).map(|t| t.what.clone()) {
                others = owned_by_other(&f);
                // "target card with flashback you own from exile".
                if f.zone().is_none() {
                    if let Some((z, other, r)) = from_zone(&rest, b, subject) {
                        b.targets[slot as usize].what =
                            TargetKind::Object(Filter::and(vec![f, z]));
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
    } else if let Some((n, r)) = parse_number(t).filter(|(_, r)| r.trim_start().starts_with("target ")) {
        (n.clone(), n, r.trim_start())
    } else {
        (Value::c(1), Value::c(1), t)
    };
    let (another, r) = match r.strip_prefix("another target ") {
        Some(r) => (true, r),
        None => (false, r.strip_prefix("target ")?),
    };
    let (filter, others, random, rest) = described(r, b, subject)?;
    if random || !continues(&rest) {
        return None;
    }
    let text = t[..t.len() - rest.trim_start().len()].trim().to_string();
    let mut spec = TargetSpec::object(filter, text.clone());
    spec.min = min;
    spec.max = max;
    let _ = another;
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
                    // The source may have moved since the ability triggered (an Aura
                    // put into a graveyard): the object it became (CR 400.7).
                    let which = match sel {
                        Sel::This => Filter::Custom(SmolStr::new(
                            crate::kw::hand_graveyard_actions::SOURCE_OR_NEXT,
                        )),
                        sel => Filter::In(Box::new(sel)),
                    };
                    return Some((
                        fixed(Sel::All(Filter::and(vec![which, f]))),
                        r.to_string(),
                    ));
                }
                return Some((fixed(sel), rest));
            }
        }
        restore(b);
    }
    let (qty, r) = quantity(s)?;
    let Some((filter, others, random, rest)) = described(r, b, subject) else {
        restore(b);
        return None;
    };
    Some((
        Item {
            kind: Kind::Chosen {
                qty,
                filter,
                random,
                chooser: subject.who.clone(),
            },
            others_zone: others,
        },
        rest,
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
    loop {
        let t = rest.trim_start().to_string();
        let next = [", and/or ", ", and ", ", ", "and/or ", "and "]
            .iter()
            .find_map(|sep| t.strip_prefix(sep));
        let Some(n) = next else { break };
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
        for i in out.iter_mut() {
            let Some(f) = item_filter(i, b) else { continue };
            if located(&f) {
                continue;
            }
            let nf = Filter::and(vec![f, loc.clone()]);
            set_item_filter(i, nf, b);
            i.others_zone |= other;
        }
    }
    // Cards must be somewhere.
    for i in &out {
        if let Some(f) = item_filter(i, b) {
            if names_cards(&f) && !located(&f) {
                return None;
            }
        }
    }
    Some((out, rest))
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
    for p in [
        "to the top of ",
        "on top of ",
    ] {
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
        if let Some(x) = r2.strip_prefix(" and ").or_else(|| r2.strip_prefix(", and ")).or_else(|| r2.strip_prefix(", ")) {
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
        let (items, rest) = items(r, b, &subject)?;
        let (to, after) = dest_zone(&rest)?;
        (items, to, after.to_string())
    };
    let mut pre = Vec::new();
    let mut sels = Vec::new();
    for i in &items {
        let (s, p) = item_sel(i, b)?;
        sels.push(s);
        pre.extend(p);
    }
    if pre.len() > 1 {
        return None;
    }
    let sel = if sels.len() == 1 {
        sels[0].clone()
    } else {
        Sel::Union(sels.clone())
    };
    let others = items.iter().any(|i| i.others_zone);
    let tail = modifiers(&tail, &mut to, &sel, &subject, others).to_string();
    // "[... from their hand] onto the battlefield from their hand" (the zone after the
    // destination) is read with the objects; anything else left over isn't understood.
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
fn each_players_all(subject: &Subject, sels: &[Sel], pre: &[Effect], to: &Destination) -> Option<Effect> {
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

fn note_moved(b: &mut Builder) {
    if !b.named.iter().any(|(n, _)| n == MOVED) {
        b.named.push((MOVED.to_string(), Sel::Var(vars::IT)));
    }
}

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
        r.unsupported.is_empty().then(|| format!("{:#?}", r.abilities))
    }

    /// `ZM_TYPE="Sorcery" ZM_TEXT="..." cargo test -p mtg-engine --lib zone_move_grammar::tests::debug -- --nocapture`
    #[test]
    fn debug() {
        if let Ok(file) = std::env::var("ZM_FILE") {
            for line in std::fs::read_to_string(file).unwrap_or_default().lines() {
                let Some((ty, text)) = line.split_once('|') else { continue };
                let ok = compiled(ty, text).is_some();
                println!("{} {text}", if ok { "OK  " } else { "FAIL" });
            }
        }
        if let Ok(text) = std::env::var("ZM_TEXT") {
            let ty = std::env::var("ZM_TYPE").unwrap_or_else(|_| "Sorcery".into());
            println!("{}", compiled(&ty, &text).unwrap_or_else(|| "UNSUPPORTED".into()));
        }
    }
}
