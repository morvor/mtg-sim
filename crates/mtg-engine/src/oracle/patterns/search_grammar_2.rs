//! Searches by name (CR 701.23, 201.2) for objects other than a target, and other search
//! forms the search grammar (`search_grammar.rs`) doesn't read:
//!
//! * "Target player reveals their hand, then you choose a card other than a basic land
//!   card from it. Search that player's graveyard, hand, and library for all cards with
//!   the same name as the chosen card and exile them. Then that player shuffles."
//!   (Lobotomy, Shimian Specter);
//! * "For each card exiled this way, search that player's library for all cards with the
//!   same name as that card and exile them." (Haunting Echoes, Reap Intellect);
//! * "search the graveyard, hand, and library of that spell's controller for any number
//!   of cards with the same name as that spell, exile those cards, then that player
//!   shuffles" (Invasive Surgery).
//!
//! The zones are those of the named object's controller (for a card, its owner). In the
//! graveyard every such card is found ("all cards"); in the hand and the library the
//! searching player may leave some (CR 701.23b). See `kw/search_same_name.rs`.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::search_same_name::{general_effect, parse_general};
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

/// The cards a same-name search found.
const FOUND: Var = vars::USER + 2312;

fn search_same_name_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("then ").unwrap_or(l).strip_prefix("search ")?;
    let (all_zones, r) = [
        ("that player's graveyard, hand, and library for ", true),
        ("their graveyard, hand, and library for ", true),
        (
            "the graveyard, hand, and library of that spell's controller for ",
            true,
        ),
        ("that player's library for ", false),
        ("their library for ", false),
    ]
    .iter()
    .find_map(|(p, z)| r.strip_prefix(p).map(|r| (*z, r)))?;
    // "that player's": the zones of the player the text is about (a target or the
    // trigger's player), searched even if no card was chosen (CR 701.23c).
    let named_player = (!l.contains("of that spell's controller"))
        .then(|| b.it_player.clone())
        .filter(|p| matches!(p, PlayerRef::Target(_) | PlayerRef::TriggerPlayer));
    let (any_number, r) = if let Some(x) = r.strip_prefix("all cards with the same name as ") {
        (false, x)
    } else {
        (
            true,
            r.strip_prefix("any number of cards with the same name as ")?,
        )
    };
    // "and exile them", ", exile those cards[, then that player shuffles]".
    let (obj, shuffles) = if let Some(o) = r.strip_suffix(" and exile them") {
        (o, false)
    } else if let Some(o) = r.strip_suffix(", exile those cards, then that player shuffles") {
        (o, true)
    } else {
        (r.strip_suffix(", exile those cards")?, false)
    };
    if !matches!(
        obj,
        "the chosen card" | "that card" | "that spell" | "that creature" | "that permanent"
    ) {
        return None;
    }
    let saved = b.targets.len();
    let (sel, rest) = object_ref(obj, b)?;

    if !rest.trim().is_empty() || b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    let search = general_effect(all_zones, any_number, &sel, FOUND, named_player.as_ref())?;
    let who = named_player.unwrap_or_else(|| PlayerRef::ControllerOf(Box::new(sel)));
    if matches!(b.it_player, PlayerRef::You) {
        b.it_player = who.clone();
    }
    let mut v = vec![
        search,
        Effect::Exile {
            what: Sel::Var(FOUND),
            face_down: false,
            link: false,
        },
    ];
    if shuffles {
        v.push(Effect::Shuffle { who });
    }
    Some(Effect::seq(v))
}

inventory::submit! {
    EffectPattern { name: "search a player's zones for cards with the same name as an object", priority: 89, parse: search_same_name_of }
}

/// The player whose zones the last same-name search of [`search_same_name_of`] in `e`
/// searched: the named object's controller, or, for a search repeated for each of
/// several objects, the player the text is about (`it_player`).
fn general_search_player(e: &Effect, it_player: &PlayerRef) -> Option<PlayerRef> {
    use crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent;
    match e {
        Effect::Custom(n) => {
            let s = parse_general(n)?;
            Some(
                s.player
                    .unwrap_or_else(|| PlayerRef::ControllerOf(Box::new(s.object))),
            )
        }
        Effect::Seq(v) => v
            .iter()
            .rev()
            .find_map(|x| general_search_player(x, it_player)),
        Effect::ForEach { effect, .. } => {
            general_search_player(effect, it_player)?;
            (!matches!(it_player, PlayerRef::You) && !is_no_player_referent(it_player))
                .then(|| it_player.clone())
        }
        Effect::If { then, .. } => general_search_player(then, it_player),
        _ => None,
    }
}

/// "Then that player shuffles." after it: the player whose library was searched.
fn then_that_player_shuffles(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(l, "then that player shuffles" | "that player shuffles") {
        return false;
    }
    let Some(who) = general_search_player(prev, &b.it_player) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, Effect::Shuffle { who }]);
    true
}

inventory::submit! {
    FollowupPattern { name: "then that player shuffles (after a same-name search of an object)", priority: 101, apply: then_that_player_shuffles }
}

// ---------------------------------------------------------------------------
// Found cards chosen among by another player (Gifts Ungiven)
// ---------------------------------------------------------------------------

/// The cards a search found and revealed, left where they are for the instructions that
/// follow.
const REVEALED_FOUND: Var = vars::USER + 2313;

/// The search of `e` that leaves the cards it finds where they are, if `e` is one.
fn search_in_place(e: &mut Effect) -> Option<&mut SearchSpec> {
    match e {
        Effect::SearchCards(s) if s.dests.is_empty() && s.shuffle == SearchShuffle::No => {
            Some(s)
        }
        Effect::Seq(v) if v.len() == 1 => search_in_place(&mut v[0]),
        Effect::May { effect, .. } => search_in_place(effect),
        _ => None,
    }
}

/// The search, revealing the cards found, and the cards noted for what follows ("An
/// opponent chooses two of those cards.").
fn noted(e: Effect, b: &mut Builder) -> Effect {
    super::dig_grammar::mark_dug(b, Sel::Var(REVEALED_FOUND));
    b.it = Sel::Var(REVEALED_FOUND);
    Effect::seq(vec![
        e,
        Effect::Store {
            var: REVEALED_FOUND,
            sel: Sel::Var(vars::IT),
        },
    ])
}

/// "Search your library for up to four cards with different names and reveal them."
/// (Gifts Ungiven, Threats Undetected, Ecological Appreciation): the cards found are
/// revealed and stay where they are; later instructions say where they go (CR 701.23).
fn search_and_reveal_them(l: &str, b: &mut Builder) -> Option<Effect> {
    let head = end(l).strip_suffix(" and reveal them")?;
    if !head.starts_with("search ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = crate::oracle::effects::parse_clause(head, b)?;
    let Some(spec) = search_in_place(&mut e) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    spec.reveal = true;
    Some(noted(e, b))
}

inventory::submit! {
    EffectPattern { name: "search for cards and reveal them (they stay for what follows)", priority: 96, parse: search_and_reveal_them }
}

/// "If you do, reveal those cards." after "You may search your library for exactly two
/// cards ..." (Burning-Rune Demon): the cards found are revealed (only if the player
/// searched) and stay where they are for what follows.
fn if_you_do_reveal_those_cards(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(l, "if you do, reveal those cards" | "if you do, reveal them") {
        return false;
    }
    let may = matches!(prev, Effect::May { .. });
    let Some(spec) = search_in_place(prev) else {
        return false;
    };
    if !spec.optional && !may {
        return false;
    }
    spec.reveal = true;
    let old = std::mem::take(prev);
    *prev = noted(old, b);
    true
}

inventory::submit! {
    FollowupPattern { name: "if you do, reveal those cards (after a search)", priority: 96, apply: if_you_do_reveal_those_cards }
}

/// "An opponent chooses two of those cards.", "Target opponent chooses two of those
/// cards.", "An opponent chooses one of them.": among the cards dug or found, which stay
/// where they are ("the chosen cards").
fn opponent_chooses(l: &str, b: &mut Builder) -> Option<Effect> {
    // After a search that left the cards it found (see `noted`); the choice grammar reads
    // other choices.
    let found = b.named.iter().any(|(n, s)| {
        n == super::dig_grammar::DIG_MARK && matches!(s, Sel::Var(v) if *v == REVEALED_FOUND)
    });
    if !found {
        return None;
    }
    let l = end(l);
    let (chooser, r) = if let Some(r) = l.strip_prefix("an opponent chooses ") {
        (PlayerRef::EachOpponent, r)
    } else if let Some(r) = l.strip_prefix("target opponent chooses ") {
        let slot = b.add_target(
            TargetSpec::player(PlayerFilter::Opponent, "target opponent"),
            "target opponent",
        );
        (PlayerRef::Target(slot), r)
    } else {
        return None;
    };
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    if !matches!(r.trim(), "of those cards" | "of them") || n.as_const().is_none() {
        return None;
    }
    Some(super::dig_grammar::choose_in_place(chooser, n, b))
}

inventory::submit! {
    EffectPattern { name: "an opponent chooses N of those cards (found by a search)", priority: 50, parse: opponent_chooses }
}

/// The cards an earlier instruction named ("the chosen cards", "the rest"), if `s` is
/// one of those names exactly.
fn named_cards(s: &str, b: &Builder) -> Option<Sel> {
    b.named.iter().rev().find_map(|(n, sel)| (n == s).then(|| sel.clone()))
}

/// "Shuffle the chosen cards into your library", "put the rest onto the battlefield
/// [tapped]" about cards an earlier choice named (Threats Undetected, Elemental
/// Teachings): each is shuffled into (put onto the battlefield from) where it is.
fn move_named_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("shuffle ") {
        let what = named_cards(r.strip_suffix(" into your library")?, b)?;
        return Some(Effect::ShuffleIntoLibrary {
            what,
            library: PlayerRef::You,
        });
    }
    let r = l.strip_prefix("put ")?;
    let (name, dest) = r.split_once(" onto the battlefield")?;
    let what = named_cards(name, b)?;
    let mut to = Destination::battlefield().under_your_control();
    // "..., then shuffle", "..., then that player shuffles".
    let (dest, shuffle) = if let Some(d) = dest.strip_suffix(", then shuffle") {
        (d, Some(PlayerRef::You))
    } else if let Some(d) = dest.strip_suffix(", then that player shuffles") {
        if !matches!(b.it_player, PlayerRef::Target(_)) {
            return None;
        }
        (d, Some(b.it_player.clone()))
    } else {
        (dest, None)
    };
    let dest = dest.strip_suffix(" under your control").unwrap_or(dest);
    match dest {
        "" => {}
        " tapped" => to.tapped = true,
        _ => return None,
    }
    let mv = Effect::Move { what, to };
    Some(match shuffle {
        Some(who) => Effect::seq(vec![mv, Effect::Shuffle { who }]),
        None => mv,
    })
}

inventory::submit! {
    EffectPattern { name: "shuffle / put onto the battlefield the cards a choice named", priority: 990, parse: move_named_cards }
}

// ---------------------------------------------------------------------------
// A search for each object, for a card with its name (Doubling Chant)
// ---------------------------------------------------------------------------

/// The cards all the searches found ("those cards").
const ALL_FOUND: Var = vars::USER + 2315;

/// "For each creature you control, you may search your library for a creature card with
/// the same name as that creature." (Doubling Chant), "For each of those permanents, ..."
/// (Clarion Ultimatum), "For each tapped nonland permanent target opponent controls,
/// search that player's library for a card with the same name as that permanent."
/// (Dichotomancy): one search for each object, for a card with its name as the search
/// happens; the cards found stay where they are for the next instruction ("Put those
/// cards onto the battlefield, then shuffle.").
fn for_each_search_same_name(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (objs, body) = r.split_once(", ")?;
    let sel = if let Some(o) = objs.strip_prefix("of ") {
        let (sel, rest) = object_ref(o, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        sel
    } else {
        let (f, _, rest) = crate::oracle::phrases::parse_object_phrase(objs)?;
        let (f, rest) = crate::oracle::effects::bind_target_player(f, rest, b);
        if !rest.trim().is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
        Sel::All(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]))
    };
    let (optional, body) = match body.strip_prefix("you may ") {
        Some(x) => (true, x),
        None => (false, body),
    };
    let body = body.strip_prefix("search ")?;
    let (whose, body) = if let Some(x) = body.strip_prefix("your library for ") {
        (PlayerRef::You, x)
    } else if let Some(x) = body.strip_prefix("that player's library for ") {
        if !matches!(b.it_player, PlayerRef::Target(_)) {
            return None;
        }
        (b.it_player.clone(), x)
    } else {
        return None;
    };
    let (desc, noun) = body.split_once(" with the same name as that ")?;
    if !matches!(noun, "creature" | "permanent") {
        return None;
    }
    let desc = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an "))?;
    let f = super::card_flow_search::card_filter(desc, b)?;
    let search = crate::kw::search_each_same_name::effect(&crate::kw::search_each_same_name::Spec {
        objects: sel,
        whose,
        filter: f,
        optional,
    })?;
    b.it = Sel::Var(ALL_FOUND);
    b.named.push(("those cards".to_string(), Sel::Var(ALL_FOUND)));
    Some(Effect::seq(vec![
        search,
        Effect::Store {
            var: ALL_FOUND,
            sel: Sel::Var(vars::IT),
        },
    ]))
}

inventory::submit! {
    EffectPattern { name: "for each [object], search for a card with the same name as that [object]", priority: 89, parse: for_each_search_same_name }
}

/// "that shares a land type with a land you control", "that doesn't share a land type
/// with a land you control" (Hiveheart Shaman): a land type (CR 205.3i) the card has and
/// one of those objects has.
fn shares_land_type<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let (negate, r) = if let Some(r) = t.strip_prefix("that shares a land type with ") {
        (false, r)
    } else {
        (true, t.strip_prefix("that doesn't share a land type with ")?)
    };
    let (sel, rest) = super::filters_relational::object_in(r)?;
    let Sel::All(objs) = sel else {
        return None;
    };
    let alts: Vec<Filter> = crate::types::land_types()
        .into_iter()
        .map(|lt| {
            let st = crate::types::Subtype::new(lt.as_str());
            Filter::and(vec![
                Filter::Subtype(st.clone()),
                Filter::ValueCmp(
                    Box::new(Value::CountSel(Box::new(Sel::All(Filter::and(vec![
                        objs.clone(),
                        Filter::Subtype(st),
                    ]))))),
                    Cmp::Ge,
                    Box::new(Value::c(1)),
                ),
            ])
        })
        .collect();
    let f = Filter::Or(alts);
    Some((if negate { Filter::not(f) } else { f }, rest))
}

inventory::submit! { super::FilterSuffixPattern { name: "that shares / doesn't share a land type with [objects]", priority: 100, parse: shares_land_type } }

/// The cards several players' searches found (Winds of Abandon).
const FOUND_BY_EACH: Var = vars::USER + 2316;

/// "Those players put those cards onto the battlefield tapped, then shuffle." after "For
/// each creature exiled this way, its controller searches their library for a basic land
/// card." (Winds of Abandon): each searcher shuffles their library (CR 701.23), and every
/// card found is put onto the battlefield under its owner's control (the player who found
/// it, CR 110.2a) at the same time.
fn those_players_put_found(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let tapped = match end(l) {
        "those players put those cards onto the battlefield tapped, then shuffle" => true,
        "those players put those cards onto the battlefield, then shuffle" => false,
        _ => return false,
    };
    // The last instruction: a search for each object, leaving the cards found.
    let last = match &mut *prev {
        Effect::Seq(v) => v.last_mut(),
        e => Some(e),
    };
    let Some(Effect::ForEach { effect, .. }) = last else {
        return false;
    };
    let Effect::SearchCards(spec) = &mut **effect else {
        return false;
    };
    if !spec.dests.is_empty() || spec.shuffle != SearchShuffle::No {
        return false;
    }
    spec.shuffle = SearchShuffle::After;
    let search = std::mem::replace(&mut **effect, Effect::Noop);
    **effect = Effect::seq(vec![
        search,
        Effect::Store {
            var: FOUND_BY_EACH,
            sel: Sel::Union(vec![Sel::Var(FOUND_BY_EACH), Sel::Var(vars::IT)]),
        },
    ]);
    let mut to = Destination::battlefield();
    to.tapped = tapped;
    // Each under its owner's control (the player who found it).
    to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(FOUND_BY_EACH))));
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: FOUND_BY_EACH,
            sel: Sel::None,
        },
        old,
        Effect::Move {
            what: Sel::Var(FOUND_BY_EACH),
            to,
        },
    ]);
    let _ = b;
    true
}

inventory::submit! {
    FollowupPattern { name: "those players put those cards onto the battlefield, then shuffle (after a search for each)", priority: 101, apply: those_players_put_found }
}

/// The cards a search found and revealed, for a condition about them.
const REVEALED_BY_SEARCH: Var = vars::USER + 2317;
/// The card of them chosen at random.
const AT_RANDOM: Var = vars::USER + 2318;

/// "If you reveal three cards with different names, [effect]" after "Search your library
/// for three creature cards and reveal them." (Signal the Clans): whether the cards found
/// (and revealed) have different names; "them" in the effect are those cards.
fn if_you_reveal_with_different_names(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you reveal ") else {
        return false;
    };
    let Some((n, rest)) = crate::oracle::phrases::parse_number(r) else {
        return false;
    };
    let Some(body) = rest
        .trim_start()
        .strip_prefix("cards with different names, ")
    else {
        return false;
    };
    let revealed = match &*prev {
        Effect::SearchCards(s) => s.reveal && s.dests.is_empty(),
        _ => false,
    };
    if !revealed || n.as_const().is_none() {
        return false;
    }
    // "choose one of them at random and put that card into your hand"; "the rest" are
    // the others.
    if body != "choose one of them at random and put that card into your hand" {
        return false;
    }
    let then = Effect::seq(vec![
        Effect::Store {
            var: AT_RANDOM,
            sel: Sel::AtRandom {
                filter: Filter::In(Box::new(Sel::Var(REVEALED_BY_SEARCH))),
                count: Value::c(1),
                store: None,
            },
        },
        Effect::Move {
            what: Sel::Var(AT_RANDOM),
            to: Destination::zone(ZoneKind::Hand),
        },
    ]);
    b.named.push((
        "the rest".to_string(),
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(REVEALED_BY_SEARCH))),
            Filter::not(Filter::In(Box::new(Sel::Var(AT_RANDOM)))),
        ])),
    ));
    let cond = Condition::Compare(
        Value::Count(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(REVEALED_BY_SEARCH))),
            Filter::Together(TargetGroup::DifferentNames),
        ])),
        Cmp::Ge,
        n,
    );
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Store {
            var: REVEALED_BY_SEARCH,
            sel: Sel::Var(vars::IT),
        },
        Effect::Store {
            var: AT_RANDOM,
            sel: Sel::None,
        },
        Effect::If {
            cond,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! {
    FollowupPattern { name: "if you reveal N cards with different names, [effect] (after a search)", priority: 101, apply: if_you_reveal_with_different_names }
}
