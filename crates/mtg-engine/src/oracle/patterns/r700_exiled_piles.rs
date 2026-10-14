//! Piles formed as cards are exiled, face-down piles, three piles, piles chosen at random,
//! each player's own piles, and ordered face-down piles in exile (CR 700.3, 406.3, 406.4,
//! 701.24a; see `piles.rs`):
//!
//! * "Exile the top four cards of your library in a face-down pile, then exile the top
//!   four cards of your library in a face-up pile." (Abstract Performance; "in another
//!   face-down pile", Hostile Negotiations), "Exile any number of them in a face-down pile
//!   and the rest in a face-up pile." (The Celestial Toymaker);
//! * "Look at the cards in the other pile." / "... in each pile", "turn a pile of your
//!   choice face up", "Put that pile into your graveyard.", "You may cast a spell from
//!   among them without paying its mana cost. Put the rest into your hand.";
//! * "target opponent looks at the top ten cards of your library and separates them into
//!   two face-down piles. Exile one of those piles. Search the other pile for a card, put
//!   it into your hand, then shuffle the rest of that pile into your library." (Phyrexian
//!   Portal);
//! * "Put all cards from the pile of your choice onto the battlefield under your control
//!   and the rest into their owners' graveyards." (Boneyard Parley); "You may play lands
//!   and cast spells from one of those piles. If you cast a spell this way, you cast it
//!   without paying its mana cost." (Brilliant Ultimatum);
//! * each player's piles: "Each opponent separates the creatures they control into two
//!   piles. For each opponent, you choose one of their piles. Each opponent sacrifices the
//!   creatures in their chosen pile." (Make an Example), "Each player separates all
//!   nontoken lands they control into two piles. For each player, one of their piles is
//!   chosen by one of their opponents of their choice. Destroy all lands in the chosen
//!   piles. Tap all lands in the other piles." (Bend or Break), "Starting with you, each
//!   player separates all permanents they control into three piles. Then each player
//!   chooses one of their piles at random and sacrifices those permanents." (Whims of the
//!   Fates);
//! * ordered face-down piles: "search your library for seven cards, exile them in a
//!   face-down pile, and shuffle that pile" with "If you would draw a card, you may instead
//!   put the top card of the pile you exiled into your hand." (Parallel Thoughts) and "The
//!   next time you would draw a card this turn, instead put the top card of the exiled pile
//!   into its owner's hand." (Mangara's Tome); "exile it and the top six cards of your
//!   library in a face-down pile. If you do, shuffle that pile and put it back on top of
//!   your library." (Triumph of Saint Katherine).

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::piles::*;
use crate::types::CardType;

fn piles(a: PileAction) -> Effect {
    Effect::Piles(Box::new(a))
}

fn choose(chooser: PlayerRef) -> Effect {
    piles(PileAction::Choose {
        chooser,
        picked_by: None,
        of: None,
        at_random: false,
    })
}

fn in_pile(var: Var, f: Vec<Filter>) -> Filter {
    let mut v = vec![Filter::In(Box::new(Sel::Var(var)))];
    v.extend(f);
    Filter::and(v)
}

fn is_pile_var(s: &Sel) -> Option<Var> {
    match s {
        Sel::Var(v) if [CHOSEN, OTHER, PILE_A, PILE_B, PILE_C].contains(v) => Some(*v),
        _ => None,
    }
}

/// "the top four cards of your library" → 4.
fn top_cards_of_library(r: &str) -> Option<(Value, &str)> {
    let r = r.strip_prefix("the top ")?;
    let (n, r) = parse_number(r)?;
    n.as_const()?;
    let r = r.trim_start().strip_prefix("cards of your library")?;
    Some((n, r))
}

/// "exile the top four cards of your library in a face-down pile[, then exile the top four
/// cards of your library in a face-up pile / in another face-down pile]".
fn exile_in_piles(l: &str, _b: &mut Builder) -> Option<Effect> {
    let parts: Vec<&str> = end(l).split(", then ").collect();
    if parts.len() > 2 {
        return None;
    }
    let mut out = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        let (n, r) = top_cards_of_library(part.strip_prefix("exile ")?)?;
        let (another, face_down) = match r.trim() {
            "in a face-down pile" => (false, true),
            "in a face-up pile" => (false, false),
            "in another face-down pile" => (true, true),
            "in another face-up pile" => (true, false),
            _ => return None,
        };
        if i == 0 && another {
            return None;
        }
        out.push(piles(PileAction::ExileAsPile {
            what: Sel::TopOfLibrary(PlayerRef::You, n),
            pile: i as u8,
            face_down,
            seen_by: None,
        }));
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "r700 exile cards in a face-down pile", priority: 80, parse: exile_in_piles } }

/// "exile any number of them in a face-down pile and the rest in a face-up pile": you
/// looked at them, so you may go on looking at the face-down ones (CR 406.3).
fn exile_any_number_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile any number of ")?;
    let r = r.strip_suffix(" in a face-down pile and the rest in a face-up pile")?;
    let what = match r {
        "them" | "those cards" => b.it.clone(),
        _ => return None,
    };
    if matches!(what, Sel::None) {
        return None;
    }
    Some(Effect::seq(vec![
        piles(PileAction::Separate {
            what,
            separator: PlayerRef::You,
            count: 0,
            face_down: false,
            own: false,
        }),
        piles(PileAction::ExileAsPile {
            what: Sel::Var(PILE_A),
            pile: 0,
            face_down: true,
            seen_by: Some(PlayerRef::You),
        }),
        piles(PileAction::ExileAsPile {
            what: Sel::Var(PILE_B),
            pile: 1,
            face_down: false,
            seen_by: None,
        }),
    ]))
}

inventory::submit! { EffectPattern { name: "r700 exile any number of them in a face-down pile and the rest in a face-up pile", priority: 80, parse: exile_any_number_face_down } }

/// "put that pile into your hand", "put that pile into your graveyard".
fn put_that_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put that pile ")?;
    let zone = match r {
        "into your hand" => ZoneKind::Hand,
        "into your graveyard" => ZoneKind::Graveyard,
        _ => return None,
    };
    Some(Effect::Move {
        what: Sel::Var(CHOSEN),
        to: Destination::zone(zone),
    })
}

inventory::submit! { EffectPattern { name: "r700 put that pile into a zone", priority: 80, parse: put_that_pile } }

/// "look at the cards in the other pile", "look at the cards in each pile" (CR 406.3).
fn look_at_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let what = match end(l) {
        "look at the cards in the other pile" => Sel::Var(OTHER),
        "look at the cards in that pile" => Sel::Var(CHOSEN),
        "look at the cards in each pile" => Sel::Union(vec![Sel::Var(PILE_A), Sel::Var(PILE_B)]),
        _ => return None,
    };
    b.it = what.clone();
    Some(piles(PileAction::Look { what }))
}

inventory::submit! { EffectPattern { name: "r700 look at the cards in a pile", priority: 80, parse: look_at_pile } }

/// "turn a pile of your choice face up".
fn turn_pile_face_up(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "turn a pile of your choice face up").then(|| {
        piles(PileAction::TurnFaceUp {
            chooser: PlayerRef::You,
        })
    })
}

inventory::submit! { EffectPattern { name: "r700 turn a pile of your choice face up", priority: 80, parse: turn_pile_face_up } }

/// "you may cast a spell from among them without paying its mana cost" (them: an exiled
/// pile): cast as the effect resolves (CR 608.2g), without paying its mana cost
/// (CR 118.9), ignoring timing permissions based on its card type.
fn cast_one_from_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you may ").unwrap_or(l);
    if l != "cast a spell from among them without paying its mana cost" {
        return None;
    }
    let var = is_pile_var(&b.it)?;
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: in_pile(
                var,
                vec![
                    Filter::InZone(ZoneKind::Exile),
                    Filter::Not(Box::new(Filter::Type(CardType::Land))),
                ],
            ),
            count: Value::c(1),
            up_to: true,
            store: None,
        },
        free: true,
        optional: false,
    })
}

inventory::submit! { EffectPattern { name: "r700 you may cast a spell from among the pile without paying its mana cost", priority: 80, parse: cast_one_from_pile } }

/// "put the rest into your hand": the cards of the exiled pile that weren't cast.
fn put_rest_of_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let zone = match end(l) {
        "put the rest into your hand" => ZoneKind::Hand,
        "put the rest into your graveyard" => ZoneKind::Graveyard,
        _ => return None,
    };
    let var = is_pile_var(&b.it)?;
    Some(Effect::Move {
        what: Sel::All(in_pile(var, vec![Filter::InZone(ZoneKind::Exile)])),
        to: Destination::zone(zone),
    })
}

inventory::submit! { EffectPattern { name: "r700 put the rest of the pile into a zone", priority: 80, parse: put_rest_of_pile } }

/// "[if your library has ten or more cards in it, ]target opponent looks at the top ten
/// cards of your library and separates them into two face-down piles".
fn separate_into_face_down_piles(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (cond, l) = match l.strip_prefix("if your library has ") {
        Some(r) => {
            let (n, r) = parse_number(r)?;
            n.as_const()?;
            let r = r.trim_start().strip_prefix("or more cards in it, ")?;
            (
                Some(Condition::Compare(
                    Value::LibrarySize(PlayerRef::You),
                    Cmp::Ge,
                    n,
                )),
                r,
            )
        }
        None => (None, l),
    };
    let (subject, r) = l.split_once(" looks at ")?;
    let (n, r) = top_cards_of_library(r)?;
    if r != " and separates them into two face-down piles" || !subject.starts_with("target ") {
        return None;
    }
    let saved = b.targets.len();
    let (who, rest) = match player_ref(subject, b) {
        Some(x) => x,
        None => {
            b.targets.truncate(saved);
            return None;
        }
    };
    if !rest.trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    let e = Effect::seq(vec![
        Effect::Dig {
            who: PlayerRef::You,
            n,
            reveal: false,
            filter: Filter::Any,
            take: Value::c(0),
            take_up_to: false,
            take_to: Destination::zone(ZoneKind::Hand),
            rest_to: Destination {
                position: LibraryPosition::FromTop(0),
                ..Destination::library_top()
            },
        },
        piles(PileAction::Separate {
            what: Sel::Var(vars::REVEALED),
            separator: who,
            count: 0,
            face_down: true,
            own: false,
        }),
    ]);
    Some(match cond {
        Some(cond) => Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
        None => e,
    })
}

inventory::submit! { EffectPattern { name: "r700 separates them into two face-down piles", priority: 80, parse: separate_into_face_down_piles } }

/// "exile one of those piles": you choose which.
fn exile_one_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "exile one of those piles").then(|| {
        Effect::seq(vec![
            choose(PlayerRef::You),
            Effect::Exile {
                what: Sel::Var(CHOSEN),
                face_down: false,
                link: false,
            },
        ])
    })
}

inventory::submit! { EffectPattern { name: "r700 exile one of those piles", priority: 80, parse: exile_one_pile } }

/// "search the other pile for a card, put it into your hand, then shuffle the rest of that
/// pile into your library".
fn search_other_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l)
        == "search the other pile for a card, put it into your hand, then shuffle the rest of that pile into your library")
        .then(|| piles(PileAction::SearchPile { what: Sel::Var(OTHER) }))
}

inventory::submit! { EffectPattern { name: "r700 search the other pile for a card", priority: 80, parse: search_other_pile } }

/// "put all cards from the pile of your choice onto the battlefield under your control and
/// the rest into their owners' graveyards".
fn pile_of_your_choice_onto_battlefield(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l)
        != "put all cards from the pile of your choice onto the battlefield under your control and the rest into their owners' graveyards"
    {
        return None;
    }
    Some(Effect::seq(vec![
        choose(PlayerRef::You),
        Effect::Move {
            what: Sel::Var(CHOSEN),
            to: Destination {
                controller: Some(PlayerRef::You),
                ..Destination::zone(ZoneKind::Battlefield)
            },
        },
        Effect::Move {
            what: Sel::Var(OTHER),
            to: Destination::zone(ZoneKind::Graveyard),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "r700 put the pile of your choice onto the battlefield", priority: 80, parse: pile_of_your_choice_onto_battlefield } }

/// "you may play lands and cast spells from one of those piles": you choose a pile, then
/// play any of its cards as the effect resolves (CR 608.2g; a land only if you could
/// play one now, CR 305.2b).
fn play_from_one_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you may ").unwrap_or(l);
    if l != "play lands and cast spells from one of those piles" {
        return None;
    }
    Some(Effect::seq(vec![
        choose(PlayerRef::You),
        Effect::PlayCard {
            who: PlayerRef::You,
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: in_pile(CHOSEN, vec![Filter::InZone(ZoneKind::Exile)]),
                count: Value::c(99),
                up_to: true,
                store: None,
            },
            free: false,
            optional: false,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "r700 play lands and cast spells from one of those piles", priority: 80, parse: play_from_one_pile } }

/// "If you cast a spell this way, you cast it without paying its mana cost." after
/// [`play_from_one_pile`].
fn cast_this_way_free(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l != "if you cast a spell this way, you cast it without paying its mana cost" {
        return false;
    }
    // "You may play ...": the choice of a pile and the play.
    let prev = match prev {
        Effect::May { effect, .. } => &mut **effect,
        e => e,
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    let from_pile = |f: &Filter| {
        matches!(f, Filter::And(v)
            if matches!(v.first(), Some(Filter::In(s)) if matches!(**s, Sel::Var(CHOSEN))))
    };
    match v.last_mut() {
        Some(Effect::PlayCard {
            what: Sel::Choose { filter, .. },
            free,
            ..
        }) if from_pile(filter) => {
            *free = true;
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "r700 if you cast a spell this way, you cast it without paying its mana cost", priority: 80, apply: cast_this_way_free } }

// --- Each player's own piles ----------------------------------------------------------

/// "[objects] they control" → the objects the separating player controls.
fn they_control(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("all ").or_else(|| s.strip_prefix("the ")).unwrap_or(s);
    let head = s.strip_suffix(" they control")?;
    let (f, _, tail) = parse_object_phrase(head)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]))
}

fn own_separation(what: Filter, count: u8) -> Effect {
    Effect::AsPlayer {
        who: PlayerRef::Iterated,
        effect: Box::new(piles(PileAction::Separate {
            what: Sel::All(what),
            separator: PlayerRef::You,
            count,
            face_down: false,
            own: true,
        })),
    }
}

/// "each opponent separates the creatures they control into two piles", "each player
/// separates all nontoken lands they control into two piles", "starting with you, each
/// player separates all permanents they control into three piles".
fn each_player_separates(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (in_order, l) = match l.strip_prefix("starting with you, ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let (who, r) = if let Some(r) = l.strip_prefix("each player separates ") {
        (PlayerRef::EachPlayer, r)
    } else if let Some(r) = l.strip_prefix("each opponent separates ") {
        (PlayerRef::EachOpponent, r)
    } else {
        return None;
    };
    let (objs, count) = if let Some(o) = r.strip_suffix(" into two piles") {
        (o, 2)
    } else if let Some(o) = r.strip_suffix(" into three piles") {
        (o, 3)
    } else {
        return None;
    };
    let what = they_control(objs)?;
    let body = own_separation(what, count);
    Some(if in_order {
        if !matches!(who, PlayerRef::EachPlayer) {
            return None;
        }
        Effect::InTurnOrder {
            first: TurnOrderStart::You,
            who: PlayerFilter::Any,
            effect: Box::new(body),
            direction: SeatSide::Left,
        }
    } else {
        Effect::ForEachPlayer {
            who,
            effect: Box::new(body),
        }
    })
}

inventory::submit! { EffectPattern { name: "r700 each player separates the permanents they control into piles", priority: 50, parse: each_player_separates } }

/// "for each opponent, you choose one of their piles", "for each player, one of their
/// piles is chosen by one of their opponents of their choice".
fn for_each_player_choose(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (who, choice) = match end(l) {
        "for each opponent, you choose one of their piles" => (
            PlayerRef::EachOpponent,
            PileAction::Choose {
                chooser: PlayerRef::You,
                picked_by: None,
                of: Some(PlayerRef::Iterated),
                at_random: false,
            },
        ),
        "for each player, one of their piles is chosen by one of their opponents of their choice" => (
            PlayerRef::EachPlayer,
            PileAction::Choose {
                chooser: PlayerRef::Each(PlayerFilter::OpponentOf(Box::new(PlayerRef::Iterated))),
                picked_by: Some(PlayerRef::Iterated),
                of: Some(PlayerRef::Iterated),
                at_random: false,
            },
        ),
        _ => return None,
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(piles(choice)),
    })
}

inventory::submit! { EffectPattern { name: "r700 for each player, one of their piles is chosen", priority: 50, parse: for_each_player_choose } }

/// "each opponent sacrifices the creatures in their chosen pile" ("their chosen pile": the
/// pile of their permanents that was chosen for them).
fn sacrifice_their_chosen_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each opponent sacrifices ")?;
    let objs = r.strip_suffix(" in their chosen pile")?;
    let objs = objs.strip_prefix("the ").or_else(|| objs.strip_prefix("all ")).unwrap_or(objs);
    let (f, _, tail) = parse_object_phrase(objs)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachOpponent,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::seq(vec![
                piles(PileAction::Recall { of: PlayerRef::You }),
                Effect::SacrificeObjects {
                    what: Sel::All(in_pile(
                        CHOSEN,
                        vec![f, Filter::ControlledBy(PlayerRel::You)],
                    )),
                },
            ])),
        }),
    })
}

inventory::submit! { EffectPattern { name: "r700 each opponent sacrifices the creatures in their chosen pile", priority: 50, parse: sacrifice_their_chosen_pile } }

/// "each player chooses one of their piles at random and sacrifices those permanents".
fn sacrifice_random_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player chooses one of their piles at random and sacrifices those permanents" {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::seq(vec![
                piles(PileAction::Choose {
                    chooser: PlayerRef::You,
                    picked_by: None,
                    of: Some(PlayerRef::You),
                    at_random: true,
                }),
                piles(PileAction::Recall { of: PlayerRef::You }),
                Effect::SacrificeObjects {
                    what: Sel::All(in_pile(
                        CHOSEN,
                        vec![Filter::ControlledBy(PlayerRel::You)],
                    )),
                },
            ])),
        }),
    })
}

inventory::submit! { EffectPattern { name: "r700 each player chooses one of their piles at random and sacrifices it", priority: 50, parse: sacrifice_random_pile } }

/// "destroy all lands in the chosen piles", "tap all lands in the other piles".
fn all_in_chosen_piles(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (destroy, r) = if let Some(r) = l.strip_prefix("destroy all ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("tap all ") {
        (false, r)
    } else {
        return None;
    };
    let (objs, var) = if let Some(o) = r.strip_suffix(" in the chosen piles") {
        (o, ALL_CHOSEN)
    } else if let Some(o) = r.strip_suffix(" in the other piles") {
        (o, ALL_OTHER)
    } else {
        return None;
    };
    let (f, _, tail) = parse_object_phrase(objs)?;
    if !end(tail).is_empty() {
        return None;
    }
    let what = Sel::All(in_pile(var, vec![f]));
    Some(if destroy {
        Effect::Destroy {
            what,
            no_regen: false,
        }
    } else {
        Effect::Tap { what }
    })
}

inventory::submit! { EffectPattern { name: "r700 all [objects] in the chosen piles", priority: 50, parse: all_in_chosen_piles } }

// --- Ordered face-down piles in exile ---------------------------------------------------

/// "search your library for seven cards, exile them in a face-down pile, and shuffle that
/// pile": the cards are exiled face down; once the pile is shuffled, no one may look at
/// them (CR 406.3, 701.24a).
fn search_exile_face_down_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("search your library for ")?;
    let (n, r) = parse_number(r)?;
    n.as_const()?;
    if r.trim_start() != "cards, exile them in a face-down pile, and shuffle that pile" {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Search {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            filter: Filter::and(vec![Filter::Card, Filter::InZone(ZoneKind::Library)]),
            count: n,
            to: Destination {
                face_down: true,
                ..Destination::zone(ZoneKind::Exile)
            },
            reveal: false,
            shuffle: false,
        },
        piles(PileAction::Shuffle {
            what: Sel::Var(vars::IT),
        }),
    ]))
}

inventory::submit! { EffectPattern { name: "r700 search your library for cards, exile them in a face-down pile, and shuffle that pile", priority: 80, parse: search_exile_face_down_pile } }

/// "put the top card of the pile you exiled into your hand", "put the top card of the
/// exiled pile into its owner's hand": the cards the source's linked ability exiled
/// (CR 607.2a), in the pile's order.
fn top_of_linked_pile_to_hand(l: &str) -> Option<Effect> {
    match end(l) {
        "put the top card of the pile you exiled into your hand"
        | "put the top card of the exiled pile into its owner's hand" => {}
        _ => return None,
    }
    Some(Effect::seq(vec![
        piles(PileAction::Top { from: Sel::Linked }),
        Effect::Move {
            what: Sel::Var(vars::IT),
            to: Destination::zone(ZoneKind::Hand),
        },
    ]))
}

/// "If you would draw a card, you may instead put the top card of the pile you exiled into
/// your hand." A replacement effect the player may apply to each draw, even with no cards
/// left in the pile (then the draw is just skipped).
fn may_instead_take_from_pile(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if you would draw a card, you may instead ")?;
    let effect = top_of_linked_pile_to_hand(r)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Draw(PlayerFilter::You),
                action: ReplacementAction::Instead(Box::new(effect)),
                self_replacement: false,
                optional: true,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r700 if you would draw a card, you may instead put the top card of the pile you exiled into your hand", priority: 80, parse: may_instead_take_from_pile } }

/// "the next time you would draw a card this turn, instead put the top card of the exiled
/// pile into its owner's hand".
fn next_draw_from_pile(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the next time you would draw a card this turn, instead ")?;
    let effect = top_of_linked_pile_to_hand(r)?;
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(effect)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "r700 the next time you would draw a card this turn, instead put the top card of the exiled pile into its owner's hand", priority: 80, parse: next_draw_from_pile } }

/// "exile it and the top six cards of your library in a face-down pile": all of them or
/// none (nothing is exiled if it's no longer in your graveyard or your library has fewer
/// than six cards).
fn exile_it_and_top_cards_in_pile(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile it and ")?;
    let r = r.strip_suffix(" in a face-down pile")?;
    let (n, rest) = top_cards_of_library(r)?;
    if !rest.is_empty() || !b.in_trigger || !matches!(b.it, Sel::This | Sel::TriggerObject) {
        return None;
    }
    let it = Sel::This;
    let cond = Condition::And(vec![
        Condition::SelMatches(it.clone(), Filter::InZone(ZoneKind::Graveyard)),
        Condition::Compare(Value::LibrarySize(PlayerRef::You), Cmp::Ge, n.clone()),
    ]);
    b.it = Sel::Var(vars::IT);
    Some(Effect::If {
        cond,
        then: Box::new(Effect::Exile {
            what: Sel::Union(vec![it, Sel::TopOfLibrary(PlayerRef::You, n)]),
            face_down: true,
            link: false,
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "r700 exile it and the top cards of your library in a face-down pile", priority: 80, parse: exile_it_and_top_cards_in_pile } }

/// "shuffle that pile and put it back on top of your library".
fn shuffle_pile_onto_library(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "shuffle that pile and put it back on top of your library" {
        return None;
    }
    Some(Effect::seq(vec![
        piles(PileAction::Shuffle {
            what: Sel::Var(vars::IT),
        }),
        piles(PileAction::PutOnLibrary {
            what: Sel::Var(vars::IT),
        }),
    ]))
}

inventory::submit! { EffectPattern { name: "r700 shuffle that pile and put it back on top of your library", priority: 80, parse: shuffle_pile_onto_library } }
