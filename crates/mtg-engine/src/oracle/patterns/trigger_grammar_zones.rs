//! Zone-change trigger events (CR 603.6, 603.10a, 700.4) that the compositional parser in
//! `triggers.rs` doesn't cover, as `[subject] [zone-change verb]`:
//!
//! * `is/are put into [a graveyard | graveyards | your graveyard | an opponent's graveyard |
//!   a player's graveyard | exile | a library | the command zone] [from [zones]]`, the zones
//!   being `the battlefield`, `anywhere`, `anywhere other than the battlefield`, `[your |
//!   their | a] library`, `your hand [or library]`, `[your | a] graveyard`, `graveyards`,
//!   joined by "and/or" or "or". Without "from", a permanent ("a permanent you control is
//!   put into a graveyard") comes from the battlefield — into a graveyard that's dying
//!   (CR 700.4) — and a card from anywhere.
//! * `is/are returned to [your | a player's | its owner's] hand [from the battlefield]`.
//! * `leave(s) [an opponent's | a] graveyard` (look back in time, CR 603.10a).
//! * `leave(s) the battlefield without dying`: to any zone but a graveyard.
//! * `dies or is put into the command zone`, `is put into the command zone [from
//!   anywhere]`, `phases out or leaves the battlefield`.
//!
//! Subjects extend the core ones with `enchanted [noun]` (the enchanted permanent), `~
//! and/or one or more other [objects]`, `[object] or a/another [object]`, `one or more
//! [objects] and/or [objects]`, and plural subtypes spelled like their singular ("one or
//! more Merfolk").
//!
//! Referents: for a permanent that left the battlefield, "it" is its last known
//! information (actions find the card, CR 400.7e) and "that player" its controller — or
//! its owner when the text names the owner's zone ("put into a player's graveyard",
//! "returned to a player's hand"); for a card put into a zone from elsewhere, "it"/"that
//! card" is the card in its new zone and "that player" its owner. A "one or more" batch
//! (CR 603.2c) has no single "it".

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::patterns::triggers::{parse_subject, Subject};
use crate::oracle::phrases::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

fn subj(filter: Filter, self_only: bool, one_or_more: bool) -> Subject {
    Subject {
        filter,
        self_only,
        one_or_more,
    }
}

/// A single object phrase with an optional article ("a creature you control", "another
/// artifact"): its filter.
fn single(s: &str) -> Option<Filter> {
    let t = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    if t == s && !s.starts_with("another ") {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(t)?;
    (!plural && end(tail).is_empty()).then_some(f)
}

/// The subjects of events that [`parse_subject`] reads besides its own (it falls back to
/// this): the extensions listed in the module documentation.
pub(crate) fn subject_ext(s: &str) -> Option<Subject> {
    let s = s.trim();
    // "enchanted Forest" (the Genjus): the permanent this Aura enchants.
    // ("Plains" reads like a plural noun.)
    if let Some(r) = s.strip_prefix("enchanted ") {
        let (_, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(subj(Filter::AttachedToSource, false, false));
    }
    // "a 1/1 creature you control": with that power and toughness.
    if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        let (pt, rest) = split_word(r);
        if let Some((p, t)) = pt.split_once('/') {
            let (p, t): (i32, i32) = (p.parse().ok()?, t.parse().ok()?);
            let f = single(&format!("a {rest}"))?;
            let stat = |v| Box::new(Value::c(v));
            return Some(subj(
                Filter::and(vec![
                    Filter::Power(Cmp::Eq, stat(p)),
                    Filter::Toughness(Cmp::Eq, stat(t)),
                    f,
                ]),
                false,
                false,
            ));
        }
        // "a commander an opponent controls" (CR 903.3).
        if let Some(q) = r.strip_prefix("commander ") {
            let f = single(&format!("a permanent {q}"))?;
            return Some(subj(Filter::and(vec![Filter::Commander, f]), false, false));
        }
    }
    // "~ and/or one or more other Vampires you control": a batch with ~ or the others.
    if let Some(r) = s.strip_prefix("~ and/or one or more other ") {
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(subj(
            Filter::Or(vec![Filter::Source, Filter::and(vec![f, Filter::Other])]),
            false,
            true,
        ));
    }
    if let Some(r) = s.strip_prefix("one or more ") {
        // "one or more creatures you control and/or creature cards in your graveyard".
        if let Some((a, b)) = r.split_once(" and/or ") {
            if let (Some((fa, _, ta)), Some((fb, _, tb))) =
                (parse_object_phrase(a), parse_object_phrase(b))
            {
                if end(ta).is_empty() && end(tb).is_empty() {
                    return Some(subj(Filter::Or(vec![fa, fb]), false, true));
                }
            }
        }
        // "one or more nontoken Merfolk you control": a plural spelled like its singular.
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(subj(f, false, true));
    }
    // "another creature you control or a land you control".
    for sep in [" or a ", " or an ", " or another "] {
        if let Some(i) = s.find(sep) {
            let (a, b) = (&s[..i], &s[i + " or ".len()..]);
            let fa = single(a)?;
            let fb = single(b)?;
            return Some(subj(Filter::Or(vec![fa, fb]), false, false));
        }
    }
    None
}

/// Whether every alternative of an object phrase is a card in no particular zone.
fn all_cards(f: &Filter) -> bool {
    match f {
        Filter::Or(v) => v.iter().all(all_cards),
        // "your commander is put into the command zone from anywhere" (a card, CR 903.3).
        f => (names_cards(f) || is_commander(f)) && !mentions_zone(f),
    }
}

fn is_commander(f: &Filter) -> bool {
    match f {
        Filter::Commander => true,
        Filter::And(v) => v.iter().any(is_commander),
        _ => false,
    }
}

fn mentions_zone(f: &Filter) -> bool {
    match f {
        Filter::InZone(_) | Filter::Permanent => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_zone),
        Filter::Not(x) => mentions_zone(x),
        _ => false,
    }
}

/// Whether an object phrase describes cards (in any zone) rather than permanents.
fn names_cards(f: &Filter) -> bool {
    match f {
        Filter::Card | Filter::PermanentCard => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(names_cards),
        _ => false,
    }
}

/// A zone in a "from [zones]" list ("your graveyard", "the battlefield", "library" after
/// "your hand or"): the zone and whose it is.
fn zone_phrase(p: &str, inherited_yours: bool) -> Option<(ZoneKind, Option<PlayerRel>)> {
    let yours = |z| Some((z, Some(PlayerRel::You)));
    Some(match p.trim() {
        "the battlefield" => (ZoneKind::Battlefield, None),
        "your graveyard" => return yours(ZoneKind::Graveyard),
        "your library" => return yours(ZoneKind::Library),
        "your hand" => return yours(ZoneKind::Hand),
        "a graveyard" | "graveyards" => (ZoneKind::Graveyard, None),
        "a library" | "libraries" => (ZoneKind::Library, None),
        "graveyard" if inherited_yours => return yours(ZoneKind::Graveyard),
        "library" if inherited_yours => return yours(ZoneKind::Library),
        "hand" if inherited_yours => return yours(ZoneKind::Hand),
        _ => return None,
    })
}

/// Where the cards come from: `None` for anywhere.
struct From {
    zones: Option<Vec<ZoneKind>>,
    owner: Option<PlayerRel>,
}

/// "the battlefield", "anywhere", "anywhere other than the battlefield", "your hand or
/// library", "graveyards and/or the battlefield", "their library" (the owner of the
/// destination graveyard: every card goes to its owner's graveyard).
fn from_zones(x: &str, dest: ZoneKind) -> Option<From> {
    let x = x.trim();
    if x == "anywhere" {
        return Some(From {
            zones: None,
            owner: None,
        });
    }
    if x == "anywhere other than the battlefield" {
        let zones = [
            ZoneKind::Library,
            ZoneKind::Hand,
            ZoneKind::Graveyard,
            ZoneKind::Stack,
            ZoneKind::Exile,
            ZoneKind::Command,
        ]
        .into_iter()
        .filter(|z| *z != dest)
        .collect();
        return Some(From {
            zones: Some(zones),
            owner: None,
        });
    }
    if x == "their library" {
        return Some(From {
            zones: Some(vec![ZoneKind::Library]),
            owner: None,
        });
    }
    let mut zones = Vec::new();
    let mut owners = Vec::new();
    let mut yours = false;
    for piece in x
        .split(" and/or ")
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
    {
        let (z, o) = zone_phrase(piece, yours)?;
        yours |= o == Some(PlayerRel::You);
        zones.push(z);
        owners.push(o);
    }
    // Every zone but the battlefield belongs to the same player (a permanent's owner
    // isn't the point of "the battlefield").
    let owner = owners
        .iter()
        .zip(&zones)
        .filter(|(_, z)| **z != ZoneKind::Battlefield)
        .map(|(o, _)| *o)
        .next()
        .flatten();
    if owners
        .iter()
        .zip(&zones)
        .any(|(o, z)| *z != ZoneKind::Battlefield && *o != owner)
    {
        return None;
    }
    Some(From {
        zones: Some(zones),
        owner,
    })
}

/// "a graveyard", "your graveyard", "an opponent's graveyard", "a player's graveyard",
/// "exile", "a library", "the command zone": (zone, whose, names the owner's zone, rest).
fn destination(r: &str) -> Option<(ZoneKind, Option<PlayerRel>, bool, &str)> {
    for (p, z, o, owner_named) in [
        ("a graveyard", ZoneKind::Graveyard, None, false),
        ("graveyards", ZoneKind::Graveyard, None, false),
        (
            "your graveyard",
            ZoneKind::Graveyard,
            Some(PlayerRel::You),
            true,
        ),
        (
            "an opponent's graveyard",
            ZoneKind::Graveyard,
            Some(PlayerRel::Opponent),
            true,
        ),
        ("a player's graveyard", ZoneKind::Graveyard, None, true),
        ("exile", ZoneKind::Exile, None, false),
        ("a library", ZoneKind::Library, None, false),
        ("the command zone", ZoneKind::Command, None, false),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') {
                return Some((z, o, owner_named, rest));
            }
        }
    }
    None
}

fn owned(f: Filter, o: Option<PlayerRel>) -> Filter {
    match o {
        Some(o) => Filter::and(vec![f, Filter::OwnedBy(o)]),
        None => f,
    }
}

fn any_of(mut v: Vec<TriggerCond>) -> TriggerCond {
    if v.len() == 1 {
        v.pop().unwrap()
    } else {
        TriggerCond::AnyOf(v)
    }
}

/// The parse for a zone change event, with referents for a permanent that left the
/// battlefield (`from_battlefield`) or for a card in its new zone.
fn finish(cond: TriggerCond, s: &Subject, from_battlefield: bool, owner_named: bool) -> Parsed {
    let (it, who) = if from_battlefield {
        let who = if owner_named {
            PlayerRef::OwnerOf(Box::new(Sel::TriggerLki))
        } else {
            PlayerRef::ControllerOf(Box::new(Sel::TriggerLki))
        };
        let it = if s.self_only {
            Sel::This
        } else {
            Sel::TriggerLki
        };
        (it, who)
    } else {
        (
            Sel::TriggerObject,
            PlayerRef::OwnerOf(Box::new(Sel::TriggerObject)),
        )
    };
    if s.one_or_more {
        return (
            TriggerCond::Batched {
                trigger: Box::new(cond),
                per: BatchPer::Batch,
            },
            Sel::None,
            who,
        );
    }
    (cond, it, who)
}

/// "is/are put into [destination] [from [zones]]".
fn put_into(v: &str, s: &Subject) -> Option<Parsed> {
    let r = v
        .strip_prefix("is put into ")
        .or_else(|| v.strip_prefix("are put into "))?;
    let (dest, dest_owner, owner_named, r) = destination(r)?;
    let r = end(r.trim_start());
    let from = if r.is_empty() {
        // "a permanent you control is put into a graveyard": a permanent is on the
        // battlefield; "one or more cards are put into exile": cards, from anywhere. (~
        // itself needs its zone said.)
        if s.self_only {
            return None;
        }
        if names_cards(&s.filter) {
            From {
                zones: None,
                owner: None,
            }
        } else {
            From {
                zones: Some(vec![ZoneKind::Battlefield]),
                owner: None,
            }
        }
    } else {
        from_zones(r.strip_prefix("from ")?, dest)?
    };
    // Cards put into a zone from anywhere are seen in that zone: every alternative of
    // the subject must be a card that can be there ("creatures you control and/or
    // creature cards in your graveyard are put into exile" needs each from its own
    // zone).
    if from.zones.is_none() && !all_cards(&s.filter) {
        return None;
    }
    let f = owned(owned(s.filter.clone(), dest_owner), from.owner);
    let from_battlefield = from.zones.as_deref() == Some(&[ZoneKind::Battlefield]);
    // A permanent leaving the battlefield isn't a card (a token can die).
    if from_battlefield && names_cards(&s.filter) {
        return None;
    }
    let cond = match &from.zones {
        // CR 700.4: "dies" means "is put into a graveyard from the battlefield".
        _ if from_battlefield && dest == ZoneKind::Graveyard => TriggerCond::Dies(f),
        None => TriggerCond::ZoneChange {
            filter: f,
            from: None,
            to: Some(dest),
        },
        Some(zones) => any_of(
            zones
                .iter()
                .map(|z| TriggerCond::ZoneChange {
                    filter: f.clone(),
                    from: Some(*z),
                    to: Some(dest),
                })
                .collect(),
        ),
    };
    // "A spell or ability ... causes ..." and other cards from several zones have no
    // single referent unless they all leave the battlefield.
    if !from_battlefield
        && from
            .zones
            .as_ref()
            .is_some_and(|z| z.contains(&ZoneKind::Battlefield))
    {
        let (c, _, p) = finish(cond, s, false, owner_named);
        return Some((c, Sel::None, p));
    }
    Some(finish(cond, s, from_battlefield, owner_named))
}

/// "is/are returned to [your | a player's | its owner's] hand [from the battlefield]".
fn returned_to_hand(v: &str, s: &Subject) -> Option<Parsed> {
    let r = v
        .strip_prefix("is returned to ")
        .or_else(|| v.strip_prefix("are returned to "))?;
    let (owner, r) = [
        ("your hand", Some(PlayerRel::You)),
        ("a player's hand", None),
        ("its owner's hand", None),
        ("their owner's hand", None),
        ("their owners' hands", None),
        ("hand", None),
    ]
    .into_iter()
    .find_map(|(p, o)| r.strip_prefix(p).map(|rest| (o, rest)))?;
    let r = r.trim_start();
    let r = r.strip_prefix("from the battlefield").unwrap_or(r);
    if !end(r).is_empty() || names_cards(&s.filter) {
        return None;
    }
    let cond = TriggerCond::ZoneChange {
        filter: owned(s.filter.clone(), owner),
        from: Some(ZoneKind::Battlefield),
        to: Some(ZoneKind::Hand),
    };
    Some(finish(cond, s, true, true))
}

/// "leave(s) [an opponent's | a] graveyard".
fn leaves_graveyard(v: &str, s: &Subject) -> Option<Parsed> {
    let r = v
        .strip_prefix("leaves ")
        .or_else(|| v.strip_prefix("leave "))?;
    let owner = match end(r) {
        "an opponent's graveyard" => Some(PlayerRel::Opponent),
        "a graveyard" => None,
        _ => return None,
    };
    let cond = TriggerCond::ZoneChange {
        filter: owned(s.filter.clone(), owner),
        from: Some(ZoneKind::Graveyard),
        to: None,
    };
    Some(finish(cond, s, false, true))
}

/// "leave(s) the battlefield without dying": to any zone but a graveyard.
fn leaves_without_dying(v: &str, s: &Subject) -> Option<Parsed> {
    if !matches!(
        end(v),
        "leaves the battlefield without dying" | "leave the battlefield without dying"
    ) {
        return None;
    }
    let cond = any_of(
        [
            ZoneKind::Hand,
            ZoneKind::Library,
            ZoneKind::Exile,
            ZoneKind::Command,
        ]
        .into_iter()
        .map(|z| TriggerCond::ZoneChange {
            filter: s.filter.clone(),
            from: Some(ZoneKind::Battlefield),
            to: Some(z),
        })
        .collect(),
    );
    Some(finish(cond, s, true, false))
}

/// "dies or is put into the command zone", "phases out or leaves the battlefield".
fn two_verbs(v: &str, s: &Subject) -> Option<Parsed> {
    let f = s.filter.clone();
    let cond = match end(v) {
        "dies or is put into the command zone" => TriggerCond::AnyOf(vec![
            TriggerCond::Dies(f.clone()),
            TriggerCond::ZoneChange {
                filter: f,
                from: Some(ZoneKind::Battlefield),
                to: Some(ZoneKind::Command),
            },
        ]),
        "phases out or leaves the battlefield" if s.self_only => TriggerCond::AnyOf(vec![
            TriggerCond::Phases {
                phased_in: false,
                filter: f.clone(),
            },
            TriggerCond::LeavesBattlefield(f),
        ]),
        _ => return None,
    };
    Some(finish(cond, s, true, false))
}

fn zone_verb(v: &str, s: &Subject) -> Option<Parsed> {
    put_into(v, s)
        .or_else(|| returned_to_hand(v, s))
        .or_else(|| leaves_graveyard(v, s))
        .or_else(|| leaves_without_dying(v, s))
        .or_else(|| two_verbs(v, s))
}

/// "[subject] [zone-change verb]".
fn zone_event(r: &str) -> Option<Parsed> {
    let r = end(r);
    for (i, ch) in r.char_indices() {
        if ch != ' ' {
            continue;
        }
        let (subj_s, verb_s) = (&r[..i], &r[i + 1..]);
        if !verb_s.starts_with("is ")
            && !verb_s.starts_with("are ")
            && !verb_s.starts_with("leave")
            && !verb_s.starts_with("dies ")
            && !verb_s.starts_with("phases ")
        {
            continue;
        }
        let Some(s) = parse_subject(subj_s) else {
            continue;
        };
        if let Some(p) = zone_verb(verb_s, &s) {
            return Some(p);
        }
    }
    None
}

inventory::submit! { TriggerPattern { name: "zone-change events (put into, returned to hand, without dying)", priority: 110, parse: zone_event } }

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Parsed {
        crate::oracle::triggers::parse_trigger_condition(s)
            .unwrap_or_else(|| panic!("failed to parse {s:?}"))
    }

    #[test]
    fn zone_changes_parse() {
        for s in [
            "when enchanted forest is put into a graveyard",
            "whenever a permanent you control is put into a graveyard",
            "whenever a permanent is put into an opponent's graveyard",
            "whenever a nontoken permanent is put into a player's graveyard from the battlefield",
            "whenever a creature card is put into an opponent's graveyard from their library",
            "whenever one or more cards are put into exile from your graveyard",
            "whenever one or more cards are put into exile from your library and/or your graveyard",
            "whenever one or more cards are put into a library from anywhere",
            "whenever a permanent is returned to a player's hand",
            "whenever ~ or another creature is returned to your hand from the battlefield",
            "whenever a creature card leaves an opponent's graveyard",
            "whenever another creature you control leaves the battlefield without dying",
            "whenever one or more creatures you control leave the battlefield without dying",
            "whenever a creature you control dies or is put into the command zone",
            "when ~ phases out or leaves the battlefield",
            "whenever a lhurgoyf permanent card is put into your graveyard from anywhere other than the battlefield",
        ] {
            p(s);
        }
    }

    #[test]
    fn permanent_into_graveyard_is_dying() {
        let (c, it, _) = p("whenever a permanent you control is put into a graveyard");
        assert!(matches!(c, TriggerCond::Dies(_)), "{c:?}");
        assert!(matches!(it, Sel::TriggerLki));
        // A card put into a graveyard has to say from where.
        assert!(zone_event("a creature card is put into a graveyard").is_none());
    }
}
