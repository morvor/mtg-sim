//! Choosing one object of each of several types (each choice separate: an object with
//! several of the types may be chosen for each of them, Sundering Titan's, Planar
//! Overlay's and Liliana, Dreadhorde General's rulings), and what happens to the chosen
//! objects or the others:
//!
//! - "choose a land of each basic land type, then destroy those lands" (Sundering Titan);
//! - "each player chooses a land they control of each basic land type. Return those lands
//!   to their owners' hands." (Planar Overlay);
//! - "choose a permanent you control of each permanent type and sacrifice the rest"
//!   (Liliana, Dreadhorde General, with any player as the subject);
//! - "For each player, you choose from among the permanents that player controls an
//!   artifact, a creature, an enchantment, and a planeswalker. Then each player
//!   sacrifices all other nonland permanents they control." (Tragic Arrogance).
//!
//! The chosen objects are kept in [`CHOSEN_EACH`]: for several players, all of theirs
//! together (each choice adds to what's there, so it's collected from every player).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::types::CardType;

const BASIC: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

/// The objects chosen.
pub const CHOSEN_EACH: Var = vars::USER + 30528;

/// The permanent types (CR 110.4).
fn permanent_types() -> Vec<Filter> {
    [
        CardType::Artifact,
        CardType::Battle,
        CardType::Creature,
        CardType::Enchantment,
        CardType::Land,
        CardType::Planeswalker,
    ]
    .into_iter()
    .map(Filter::Type)
    .collect()
}

/// One choice of a `base` object for each of `kinds`, by `chooser`, collected in
/// [`CHOSEN_EACH`].
fn choose_each(chooser: PlayerRef, base: &Filter, kinds: Vec<Filter>) -> Effect {
    let mut v = Vec::new();
    for k in kinds {
        v.push(Effect::Store {
            var: CHOSEN_EACH,
            sel: Sel::Union(vec![
                Sel::Var(CHOSEN_EACH),
                Sel::Choose {
                    chooser: chooser.clone(),
                    filter: Filter::and(vec![
                        base.clone(),
                        k,
                        Filter::InZone(ZoneKind::Battlefield),
                    ]),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
            ]),
        });
    }
    Effect::seq(v)
}

/// "a land of each basic land type" / "a land you control of each basic land type".
fn of_each_basic_land_type(s: &str) -> Option<Filter> {
    let noun = s.strip_suffix(" of each basic land type")?;
    let x = noun.strip_prefix("a ").or_else(|| noun.strip_prefix("an "))?;
    let (f, false, tail) = parse_object_phrase(x)? else {
        return None;
    };
    end(tail).is_empty().then_some(f)
}

fn basic_kinds() -> Vec<Filter> {
    BASIC.iter().map(|t| Filter::Subtype((*t).into())).collect()
}

/// "choose a land of each basic land type, then destroy those lands", "choose a land you
/// control of each basic land type".
fn choose_land_of_each_type(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose ")?;
    let (what, then) = match r.split_once(", then ") {
        Some((w, t)) => (w, Some(t)),
        None => (r, None),
    };
    let f = of_each_basic_land_type(what)?;
    let mut v = vec![choose_each(PlayerRef::You, &f, basic_kinds())];
    if let Some(t) = then {
        let t = t.replace("those lands", "them");
        let saved = std::mem::replace(&mut b.it, Sel::Var(CHOSEN_EACH));
        let targets = b.targets.len();
        let e = parse_clause(&t, b);
        b.it = saved;
        let Some(e) = e.filter(|_| b.targets.len() == targets) else {
            b.targets.truncate(targets);
            return None;
        };
        v.push(e);
    }
    b.it = Sel::Var(CHOSEN_EACH);
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "choose a land of each basic land type", priority: 60, parse: choose_land_of_each_type } }

/// "choose a permanent you control of each permanent type and sacrifice the rest".
fn keep_one_of_each_permanent_type(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose a permanent you control of each permanent type")?;
    if !matches!(r, " and sacrifice the rest" | ", then sacrifice the rest") {
        return None;
    }
    Some(Effect::KeepAndSacrificeRest {
        who: PlayerRef::You,
        among: Filter::Permanent,
        keep: permanent_types()
            .into_iter()
            .map(|t| Filter::and(vec![Filter::Permanent, t]))
            .collect(),
        up_to: false,
    })
}

inventory::submit! { EffectPattern { name: "choose a permanent you control of each permanent type and sacrifice the rest", priority: 60, parse: keep_one_of_each_permanent_type } }

/// "Return those lands to their owners' hands." after the lands were chosen ("it").
fn those_lands(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains("those lands") || !chose_each(prev) {
        return false;
    }
    let saved = std::mem::replace(&mut b.it, Sel::Var(CHOSEN_EACH));
    let targets = b.targets.len();
    let e = parse_clause(&l.replace("those lands", "them"), b);
    b.it = saved;
    let Some(e) = e.filter(|_| b.targets.len() == targets) else {
        b.targets.truncate(targets);
        return false;
    };
    let first = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![first, e]);
    true
}

inventory::submit! { FollowupPattern { name: "those lands (chosen one of each type)", priority: 60, apply: those_lands } }

/// Whether `e` ends with choices of one object of each type ([`choose_each`]).
fn chose_each(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(chose_each)
            || (v.len() >= 2
                && v.iter().all(|x| matches!(x, Effect::Store { var, .. } if *var == CHOSEN_EACH))),
        Effect::ForEachPlayer { effect, .. } | Effect::AsPlayer { effect, .. } => {
            chose_each(effect)
        }
        _ => false,
    }
}

/// "For each player, you choose from among the permanents that player controls an
/// artifact, a creature, an enchantment, and a planeswalker."
fn you_choose_for_each_player(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each player, you choose from among the permanents that player controls ")?;
    let list = r.replace(", and ", ", ").replace(" and ", ", ");
    let mut kinds = Vec::new();
    for item in list.split(", ") {
        let x = item
            .strip_prefix("an ")
            .or_else(|| item.strip_prefix("a "))?;
        kinds.push(Filter::Type(CardType::from_word(x)?));
    }
    if kinds.len() < 2 {
        return None;
    }
    let base = Filter::and(vec![
        Filter::Permanent,
        Filter::ControlledBy(PlayerRel::Iterated),
    ]);
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(choose_each(PlayerRef::You, &base, kinds)),
    })
}

inventory::submit! { EffectPattern { name: "for each player, you choose from among the permanents that player controls ...", priority: 60, parse: you_choose_for_each_player } }

/// "Then each player sacrifices all other nonland permanents they control." after the
/// above: the permanents not chosen.
fn sacrifices_all_other(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(noun) = l
        .strip_prefix("each player sacrifices all other ")
        .and_then(|r| r.strip_suffix(" they control"))
    else {
        return false;
    };
    if !chose_each(prev) {
        return false;
    }
    let Some((f, true, tail)) = parse_object_phrase(noun) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    let rest = Effect::SacrificeObjects {
        what: Sel::All(Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Battlefield),
            Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN_EACH)))),
        ])),
    };
    let first = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![first, rest]);
    true
}

inventory::submit! { FollowupPattern { name: "each player sacrifices all other [permanents] (not chosen)", priority: 60, apply: sacrifices_all_other } }
