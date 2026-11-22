//! Objects chosen by several players, or one for each value, and what happens to the
//! chosen objects or the others (CR 101.4, 608.2d):
//!
//! - "you may have it deal 2 damage to another creature you control. If you do, draw a
//!   card." (Wicked Guardian: a creature chosen, or none);
//! - "an opponent chooses a permanent you control other than ~ and exiles it" (Wormfang
//!   Crab: exiled with ~);
//! - "Choose a nonland permanent you don't control, then each other player chooses a
//!   nonland permanent they don't control that hasn't been chosen this way. Destroy all
//!   other nonland permanents." (Fortunate Few);
//! - "Choose a creature you control, then each opponent chooses a creature they control
//!   with equal or lesser power. If you chose a creature this way, exile each creature not
//!   chosen by any player this way." (Sculpted Sunburst);
//! - "For each different power among creatures on the battlefield, choose a creature with
//!   that power. Destroy each creature not chosen this way." (Celestial Judgment);
//! - "Choose an order for artifacts, creatures, and lands. Each player sacrifices one
//!   permanent of their choice of the first type, sacrifices two of their choice of the
//!   second type, then sacrifices three of their choice of the third type." (Rite of
//!   Ruin).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// The objects chosen (collected from every player).
const CHOSEN_OBJECTS: Var = crate::choices::CHOSEN_OBJECTS_VAR;
/// The object the controller chose first ("Choose a creature you control, then ...").
const YOURS: Var = vars::USER + 30530;

fn choose(chooser: PlayerRef, filter: Filter, up_to: bool) -> Sel {
    Sel::Choose {
        chooser,
        filter: Filter::and(vec![filter, Filter::InZone(ZoneKind::Battlefield)]),
        count: Value::c(1),
        up_to,
        store: None,
    }
}

fn add_chosen(sel: Sel) -> Effect {
    Effect::Store {
        var: CHOSEN_OBJECTS,
        sel: Sel::Union(vec![Sel::Var(CHOSEN_OBJECTS), sel]),
    }
}

fn not_chosen() -> Filter {
    Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN_OBJECTS))))
}

/// "you may have it deal 2 damage to another creature you control".
fn have_it_deal_damage_to_another(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "you may" is read around it: then the creature is chosen as it does it.
    let r = ["you may have it deal ", "you may have ~ deal ", "have it deal ", "have ~ deal "]
        .iter()
        .find_map(|p| l.strip_prefix(p))?;
    let (n, r) = parse_number(r)?;
    let r = r.trim_start().strip_prefix("damage to another ")?;
    let noun = r.strip_suffix(" you control")?;
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let filter = Filter::and(vec![f, Filter::Other, Filter::ControlledBy(PlayerRel::You)]);
    let up_to = l.starts_with("you may ");
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN_OBJECTS,
            sel: choose(PlayerRef::You, filter, up_to),
        },
        Effect::If {
            cond: Condition::SelNonEmpty(Sel::Var(CHOSEN_OBJECTS)),
            then: Box::new(Effect::DealDamage {
                source: Sel::This,
                amount: n,
                to: Sel::Var(CHOSEN_OBJECTS),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "you may have it deal N damage to another creature you control", priority: 60, parse: have_it_deal_damage_to_another } }

/// "If you do, [instruction]" after the above: if a creature was chosen (Wicked
/// Guardian's rulings: even if the damage is prevented).
fn if_you_chose_one(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you do, ") else {
        return false;
    };
    let v = match prev {
        Effect::May { effect, .. } => match effect.as_mut() {
            Effect::Seq(v) => v,
            _ => return false,
        },
        Effect::Seq(v) => v,
        _ => return false,
    };
    let is_mine = matches!(v.as_slice(), [Effect::Store { var, sel: Sel::Choose { .. } }, Effect::If { .. }] if *var == CHOSEN_OBJECTS);
    if !is_mine {
        return false;
    }
    let targets = b.targets.len();
    let Some(e) = parse_clause(r, b) else {
        b.targets.truncate(targets);
        return false;
    };
    v.push(Effect::If {
        cond: Condition::SelNonEmpty(Sel::Var(CHOSEN_OBJECTS)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "if you do (chose a creature to damage)", priority: 60, apply: if_you_chose_one } }

/// "an opponent chooses a permanent you control other than ~ and exiles it".
fn opponent_chooses_and_exiles(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("an opponent chooses ")?;
    let (what, verb) = r.rsplit_once(" and ")?;
    if verb != "exiles it" {
        return None;
    }
    let x = what.strip_prefix("a ").or_else(|| what.strip_prefix("an "))?;
    let (x, other) = match x.strip_suffix(" other than ~") {
        Some(x) => (x, true),
        None => (x, false),
    };
    let noun = x.strip_suffix(" you control")?;
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let mut parts = vec![f, Filter::ControlledBy(PlayerRel::You)];
    if other {
        parts.push(Filter::Other);
    }
    Some(Effect::seq(vec![
        // You choose which opponent (CR 601.7a, by analogy), who chooses the permanent.
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        },
        Effect::Store {
            var: CHOSEN_OBJECTS,
            sel: choose(PlayerRef::ChosenOpponent, Filter::and(parts), false),
        },
        Effect::Exile {
            what: Sel::Var(CHOSEN_OBJECTS),
            face_down: false,
            link: true,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "an opponent chooses a permanent you control and exiles it", priority: 60, parse: opponent_chooses_and_exiles } }

/// "Choose a nonland permanent you don't control, then each other player chooses a
/// nonland permanent they don't control that hasn't been chosen this way."
fn fortunate_few_choices(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (mine, theirs) = r.split_once(", then each other player chooses ")?;
    let mine = mine.strip_prefix("a ").or_else(|| mine.strip_prefix("an "))?;
    let noun = mine.strip_suffix(" you don't control")?;
    let theirs = theirs
        .strip_prefix("a ")
        .or_else(|| theirs.strip_prefix("an "))?
        .strip_suffix(" they don't control that hasn't been chosen this way")?;
    if noun != theirs {
        return None;
    }
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let mine = Filter::and(vec![f.clone(), Filter::not(Filter::ControlledBy(PlayerRel::You))]);
    let theirs = Filter::and(vec![
        f,
        Filter::not(Filter::ControlledBy(PlayerRel::Iterated)),
        not_chosen(),
    ]);
    Some(Effect::seq(vec![
        add_chosen(choose(PlayerRef::You, mine, false)),
        // In turn order starting with the active player (Fortunate Few's rulings), each
        // knowing the earlier choices.
        Effect::ForEachPlayer {
            who: PlayerRef::EachOtherPlayer,
            effect: Box::new(add_chosen(choose(PlayerRef::Iterated, theirs, false))),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choose a [permanent], then each other player chooses one that hasn't been chosen", priority: 60, parse: fortunate_few_choices } }

/// "Destroy all other nonland permanents.", "Destroy each creature not chosen this way.",
/// "exile each creature not chosen by any player this way" after objects were chosen.
fn the_others(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !chose_objects(prev) {
        return false;
    }
    // "If you chose a creature this way, ...": only if you did (Sculpted Sunburst).
    let (cond, l) = match l.strip_prefix("if you chose a creature this way, ") {
        Some(r) => (Some(Condition::SelNonEmpty(Sel::Var(YOURS))), r),
        None => (None, l),
    };
    let (verb, noun) = if let Some(r) = l.strip_prefix("destroy all other ") {
        ("destroy", r.to_string())
    } else if let Some(n) = ["destroy each ", "exile each "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
        .and_then(|r| {
            r.strip_suffix(" not chosen this way")
                .or_else(|| r.strip_suffix(" not chosen by any player this way"))
        })
    {
        (if l.starts_with("destroy") { "destroy" } else { "exile" }, n.to_string())
    } else {
        return false;
    };
    let Some((f, _, tail)) = parse_object_phrase(&noun) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    let what = Sel::All(Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Battlefield),
        not_chosen(),
    ]));
    let e = if verb == "destroy" {
        Effect::Destroy {
            what,
            no_regen: false,
        }
    } else {
        Effect::Exile {
            what,
            face_down: false,
            link: false,
        }
    };
    let e = match cond {
        Some(c) => Effect::If {
            cond: c,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
        None => e,
    };
    let _ = b;
    let first = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![first, e]);
    true
}

inventory::submit! { FollowupPattern { name: "the other [objects] (not chosen this way)", priority: 40, apply: the_others } }

/// Whether `e` collected chosen objects in [`CHOSEN_OBJECTS`].
fn chose_objects(e: &Effect) -> bool {
    let d = format!("{e:?}");
    (d.contains(&format!("var: {CHOSEN_OBJECTS}")) && d.contains("Union"))
        || d.contains(crate::choices::FOR_EACH_DIFFERENT_POWER)
}

/// "Choose a creature you control, then each opponent chooses a creature they control
/// with equal or lesser power."
fn equal_or_lesser_power(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (mine, theirs) = r.split_once(", then each opponent chooses ")?;
    let mine = mine.strip_prefix("a ").or_else(|| mine.strip_prefix("an "))?;
    let noun = mine.strip_suffix(" you control")?;
    let theirs = theirs
        .strip_prefix("a ")
        .or_else(|| theirs.strip_prefix("an "))?
        .strip_suffix(" they control with equal or lesser power")?;
    if noun != theirs {
        return None;
    }
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let mine = Filter::and(vec![f.clone(), Filter::ControlledBy(PlayerRel::You)]);
    let theirs = Filter::and(vec![
        f,
        Filter::ControlledBy(PlayerRel::Iterated),
        Filter::Power(
            Cmp::Le,
            Box::new(Value::PowerOf(Box::new(Sel::Var(YOURS)))),
        ),
    ]);
    Some(Effect::seq(vec![
        Effect::Store {
            var: YOURS,
            sel: choose(PlayerRef::You, mine, false),
        },
        add_chosen(Sel::Var(YOURS)),
        // Only if you chose one: their creatures are compared with yours.
        Effect::If {
            cond: Condition::SelNonEmpty(Sel::Var(YOURS)),
            then: Box::new(Effect::ForEachPlayer {
                who: PlayerRef::EachOpponent,
                effect: Box::new(add_chosen(choose(PlayerRef::Iterated, theirs, false))),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "each opponent chooses a creature with equal or lesser power", priority: 60, parse: equal_or_lesser_power } }

/// "For each different power among creatures on the battlefield, choose a creature with
/// that power." (`choices.rs`, [`crate::choices::FOR_EACH_DIFFERENT_POWER`]).
fn for_each_different_power(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l)
        == "for each different power among creatures on the battlefield, choose a creature with that power")
        .then(|| Effect::Custom(SmolStr::new(crate::choices::FOR_EACH_DIFFERENT_POWER)))
}

inventory::submit! { EffectPattern { name: "for each different power among creatures, choose a creature with that power", priority: 60, parse: for_each_different_power } }

/// "Choose an order for artifacts, creatures, and lands." (Rite of Ruin)
fn choose_an_order(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose an order for ")?;
    let list = r.replace(", and ", ", ").replace(" and ", ", ");
    let mut words = Vec::new();
    for w in list.split(", ") {
        let t = w.strip_suffix('s')?;
        crate::types::CardType::from_word(t)?;
        words.push(t.to_string());
    }
    if words.len() < 2 {
        return None;
    }
    let n = words.len() as u8;
    Some(Effect::Choose {
        who: PlayerRef::You,
        kind: ChoiceKind::Distinct(n, words),
    })
}

inventory::submit! { EffectPattern { name: "choose an order for [card types]", priority: 60, parse: choose_an_order } }

/// "Each player sacrifices one permanent of their choice of the first type, sacrifices two
/// of their choice of the second type, then sacrifices three of their choice of the third
/// type."
fn sacrifices_by_chosen_order(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player ")?;
    let parts: Vec<&str> = r
        .split(", then ")
        .flat_map(|p| p.split(", "))
        .collect();
    let mut v = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        let r = p.strip_prefix("sacrifices ")?;
        let (n, r) = parse_number(r)?;
        let r = r.trim_start();
        let r = r
            .strip_prefix("permanent ")
            .or_else(|| r.strip_prefix("permanents "))
            .unwrap_or(r);
        let ord = ["first", "second", "third", "fourth"].get(i)?;
        if r != format!("of their choice of the {ord} type") {
            return None;
        }
        v.push(Effect::Sacrifice {
            who: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::Permanent,
                Filter::Custom(crate::choices::nth_chosen_card_type(i).into()),
            ]),
            count: n,
        });
    }
    if v.len() < 2 {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::seq(v)),
        }),
    })
}

inventory::submit! { EffectPattern { name: "sacrifices N of their choice of the first/second/third type", priority: 60, parse: sacrifices_by_chosen_order } }
