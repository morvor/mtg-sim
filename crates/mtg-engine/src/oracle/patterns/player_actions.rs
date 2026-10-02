//! Instructions for "you" that are mostly given to other players ("target player mills
//! half their library", "each player sacrifices all artifacts they control", "each player
//! chooses six lands they control, then sacrifices the rest"); `player_subjects` rewords
//! those for "you" and has the player perform them.
//!
//! - "mill half your library, rounded down", "exile the top half of your library, rounded
//!   up", "lose a third of your life, rounded up" (CR 107.1a), "mill three times X cards".
//! - "sacrifice all [permanents] you control" (CR 701.21a).
//! - "shuffle all [permanents] you own into your library", "shuffle your hand, graveyard,
//!   and all permanents you own into your library" (CR 701.24).
//! - "choose N [permanents] you control, then sacrifice the rest": each player performing
//!   it chooses in APNAP order and the rest are sacrificed at the same time (CR 101.4,
//!   101.4c); "choose N cards in your hand/graveyard and discard/exile/shuffle the rest".
//! - "untap a land you control", "reveal your hand".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;


/// ", rounded up" / ", rounded down" at the end of `s`: (the rest, rounded up).
fn rounded(s: &str) -> Option<(&str, bool)> {
    if let Some(r) = s.strip_suffix(", rounded up") {
        Some((r, true))
    } else {
        s.strip_suffix(", rounded down").map(|r| (r, false))
    }
}

/// "half"/"a third" → the divisor.
fn fraction(s: &str) -> Option<(i32, &str)> {
    if let Some(r) = s.strip_prefix("half of ") {
        Some((2, r))
    } else if let Some(r) = s.strip_prefix("half ") {
        Some((2, r))
    } else if let Some(r) = s.strip_prefix("a third of ") {
        Some((3, r))
    } else {
        None
    }
}

/// "two times x" / "three times x" → that multiple of X.
fn times(s: &str) -> Option<(Value, &str)> {
    let (n, r) = parse_number(s)?;
    let n = n.as_const()?;
    let r = r.trim_start().strip_prefix("times ")?;
    let (x, r) = parse_number(r)?;
    if !matches!(x, Value::X) {
        return None;
    }
    Some((Value::Mul(Box::new(Value::c(n)), Box::new(Value::X)), r))
}

fn fractions(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    if let Some(r) = l.strip_prefix("mill ") {
        if let Some((r, up)) = rounded(r) {
            let (d, r) = fraction(r)?;
            if r != "your library" {
                return None;
            }
            return Some(Effect::Mill {
                who: PlayerRef::You,
                n: Value::Div(Box::new(Value::LibrarySize(PlayerRef::You)), d, up),
            });
        }
        let (n, r) = times(r)?;
        if !matches!(r.trim(), "cards" | "card") {
            return None;
        }
        return Some(Effect::Mill {
            who: PlayerRef::You,
            n,
        });
    }
    if let Some(r) = l.strip_prefix("exile the top ") {
        let (r, up) = rounded(r)?;
        let (d, r) = fraction(r)?;
        if r != "your library" {
            return None;
        }
        return Some(Effect::Exile {
            what: Sel::TopOfLibrary(
                PlayerRef::You,
                Value::Div(Box::new(Value::LibrarySize(PlayerRef::You)), d, up),
            ),
            face_down: false,
            link: false,
        });
    }
    if let Some(r) = l.strip_prefix("lose ") {
        if let Some((r, up)) = rounded(r) {
            let (d, r) = fraction(r)?;
            if r != "your life" {
                return None;
            }
            return Some(Effect::LoseLife {
                who: PlayerRef::You,
                n: Value::Div(Box::new(Value::LifeTotal(PlayerRef::You)), d, up),
            });
        }
        let (n, r) = times(r)?;
        if r.trim() != "life" {
            return None;
        }
        return Some(Effect::LoseLife {
            who: PlayerRef::You,
            n,
        });
    }
    None
}

inventory::submit! { EffectPattern { name: "player actions: half/third of your library or life, N times X", priority: 400, parse: fractions } }

/// `f` without "you control" (the permanents of each player performing the instruction).
fn without_controller(f: Filter) -> Option<Filter> {
    match f {
        Filter::ControlledBy(PlayerRel::You) => None,
        Filter::And(v) => {
            let v: Vec<Filter> = v.into_iter().filter_map(without_controller).collect();
            Some(Filter::and(v))
        }
        f => Some(f),
    }
}

/// Whether `f` restricts to permanents "you control".
fn you_control(f: &Filter) -> bool {
    match f {
        Filter::ControlledBy(PlayerRel::You) => true,
        Filter::And(v) => v.iter().any(you_control),
        _ => false,
    }
}

/// "[objects] you control" as a filter that includes "you control".
fn permanents_you_control(s: &str) -> Option<Filter> {
    let (f, _, rest) = parse_object_phrase(s)?;
    if !end(rest).is_empty() || !you_control(&f) || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some(f)
}

/// "sacrifice all [other] [permanents] you control".
fn sacrifice_all(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("sacrifice all ")?;
    let f = permanents_you_control(r)?;
    Some(Effect::SacrificeObjects {
        what: Sel::All(Filter::and(vec![Filter::Permanent, f])),
    })
}

inventory::submit! { EffectPattern { name: "player actions: sacrifice all [permanents] you control", priority: 400, parse: sacrifice_all } }

/// "shuffle all [permanents] you own into your library", "shuffle your hand, graveyard,
/// and all permanents you own into your library", "shuffle all cards from your hand and
/// all permanents you own into your library".
fn shuffle_into_library(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("shuffle ")?;
    let what = r.strip_suffix(" into your library")?;
    let parts: Vec<&str> = what
        .split(", and ")
        .flat_map(|p| p.split(" and "))
        .flat_map(|p| p.split(", "))
        .collect();
    let mut sels = Vec::new();
    for p in parts {
        let zone = |z: ZoneKind| {
            Sel::All(Filter::and(vec![
                Filter::InZone(z),
                Filter::OwnedBy(PlayerRel::You),
            ]))
        };
        let s = match p {
            "your hand" | "all cards from your hand" | "all the cards in your hand" => {
                zone(ZoneKind::Hand)
            }
            "graveyard" | "your graveyard" => zone(ZoneKind::Graveyard),
            _ => {
                let r = p.strip_prefix("all ")?;
                let (f, plural, rest) = parse_object_phrase(r)?;
                if !plural || !end(rest).is_empty() {
                    return None;
                }
                // "permanents you own": on the battlefield, whoever controls them.
                if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
                    return None;
                }
                let owned = format!("{f:?}").contains("OwnedBy(You)");
                if !owned {
                    return None;
                }
                Sel::All(Filter::and(vec![Filter::Permanent, f]))
            }
        };
        sels.push(s);
    }
    if sels.is_empty() || sels.len() == 1 && !what.starts_with("all ") {
        return None;
    }
    let what = if sels.len() == 1 {
        sels.pop()?
    } else {
        Sel::Union(sels)
    };
    Some(Effect::ShuffleIntoLibrary {
        what,
        library: PlayerRef::You,
    })
}

inventory::submit! { EffectPattern { name: "player actions: shuffle all [permanents] you own into your library", priority: 400, parse: shuffle_into_library } }

/// "choose six lands you control, then sacrifice the rest", "choose up to two creatures
/// you control, then sacrifice the rest", "choose a creature or planeswalker you control,
/// then sacrifice the rest", "choose five lands you control and sacrifice the rest",
/// "choose from the lands you control a land of each basic land type, then sacrifice the
/// rest".
fn keep_and_sacrifice(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("choose ")?;
    let r = r
        .strip_suffix(", then sacrifice the rest")
        .or_else(|| r.strip_suffix(" and sacrifice the rest"))?;
    // "from the lands you control a land of each basic land type"
    if let Some(r) = r.strip_prefix("from the ") {
        let (among, each) = r.split_once(" you control a ")?;
        let (f, plural, rest) = parse_object_phrase(among)?;
        if !plural || !end(rest).is_empty() {
            return None;
        }
        let noun = each.strip_suffix(" of each basic land type")?;
        let (nf, _, rest) = parse_object_phrase(noun)?;
        if !end(rest).is_empty() || format!("{nf:?}") != format!("{f:?}") {
            return None;
        }
        let keep = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .iter()
            .map(|t| Filter::and(vec![f.clone(), Filter::Subtype((*t).into())]))
            .collect();
        return Some(Effect::KeepAndSacrificeRest {
            who: PlayerRef::You,
            among: f,
            keep,
            up_to: false,
        });
    }
    let (up_to, r) = match r.strip_prefix("up to ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (n, rest) = parse_number(r)?;
    let n = n.as_const()?;
    if !(1..=20).contains(&n) {
        return None;
    }
    let f = permanents_you_control(rest)?;
    let among = without_controller(f)?;
    if format!("{among:?}").contains("You") {
        return None;
    }
    let among = Filter::and(vec![Filter::Permanent, among]);
    Some(Effect::KeepAndSacrificeRest {
        who: PlayerRef::You,
        among: among.clone(),
        keep: vec![among; n as usize],
        up_to,
    })
}

inventory::submit! { EffectPattern { name: "player actions: choose N permanents you control, then sacrifice the rest", priority: 400, parse: keep_and_sacrifice } }

/// The chosen cards of "choose N cards in your hand, then [verb] the rest".
const KEPT: Var = vars::USER + 2745;

/// "choose a card in your hand and discard the rest", "choose two cards in your graveyard
/// and exile the rest", "choose up to seven cards in your hand, then shuffle the rest into
/// your library".
fn keep_cards_rest(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("choose ")?;
    let (choice, verb) = r
        .split_once(", then ")
        .or_else(|| r.split_once(" and "))?;
    let (up_to, choice) = match choice.strip_prefix("up to ") {
        Some(c) => (true, c),
        None => (false, choice),
    };
    let (n, rest) = parse_number(choice)?;
    n.as_const()?;
    let zone = match rest.trim() {
        "card in your hand" | "cards in your hand" => ZoneKind::Hand,
        "card in your graveyard" | "cards in your graveyard" => ZoneKind::Graveyard,
        _ => return None,
    };
    let cards = Filter::and(vec![
        Filter::InZone(zone),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    let rest_cards = Filter::and(vec![
        cards.clone(),
        Filter::not(Filter::In(Box::new(Sel::Var(KEPT)))),
    ]);
    let act = match (verb, zone) {
        ("discard the rest", ZoneKind::Hand) => Effect::Discard {
            who: PlayerRef::You,
            n: Value::CountSel(Box::new(Sel::All(rest_cards.clone()))),
            random: false,
            filter: rest_cards,
        },
        ("exile the rest", _) => Effect::Exile {
            what: Sel::All(rest_cards),
            face_down: false,
            link: false,
        },
        ("shuffle the rest into your library", _) => Effect::ShuffleIntoLibrary {
            what: Sel::All(rest_cards),
            library: PlayerRef::You,
        },
        _ => return None,
    };
    Some(Effect::seq(vec![
        Effect::Store {
            var: KEPT,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: cards,
                count: n,
                up_to,
                store: None,
            },
        },
        act,
    ]))
}

inventory::submit! { EffectPattern { name: "player actions: choose N cards in your hand, then discard the rest", priority: 400, parse: keep_cards_rest } }

/// "untap a land you control" (not targeted: chosen as it resolves).
fn untap_one(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("untap ")?;
    let (n, rest) = parse_number(r)?;
    if !matches!(n, Value::Const(1)) || !(r.starts_with("a ") || r.starts_with("an ")) {
        return None;
    }
    let f = permanents_you_control(rest)?;
    Some(Effect::Untap {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![Filter::Permanent, f]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "player actions: untap a [permanent] you control", priority: 400, parse: untap_one } }

/// "reveal your hand".
fn reveal_your_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    if l != "reveal your hand" {
        return None;
    }
    let _ = b;
    Some(Effect::RevealHand {
        who: PlayerRef::You,
    })
}

inventory::submit! { EffectPattern { name: "player actions: reveal your hand", priority: 400, parse: reveal_your_hand } }


/// "put a rope counter on a creature you control" (not targeted: chosen as it resolves).
fn counter_on_one(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("put ")?;
    let (n, r) = parse_number(r)?;
    let (kind, rest) = crate::oracle::costs::counter_kind(r)?;
    let rest = rest
        .trim_start()
        .strip_prefix("counters on ")
        .or_else(|| rest.trim_start().strip_prefix("counter on "))?;
    let one = rest.strip_prefix("a ").or_else(|| rest.strip_prefix("an "))?;
    let f = permanents_you_control(one)?;
    Some(Effect::AddCounters {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![Filter::Permanent, f]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        kind,
        n,
    })
}

inventory::submit! { EffectPattern { name: "player actions: put a counter on a [permanent] you control", priority: 400, parse: counter_on_one } }

/// "mill cards equal to [amount]", "discard cards equal to [amount]".
fn cards_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let (verb, r) = split_word(l);
    let r = r.strip_prefix("cards equal to ")?;
    let (n, tail) = crate::oracle::statics::parse_value_phrase(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    match verb {
        "mill" => Some(Effect::Mill {
            who: PlayerRef::You,
            n,
        }),
        "discard" => Some(Effect::Discard {
            who: PlayerRef::You,
            n,
            random: false,
            filter: Filter::Any,
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "player actions: mill/discard cards equal to [amount]", priority: 400, parse: cards_equal_to } }

/// "draw up to three cards": the player chooses how many, then draws that many.
fn draw_up_to(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let r = l.strip_prefix("draw up to ")?;
    let (n, rest) = parse_card_count(r)?;
    let n = n.as_const()?;
    if !end(rest).is_empty() {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Number { min: 0, max: n },
        },
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::Chosen,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "player actions: draw up to N cards", priority: 400, parse: draw_up_to } }
