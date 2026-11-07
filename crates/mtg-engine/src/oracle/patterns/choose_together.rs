//! Choices of several objects, not targeted, that must have a relationship with each
//! other (`Filter::Together`, see `target_groups::choose_together`): "Search your library
//! for up to three artifact cards with different names" (Saheeli Rai), "search your
//! library for any number of creature cards with total mana value 6 or less" (Protean
//! Hulk), "Choose any number of artifact tokens and/or creature tokens you control with
//! different names" (Battle for Bretagard), "choose any number of creatures with
//! different powers" (Sigardian Zealot). Objects with different names each have a name
//! and no two share one (CR 201.2b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, find_group_phrase, parse_object_phrase};

/// The variable holding the chosen objects.
const CHOSEN: Var = vars::USER + 1791;

fn together(grp: TargetGroup) -> Filter {
    Filter::Together(grp)
}

/// "search your library for up to N [cards] with different names, put them ...": the
/// search, with the cards found required to have the relationship.
fn search_with_group(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("search ") || l.contains("target") {
        return None;
    }
    let (i, j, grp) = find_group_phrase(l)?;
    // Part of what's searched for, not of a later instruction.
    if l[..i].contains(", ") || l[..i].contains(" and ") {
        return None;
    }
    let plain = format!("{}{}", &l[..i], &l[j..]);
    let mut e = parse_clause(&plain, b)?;
    let search = match &mut e {
        Effect::Search { filter, .. } => Some(filter),
        Effect::Seq(v) => v.iter_mut().find_map(|x| match x {
            Effect::Search { filter, .. } => Some(filter),
            _ => None,
        }),
        _ => None,
    }?;
    *search = Filter::and(vec![search.clone(), together(grp)]);
    Some(e)
}

inventory::submit! { EffectPattern { name: "search for cards with a relationship (different names, total mana value)", priority: 90, parse: search_with_group } }

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
fn choose_any_number_with_group(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose any number of ")?;
    let (i, j, grp) = find_group_phrase(r)?;
    if j != r.len() {
        return None;
    }
    let f = plural_objects(&r[..i])?;
    if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    b.it = Sel::Var(CHOSEN);
    b.named.push(("them".into(), Sel::Var(CHOSEN)));
    Some(Effect::Store {
        var: CHOSEN,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, together(grp)]),
            count: Value::Count(Filter::Any),
            up_to: true,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "choose any number of [objects] with a relationship (different names, powers)", priority: 110, parse: choose_any_number_with_group } }

/// The variable bound to each chosen object in turn.
const EACH_CHOSEN: Var = vars::USER + 1792;

/// "For each of them, create a token that's a copy of it." after
/// [`choose_any_number_with_group`]: the instruction is followed once for each
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
