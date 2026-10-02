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
pub const THAT_MANY: Var = crate::kw::hand_graveyard_actions::THAT_MANY_VAR;

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
    // "the cards in your hand", "each Goblin card milled this way": all of them.
    if let Some(r) = s.strip_prefix("the ") {
        if r.starts_with("cards in ") || r.starts_with("cards from ") {
            return Some((Qty::All, r));
        }
    }
    if let Some(r) = s.strip_prefix("each ") {
        return Some((Qty::All, r));
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
    let (f, rest) = card_qualifiers(f, rest)?;
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
        let (f2, _, r2) = parse_object_phrase(t.strip_prefix("other than ")?)?;
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
    // "target creature's controller reveals ...": that creature is a target.
    for (p, pf) in [
        ("up to one target player ", PlayerFilter::Any),
        ("up to one target opponent ", PlayerFilter::Opponent),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            let mut spec = TargetSpec::player(pf, p.trim_end());
            spec.min = 0;
            let it = b.it.clone();
            let slot = b.add_target(spec, p.trim_end());
            b.it = it;
            b.it_player = PlayerRef::Target(slot);
            let r = r.strip_prefix(&format!("{verb}s "))?;
            return Some((PlayerRef::Target(slot), true, r.to_string()));
        }
    }
    if let Some(r) = l.strip_prefix("target creature's controller ") {
        let r = r.strip_prefix(&format!("{verb}s "))?;
        let (spec, tail) = parse_target("target creature")?;
        if !tail.is_empty() {
            return None;
        }
        let slot = b.add_target(spec, "target creature");
        let who = PlayerRef::ControllerOf(Box::new(Sel::Target(slot)));
        b.it_player = who.clone();
        return Some((who, true, r.to_string()));
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
        // Each of several players acting in turn: each one's own number.
        Effect::Custom(SmolStr::new(
            crate::kw::hand_graveyard_actions::RECORD_THAT_MANY,
        )),
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
    // "You discard your hand", "that player discards their hand": the whole hand (even a
    // hand of no cards is discarded).
    if ["your hand", "their hand", "his or her hand"].contains(&end(&r)) {
        let own = match end(&r) {
            "your hand" => !third,
            _ => third,
        };
        if !own {
            return None;
        }
        return Some(record(Effect::DiscardHand { who }, b));
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
    if !acted(b) || !(l.contains("that many") || l.contains("that much")) {
        return None;
    }
    // X must not mean anything else in the clause.
    if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    if let Some(e) = that_many_part(l, b) {
        return Some(e);
    }
    // "draw that many cards and add that much {R}": each part on its own.
    let (x, y) = l.split_once(" and ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let part = |s: &str, b: &mut Builder| {
        if s.contains("that many") || s.contains("that much") {
            that_many_part(s, b)
        } else {
            parse_clause(s, b)
        }
    };
    match (part(x, b), part(y, b)) {
        (Some(ex), Some(ey)) => Some(Effect::seq(vec![ex, ey])),
        _ => {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            None
        }
    }
}

/// One instruction using "that many" ("draw that many cards plus one") or "that much
/// [mana]" ("add that much {R}").
fn that_many_part(l: &str, b: &mut Builder) -> Option<Effect> {
    let that_many = Value::Custom(SmolStr::new(crate::kw::hand_graveyard_actions::THAT_MANY));
    // "add that much {R}": one mana of that type for each.
    if let Some(r) = l.strip_prefix("add that much {") {
        let e = parse_clause(&format!("add {{{r}"), b)?;
        return match e {
            Effect::AddMana {
                mana: ManaProduction::Fixed(ref syms),
                ..
            } if syms.len() == 1 => scale(e, that_many),
            _ => None,
        };
    }
    if !l.contains("that many") {
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
        that_many
    } else {
        Value::Sum(vec![that_many, Value::Const(plus)])
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
        // The cards the latest draw instruction drew (see `Effect::Draw`).
        "drawn" => vars::REVEALED,
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
/// card exiled this way", "For each creature card exiled this way, you gain 1 life.".
fn p_for_each_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, thing) = match l.strip_prefix("for each ") {
        Some(r) => {
            let (thing, clause) = r.split_once(", ")?;
            (clause, thing)
        }
        None => l.rsplit_once(" for each ")?,
    };
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
    let r = scale(e, count);
    if r.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    r
}

/// `e` performed `count` times over, as one instruction (the number is determined once,
/// CR 608.2h): an amount multiplied, a cost paid that many times (CR 118.12), that many
/// cards chosen at once.
fn scale(e: Effect, count: Value) -> Option<Effect> {
    let times = |k: i32| {
        if k == 1 {
            count.clone()
        } else {
            Value::Mul(Box::new(Value::c(k)), Box::new(count.clone()))
        }
    };
    match e {
        // "Add {C}{C} for each card revealed this way".
        Effect::AddMana {
            who,
            mana: ManaProduction::Fixed(syms),
            restriction,
        } if syms.len() > 1 && syms.iter().all(|s| *s == syms[0]) => Some(Effect::AddMana {
            who,
            mana: ManaProduction::Amount(syms[0], times(syms.len() as i32)),
            restriction,
        }),
        // "Counter target spell unless its controller pays {1} for each card revealed this
        // way": the mana cost paid that many times.
        Effect::PayOptional {
            who,
            cost,
            then,
            otherwise,
        } if cost.parts.is_empty() && cost.mana.is_some() && matches!(*then, Effect::Noop) => {
            Some(Effect::PayOptional {
                who,
                cost: Cost {
                    mana: None,
                    parts: vec![CostPart::Repeated {
                        cost: Box::new(cost),
                        times: count,
                    }],
                },
                then,
                otherwise,
            })
        }
        // "Return an enchantment card from your graveyard to your hand for each card
        // revealed this way": that many cards chosen.
        Effect::Move {
            what:
                Sel::Choose {
                    chooser,
                    filter,
                    count: Value::Const(k),
                    up_to,
                    store,
                },
            to,
        } => Some(Effect::Move {
            what: Sel::Choose {
                chooser,
                filter,
                count: times(k),
                up_to,
                store,
            },
            to,
        }),
        Effect::AsPlayer { who, effect } => Some(Effect::AsPlayer {
            who,
            effect: Box::new(scale(*effect, count)?),
        }),
        // "that player loses 1 life and you gain 1 life": each part.
        Effect::Seq(v) => Some(Effect::Seq(
            v.into_iter()
                .map(|e| scale(e, count.clone()))
                .collect::<Option<_>>()?,
        )),
        e => super::damage_removal_foreach::multiply(e, count),
    }
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: for each card [verb] this way", priority: 960, parse: p_for_each_this_way } }

/// "For each card exiled this way, that player loses 1 life and you gain 1 life.": the
/// whole instruction after the comma is scaled (read as one sentence, before it would be
/// split into separate instructions).
fn f_for_each_this_way_first(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("for each ") else {
        return false;
    };
    let Some((thing, clause)) = r.split_once(", ") else {
        return false;
    };
    if !thing.ends_with(" this way") || !clause.contains(" and ") || !acted(b) {
        return false;
    }
    let Some(e) = p_for_each_this_way(l, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "hand/graveyard grammar: for each card [verb] this way, A and B", priority: 40, apply: f_for_each_this_way_first } }

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
        // "Exile up to two target cards from a single graveyard. If this spell was kicked,
        // instead exile target player's graveyard.": either way.
        Effect::If {
            then, otherwise, ..
        } => ends_with_exile(then) && ends_with_exile(otherwise),
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
        None if r.contains(" from target ") => {
            let (sel, rest) = targets_in_target_players_graveyard(&r, b)?;
            (sel, rest.to_string())
        }
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

/// `clause` (with "x" standing for a number) read with x = `v`.
fn clause_with_value(clause: &str, v: &Value, b: &mut Builder) -> Option<Effect> {
    if clause
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| *w == "x")
        .count()
        != 1
    {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::patterns::value_grammar::with_x_defined(true, || parse_clause(clause, b));
    let Some(e) = e else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    super::r107_numbers::substitute_x(&e, v)
}

/// "draw cards equal to the number of cards discarded this way", "target player draws as
/// many cards as they discarded this way": the cards the earlier action affected.
fn p_draw_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    let (head, thing) = if let Some((h, t)) = l.split_once(" cards equal to the number of ") {
        (h.to_string(), t.to_string())
    } else {
        // "draws as many cards as they discarded this way".
        let (h, t) = l.split_once(" as many cards as ")?;
        let verb = t
            .strip_prefix("they ")
            .or_else(|| t.strip_prefix("you "))
            .or_else(|| t.strip_prefix("that player "))?;
        (h.to_string(), format!("cards {verb}"))
    };
    if !head.ends_with("draw") && !head.ends_with("draws") {
        return None;
    }
    let (v, tail) = this_way_count(&thing, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    clause_with_value(&format!("{head} x cards"), &v, b)
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: draw cards equal to the number [verb] this way", priority: 960, parse: p_draw_this_way } }

/// "X target cards from target player's graveyard", "any number of target artifact cards
/// from target player's graveyard": the player is chosen first (its target slot comes
/// first), and the cards must be in that player's graveyard (CR 115.1, 601.2c).
fn targets_in_target_players_graveyard<'a>(s: &'a str, b: &mut Builder) -> Option<(Sel, &'a str)> {
    for (p, pf) in [
        (" from target player's graveyard", PlayerFilter::Any),
        (" from target opponent's graveyard", PlayerFilter::Opponent),
    ] {
        let Some(i) = s.find(p) else { continue };
        let (head, rest) = (&s[..i], &s[i + p.len()..]);
        if !head.contains("target ") {
            return None;
        }
        let probe = format!("{head} from a graveyard");
        let (mut spec, tail) = parse_target(&probe)?;
        if !tail.trim().is_empty() {
            return None;
        }
        let TargetKind::Object(f) = &spec.what else {
            return None;
        };
        if !has_card_head(f) {
            return None;
        }
        let it = b.it.clone();
        let player = b.add_target(TargetSpec::player(pf, p.trim_start().trim_start_matches("from ")), "target player");
        spec.what = TargetKind::Object(Filter::and(vec![
            f.clone(),
            Filter::OwnedBy(PlayerRel::Target(player)),
        ]));
        b.it = it;
        let slot = b.add_target(spec, head);
        b.it_player = PlayerRef::Target(player);
        return Some((Sel::Target(slot), rest));
    }
    None
}

/// "Exile X target cards from target player's graveyard."
fn p_exile_targets_from_target_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (what, rest) = targets_in_target_players_graveyard(r, b)?;
    if !end(rest).is_empty() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    Some(record(
        Effect::Exile {
            what,
            face_down: false,
            link: false,
        },
        b,
    ))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: exile target cards from target player's graveyard", priority: 960, parse: p_exile_targets_from_target_player } }

/// Qualifiers after a card noun that the shared object phrase doesn't read: "that are
/// black or red" (adjectives), "milled this way" / "discarded this way" (the cards the
/// earlier instruction put into a graveyard, CR 400.7).
fn card_qualifiers(f: Filter, rest: &str) -> Option<(Filter, &str)> {
    let t = rest.trim_start();
    if let Some(r) = t.strip_prefix("that are ") {
        // "black or red": the adjectives before a stand-in noun.
        let (adj, tail) = match r.find(|c: char| c == ',' || c == '.') {
            Some(i) => (&r[..i], &r[i..]),
            None => {
                // "from all graveyards" may follow.
                match r.find(" from ").or_else(|| r.find(" in ")) {
                    Some(i) => (&r[..i], &r[i..]),
                    None => (r, ""),
                }
            }
        };
        let probe = format!("{adj} cards");
        let (g, _, t2) = parse_object_phrase(&probe)?;
        if !t2.trim().is_empty() {
            return None;
        }
        let g = match g {
            Filter::And(v) => Filter::and(v.into_iter().filter(|x| !matches!(x, Filter::Card)).collect()),
            g => g,
        };
        return Some((Filter::and(vec![f, g]), tail));
    }
    for (p, var) in [
        ("milled this way", vars::IT),
        ("put into graveyards this way", vars::IT),
        ("put into a graveyard this way", vars::IT),
        ("discarded this way", crate::discard_rules::DISCARDED),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if !word_end(r) {
                return None;
            }
            return Some((
                Filter::and(vec![
                    f,
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::In(Box::new(Sel::Var(var))),
                ]),
                r,
            ));
        }
    }
    Some((f, rest))
}

/// The cards one of the distribution's steps picked, and all the steps so far.
const PICK: Var = vars::USER + 6103;
const DISTRIBUTED: Var = vars::USER + 6104;
const GROUP: Var = vars::USER + 6105;

/// "Put one of them into your hand, put one of them on the bottom of your library, and
/// exile one of them.", "Put one of those cards into your hand, one into your graveyard,
/// and one on the bottom of your library.": the looked-at cards are distributed, one per
/// destination, each chosen among those not yet distributed.
fn p_distribute(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("put one of ") {
        return None;
    }
    let items: Vec<&str> = l
        .split(", and ")
        .flat_map(|x| x.split(", "))
        .collect();
    if items.len() < 2 {
        return None;
    }
    let mut group: Option<Sel> = None;
    let mut steps = vec![Effect::Store {
        var: DISTRIBUTED,
        sel: Sel::None,
    }];
    let mut exiled = None;
    for (i, item) in items.iter().enumerate() {
        let item = item.trim();
        let (exile, r) = match item.strip_prefix("exile ") {
            Some(r) => (true, r),
            None => (false, item.strip_prefix("put ").unwrap_or(item)),
        };
        // "one of them", "one of those cards", or (after the first) "one".
        let r = r.strip_prefix("one")?;
        let r = match ["of them", "of those cards"]
            .iter()
            .find_map(|p| r.trim_start().strip_prefix(p).filter(|x| word_end(x)))
        {
            Some(r2) => {
                if group.is_none() {
                    let pron = &r.trim_start()[3..r.trim_start().len() - r2.len()];
                    group = Some(super::pronoun_groups::plural_object_ref(pron, b)??.0);
                }
                r2
            }
            None if i > 0 => r,
            None => return None,
        };
        if i == 0 {
            // The group as it is now ("them" may be re-bound by the moves).
            steps.push(Effect::Store {
                var: GROUP,
                sel: group.clone()?,
            });
        }
        let pick = Effect::Store {
            var: PICK,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    Filter::In(Box::new(Sel::Var(GROUP))),
                    Filter::not(Filter::In(Box::new(Sel::Var(DISTRIBUTED)))),
                ]),
                count: Value::Const(1),
                up_to: false,
                store: None,
            },
        };
        let act = if exile {
            if !end(r).is_empty() {
                return None;
            }
            exiled = Some(());
            Effect::Exile {
                what: Sel::Var(PICK),
                face_down: false,
                link: false,
            }
        } else {
            let (to, tail) = destination(r, b, &PlayerRef::You)?;
            if !end(tail).is_empty() {
                return None;
            }
            Effect::Move {
                what: Sel::Var(PICK),
                to,
            }
        };
        steps.push(pick);
        steps.push(Effect::Store {
            var: DISTRIBUTED,
            sel: Sel::Union(vec![Sel::Var(DISTRIBUTED), Sel::Var(PICK)]),
        });
        steps.push(act);
    }
    let _ = exiled;
    Some(Effect::seq(steps))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: put one of them ..., one ..., and one ...", priority: 960, parse: p_distribute } }

/// "You may play the exiled card this turn." after distributing looked-at cards with an
/// exile step (Expressive Iteration): a permission for the card exiled that way.
fn f_play_distributed_exile(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "you may play the exiled card this turn" {
        return false;
    }
    let Effect::Seq(v) = &*prev else {
        return false;
    };
    let Some(i) = v.iter().position(|e| {
        matches!(e, Effect::Exile { what: Sel::Var(PICK), .. })
    }) else {
        return false;
    };
    let mut v = v.clone();
    v.insert(
        i + 1,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::Var(vars::IT),
            duration: Duration::EndOfTurn,
            free: false,
        },
    );
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { super::FollowupPattern { name: "hand/graveyard grammar: you may play the exiled card this turn", priority: 960, apply: f_play_distributed_exile } }

/// "Look at the top card of each player's library.": each library's top card(s), in turn
/// (CR 401.2: looking doesn't move them).
fn p_look_each_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("look at the top ")?;
    let (n, r) = match r.strip_prefix("card of ") {
        Some(r) => (Value::Const(1), r),
        None => {
            let (n, r) = parse_number(r)?;
            (n, r.trim_start().strip_prefix("cards of ")?)
        }
    };
    let who = match r {
        "each player's library" => PlayerRef::EachPlayer,
        "each opponent's library" => PlayerRef::EachOpponent,
        _ => return None,
    };
    let _ = b;
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::Dig {
            who: PlayerRef::Iterated,
            n,
            reveal: false,
            filter: Filter::Any,
            take: Value::Const(0),
            take_up_to: true,
            take_to: Destination::zone(ZoneKind::Hand),
            rest_to: Destination {
                position: LibraryPosition::FromTop(0),
                ..Destination::zone(ZoneKind::Library)
            },
        }),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: look at the top card of each player's library", priority: 960, parse: p_look_each_library } }

/// "Look at its controller's hand.", "look at that player's hand".
fn p_look_at_players_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("look at ")?;
    let who = r.strip_suffix("'s hand")?;
    if who.starts_with("target ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (who, rest) = player_ref(who, b)?;
    if !rest.trim().is_empty() || b.targets.len() != saved.0 {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    Some(Effect::LookAtHand { who })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: look at [player]'s hand", priority: 960, parse: p_look_at_players_hand } }

/// "Draw three cards, untap up to two lands, then discard a card.": three or more
/// instructions in order.
fn p_comma_list_then(l: &str, b: &mut Builder) -> Option<Effect> {
    let (head, last) = end(l).rsplit_once(", then ")?;
    let parts: Vec<&str> = head.split(", ").collect();
    if parts.len() < 2 {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut v = Vec::new();
    for p in parts.iter().chain(std::iter::once(&last)) {
        let first = p.split(' ').next().unwrap_or("");
        // Imperative instructions only (no subject to carry over).
        if !["draw", "untap", "discard", "tap", "scry", "mill", "surveil", "put", "exile", "return", "sacrifice", "shuffle", "create", "gain", "lose"].contains(&first) {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            return None;
        }
        match parse_clause(p, b) {
            Some(e) => v.push(e),
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
                return None;
            }
        }
    }
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: A, B, then C", priority: 960, parse: p_comma_list_then } }

/// "~ gets +2/+0 for every seven cards in your graveyard.": +2/+0 for each complete group
/// of seven (the count divided by seven, rounded down).
fn s_for_every_n(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (head, r) = l.split_once(" for every ")?;
    let (n, thing) = parse_number(r)?;
    let Value::Const(n) = n else { return None };
    if n < 2 {
        return None;
    }
    let rewritten = format!("{head} for each {}", thing.trim_start());
    let mut abilities = crate::oracle_ext::parse_static_ext(&rewritten, text, ctx)?;
    let per_group = |v: &Value| -> Option<Value> {
        Some(match v {
            Value::Const(_) => v.clone(),
            Value::Mul(k, c) if matches!(**k, Value::Const(_)) => {
                Value::Mul(k.clone(), Box::new(Value::Div(c.clone(), n, false)))
            }
            c => Value::Div(Box::new(c.clone()), n, false),
        })
    };
    for a in &mut abilities {
        let a = std::sync::Arc::make_mut(a);
        let AbilityKind::Static(st) = &mut a.kind else {
            return None;
        };
        let StaticEffect::Continuous { mods, .. } = &mut st.effect else {
            return None;
        };
        for m in mods.iter_mut() {
            let Modification::ModifyPT(p, t) = m else {
                return None;
            };
            *m = Modification::ModifyPT(per_group(p)?, per_group(t)?);
        }
    }
    Some(abilities)
}

inventory::submit! { super::StaticPattern { name: "hand/graveyard grammar: +N/+N for every N [things]", priority: 960, parse: s_for_every_n } }

/// After "the number of": counts about cards in hands and graveyards — "cards revealed
/// this way" ([`this_way_count`]), "cards in the hand of the opponent with the most cards
/// in hand", "card types among cards in your opponents' graveyards".
pub fn count_phrase(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = this_way_count(r, b) {
        return Some(v);
    }
    for (p, f) in [
        ("cards in the hand of the opponent with the most cards in hand", PlayerFilter::Opponent),
        ("cards in the hand of the player with the most cards in hand", PlayerFilter::Any),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) {
                return Some((
                    Value::OverPlayers(AggOp::Max, f, Box::new(Value::HandSize(PlayerRef::Iterated))),
                    rest.to_string(),
                ));
            }
        }
    }
    for p in ["card types among cards in ", "card type among cards in "] {
        if let Some(z) = r.strip_prefix(p) {
            let owner = if let Some(rest) = z
                .strip_prefix("your opponents' graveyards")
                .or_else(|| z.strip_prefix("opponents' graveyards"))
            {
                (Filter::OwnedBy(PlayerRel::Opponent), rest)
            } else {
                return None;
            };
            if !word_end(owner.1) {
                return None;
            }
            return Some((
                Value::CardTypesAmong(Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Graveyard),
                    owner.0,
                ])),
                owner.1.to_string(),
            ));
        }
    }
    None
}

/// "there are ten or more cards in a single graveyard": some graveyard has that many.
fn c_single_graveyard(c: &str) -> Option<Condition> {
    // "a graveyard has twenty or more cards in it".
    let n = if let Some(r) = end(c).strip_prefix("a graveyard has ") {
        let (n, r) = parse_number(r)?;
        if r.trim() != "or more cards in it" {
            return None;
        }
        n
    } else {
        let r = end(c).strip_prefix("there are ")?;
        let (n, r) = parse_number(r)?;
        let r = r.trim_start().strip_prefix("or more cards in a single graveyard")?;
        if !r.trim().is_empty() {
            return None;
        }
        n
    };
    Some(Condition::Compare(
        Value::OverPlayers(
            AggOp::Max,
            PlayerFilter::Any,
            Box::new(Value::CardsInGraveyard(PlayerRef::Iterated, Filter::Card)),
        ),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { super::ConditionPattern { name: "hand/graveyard grammar: N or more cards in a single graveyard", priority: 960, parse: c_single_graveyard } }

/// "Exile target creature card from a graveyard that was put there this turn.", "Exile all
/// creature cards in all graveyards that were put there from the battlefield this turn.":
/// the cards in graveyards are limited to those put there this turn (from that zone).
fn p_put_there_this_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    use crate::kw::hand_graveyard_actions::PUT_THERE_THIS_TURN;
    let l = end(l);
    let (clause, from) = [
        (" that was put there this turn", ""),
        (" that were put there this turn", ""),
        (" that was put there from anywhere this turn", ""),
        (" that were put there from anywhere this turn", ""),
        (" that was put there from the battlefield this turn", "battlefield"),
        (" that were put there from the battlefield this turn", "battlefield"),
    ]
    .iter()
    .find_map(|(p, from)| l.contains(p).then(|| (l.replacen(p, "", 1), *from)))?;
    let custom = Filter::Custom(SmolStr::new(format!("{PUT_THERE_THIS_TURN}{from}")));
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut e = parse_clause(&clause, b)?;
    let in_gy = |f: &Filter| f.zone() == Some(ZoneKind::Graveyard);
    let mut patched = 0;
    for t in b.targets[saved.0..].iter_mut() {
        if let TargetKind::Object(f) = &t.what {
            if in_gy(f) {
                t.what = TargetKind::Object(Filter::and(vec![f.clone(), custom.clone()]));
                patched += 1;
            }
        }
    }
    let exile = match &mut e {
        Effect::Seq(v) => v.first_mut(),
        e => Some(e),
    };
    if let Some(Effect::Exile { what: Sel::All(f), .. }) = exile {
        if in_gy(f) {
            *f = Filter::and(vec![f.clone(), custom.clone()]);
            patched += 1;
        }
    }
    if patched != 1 {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: cards put into a graveyard this turn", priority: 960, parse: p_put_there_this_turn } }

/// "~ has all activated abilities of all Elf cards in your graveyard.", "As long as ~ is
/// on the battlefield, it has all activated abilities of all creature cards in all
/// graveyards.": the permanent gains those abilities (layer 6, CR 613.1f), as the cards
/// have them now.
fn s_activated_abilities_of_graveyard_cards(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    use crate::kw::hand_graveyard_actions::ACTIVATED_ABILITIES_OF_GRAVEYARD;
    let l = end(l);
    let r = l
        .strip_prefix("as long as ~ is on the battlefield, it has all activated abilities of all ")
        .or_else(|| l.strip_prefix("~ has all activated abilities of all "))?;
    let (kind, zone) = r.split_once(" cards in ")?;
    let scope = match zone {
        "all graveyards" => "all",
        "your graveyard" => "your",
        _ => return None,
    };
    let known = crate::types::CardType::from_word(kind).is_some()
        || crate::types::is_creature_type(&kind_title(kind));
    if !known {
        return None;
    }
    let name = format!("{ACTIVATED_ABILITIES_OF_GRAVEYARD}{scope}:{}", match crate::types::CardType::from_word(kind) {
        Some(_) => kind.to_string(),
        None => kind_title(kind),
    });
    let st = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::Custom {
            name: SmolStr::new(name),
            layer: Layer::L6Ability,
        }],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

/// "elf" → "Elf".
fn kind_title(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

inventory::submit! { super::StaticPattern { name: "hand/graveyard grammar: has all activated abilities of cards in graveyards", priority: 960, parse: s_activated_abilities_of_graveyard_cards } }

/// "draw half X cards, rounded down", "you gain half X life and draw half X cards" (with
/// "Round down each time." following): X halved, rounded as the text says (CR 107.1a).
fn p_half_x(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("half x ") {
        return None;
    }
    let (body, up) = if let Some(h) = l.strip_suffix(", rounded down") {
        (h, false)
    } else if let Some(h) = l.strip_suffix(", rounded up") {
        (h, true)
    } else if crate::oracle::raw_text()
        .to_lowercase()
        .contains(&format!("{l}. round down each time"))
    {
        (l, false)
    } else {
        return None;
    };
    // Every X is halved.
    let rewritten = body.replace("half x ", "x ");
    if rewritten.contains("half ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = parse_clause(&rewritten, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    super::r107_numbers::substitute_x(&e, &Value::Div(Box::new(Value::X), 2, up))
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: half X, rounded down", priority: 960, parse: p_half_x } }

/// "Round down each time." after an instruction with "half X" (read with it).
fn f_round_down_each_time(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    end(l) == "round down each time" && format!("{prev:?}").contains("Div(X, 2, false)")
}

inventory::submit! { super::FollowupPattern { name: "hand/graveyard grammar: round down each time", priority: 960, apply: f_round_down_each_time } }

/// "If target opponent has more cards in hand than you, draw cards equal to the
/// difference.": the number is determined once (CR 121.2).
fn p_draw_hand_difference(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let who = r.strip_suffix(" has more cards in hand than you, draw cards equal to the difference")?;
    if !who.starts_with("target ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (who, rest) = player_ref(who, b)?;
    if !rest.trim().is_empty() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    let them = Value::HandSize(who);
    let you = Value::HandSize(PlayerRef::You);
    Some(Effect::If {
        cond: Condition::Compare(them.clone(), Cmp::Gt, you.clone()),
        then: Box::new(Effect::Draw {
            who: PlayerRef::You,
            n: Value::Diff(Box::new(them), Box::new(you)),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: if target opponent has more cards in hand, draw the difference", priority: 960, parse: p_draw_hand_difference } }

/// "draw cards equal to the mana value of the sacrificed permanent": read as "the
/// sacrificed permanent's mana value" (and likewise power and toughness).
fn p_equal_to_stat_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for stat in ["mana value", "power", "toughness"] {
        let p = format!(" equal to the {stat} of ");
        if let Some((head, obj)) = l.split_once(&p) {
            if obj.contains(" equal to ") || obj.starts_with("each ") || obj.contains(" among ") {
                return None;
            }
            let rewritten = format!("{head} equal to {obj}'s {stat}");
            let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
            let e = parse_clause(&rewritten, b);
            if e.is_none() {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
            }
            return e;
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: equal to the [stat] of [object]", priority: 960, parse: p_equal_to_stat_of } }

/// "exile cards equal to its power from the top of its owner's library": the top that
/// many cards.
fn p_exile_cards_equal_from_top(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile cards equal to ")?;
    let (value, lib) = r.split_once(" from the top of ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some((v, rest)) = crate::oracle::statics::parse_value_phrase(value, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    if !end(&rest).is_empty() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    clause_with_value(&format!("exile the top x cards of {lib}"), &v, b)
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: exile cards equal to [value] from the top of [library]", priority: 960, parse: p_exile_cards_equal_from_top } }

/// "you gain life and draw cards equal to its power": both amounts are the value.
fn p_gain_and_draw_equal(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you gain life and draw cards equal to ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(&format!("you gain life equal to {r} and draw cards equal to {r}"), b);
    if e.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    e
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: gain life and draw cards equal to [value]", priority: 960, parse: p_gain_and_draw_equal } }

/// "each of that player's opponents may draw a card", "each of that player's opponents
/// draws three cards": the opponents of that player (each in turn, CR 101.4) — read as
/// "each opponent ..." from that player's point of view. Instructions that also mention
/// "you" aren't read this way.
fn p_each_of_that_players_opponents(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each of that player's opponents ")?;
    if r.split(|c: char| !c.is_alphanumeric()).any(|w| w == "you" || w == "your") {
        return None;
    }
    use super::oracle_hardening_referents::is_no_player_referent;
    if is_no_player_referent(&b.it_player) {
        return None;
    }
    let who = b.it_player.clone();
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = parse_clause(&format!("each opponent {r}"), b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: each of that player's opponents", priority: 960, parse: p_each_of_that_players_opponents } }

/// "draw cards equal to the number of Zombies you control or the number of Zombie cards
/// in your graveyard, whichever is greater".
fn p_whichever_is_greater(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, up) = if let Some(h) = l.strip_suffix(", whichever is greater") {
        (h, true)
    } else {
        (l.strip_suffix(", whichever is less")?, false)
    };
    let (clause, values) = head.split_once(" equal to ")?;
    let (a, c) = values.split_once(" or ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let (va, ra) = crate::oracle::statics::parse_value_phrase(a, b)?;
        let (vc, rc) = crate::oracle::statics::parse_value_phrase(c, b)?;
        if !end(&ra).is_empty() || !end(&rc).is_empty() {
            return None;
        }
        let v = if up {
            Value::Max(Box::new(va), Box::new(vc))
        } else {
            Value::Min(Box::new(va), Box::new(vc))
        };
        let words: Vec<&str> = clause.split(' ').collect();
        let (verb, noun) = (words.first()?, words.get(1..)?.join(" "));
        clause_with_value(&format!("{verb} x {noun}"), &v, b)
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: A or B, whichever is greater", priority: 960, parse: p_whichever_is_greater } }

/// "investigate once for each opponent who has more cards in hand than you".
fn p_investigate_per_opponent_with_more_cards(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "investigate once for each opponent who has more cards in hand than you" {
        return None;
    }
    Some(Effect::KeywordAction {
        action: KeywordAction::Investigate,
        who: PlayerRef::You,
        what: Sel::None,
        n: Value::CountPlayers(PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::HandSize(Cmp::Gt, Box::new(Value::HandSize(PlayerRef::You))),
        ])),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: investigate for each opponent with more cards in hand", priority: 960, parse: p_investigate_per_opponent_with_more_cards } }

/// "you have more life than an opponent" (some opponent has less life than you), "you
/// have more life than each opponent".
fn c_more_life_than(c: &str) -> Option<Condition> {
    let op = match end(c) {
        "you have more life than an opponent" => AggOp::Min,
        "you have more life than each opponent" => AggOp::Max,
        _ => return None,
    };
    Some(Condition::Compare(
        Value::LifeTotal(PlayerRef::You),
        Cmp::Gt,
        Value::OverPlayers(op, PlayerFilter::Opponent, Box::new(Value::LifeTotal(PlayerRef::Iterated))),
    ))
}

inventory::submit! { super::ConditionPattern { name: "hand/graveyard grammar: you have more life than an opponent", priority: 960, parse: c_more_life_than } }

/// "The owner of target permanent shuffles it into their library, then reveals the top
/// card of their library.": the shuffle, then that player's next instruction.
fn p_owner_shuffles_then(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, then) = l.split_once(", then ")?;
    if then.starts_with("draw") || then.starts_with("you ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let (obj, rest) = match first.strip_prefix("the owner of ") {
            Some(r) => r.split_once(" shuffles ")?,
            None => first.split_once("'s owner shuffles ")?,
        };
        if !["it into their library", "that card into their library"].contains(&rest) {
            return None;
        }
        let (what, tail) = crate::oracle::effects::object_ref(obj, b)?;
        if !tail.trim().is_empty() || matches!(what, Sel::All(_) | Sel::Players(_) | Sel::None | Sel::Choose { .. }) {
            return None;
        }
        let owner = PlayerRef::OwnerOf(Box::new(what.clone()));
        b.it = what.clone();
        let next = parse_clause(&format!("its owner {then}"), b)?;
        b.it_player = owner.clone();
        Some(Effect::seq(vec![
            Effect::ShuffleIntoLibrary {
                what,
                library: owner,
            },
            next,
        ]))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: the owner of [object] shuffles it into their library, then ...", priority: 960, parse: p_owner_shuffles_then } }

/// "exile it unless you discard a creature card": the instruction happens unless the
/// player discards (CR 118.12a: a cost paid while the ability resolves).
fn p_unless_you_discard(l: &str, b: &mut Builder) -> Option<Effect> {
    let (eff, cost) = end(l).split_once(" unless you discard ")?;
    let (cost, _) = crate::oracle::costs::parse_cost(&format!("discard {cost}"))?;
    if !cost.parts.iter().all(|p| matches!(p, CostPart::Discard { .. })) || cost.mana.is_some() {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(effect) = parse_clause(eff, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: [instruction] unless you discard [cards]", priority: 960, parse: p_unless_you_discard } }

/// "If it's a noncreature, nonland card, you may reveal it and put it into your hand.":
/// a condition whose card description contains a comma (the condition ends at "card, ").
fn p_if_its_a_listed_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("if it's a ").or_else(|| l.strip_prefix("if it's an "))?;
    let (desc, rest) = r.split_once(" card, ")?;
    if !desc.contains(", ") {
        return None;
    }
    let cond = super::conditions_referents::parse_condition_with(
        &format!("it's a {desc} card"),
        b,
    )?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(then) = crate::oracle::effects::parse_sentence(rest, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: if it's a [A], [B] card, ...", priority: 40, parse: p_if_its_a_listed_card } }

/// "Reveal cards from the top of your library until you reveal a nonland card, then put
/// all cards revealed this way into your hand.": the found card and the rest go to the
/// same zone.
fn p_reveal_until_put_all(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, tail) = l.split_once(", then put all cards revealed this way ")?;
    // Read with the rest going elsewhere, then sent where the found card goes.
    let probe = format!("{head}, then put that card {tail} and the rest into your graveyard");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    match parse_clause(&probe, b) {
        Some(Effect::RevealUntil {
            who,
            filter,
            found_to,
            ..
        }) => Some(Effect::RevealUntil {
            who,
            filter,
            rest_to: found_to.clone(),
            found_to,
        }),
        _ => {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: reveal until, then put all cards revealed this way ...", priority: 960, parse: p_reveal_until_put_all } }

/// "If you do, you may put that card on the bottom of that player's library.": an
/// optional instruction where a clause is expected (as a sentence reads it).
fn p_you_may_clause(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = parse_clause(r, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::May {
        who: PlayerRef::You,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: you may [instruction] (in a clause)", priority: 990, parse: p_you_may_clause } }

/// "If you do or if you control another Dinosaur, you gain 3 life.": either the optional
/// instruction was followed or the other condition holds.
fn p_if_you_do_or_if(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if you do or if ")?;
    let (c, rest) = r.split_once(", ")?;
    let cond = super::conditions_referents::parse_condition_with(c, b)
        .or_else(|| crate::oracle::statics::parse_condition(c, b.ctx))?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(then) = parse_clause(rest, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::If {
        cond: Condition::Or(vec![Condition::PrevHappened, cond]),
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: if you do or if [condition], ...", priority: 960, parse: p_if_you_do_or_if } }

/// "you return a land card from your graveyard to your hand" (after ", then" following
/// another player's instruction): the imperative with "you" as its subject.
fn p_you_verb(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you ")?;
    let verb = r.split(' ').next()?;
    if ![
        "return", "put", "exile", "sacrifice", "search", "shuffle", "reveal", "look", "create",
        "destroy", "tap", "untap", "mill", "scry", "surveil", "investigate",
    ]
    .contains(&verb)
    {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(r, b);
    if e.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    e
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: you [verb] ...", priority: 990, parse: p_you_verb } }

/// "If a red card is discarded this way, ~ deals 4 damage to any target.", "If fewer than
/// two cards were discarded this way, you draw cards equal to the difference.": about the
/// cards the earlier instruction discarded (revealed, exiled).
fn p_if_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (c, rest) = r.split_once(" this way, ")?;
    // "fewer than two cards were discarded", "a red card is discarded".
    let (fewer, c) = match c.strip_prefix("fewer than ") {
        Some(x) => {
            let (n, x) = parse_number(x)?;
            (Some(n.as_const()?), x.trim_start().to_string())
        }
        None => (None, c.to_string()),
    };
    let (noun, verb) = c
        .split_once(" is ")
        .or_else(|| c.split_once(" was "))
        .or_else(|| c.split_once(" were "))
        .or_else(|| c.split_once(" are "))?;
    let noun = match fewer {
        Some(_) => noun,
        None => noun
            .strip_prefix("a ")
            .or_else(|| noun.strip_prefix("an "))?,
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (count, tail) = this_way_count(&format!("{noun} {verb} this way"), b)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let parsed = match fewer {
        // "you draw cards equal to the difference".
        Some(n) => {
            let diff = Value::Diff(Box::new(Value::Const(n)), Box::new(count.clone()));
            let r = rest.strip_prefix("you ").unwrap_or(rest);
            let r = r.strip_suffix(" equal to the difference")?;
            let words: Vec<&str> = r.split(' ').collect();
            let clause = format!("{} x {}", words.first()?, words.get(1..)?.join(" "));
            clause_with_value(&clause, &diff, b).map(|e| (Condition::Compare(count, Cmp::Lt, Value::Const(n)), e))
        }
        None => parse_clause(rest, b).map(|e| (Condition::Compare(count, Cmp::Ge, Value::Const(1)), e)),
    };
    let Some((cond, then)) = parsed else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: if a [card] is [verb] this way, ...", priority: 960, parse: p_if_this_way } }

/// "Exile the bottom card of target player's graveyard." (CR 404.2: a graveyard's order
/// can't change; its bottom card is the one put there earliest), "Exile all but the bottom
/// card of target player's library."
fn p_bottom_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let fail = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
        None
    };
    if let Some(r) = l.strip_prefix("exile the bottom card of ") {
        let who = r.strip_suffix("'s graveyard")?;
        let (who, rest) = player_ref(who, b)?;
        if !rest.trim().is_empty() {
            return fail(b);
        }
        let owner = super::value_grammar::owned_by(&who);
        return Some(record(
            Effect::Exile {
                what: Sel::All(Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Graveyard),
                    owner,
                    Filter::Custom(SmolStr::new(crate::kw::hand_graveyard_actions::BOTTOM_OF_GRAVEYARD)),
                ])),
                face_down: false,
                link: false,
            },
            b,
        ));
    }
    if let Some(r) = l.strip_prefix("exile all but the bottom card of ") {
        let who = r.strip_suffix("'s library")?;
        let (who, rest) = player_ref(who, b)?;
        if !rest.trim().is_empty() {
            return fail(b);
        }
        let n = Value::Diff(Box::new(Value::LibrarySize(who.clone())), Box::new(Value::Const(1)));
        return Some(Effect::Exile {
            what: Sel::TopOfLibrary(who, n),
            face_down: false,
            link: false,
        });
    }
    None
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: the bottom card of a graveyard or library", priority: 960, parse: p_bottom_card } }

/// "You have no maximum hand size for the rest of the game.", "You have no maximum hand
/// size until your next turn." (CR 402.2): a player effect for that long.
fn p_no_max_hand_size(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you have no maximum hand size")?;
    let duration = match r {
        " for the rest of the game" => Duration::Permanent,
        "" => return None,
        r => match crate::oracle::effects::duration_suffix(&format!("x{r}")) {
            (d, "x") if !matches!(d, Duration::Permanent) => d,
            _ => return None,
        },
    };
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::MaxHandSize(None),
        duration,
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: you have no maximum hand size for a duration", priority: 960, parse: p_no_max_hand_size } }

/// "Exile two cards from your graveyard. If you can't, sacrifice ~ and draw a card." (Egon
/// rulings: with fewer cards, none are exiled and the other instruction happens instead).
fn f_if_you_cant(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you can't, ") else {
        return false;
    };
    // The previous instruction: exiling an exact number of cards the player chooses.
    let exile = match &*prev {
        Effect::Seq(v) => v.first(),
        e => Some(e),
    };
    let Some(Effect::Exile {
        what:
            Sel::Choose {
                filter,
                count: Value::Const(n),
                up_to: false,
                ..
            },
        ..
    }) = exile
    else {
        return false;
    };
    let (filter, n) = (filter.clone(), *n);
    let Some(otherwise) = parse_clause(r, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::If {
        cond: Condition::Compare(Value::Count(filter), Cmp::Ge, Value::Const(n)),
        then: Box::new(old),
        otherwise: Box::new(otherwise),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "hand/graveyard grammar: if you can't, ...", priority: 960, apply: f_if_you_cant } }

/// "exile ~ from your graveyard" (after it died): only if it's still there.
fn p_exile_self_from_graveyard(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "exile ~ from your graveyard" {
        return None;
    }
    Some(Effect::Exile {
        what: Sel::All(Filter::and(vec![
            Filter::Custom(SmolStr::new(crate::kw::hand_graveyard_actions::SOURCE_OR_NEXT)),
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ])),
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: exile ~ from your graveyard", priority: 960, parse: p_exile_self_from_graveyard } }

/// "If you control a Fish, Octopus, Otter, Seal, Serpent, or Whale, draw a card.": a
/// condition containing commas (the condition ends at the comma where both it and the
/// instruction after it can be read).
fn p_if_condition_with_commas(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let first = r.find(", ")?;
    let mut at = first + 2;
    while let Some(i) = r[at..].find(", ") {
        let split = at + i;
        at = split + 2;
        let (c, rest) = (&r[..split], &r[split + 2..]);
        let cond = super::conditions_referents::parse_condition_with(c, b)
            .or_else(|| crate::oracle::statics::parse_condition(c, b.ctx));
        let Some(cond) = cond else { continue };
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        match crate::oracle::effects::parse_sentence(rest, b) {
            Some(then) => {
                return Some(Effect::If {
                    cond,
                    then: Box::new(then),
                    otherwise: Box::new(Effect::Noop),
                })
            }
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
            }
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: if [condition with commas], ...", priority: 990, parse: p_if_condition_with_commas } }

/// "up to one target player mills cards equal to ~'s power": the instruction for a
/// target player that may be left unchosen (then nothing happens).
fn p_up_to_one_target_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (r, who) = if let Some(r) = l.strip_prefix("up to one target player ") {
        (r, "target player")
    } else {
        (l.strip_prefix("up to one target opponent ")?, "target opponent")
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = parse_clause(&format!("{who} {r}"), b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    // Exactly one new target: the player.
    if b.targets.len() != saved.0 + 1 || !matches!(b.targets[saved.0].what, TargetKind::Player(_)) {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    b.targets[saved.0].min = 0;
    Some(e)
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: up to one target player [instruction]", priority: 990, parse: p_up_to_one_target_player } }

/// "you control a Fish, Octopus, Otter, Seal, Serpent, or Whale": a permanent of any of
/// the listed kinds.
fn c_you_control_listed(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you control ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    if !r.contains(", ") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !tail.trim().is_empty() || super::statics::mentions_other_zones(&f) {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)])),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "hand/graveyard grammar: you control a [A, B, or C]", priority: 960, parse: c_you_control_listed } }

/// "If the top card of target player's graveyard is a creature card, put that card on top
/// of that player's library." (CR 404.2: the top card is the one put there latest).
fn p_if_top_card_of_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if the top card of ")?;
    let (who, r) = r.split_once("'s graveyard is ")?;
    let (kind, rest) = r.split_once(", ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let (who, extra) = player_ref(who, b)?;
        if !extra.trim().is_empty() {
            return None;
        }
        let kind = kind.strip_prefix("a ").or_else(|| kind.strip_prefix("an "))?;
        let (f, _, tail) = parse_object_phrase(kind)?;
        if !tail.trim().is_empty() || !has_card_head(&f) {
            return None;
        }
        let top = Filter::and(vec![
            Filter::Card,
            Filter::InZone(ZoneKind::Graveyard),
            super::value_grammar::owned_by(&who),
            Filter::Custom(SmolStr::new(crate::kw::hand_graveyard_actions::TOP_OF_GRAVEYARD)),
        ]);
        b.it = Sel::Var(CHOSEN);
        b.it_player = who;
        let then = crate::oracle::effects::parse_sentence(rest, b)?;
        Some(Effect::seq(vec![
            Effect::Store {
                var: CHOSEN,
                sel: Sel::All(top.clone()),
            },
            Effect::If {
                cond: Condition::Compare(
                    Value::Count(Filter::and(vec![top, f])),
                    Cmp::Ge,
                    Value::Const(1),
                ),
                then: Box::new(then),
                otherwise: Box::new(Effect::Noop),
            },
        ]))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "hand/graveyard grammar: if the top card of a graveyard is ...", priority: 960, parse: p_if_top_card_of_graveyard } }
