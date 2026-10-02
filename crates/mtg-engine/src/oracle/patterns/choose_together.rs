//! Choices of several objects, not targeted, that must have a relationship with each
//! other (`Filter::Together`, see `target_groups::choose_together`): "Search your library
//! for up to three artifact cards with different names" (Saheeli Rai), "Choose any number
//! of artifact tokens and/or creature tokens you control with different names" (Battle
//! for Bretagard). Objects with different names each have a name and no two share one
//! (CR 201.2b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

/// The variable holding the chosen objects.
const CHOSEN: Var = vars::USER + 1791;

const DIFFERENT_NAMES: &str = " with different names";

fn different_names() -> Filter {
    Filter::Together(Box::new(TargetGroup::DifferentNames))
}

/// "search your library for up to N [cards] with different names, put them ...": the
/// search, with the cards found required to have different names.
fn search_with_different_names(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, _) = l.split_once(DIFFERENT_NAMES)?;
    if !head.starts_with("search ") {
        return None;
    }
    let plain = l.replacen(DIFFERENT_NAMES, "", 1);
    let mut e = parse_clause(&plain, b)?;
    let search = match &mut e {
        Effect::Search { filter, .. } => Some(filter),
        Effect::Seq(v) => v.iter_mut().find_map(|x| match x {
            Effect::Search { filter, .. } => Some(filter),
            _ => None,
        }),
        _ => None,
    }?;
    *search = Filter::and(vec![search.clone(), different_names()]);
    Some(e)
}

inventory::submit! { EffectPattern { name: "search for cards with different names", priority: 90, parse: search_with_different_names } }

/// A plural object phrase, all of it: "creature tokens you control", or alternatives
/// joined by "and/or" that share the last one's suffix ("artifact tokens and/or creature
/// tokens you control").
fn plural_objects(r: &str) -> Option<Filter> {
    let whole = |s: &str| -> Option<Filter> {
        let (f, true, tail) = parse_object_phrase(s)? else {
            return None;
        };
        end(tail).is_empty().then_some(f)
    };
    let Some((first, last)) = r.split_once(" and/or ") else {
        return whole(r);
    };
    // The last alternative's words after its noun ("you control") describe both.
    let noun = first.rsplit(' ').next()?;
    let suffix = last.split_once(&format!("{noun} ")).map_or("", |(_, s)| s);
    let first = if suffix.is_empty() {
        first.to_string()
    } else {
        format!("{first} {suffix}")
    };
    Some(Filter::Or(vec![whole(&first)?, whole(last)?]))
}

/// "choose any number of [objects] with different names": they're chosen as the effect
/// happens and are "them" afterward ("For each of them, create a token that's a copy of
/// it").
fn choose_any_number_with_different_names(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose any number of ")?;
    let r = r.strip_suffix(DIFFERENT_NAMES)?;
    let f = plural_objects(r)?;
    if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    b.it = Sel::Var(CHOSEN);
    b.named.push(("them".into(), Sel::Var(CHOSEN)));
    Some(Effect::Store {
        var: CHOSEN,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, different_names()]),
            count: Value::Count(Filter::Any),
            up_to: true,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "choose any number of [objects] with different names", priority: 110, parse: choose_any_number_with_different_names } }

/// The variable bound to each chosen object in turn.
const EACH_CHOSEN: Var = vars::USER + 1792;

/// "For each of them, create a token that's a copy of it." after
/// [`choose_any_number_with_different_names`]: the instruction is followed once for each
/// chosen object, "it" being that object.
fn for_each_of_them(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each of them, ")?;
    if !matches!(b.it, Sel::Var(CHOSEN)) {
        return None;
    }
    let saved = b.it.clone();
    b.it = Sel::Var(EACH_CHOSEN);
    let e = parse_clause(r, b);
    b.it = saved;
    Some(Effect::ForEach {
        sel: Sel::Var(CHOSEN),
        var: EACH_CHOSEN,
        effect: Box::new(e?),
    })
}

inventory::submit! { EffectPattern { name: "for each of them (the chosen objects), [instruction]", priority: 90, parse: for_each_of_them } }
