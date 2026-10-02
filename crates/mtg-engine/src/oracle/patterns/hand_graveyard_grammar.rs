//! Card-flow grammar for cards outside the battlefield: a player's actions on cards in
//! hands, graveyards and libraries, read compositionally.
//!
//! ```text
//! action   := [subject] ["may"] verb cards [destination]
//! subject  := "" | "you" | player phrase ("target player", "that player", "each player")
//! verb     := "discard" | "exile" | "reveal" | "put" | "shuffle"   (third person: "-s")
//! cards    := quantity [adjectives] "card(s)" [qualifiers] [zone]
//! quantity := "a" | N | "X" | "up to" N | "any number of" | "one or more" | "all [the]"
//!           | "cards equal to" value
//! zone     := ("from" | "in") (possessive | "a" | "each" | "all") ("hand" | "graveyard"
//!             | "library" | "hands and graveyards" ...)
//! dest     := "on top of" / "on the bottom of" possessive "library" ["in any order"]
//!           | "into" possessive ("hand" | "graveyard" | "library")
//! ```
//!
//! The player who performs the action chooses the cards (CR 701.9a for discarding, CR
//! 701.20a for revealing, CR 608.2d), unless the action takes all of them. "That many"
//! afterwards is the number of cards the action affected, and "[cards] discarded / exiled
//! / revealed this way" are those cards.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// The cards a choice picked before the action is performed on them.
pub const CHOSEN: Var = vars::USER + 6100;
/// The cards the most recent action of this grammar affected ("[cards] exiled this way").
pub const AFFECTED: Var = vars::USER + 6101;
/// The number of cards it affected ("that many").
pub const THAT_MANY: Var = vars::USER + 6102;

/// Marks, in [`Builder::named`], that an action of this grammar happened earlier in the
/// text, so "that many" and "this way" have an antecedent.
const ACTED: &str = "\u{1}hand/graveyard action";

/// How many cards an action takes.
#[derive(Clone, Debug)]
pub enum Qty {
    Exactly(Value),
    UpTo(Value),
    AnyNumber,
    OneOrMore,
    All,
}

/// Cards described by a phrase: how many, and which (the filter includes the zone and
/// the owner where the phrase names them).
#[derive(Clone, Debug)]
pub struct Cards {
    pub qty: Qty,
    pub filter: Filter,
    pub zone: Option<ZoneKind>,
    /// The owner the zone phrase named ("your graveyard", "target player's hand").
    pub owner: Option<PlayerRef>,
    /// The zone phrase named no single player's zone ("from all graveyards").
    pub any_owner: bool,
}

fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';'])
}

/// The quantity at the start of a card phrase.
fn quantity(s: &str) -> Option<(Qty, &str)> {
    let s = s.trim_start();
    if let Some(r) = s
        .strip_prefix("all the ")
        .or_else(|| s.strip_prefix("all "))
    {
        return Some((Qty::All, r));
    }
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((Qty::AnyNumber, r));
    }
    if let Some(r) = s.strip_prefix("one or more ") {
        return Some((Qty::OneOrMore, r));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        return Some((Qty::UpTo(n), r));
    }
    let (n, r) = parse_number(s)?;
    Some((Qty::Exactly(n), r))
}

fn zone_word(s: &str) -> Option<(Vec<ZoneKind>, &str)> {
    for (p, z) in [
        ("hands and graveyards", vec![ZoneKind::Hand, ZoneKind::Graveyard]),
        ("hand and graveyard", vec![ZoneKind::Hand, ZoneKind::Graveyard]),
        ("hands", vec![ZoneKind::Hand]),
        ("hand", vec![ZoneKind::Hand]),
        ("graveyards", vec![ZoneKind::Graveyard]),
        ("graveyard", vec![ZoneKind::Graveyard]),
        ("library", vec![ZoneKind::Library]),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) {
                return Some((z, r));
            }
        }
    }
    None
}

/// The player a third-person possessive ("their", "that player's") refers to.
fn their(b: &Builder, subject: Option<&PlayerRef>) -> Option<PlayerRef> {
    use super::oracle_hardening_referents::is_no_player_referent;
    match subject {
        Some(p) if !matches!(p, PlayerRef::You) => Some(p.clone()),
        _ if !is_no_player_referent(&b.it_player) => Some(b.it_player.clone()),
        _ => None,
    }
}

/// Owner phrase of a zone: "your", "their", "that player's", "target player's", "each
/// player's", "an opponent's", "a", "each", "all", "all opponents'". Returns the owner
/// (None for every player's zone) and the rest.
fn owner_phrase<'a>(
    s: &'a str,
    b: &mut Builder,
    subject: Option<&PlayerRef>,
) -> Option<(Option<Filter>, Option<PlayerRef>, &'a str)> {
    use super::value_grammar::owned_by;
    if let Some(r) = s.strip_prefix("your ") {
        return Some((Some(Filter::OwnedBy(PlayerRel::You)), Some(PlayerRef::You), r));
    }
    for p in ["their ", "his or her "] {
        if let Some(r) = s.strip_prefix(p) {
            let who = their(b, subject)?;
            // "Each player discards all the cards in their hand": each one's own.
            if matches!(who, PlayerRef::EachPlayer | PlayerRef::EachOpponent) {
                return Some((
                    Some(Filter::OwnedBy(PlayerRel::Iterated)),
                    Some(PlayerRef::Iterated),
                    r,
                ));
            }
            return Some((Some(owned_by(&who)), Some(who), r));
        }
    }
    for p in [
        "an opponent's ",
        "all opponents' ",
        "each opponent's ",
        "your opponents' ",
        "opponents' ",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((Some(Filter::OwnedBy(PlayerRel::Opponent)), None, r));
        }
    }
    for p in ["a single ", "a ", "each ", "all ", "each player's ", "all players' "] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((None, None, r));
        }
    }
    if zone_word(s).is_some_and(|(z, _)| z == vec![ZoneKind::Graveyard] && s.starts_with("graveyards")) {
        return Some((None, None, s));
    }
    // "that player's", "target opponent's", "defending player's", "its owner's".
    let (who, rest) = possessive_player(s, b)?;
    let rest = rest.strip_prefix("'s ")?;
    Some((Some(owned_by(&who)), Some(who), rest))
}

/// "that player", "target player", "target opponent", "defending player", "its owner",
/// "its controller", "that creature's controller" (adds a target for "target ...").
fn possessive_player<'a>(s: &'a str, b: &mut Builder) -> Option<(PlayerRef, &'a str)> {
    use super::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
    if let Some(r) = s.strip_prefix("that player") {
        if is_no_player_referent(&b.it_player) {
            return None;
        }
        return Some((b.it_player.clone(), r));
    }
    if let Some(r) = s.strip_prefix("defending player") {
        return Some((PlayerRef::DefendingPlayer, r));
    }
    for (p, owner) in [
        ("its owner", true),
        ("its controller", false),
        ("that creature's controller", false),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if is_no_referent(&b.it) {
                return None;
            }
            let sel = Box::new(b.it.clone());
            return Some((
                if owner {
                    PlayerRef::OwnerOf(sel)
                } else {
                    PlayerRef::ControllerOf(sel)
                },
                r,
            ));
        }
    }
    for (p, pf) in [
        ("target player", PlayerFilter::Any),
        ("target opponent", PlayerFilter::Opponent),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            let it = b.it.clone();
            let slot = b.add_target(TargetSpec::player(pf, p), p);
            b.it = it;
            b.it_player = PlayerRef::Target(slot);
            return Some((PlayerRef::Target(slot), r));
        }
    }
    None
}

/// "from your graveyard", "in target player's hand", "from all graveyards", "from all
/// opponents' hands and graveyards": the zone(s) and the owner filter.
fn zone_phrase<'a>(
    s: &'a str,
    b: &mut Builder,
    subject: Option<&PlayerRef>,
) -> Option<(Vec<ZoneKind>, Option<Filter>, Option<PlayerRef>, &'a str)> {
    let s = s.trim_start();
    let r = s
        .strip_prefix("from ")
        .or_else(|| s.strip_prefix("in "))?;
    let saved = (b.targets.len(), b.it_player.clone());
    let Some((owner, who, r)) = owner_phrase(r, b, subject) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    let Some((zones, rest)) = zone_word(r) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    Some((zones, owner, who, rest))
}

fn has_card_head(f: &Filter) -> bool {
    match f {
        Filter::Card => true,
        Filter::And(v) => v.iter().any(has_card_head),
        Filter::Or(v) => !v.is_empty() && v.iter().all(has_card_head),
        _ => false,
    }
}

/// Removes zone and owner parts of a filter (the shared phrase parser reads "from your
/// graveyard" itself): returns the remaining filter, the zone and the owner.
fn split_zone(f: Filter) -> (Filter, Option<ZoneKind>, Option<Filter>) {
    match f {
        Filter::And(v) => {
            let mut zone = None;
            let mut owner = None;
            let mut rest = Vec::new();
            for x in v {
                match x {
                    Filter::InZone(z) => zone = Some(z),
                    Filter::OwnedBy(_) | Filter::OwnedByPlayer(_) => owner = Some(x),
                    x => rest.push(x),
                }
            }
            (Filter::and(rest), zone, owner)
        }
        Filter::InZone(z) => (Filter::Any, Some(z), None),
        f => (f, None, None),
    }
}

/// A card phrase: "two cards from your graveyard", "any number of black cards in your
/// hand", "all creature cards of that type", "up to two target ..." is not (targets are
/// parsed by the target grammar). `subject` is the player performing the action (for
/// "their").
pub fn cards<'a>(s: &'a str, b: &mut Builder, subject: Option<&PlayerRef>) -> Option<(Cards, String)> {
    let s = s.trim_start();
    // "cards equal to [value] from ...".
    let (qty, r, owned_rest);
    if let Some(r2) = s.strip_prefix("cards equal to ") {
        let (v, rest) = crate::oracle::statics::parse_value_phrase(r2, b)?;
        qty = Qty::Exactly(v);
        owned_rest = format!("cards {}", rest.trim_start());
        r = owned_rest.as_str();
    } else {
        let (q, r2) = quantity(s)?;
        qty = q;
        r = r2;
    }
    if r.trim_start().starts_with("target ") {
        return None;
    }
    let (f, _plural, rest) = parse_object_phrase(r)?;
    if !has_card_head(&f) {
        return None;
    }
    let (mut filter, mut zone, mut owner_f) = split_zone(f);
    let mut owner = owner_f.as_ref().map(|o| match o {
        Filter::OwnedBy(PlayerRel::You) => Some(PlayerRef::You),
        _ => None,
    });
    let mut any_owner = zone.is_some() && owner_f.is_none();
    let mut rest = rest.to_string();
    let mut zones_or: Option<Vec<ZoneKind>> = None;
    if zone.is_none() {
        if let Some((zones, of, who, r2)) = zone_phrase(&rest, b, subject) {
            any_owner = of.is_none();
            owner_f = of;
            owner = Some(who);
            if zones.len() == 1 {
                zone = Some(zones[0]);
            } else {
                zones_or = Some(zones);
            }
            rest = r2.to_string();
        }
    }
    // Qualifiers after the zone ("with four or more card types among them" isn't one).
    let t = rest.trim_start().to_string();
    if t.starts_with("other than ") {
        let probe = format!("cards {}", t.strip_prefix("other than ")?);
        let (f2, _, r2) = parse_object_phrase(&probe)?;
        let f2 = split_zone(f2).0;
        filter = Filter::and(vec![filter, Filter::not(f2)]);
        rest = r2.to_string();
    }
    let mut parts = vec![filter];
    if let Some(z) = zone {
        parts.push(Filter::InZone(z));
    }
    if let Some(zs) = zones_or {
        parts.push(Filter::Or(zs.into_iter().map(Filter::InZone).collect()));
        zone = Some(ZoneKind::Graveyard);
    }
    if let Some(o) = owner_f {
        parts.push(o);
    }
    Some((
        Cards {
            qty,
            filter: Filter::and(parts),
            zone,
            owner: owner.flatten(),
            any_owner,
        },
        rest,
    ))
}

/// The player performing an action and the rest of the clause after the verb, which must
/// be one of `verbs` (imperative/first person) or its third-person form.
fn subject_verb<'a>(
    l: &'a str,
    b: &mut Builder,
    verb: &str,
) -> Option<(PlayerRef, bool, String)> {
    if let Some(r) = l.strip_prefix(&format!("{verb} ")) {
        return Some((PlayerRef::You, false, r.to_string()));
    }
    if let Some(r) = l.strip_prefix(&format!("you {verb} ")) {
        return Some((PlayerRef::You, false, r.to_string()));
    }
    if l.starts_with("you ") {
        return None;
    }
    let (who, rest) = player_ref(l, b)?;
    let r = rest.trim_start().strip_prefix(&format!("{verb}s "))?;
    Some((who, true, r.to_string()))
}

/// The cards the player chooses: all of them, or a choice of the given size.
fn selection(c: &Cards, chooser: PlayerRef) -> Sel {
    let (count, up_to) = match &c.qty {
        Qty::All => return Sel::All(c.filter.clone()),
        Qty::Exactly(v) => (v.clone(), false),
        Qty::UpTo(v) => (v.clone(), true),
        Qty::AnyNumber | Qty::OneOrMore => (
            Value::CountSel(Box::new(Sel::All(c.filter.clone()))),
            true,
        ),
    };
    Sel::Choose {
        chooser,
        filter: c.filter.clone(),
        count,
        up_to,
        store: None,
    }
}

fn is_single_player(p: &PlayerRef) -> bool {
    !matches!(
        p,
        PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
    )
}

/// After an action: its cards are "this way", their number is "that many".
fn record(e: Effect, b: &mut Builder) -> Effect {
    if !b.named.iter().any(|(n, _)| n == ACTED) {
        b.named.push((ACTED.to_string(), Sel::Var(AFFECTED)));
    }
    b.it = Sel::Var(AFFECTED);
    Effect::seq(vec![
        e,
        Effect::StoreValue {
            var: THAT_MANY,
            value: Value::CountSel(Box::new(Sel::Var(vars::IT))),
        },
        Effect::Store {
            var: AFFECTED,
            sel: Sel::Var(vars::IT),
        },
    ])
}

/// Whether an action of this grammar happened earlier in the text.
pub fn acted(b: &Builder) -> bool {
    b.named.iter().any(|(n, _)| n == ACTED)
}

// ---------------------------------------------------------------------------
// Discard
// ---------------------------------------------------------------------------

/// "discard any number of cards", "discard all creature cards", "target player discards
/// all cards of that color", "discard up to two land cards", "discard cards equal to
/// ...".
fn p_discard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, third, r) = subject_verb(l, b, "discard")?;
    // "discard one of them": among the cards the text is about.
    if let Some((Sel::Choose { filter, count, .. }, tail)) = some_of_them(&r, b, &who) {
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Effect::Discard {
            who,
            n: count,
            random: false,
            filter,
        });
    }
    let r = r.strip_suffix(" at random").map_or(r.clone(), str::to_string);
    let (c, rest) = cards(&r, b, Some(&who))?;
    if !end(&rest).is_empty() {
        return None;
    }
    // Only cards in the discarding player's hand (the zone is implied).
    if c.zone.is_some_and(|z| z != ZoneKind::Hand) || c.any_owner {
        return None;
    }
    let _ = third;
    let (adj, _, _) = split_zone(c.filter.clone());
    let e = match &c.qty {
        Qty::Exactly(n) => Effect::Discard {
            who: who.clone(),
            n: n.clone(),
            random: false,
            filter: adj,
        },
        // The whole hand (even a hand of no cards is discarded).
        Qty::All if matches!(adj, Filter::Any | Filter::Card) => Effect::DiscardHand { who: who.clone() },
        // CR 701.9a: all of them; the player chooses none.
        Qty::All => Effect::Discard {
            who: who.clone(),
            n: Value::HandSize(who.clone()),
            random: false,
            filter: adj,
        },
        _ => {
            let per = |p: PlayerRef| {
                let chooser_cards = Cards {
                    filter: Filter::and(vec![
                        adj.clone(),
                        Filter::Card,
                        Filter::InZone(ZoneKind::Hand),
                        Filter::OwnedByPlayer(Box::new(p.clone())),
                    ]),
                    ..c.clone()
                };
                Effect::seq(vec![
                    Effect::Store {
                        var: CHOSEN,
                        sel: selection(&chooser_cards, p.clone()),
                    },
                    Effect::Discard {
                        who: p,
                        n: Value::CountSel(Box::new(Sel::Var(CHOSEN))),
                        random: false,
                        filter: Filter::In(Box::new(Sel::Var(CHOSEN))),
                    },
                ])
            };
            if is_single_player(&who) {
                per(who.clone())
            } else {
                return None;
            }
        }
    };
    Some(record(e, b))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: discard cards", priority: 960, parse: p_discard } }

// ---------------------------------------------------------------------------
// Exile
// ---------------------------------------------------------------------------

/// "exile two cards from your graveyard", "exile all cards from all graveyards", "target
/// opponent exiles all cards from their hand", "exile any number of creature cards from
/// your graveyard".
fn p_exile(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, _third, r) = subject_verb(l, b, "exile")?;
    let (c, rest) = cards(&r, b, Some(&who))?;
    if !end(&rest).is_empty() || c.zone.is_none() {
        return None;
    }
    if !is_single_player(&who) {
        return None;
    }
    let e = Effect::Exile {
        what: selection(&c, who),
        face_down: false,
        link: false,
    };
    Some(record(e, b))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: exile cards", priority: 960, parse: p_exile } }

// ---------------------------------------------------------------------------
// Reveal
// ---------------------------------------------------------------------------

/// "reveal any number of black cards in your hand", "you may reveal a Dinosaur card from
/// your hand", "reveal up to five nonland cards from your hand".
fn p_reveal(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, _third, r) = subject_verb(l, b, "reveal")?;
    if !is_single_player(&who) {
        return None;
    }
    let (c, rest) = cards(&r, b, Some(&who))?;
    if !end(&rest).is_empty() || c.zone != Some(ZoneKind::Hand) {
        return None;
    }
    // A player reveals cards from their own hand.
    match (&c.owner, &who) {
        (Some(PlayerRef::You), PlayerRef::You) => {}
        (Some(o), w) if format!("{o:?}") == format!("{w:?}") => {}
        _ => return None,
    }
    let e = Effect::seq(vec![
        Effect::Store {
            var: crate::kw::reveal_from_hand::REVEALED,
            sel: selection(&c, who),
        },
        Effect::Custom(SmolStr::new(crate::kw::reveal_from_hand::REVEAL_CHOSEN)),
        Effect::Store {
            var: vars::IT,
            sel: Sel::Var(crate::kw::reveal_from_hand::REVEALED),
        },
    ]);
    Some(record(e, b))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: reveal cards from hand", priority: 960, parse: p_reveal } }

// ---------------------------------------------------------------------------
// Put / shuffle
// ---------------------------------------------------------------------------

/// The destination after the cards: "on top of your library", "on the bottom of their
/// library in any order", "into your hand", "into that player's graveyard".
fn destination<'a>(s: &'a str, b: &mut Builder, subject: &PlayerRef) -> Option<(Destination, &'a str)> {
    destination_of(s, b, subject, false)
}

/// [`destination`]; `owner_implied`: the zone is the moved object's owner's whatever the
/// text calls it ("its owner's library", "their library" after "target card from an
/// opponent's graveyard"), as objects only ever go to their owner's hand, library or
/// graveyard (CR 400.3).
fn destination_of<'a>(
    s: &'a str,
    b: &mut Builder,
    subject: &PlayerRef,
    owner_implied: bool,
) -> Option<(Destination, &'a str)> {
    let s = s.trim_start();
    let (to, r) = if let Some(r) = s.strip_prefix("on top of ") {
        (Destination::library_top(), r)
    } else if let Some(r) = s.strip_prefix("on the bottom of ") {
        (Destination::library_bottom(), r)
    } else if let Some(r) = s.strip_prefix("into ") {
        (Destination::zone(ZoneKind::Hand), r)
    } else {
        return None;
    };
    let implied = ["its owner's ", "their owner's ", "their owners' ", "its owners' "]
        .iter()
        .find_map(|p| r.strip_prefix(p))
        .or_else(|| {
            owner_implied
                .then(|| r.strip_prefix("their ").or_else(|| r.strip_prefix("his or her ")))
                .flatten()
        });
    let r = match implied {
        Some(r) => r,
        None => owner_phrase(r, b, Some(subject))?.2,
    };
    let (zones, mut rest) = zone_word(r)?;
    let [zone] = zones[..] else { return None };
    let mut to = to;
    if s.starts_with("into ") {
        to = Destination::zone(zone);
    } else if zone != ZoneKind::Library {
        return None;
    }
    if let Some(r2) = rest.strip_prefix(" in any order") {
        rest = r2;
    } else if let Some(r2) = rest.strip_prefix(" in a random order") {
        to.position = LibraryPosition::BottomRandom;
        rest = r2;
    }
    Some((to, rest))
}

/// "put a card from your hand on the bottom of your library", "put any number of cards
/// from your hand on the bottom of your library", "[player] puts a card from their hand
/// on top of their library", "put all the cards from your graveyard on the bottom of your
/// library in a random order".
fn p_put(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, _third, r) = subject_verb(l, b, "put")?;
    if !is_single_player(&who) {
        return None;
    }
    let (c, rest) = cards(&r, b, Some(&who))?;
    c.zone?;
    let (to, tail) = destination(&rest, b, &who)?;
    if !end(tail).is_empty() || Some(to.zone) == c.zone {
        return None;
    }
    let e = Effect::Move {
        what: selection(&c, who),
        to,
    };
    Some(record(e, b))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: put cards", priority: 960, parse: p_put } }

/// "shuffle a card from your hand into your library", "shuffle any number of cards from
/// your hand into your library", "shuffle all creature cards from target player's
/// graveyard into that player's library".
fn p_shuffle(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, _third, r) = subject_verb(l, b, "shuffle")?;
    if !is_single_player(&who) {
        return None;
    }
    let (c, rest) = cards(&r, b, Some(&who))?;
    c.zone?;
    let r = rest.trim_start().strip_prefix("into ")?;
    let (_, lib_owner, r) = owner_phrase(r, b, Some(&who))?;
    let (zones, tail) = zone_word(r)?;
    if zones != vec![ZoneKind::Library] || !end(tail).is_empty() {
        return None;
    }
    // The cards go into their owner's library (CR 400.3): the library named must be the
    // owner's.
    let library = lib_owner.or(c.owner.clone())?;
    if c.owner.is_none() || format!("{:?}", c.owner) != format!("{:?}", Some(library.clone())) {
        return None;
    }
    let e = Effect::ShuffleIntoLibrary {
        what: selection(&c, who),
        library,
    };
    Some(record(e, b))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: shuffle cards into library", priority: 960, parse: p_shuffle } }

// ---------------------------------------------------------------------------
// "that many"
// ---------------------------------------------------------------------------

/// "draw that many cards", "draws that many cards plus one", "that player loses that much
/// life": after an action of this grammar, its number of cards.
fn p_that_many(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !acted(b) || !l.contains("that many") {
        return None;
    }
    // X must not mean anything else in the clause.
    if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    // "that many cards plus one".
    let (l, plus) = match l.rsplit_once(" plus ") {
        Some((head, n)) if head.contains("that many") => match parse_number(n) {
            Some((Value::Const(k), tail)) if tail.trim().is_empty() => (head, k),
            _ => (l, 0),
        },
        _ => (l, 0),
    };
    let rewritten = l.replace("that many", "x");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::patterns::value_grammar::with_x_defined(true, || {
        parse_clause(&rewritten, b)
    });
    let Some(e) = e else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    let v = if plus == 0 {
        Value::Var(THAT_MANY)
    } else {
        Value::Sum(vec![Value::Var(THAT_MANY), Value::Const(plus)])
    };
    super::r107_numbers::substitute_x(&e, &v)
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: that many", priority: 960, parse: p_that_many } }

// ---------------------------------------------------------------------------
// "this way"
// ---------------------------------------------------------------------------

/// "cards revealed this way", "creature card exiled this way", "cards discarded this
/// way": how many of the cards the earlier action affected match the description (they
/// are counted where they went, CR 400.7).
pub fn this_way_count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    let i = r.find(" this way")?;
    let (head, rest) = (&r[..i], &r[i + " this way".len()..]);
    if !word_end(rest) {
        return None;
    }
    let (noun, verb) = head.rsplit_once(' ')?;
    let var = match verb {
        "discarded" => crate::discard_rules::DISCARDED,
        "revealed" if acted(b) => crate::kw::reveal_from_hand::REVEALED,
        "exiled" if acted(b) => AFFECTED,
        _ => return None,
    };
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !tail.trim().is_empty() || !has_card_head(&f) {
        return None;
    }
    let f = match f {
        Filter::Card => Filter::In(Box::new(Sel::Var(var))),
        f => Filter::and(vec![f, Filter::In(Box::new(Sel::Var(var)))]),
    };
    Some((Value::Count(f), rest.to_string()))
}

/// "[instruction] for each card revealed this way", "you gain 2 life for each creature
/// card exiled this way".
fn p_for_each_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let (clause, thing) = end(l).rsplit_once(" for each ")?;
    if !thing.ends_with(" this way") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (count, tail) = this_way_count(thing, b)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let Some(e) = parse_clause(clause, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    super::damage_removal_foreach::multiply(e, count)
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: for each card [verb] this way", priority: 960, parse: p_for_each_this_way } }

// ---------------------------------------------------------------------------
// "A or B"
// ---------------------------------------------------------------------------

/// "sacrifice an artifact or discard a card", "discard a card or sacrifice a land": the
/// player chooses which instruction to follow; "if you do" afterwards is about the one
/// they followed (whether it was done). Both must be complete instructions for the same
/// player without targets.
fn p_either_or(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let mut start = 0;
    while let Some(i) = l[start..].find(" or ") {
        let at = start + i;
        start = at + 4;
        let (x, y) = (&l[..at], &l[at + 4..]);
        let first = x.split(' ').next()?;
        let second = y.split(' ').next()?;
        let verbs = ["sacrifice", "discard", "exile", "return", "reveal", "tap", "put"];
        if !verbs.contains(&first) || !verbs.contains(&second) {
            continue;
        }
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let ex = crate::oracle::effects::parse_simple(x, b);
        let ey = crate::oracle::effects::parse_simple(y, b);
        match (ex, ey) {
            (Some(ex), Some(ey)) if b.targets.len() == saved.0 => {
                b.it = saved.1;
                return Some(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options: vec![(x.to_string(), ex), (y.to_string(), ey)],
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

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: [instruction] or [instruction]", priority: 960, parse: p_either_or } }

/// Whether `e` ends with an exile instruction (possibly optional or conditional).
fn ends_with_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exile),
        Effect::May { effect, .. } => ends_with_exile(effect),
        _ => false,
    }
}

/// A sentence about the cards the previous sentence exiled ("Exile up to two target cards
/// from graveyards. You gain 2 life for each creature card exiled this way."): those cards
/// are recorded as this grammar's affected cards, then the sentence is read.
fn f_exiled_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains(" exiled this way") || acted(b) || !ends_with_exile(prev) {
        return false;
    }
    b.named.push((ACTED.to_string(), Sel::Var(AFFECTED)));
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = crate::oracle::effects::parse_sentence(l, b) else {
        b.named.retain(|(n, _)| n != ACTED);
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Store {
            var: AFFECTED,
            sel: Sel::Var(vars::IT),
        },
        e,
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "hand/graveyard grammar: [cards] exiled this way", priority: 960, apply: f_exiled_this_way } }

// ---------------------------------------------------------------------------
// Objects the text named earlier: "put that card into their graveyard", "put one of
// them into your graveyard", "reveal it and put it into your hand".
// ---------------------------------------------------------------------------

/// "one of them", "two of those cards", "one of them": a choice among the cards the text
/// is about (the looked-at or revealed cards), made by `chooser`.
fn some_of_them<'a>(s: &'a str, b: &mut Builder, chooser: &PlayerRef) -> Option<(Sel, &'a str)> {
    let (n, r) = parse_number(s)?;
    n.as_const()?;
    let r = r.trim_start().strip_prefix("of ")?;
    let group = ["them", "those cards"]
        .iter()
        .find_map(|p| r.strip_prefix(p).filter(|x| word_end(x)).map(|x| (*p, x)))?;
    let (sel, _) = super::pronoun_groups::plural_object_ref(group.0, b)??;
    Some((
        Sel::Choose {
            chooser: chooser.clone(),
            filter: Filter::In(Box::new(sel)),
            count: n,
            up_to: false,
            store: None,
        },
        group.1,
    ))
}

/// "put that card into their graveyard", "put it on the bottom of that player's
/// library", "put one of them into your graveyard".
fn p_put_object(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, _third, r) = subject_verb(l, b, "put")?;
    if !is_single_player(&who) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (what, rest) = match some_of_them(&r, b, &who) {
        Some((sel, rest)) => (sel, rest.to_string()),
        None => match chosen_permanent(&r, &who) {
            Some(x) => x,
            None => crate::oracle::effects::object_ref(&r, b)?,
        },
    };
    let Some((to, tail)) = destination_of(&rest, b, &who, true) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    if !end(tail).is_empty() || to.zone == ZoneKind::Battlefield {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    Some(Effect::Move { what, to })
}

/// "a creature you control", "two lands you control": permanents the player chooses.
fn chosen_permanent(s: &str, who: &PlayerRef) -> Option<(Sel, String)> {
    let (n, r) = parse_number(s)?;
    n.as_const()?;
    let (f, _, rest) = parse_object_phrase(r)?;
    if super::statics::mentions_other_zones(&f)
        || !super::statics::filter_mentions(&f, &|x| {
            matches!(x, Filter::ControlledBy(PlayerRel::You))
        })
        || !matches!(who, PlayerRef::You)
    {
        return None;
    }
    Some((
        Sel::Choose {
            chooser: who.clone(),
            filter: f,
            count: n,
            up_to: false,
            store: None,
        },
        rest.to_string(),
    ))
}

/// "shuffle target nontoken permanent you control into its owner's library", "shuffle
/// enchanted creature into its owner's library", "that creature's owner shuffles it into
/// their library".
fn p_shuffle_object(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (what, rest) = if let Some(r) = l.strip_prefix("shuffle ") {
        crate::oracle::effects::object_ref(r, b)?
    } else {
        // "[the object's] owner shuffles it into their library".
        let (owner_of, r) = l.split_once("'s owner shuffles ")?;
        let (sel, t) = crate::oracle::effects::object_ref(owner_of, b)?;
        let (sel2, rest) = crate::oracle::effects::object_ref(r, b)?;
        if !t.trim().is_empty() || format!("{sel:?}") != format!("{sel2:?}") {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            return None;
        }
        let rest = rest.trim_start().strip_prefix("into their library")?;
        return end(rest).is_empty().then(|| Effect::ShuffleIntoLibrary {
            what: sel.clone(),
            library: PlayerRef::OwnerOf(Box::new(sel)),
        });
    };
    let ok = ["into its owner's library", "into their owner's library", "into their owners' libraries"]
        .iter()
        .any(|p| rest.trim() == *p);
    if !ok || matches!(what, Sel::Choose { .. }) {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    Some(Effect::ShuffleIntoLibrary {
        library: PlayerRef::OwnerOf(Box::new(what.clone())),
        what,
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: shuffle [object] into its owner's library", priority: 960, parse: p_shuffle_object } }

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: put [object] into [zone]", priority: 960, parse: p_put_object } }

/// "reveal it and put it into your hand", "reveal that card and put it into your hand":
/// the card is revealed (CR 701.20a), then moved.
fn p_reveal_and_put(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("reveal ")?;
    let (obj, put) = r.split_once(" and put ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (sel, tail) = crate::oracle::effects::object_ref(obj, b)?;
    if !tail.trim().is_empty() || b.targets.len() != saved.0 {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    let put = put
        .strip_prefix("it ")
        .or_else(|| put.strip_prefix("that card "))
        .or_else(|| put.strip_prefix("them "))?;
    let (to, tail) = destination(put, b, &PlayerRef::You)?;
    if !end(tail).is_empty() || to.zone != ZoneKind::Hand {
        return None;
    }
    use crate::kw::reveal_from_hand::{REVEALED, REVEAL_CHOSEN};
    Some(Effect::seq(vec![
        Effect::Store {
            var: REVEALED,
            sel: sel.clone(),
        },
        Effect::Custom(SmolStr::new(REVEAL_CHOSEN)),
        Effect::Move { what: sel, to },
    ]))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: reveal it and put it into your hand", priority: 960, parse: p_reveal_and_put } }

// ---------------------------------------------------------------------------
// Statics granting keywords to cards in a hand or graveyard
// ---------------------------------------------------------------------------

/// Keywords that function while their card is in `zone` (and do what the card says
/// there), with whether a granted one may be costless ("its flashback cost is equal to
/// its mana cost": see `kw/flashback.rs`, `kw/mayhem.rs`, `kw/madness.rs`).
fn grantable_in(zone: ZoneKind, k: crate::keywords::KeywordKind) -> Option<bool> {
    use crate::keywords::KeywordKind::*;
    match (zone, k) {
        (ZoneKind::Graveyard, Flashback | Mayhem) => Some(true),
        (ZoneKind::Graveyard, Unearth | Embalm | Eternalize | Scavenge | Retrace) => Some(false),
        (ZoneKind::Hand, Madness) => Some(true),
        (ZoneKind::Hand, Cycling | Ninjutsu) => Some(false),
        _ => None,
    }
}

/// "Each instant and sorcery card in your graveyard has flashback. The flashback cost is
/// equal to that card's mana cost.", "During your turn, each Lesson card in your graveyard
/// has flashback {1}.", "Each creature card in your graveyard that's a Cleric, Rogue,
/// Warrior, and/or Wizard has unearth {1}{B}.", "Each nonland card in your graveyard has
/// mayhem. The mayhem cost is equal to its mana cost.": a static ability of the permanent
/// that grants a keyword to the cards in that zone (layer 6, CR 613.1f), which functions
/// there (CR 113.6).
fn s_zone_keyword_grant(
    l: &str,
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, l) = match l.strip_prefix("during your turn, ") {
        Some(r) => (Some(Condition::YourTurn), r),
        None => (None, l),
    };
    let r = l.strip_prefix("each ")?;
    let (subject, rest) = r.split_once(" has ")?;
    // The keyword, and the sentence about its cost.
    let (granted, cost_sentence) = match rest.split_once(". ") {
        Some((g, c)) => (g, Some(c)),
        None => (rest, None),
    };
    // Subject: "[quality] card in your graveyard [that's a A, B, and/or C]".
    let (head, relative) = match subject.split_once(" that's ") {
        Some((h, rel)) => (h, Some(rel)),
        None => (subject, None),
    };
    let (f, _, tail) = parse_object_phrase(head)?;
    if !tail.trim().is_empty() || !has_card_head(&f) {
        return None;
    }
    let zone = f.zone()?;
    if !matches!(zone, ZoneKind::Graveyard | ZoneKind::Hand) {
        return None;
    }
    let mut parts = vec![f];
    if let Some(rel) = relative {
        let rel = rel
            .strip_prefix("a ")
            .or_else(|| rel.strip_prefix("an "))
            .unwrap_or(rel);
        let norm = rel.replace(", and/or ", ", ").replace(" and/or ", ", ").replace(", or ", ", ");
        let mut alts = Vec::new();
        for w in norm.split(", ") {
            let (f, _, t) = parse_object_phrase(w.trim())?;
            if !t.trim().is_empty() {
                return None;
            }
            alts.push(f);
        }
        parts.push(Filter::Or(alts));
    }
    let kws: Vec<crate::keywords::Keyword> =
        crate::oracle::keywords::parse_keyword_line(granted, ctx)?
            .into_iter()
            .map(|a| match &a.kind {
                AbilityKind::Keyword(k) => Some(k.clone()),
                _ => None,
            })
            .collect::<Option<_>>()?;
    let [kw] = &kws[..] else { return None };
    let costless_ok = grantable_in(zone, kw.kind)?;
    match (&kw.cost, cost_sentence) {
        (Some(_), None) => {}
        (None, Some(c)) if costless_ok => {
            let name = kw.kind.name().to_lowercase();
            let ok = [
                format!("the {name} cost is equal to its mana cost"),
                format!("the {name} cost is equal to that card's mana cost"),
                format!("its {name} cost is equal to its mana cost"),
            ];
            if !ok.iter().any(|o| o == end(c)) {
                return None;
            }
        }
        _ => return None,
    }
    let mut st = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(parts),
        mods: vec![Modification::AddKeyword(kw.clone())],
    });
    st.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { super::StaticPattern { name: "hand/graveyard grammar: cards in a zone have [keyword]", priority: 960, parse: s_zone_keyword_grant } }

// ---------------------------------------------------------------------------
// Whole zones and lists: "exile your hand", "exile all creatures and graveyards",
// "exile any number of target players' graveyards", "exile ~ and all cards from all
// graveyards"
// ---------------------------------------------------------------------------

/// A whole hand or graveyard: "your hand", "that player's graveyard", "target player's
/// graveyard", "graveyards", "all graveyards", "any number of target players'
/// graveyards". Returns the cards in it.
fn whole_zone<'a>(s: &'a str, b: &mut Builder) -> Option<(Filter, &'a str)> {
    let s = s.trim_start();
    for p in ["any number of target players' ", "any number of target opponents' "] {
        if let Some(r) = s.strip_prefix(p) {
            let (zones, rest) = zone_word(r)?;
            let [zone] = zones[..] else { return None };
            let pf = if p.contains("opponents") {
                PlayerFilter::Opponent
            } else {
                PlayerFilter::Any
            };
            let mut spec = TargetSpec::player(pf, p.trim_end());
            spec.min = 0;
            spec.max = Value::Const(99);
            let it = b.it.clone();
            let slot = b.add_target(spec, p.trim_end());
            b.it = it;
            return Some((
                Filter::and(vec![
                    Filter::InZone(zone),
                    Filter::OwnedBy(PlayerRel::Target(slot)),
                ]),
                rest,
            ));
        }
    }
    let saved = (b.targets.len(), b.it_player.clone());
    let (owner, rest) = match s.strip_prefix("all ").or_else(|| s.strip_prefix("each ")) {
        Some(r) if r.starts_with("graveyards") || r.starts_with("hands") || r.starts_with("graveyard") => (None, r),
        _ if s.starts_with("graveyards") || s.starts_with("hands") => (None, s),
        _ => match owner_phrase(s, b, None) {
            Some((Some(o), _, r)) => (Some(o), r),
            _ => {
                b.targets.truncate(saved.0);
                b.it_player = saved.1;
                return None;
            }
        },
    };
    let Some((zones, rest)) = zone_word(rest) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    let zone = if zones.len() == 1 {
        Filter::InZone(zones[0])
    } else {
        Filter::Or(zones.into_iter().map(Filter::InZone).collect())
    };
    let mut parts = vec![Filter::Card, zone];
    parts.extend(owner);
    Some((Filter::and(parts), rest))
}

/// One item of an exile list: a whole zone, cards in a zone, "~", or "all [permanents]
/// [from the battlefield]".
fn exile_item<'a>(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    if let Some((f, rest)) = whole_zone(s, b) {
        return Some((Sel::All(f), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("~") {
        if word_end(r) {
            return Some((Sel::This, r.to_string()));
        }
    }
    let saved = b.targets.len();
    if let Some((c, rest)) = cards(s, b, Some(&PlayerRef::You)) {
        if c.zone.is_some() && matches!(c.qty, Qty::All) {
            return Some((Sel::All(c.filter), rest));
        }
    }
    b.targets.truncate(saved);
    let r = s.strip_prefix("all ")?;
    let (f, _, rest) = parse_object_phrase(r)?;
    if super::statics::mentions_other_zones(&f) {
        return None;
    }
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("from the battlefield").unwrap_or(rest);
    Some((Sel::All(f), rest.to_string()))
}

/// "Exile all creatures and graveyards.", "Exile ~ and all cards from all graveyards.",
/// "Exile all artifacts, creatures, and lands from the battlefield, all cards from all
/// graveyards, and all cards from all hands.", "exile your hand", "exile any number of
/// target players' graveyards": everything named is exiled at once.
fn p_exile_list(l: &str, b: &mut Builder) -> Option<Effect> {
    let mut r = end(l).strip_prefix("exile ")?.to_string();
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut items = Vec::new();
    loop {
        let Some((sel, rest)) = exile_item(&r, b) else {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            return None;
        };
        items.push(sel);
        let t = rest.trim_start();
        if t.is_empty() {
            break;
        }
        r = match t
            .strip_prefix(", and ")
            .or_else(|| t.strip_prefix("and "))
            .or_else(|| t.strip_prefix(", "))
        {
            Some(x) => x.to_string(),
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
                return None;
            }
        };
    }
    let what = if items.len() == 1 {
        items.pop()?
    } else {
        Sel::Union(items)
    };
    Some(record(
        Effect::Exile {
            what,
            face_down: false,
            link: false,
        },
        b,
    ))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: exile zones and lists", priority: 970, parse: p_exile_list } }

/// "sacrifice any number of permanents you control", "sacrifice any number of other
/// permanents": the player chooses which (CR 701.21a); "that many" afterwards is the
/// number sacrificed.
fn p_sacrifice_any(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("sacrifice any number of ")?;
    let (f, _, rest) = parse_object_phrase(r)?;
    if !rest.trim().is_empty() || super::statics::mentions_other_zones(&f) {
        return None;
    }
    let f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
    let e = Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: f.clone(),
                count: Value::CountSel(Box::new(Sel::All(f))),
                up_to: true,
                store: None,
            },
        },
        Effect::SacrificeObjects {
            what: Sel::Var(CHOSEN),
        },
    ]);
    Some(record(e, b))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: sacrifice any number", priority: 960, parse: p_sacrifice_any } }

/// "draw three cards, then discard one of them", "draw two cards, then put one of them
/// on the bottom of your library": "them" are the cards drawn.
fn p_draw_then_of_them(l: &str, b: &mut Builder) -> Option<Effect> {
    let (first, second) = end(l).split_once(", then ")?;
    if !second.contains(" of them") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let draw = crate::oracle::effects::parse_simple(first, b)
        .filter(|e| matches!(e, Effect::Draw { who: PlayerRef::You, .. }));
    let Some(draw) = draw else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    b.it = Sel::Var(vars::REVEALED);
    let Some(then) = crate::oracle::effects::parse_simple(second, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::seq(vec![draw, then]))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: draw, then [verb] N of them", priority: 960, parse: p_draw_then_of_them } }

/// "Reveal the top card of your library. If it's a creature card, put it into your hand.
/// Otherwise, you may put it into your graveyard.": the revealed card goes where the
/// condition says if it qualifies; otherwise its owner may move it (or leave it on top).
fn f_otherwise_may_put_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("otherwise, you may put it ") else {
        return false;
    };
    let Effect::Dig {
        who: PlayerRef::You,
        n: Value::Const(1),
        reveal,
        take: Value::Const(1),
        take_up_to: false,
        filter,
        take_to,
        rest_to,
    } = &*prev
    else {
        return false;
    };
    if rest_to.zone != ZoneKind::Library || !matches!(rest_to.position, LibraryPosition::FromTop(0)) {
        return false;
    }
    let Some((to, tail)) = destination(r, b, &PlayerRef::You) else {
        return false;
    };
    if !end(tail).is_empty() || to.zone == take_to.zone {
        return false;
    }
    let look = Effect::Dig {
        who: PlayerRef::You,
        n: Value::Const(1),
        reveal: *reveal,
        filter: Filter::Any,
        take: Value::Const(0),
        take_up_to: true,
        take_to: take_to.clone(),
        rest_to: rest_to.clone(),
    };
    let it = Sel::Var(vars::IT);
    *prev = Effect::seq(vec![
        look,
        Effect::If {
            cond: Condition::SelMatches(it.clone(), filter.clone()),
            then: Box::new(Effect::Move {
                what: it.clone(),
                to: take_to.clone(),
            }),
            otherwise: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::Move { what: it, to }),
            }),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "hand/graveyard grammar: otherwise, you may put it ...", priority: 89, apply: f_otherwise_may_put_it } }
