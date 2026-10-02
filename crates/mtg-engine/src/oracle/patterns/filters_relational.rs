//! Relational qualifiers in object phrases (see `relational.rs` for the engine side):
//!
//! * comparisons with a referent or a value: "with lesser mana value", "with greater
//!   power", "with power less than or equal to that creature's power", "with mana value
//!   less than or equal to the number of lands you control", "with the same mana value as
//!   that permanent", "with toughness greater than its power" (the object's own power),
//!   "with total power and toughness 5 or less";
//! * extremes: "with the greatest power among creatures you control", "with the lowest
//!   mana value" (among the objects the phrase describes), ties included;
//! * sharing: "that shares a color with it", "that shares a creature type with enchanted
//!   creature", "with the same name as that creature";
//! * exclusions: "other than enchanted creature", "other than a basic land card";
//! * group requirements on objects chosen together: "with different names", "with total
//!   mana value 6 or less" ([`Filter::Together`], lifted into the target slot for targets).
//!
//! The qualifiers are parsed without the effect parser's [`Builder`]: "it", "that
//! creature" and the like become the [`REFERENT`] placeholder, which the effect parser
//! replaces with what the pronoun means where the phrase is used ([`resolve_referent`]).
//! An ability that still contains the placeholder, or a group requirement where nothing
//! checks it, isn't understood ([`unresolved`]).

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// What "it" / "that creature" in a qualifier refers to, until the effect parser resolves
/// it (never stored by any instruction).
pub const REFERENT: Var = 0x7ffd;

pub fn referent() -> Sel {
    Sel::Var(REFERENT)
}

/// "They" in "among creatures they control": the player the instruction is performed
/// for, until the effect parser resolves it ([`resolve_they`]).
pub const THEY: PlayerRel = PlayerRel::Target(0xfe);

fn they_needle() -> String {
    serde_json::to_string(&Filter::ControlledBy(THEY)).unwrap_or_default()
}

/// Replaces "they" in the filters of instructions each player performs for themselves
/// ("each opponent sacrifices a creature with the greatest power among creatures they
/// control") with that player ([`PlayerRel::Iterated`]).
pub fn resolve_they(e: Effect) -> Effect {
    let needle = they_needle();
    let Ok(mut v) = serde_json::to_value(&e) else {
        return e;
    };
    if !v.to_string().contains(&needle) {
        return e;
    }
    let iterated = serde_json::to_string(&Filter::ControlledBy(PlayerRel::Iterated))
        .unwrap_or_default();
    fn walk(v: &mut serde_json::Value, needle: &str, with: &str) {
        match v {
            serde_json::Value::Object(m) => {
                for key in ["Sacrifice", "ForEachPlayer"] {
                    if let Some(inner) = m.get_mut(key) {
                        let s = inner.to_string().replace(needle, with);
                        if let Ok(x) = serde_json::from_str(&s) {
                            *inner = x;
                        }
                    }
                }
                for x in m.values_mut() {
                    walk(x, needle, with);
                }
            }
            serde_json::Value::Array(xs) => xs.iter_mut().for_each(|x| walk(x, needle, with)),
            _ => {}
        }
    }
    walk(&mut v, &needle, &iterated);
    serde_json::from_value(v).unwrap_or(e)
}

fn referent_needle() -> String {
    format!("{{\"Var\":{REFERENT}}}")
}

/// Whether the filter mentions the placeholder.
pub fn mentions_referent<T: serde::Serialize>(x: &T) -> bool {
    serde_json::to_string(x).is_ok_and(|s| s.contains(&referent_needle()))
}

/// `x` with the placeholder replaced by `it`. None if `it` has no antecedent (or the
/// placeholder stood where no object can).
pub fn substitute<T: serde::Serialize + serde::de::DeserializeOwned>(x: &T, it: &Sel) -> Option<T> {
    use crate::oracle::patterns::oracle_hardening_referents::is_no_referent;
    let s = serde_json::to_string(x).ok()?;
    let needle = referent_needle();
    if !s.contains(&needle) {
        return serde_json::from_str(&s).ok();
    }
    if is_no_referent(it) {
        return None;
    }
    let with = serde_json::to_string(it).ok()?;
    serde_json::from_str(&s.replace(&needle, &with)).ok()
}

/// Replaces the placeholder in a filter the effect parser is about to use with what "it"
/// means there (`b.it`). Returns None if "it" has no antecedent. "Other" in a phrase
/// related to "it" means other than it ("each other creature that shares a color with
/// it", "other creatures you control that share a creature type with it").
pub fn resolve_referent(f: Filter, b: &Builder) -> Option<Filter> {
    if !mentions_referent(&f) {
        return Some(f);
    }
    let it = super::pronoun_groups::singular_it(b);
    let f = if matches!(it, Sel::This) {
        f
    } else {
        other_than_it(f, &it)
    };
    substitute(&f, &it)
}

/// A filter whose top-level "other" (not the source) means other than `it`.
fn other_than_it(f: Filter, it: &Sel) -> Filter {
    let not_it = || Filter::not(Filter::In(Box::new(it.clone())));
    match f {
        Filter::Other => not_it(),
        Filter::And(v) => Filter::And(
            v.into_iter()
                .map(|x| match x {
                    Filter::Other => not_it(),
                    x => x,
                })
                .collect(),
        ),
        f => f,
    }
}

/// Whether a compiled ability still contains the placeholder, or a group requirement
/// ([`Filter::Together`]) somewhere other than a search's or a choice's filter, where
/// nothing would check it.
pub fn unresolved(a: &AbilityDef) -> bool {
    let Ok(mut v) = serde_json::to_value(&a.kind) else {
        return false;
    };
    let s = v.to_string();
    if s.contains(&referent_needle()) || s.contains(&they_needle()) {
        return true;
    }
    if !s.contains("\"Together\"") {
        return false;
    }
    fn strip_checked(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                // Counting objects "with different names" counts the most of them that
                // have different names (`relational::count`).
                if let Some(f) = m.get("Count") {
                    let ok = serde_json::from_value::<Filter>(f.clone())
                        .is_ok_and(|f| crate::relational::countable(&f));
                    if ok {
                        m.remove("Count");
                    }
                }
                for key in ["Search", "Choose"] {
                    if let Some(serde_json::Value::Object(inner)) = m.get_mut(key) {
                        if let Some(f) = inner.get("filter") {
                            let ok = serde_json::from_value::<Filter>(f.clone())
                                .is_ok_and(|f| !crate::relational::has_nested_group(&f));
                            if ok {
                                inner.remove("filter");
                            }
                        }
                    }
                }
                for x in m.values_mut() {
                    strip_checked(x);
                }
            }
            serde_json::Value::Array(xs) => xs.iter_mut().for_each(strip_checked),
            _ => {}
        }
    }
    strip_checked(&mut v);
    v.to_string().contains("\"Together\"")
}

fn dummy_ctx() -> CompileContext<'static> {
    use std::sync::OnceLock;
    static TL: OnceLock<TypeLine> = OnceLock::new();
    CompileContext {
        card_name: "",
        full_name: "",
        type_line: TL.get_or_init(TypeLine::default),
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    }
}

/// Runs a value or object parser of the effect parser with "it" as the placeholder and
/// no "that player"; fails if it introduced targets (a qualifier can't).
fn with_builder<T>(f: impl FnOnce(&mut Builder) -> Option<T>) -> Option<T> {
    let ctx = dummy_ctx();
    let mut b = Builder::new(&ctx);
    b.it = referent();
    b.it_player = crate::oracle::patterns::oracle_hardening_referents::no_player_referent();
    b.in_trigger = true;
    b.sentences = 1;
    let r = f(&mut b)?;
    b.targets.is_empty().then_some(r)
}

/// `rest` (a suffix of `t` possibly trimmed at its start) as a slice of `t`.
fn tail_of<'a>(t: &'a str, rest: &str) -> Option<&'a str> {
    let rest = rest.trim_start();
    let at = t.len().checked_sub(rest.len())?;
    (t.get(at..)? == rest).then(|| &t[at..])
}

/// A value phrase ("the number of lands you control", "that creature's power", "X").
pub fn value_in(t: &str) -> Option<(Value, &str)> {
    // "that damage": the damage dealt by the triggering event ("Whenever ~ deals combat
    // damage to a player, you may put an artifact card with mana value less than or equal
    // to that damage from your hand onto the battlefield").
    if let Some(r) = t.strip_prefix("that damage") {
        if word_end(r) {
            return Some((Value::EventAmount, r));
        }
    }
    let (v, rest) = with_builder(|b| super::r107_numbers::value_phrase(t, b))?;
    // A value naming an object ("the number of counters on it") ends at the object.
    let rest = tail_of(t, &rest)?;
    Some((v, rest))
}

fn exiled_with_source() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

/// An object a qualifier relates to: "it", "that creature", "~", "enchanted creature",
/// "the sacrificed creature", "the exiled card", "a legendary creature you control" (any
/// of them), "creatures you control".
pub fn object_in(t: &str) -> Option<(Sel, &str)> {
    let t = t.trim_start();
    let word_end = |r: &str| r.is_empty() || r.starts_with([' ', ',', '.', '\'']);
    for (p, sel) in [
        ("the sacrificed creature", Sel::Var(vars::SACRIFICED)),
        ("the sacrificed permanent", Sel::Var(vars::SACRIFICED)),
        ("the sacrificed artifact", Sel::Var(vars::SACRIFICED)),
        // Cards a linked ability of the source exiled (CR 607.2a).
        ("the exiled card", Sel::All(exiled_with_source())),
        ("a card exiled with ~", Sel::All(exiled_with_source())),
        ("cards exiled with ~", Sel::All(exiled_with_source())),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if word_end(r) {
                return Some((sel, r));
            }
        }
    }
    if let Some(r) = t.strip_prefix("a ").or_else(|| t.strip_prefix("an ")) {
        let (f, plural, rest) = parse_object_phrase(r)?;
        if plural {
            return None;
        }
        return Some((Sel::All(f), rest));
    }
    let (sel, rest) = with_builder(|b| crate::oracle::effects::object_ref(t, b))?;
    // Only fixed objects or groups: no choices made while the qualifier is checked.
    if matches!(sel, Sel::Choose { .. }) {
        return None;
    }
    Some((sel, tail_of(t, &rest)?))
}

// ---------------------------------------------------------------------------
// Values of objects
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Power,
    Toughness,
    ManaValue,
}

/// "power", "toughness", "mana value" at the start of `t`.
pub fn stat_word(t: &str) -> Option<(Stat, &str)> {
    for (p, s) in [
        ("power", Stat::Power),
        ("toughness", Stat::Toughness),
        ("mana value", Stat::ManaValue),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((s, r));
            }
        }
    }
    None
}

/// The stat of the selected objects.
pub fn stat_of(stat: Stat, sel: Sel) -> Value {
    let sel = Box::new(sel);
    match stat {
        Stat::Power => Value::PowerOf(sel),
        Stat::Toughness => Value::ToughnessOf(sel),
        Stat::ManaValue => Value::ManaValueOf(sel),
    }
}

/// The stat of the object being tested ([`vars::TESTED`]).
pub fn tested(stat: Stat) -> Value {
    stat_of(stat, Sel::Var(vars::TESTED))
}

/// An object whose stat compares with the value.
pub fn stat_filter(stat: Stat, cmp: Cmp, v: Value) -> Filter {
    let v = Box::new(v);
    match stat {
        Stat::Power => Filter::Power(cmp, v),
        Stat::Toughness => Filter::Toughness(cmp, v),
        Stat::ManaValue => Filter::ManaValue(cmp, v),
    }
}

/// "less than or equal to ", "greater than ", "equal to ", ...
fn cmp_phrase(t: &str) -> Option<(Cmp, &str)> {
    for (p, c) in [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
        ("equal to ", Cmp::Eq),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            return Some((c, r));
        }
    }
    None
}

/// "N or less", "N or greater", "N or more", "less than or equal to [value]": a bound.
fn bound(t: &str) -> Option<(Cmp, Value, &str)> {
    if let Some((cmp, r)) = cmp_phrase(t) {
        let (v, rest) = value_in(r)?;
        return Some((cmp, v, rest));
    }
    let (n, r) = parse_number(t)?;
    let r = r.trim_start();
    if let Some(rest) = r.strip_prefix("or less") {
        return Some((Cmp::Le, n, rest));
    }
    if let Some(rest) = r.strip_prefix("or greater").or_else(|| r.strip_prefix("or more")) {
        return Some((Cmp::Ge, n, rest));
    }
    None
}

/// "its power", "their toughness": the tested object's own stat (in "with toughness
/// greater than its power" the pronoun is the object described).
fn own_stat(t: &str) -> Option<(Stat, &str)> {
    let r = t.strip_prefix("its ").or_else(|| t.strip_prefix("their "))?;
    stat_word(r)
}

fn word_end(r: &str) -> bool {
    r.is_empty() || r.starts_with([' ', ',', '.'])
}

/// "with lesser mana value", "with greater power than ~", "with the same mana value as that
/// permanent", "with power less than or equal to the number of Islands you control",
/// "with toughness greater than its power", "with total power and toughness 5 or less",
/// "with total mana value 6 or less" (a group requirement).
fn with_comparison<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("with ")?;
    // "with lesser power [than ...]"
    for (p, cmp) in [("lesser ", Cmp::Lt), ("greater ", Cmp::Gt)] {
        if let Some(x) = r.strip_prefix(p) {
            let (stat, rest) = stat_word(x)?;
            let (obj, rest) = match rest.trim_start().strip_prefix("than ") {
                Some(o) => object_in(o)?,
                None => (referent(), rest),
            };
            return Some((stat_filter(stat, cmp, stat_of(stat, obj)), rest));
        }
    }
    // "with the same total power and toughness [as ...]" (Wild Pair).
    if let Some(x) = r.strip_prefix("the same total power and toughness") {
        let (obj, rest) = match x.trim_start().strip_prefix("as ") {
            Some(o) => object_in(o)?,
            None => (referent(), x),
        };
        let total = |s: Sel| Value::Sum(vec![stat_of(Stat::Power, s.clone()), stat_of(Stat::Toughness, s)]);
        let f = Filter::ValueCmp(
            Box::new(total(Sel::Var(vars::TESTED))),
            Cmp::Eq,
            Box::new(total(obj)),
        );
        return Some((f, rest));
    }
    // "with the same mana value [as that permanent]"
    if let Some(x) = r.strip_prefix("the same ") {
        let (stat, rest) = stat_word(x)?;
        let (obj, rest) = match rest.trim_start().strip_prefix("as ") {
            Some(o) => object_in(o)?,
            None => (referent(), rest),
        };
        return Some((stat_filter(stat, Cmp::Eq, stat_of(stat, obj)), rest));
    }
    // "with total power and toughness 5 or less": each object's own total.
    if let Some(x) = r.strip_prefix("total power and toughness ") {
        let (cmp, v, rest) = bound(x)?;
        let sum = Value::Sum(vec![tested(Stat::Power), tested(Stat::Toughness)]);
        return Some((Filter::ValueCmp(Box::new(sum), cmp, Box::new(v)), rest));
    }
    // "with total mana value 6 or less": the objects chosen together.
    if let Some(x) = r.strip_prefix("total ") {
        let (stat, x) = stat_word(x)?;
        let (cmp, v, rest) = bound(x.trim_start())?;
        if cmp != Cmp::Le {
            return None;
        }
        let stat = match stat {
            Stat::Power => TotalStat::Power,
            Stat::Toughness => TotalStat::Toughness,
            Stat::ManaValue => TotalStat::ManaValue,
        };
        return Some((
            Filter::Together(TargetGroup::TotalAtMost(stat, Box::new(v))),
            rest,
        ));
    }
    // "with power less than or equal to [value]", "with toughness greater than its power"
    let (stat, x) = stat_word(r)?;
    let (cmp, x) = cmp_phrase(x.trim_start())?;
    if let Some((other, rest)) = own_stat(x) {
        return Some((
            Filter::ValueCmp(Box::new(tested(stat)), cmp, Box::new(tested(other))),
            rest,
        ));
    }
    // "with mana value less than or equal to the number of cards in its controller's
    // graveyard" (Drown in the Loch): "its" is the object described.
    if x.contains("its controller") || x.contains("its owner") {
        let its = |p: &str| {
            let sel = Box::new(Sel::Var(vars::TESTED));
            if p == "controller" {
                PlayerRef::ControllerOf(sel)
            } else {
                PlayerRef::OwnerOf(sel)
            }
        };
        let mut found = None;
        for p in ["controller", "owner"] {
            for (zone, hand) in [("graveyard", false), ("hand", true)] {
                let phrase = format!("the number of cards in its {p}'s {zone}");
                if let Some(r) = x.strip_prefix(phrase.as_str()) {
                    let v = if hand {
                        Value::HandSize(its(p))
                    } else {
                        Value::GraveyardSize(its(p))
                    };
                    found = Some((v, r));
                }
            }
        }
        let (v, rest) = found?;
        return Some((
            Filter::ValueCmp(Box::new(tested(stat)), cmp, Box::new(v)),
            rest,
        ));
    }
    let (v, rest) = value_in(x)?;
    Some((stat_filter(stat, cmp, v), rest))
}

/// "whose power and toughness aren't equal" (Gilt-Leaf Winnower).
fn whose_comparison<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for (p, cmp) in [
        ("whose power and toughness aren't equal", Cmp::Ne),
        ("whose power and toughness are equal", Cmp::Eq),
    ] {
        if let Some(rest) = t.strip_prefix(p) {
            let f = Filter::ValueCmp(
                Box::new(tested(Stat::Power)),
                cmp,
                Box::new(tested(Stat::Toughness)),
            );
            return Some((f, rest));
        }
    }
    None
}

/// "with the greatest power among creatures you control", "with the lowest mana value"
/// (among the objects the phrase describes): an object whose stat is the greatest (or
/// least) of the group, tied or not.
fn with_extreme<'a>(t: &'a str, so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("with the ")?;
    let (greatest, r) = if let Some(x) = r
        .strip_prefix("greatest ")
        .or_else(|| r.strip_prefix("highest "))
    {
        (true, x)
    } else if let Some(x) = r
        .strip_prefix("least ")
        .or_else(|| r.strip_prefix("lowest "))
    {
        (false, x)
    } else {
        return None;
    };
    let (stat, rest) = stat_word(r)?;
    let (among, explicit, rest) = match rest.trim_start().strip_prefix("among ") {
        Some(x) => {
            let (f, plural, rest) = parse_object_phrase(x)?;
            if !plural {
                return None;
            }
            // "among creatures they control": the player each instruction is performed
            // for (the sacrificing player).
            let t2 = rest.trim_start();
            let (f, rest) = match t2
                .strip_prefix("they control")
                .or_else(|| t2.strip_prefix("that player controls"))
            {
                Some(r2) if word_end(r2) => (Filter::and(vec![f, Filter::ControlledBy(THEY)]), r2),
                _ => (f, rest),
            };
            (f, true, rest)
        }
        None => (so_far.clone(), false, rest),
    };
    if matches!(among, Filter::Any) || crate::relational::has_nested_group(&among) {
        return None;
    }
    let ext = Value::Extreme(
        Box::new(tested(stat)),
        Box::new(Sel::All(among.clone())),
        greatest,
    );
    let cmp = if greatest { Cmp::Ge } else { Cmp::Le };
    let f = Filter::ValueCmp(Box::new(tested(stat)), cmp, Box::new(ext));
    let f = if explicit {
        Filter::and(vec![among, f])
    } else {
        f
    };
    Some((f, rest))
}

// ---------------------------------------------------------------------------
// Sharing, names, exclusions
// ---------------------------------------------------------------------------

/// "that shares a color with it", "that share a creature type with enchanted creature",
/// "that doesn't share a card type with ...".
fn shares_with<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let (negate, r) = if let Some(r) = t
        .strip_prefix("that shares a ")
        .or_else(|| t.strip_prefix("that share a "))
    {
        (false, r)
    } else if let Some(r) = t
        .strip_prefix("that doesn't share a ")
        .or_else(|| t.strip_prefix("that don't share a "))
    {
        (true, r)
    } else {
        return None;
    };
    let (kind, r) = if let Some(x) = r.strip_prefix("creature type with ") {
        (0, x)
    } else if let Some(x) = r.strip_prefix("card type with ") {
        (1, x)
    } else if let Some(x) = r.strip_prefix("color with ") {
        (2, x)
    } else {
        return None;
    };
    let (sel, rest) = object_in(r)?;
    let sel = Box::new(sel);
    let f = match kind {
        0 => Filter::SharesCreatureType(sel),
        1 => Filter::SharesCardType(sel),
        _ => Filter::SharesColor(sel),
    };
    Some((if negate { Filter::not(f) } else { f }, rest))
}

/// "with the same name as that creature", "with different names" (a group requirement),
/// "with a different name than each Aura you control".
fn with_names<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    if let Some(rest) = t.strip_prefix("with different names") {
        if word_end(rest) {
            return Some((Filter::Together(TargetGroup::DifferentNames), rest));
        }
        return None;
    }
    if let Some(r) = t.strip_prefix("with the same name as ") {
        let (sel, rest) = object_in(r)?;
        return Some((Filter::SameNameAs(Box::new(sel)), rest));
    }
    if let Some(r) = t.strip_prefix("with a different name than ") {
        let (sel, rest) = object_in(r)?;
        return Some((Filter::DifferentNameFrom(Box::new(sel)), rest));
    }
    None
}

/// "other than enchanted creature", "other than that creature", "other than a basic land
/// card".
fn other_than<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("other than ")?;
    if let Some(x) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
        let (f, plural, rest) = parse_object_phrase(x)?;
        if plural {
            return None;
        }
        return Some((Filter::not(f), rest));
    }
    let (sel, rest) = object_in(r)?;
    if matches!(sel, Sel::All(_)) {
        return None;
    }
    let mut f = vec![Filter::not(Filter::In(Box::new(sel)))];
    let mut rest = rest;
    // "other than that creature or ~": neither of them.
    while let Some(r2) = rest
        .trim_start()
        .strip_prefix("or ")
        .or_else(|| rest.trim_start().strip_prefix("and "))
    {
        let Some((sel2, r3)) = object_in(r2).filter(|(s, _)| !matches!(s, Sel::All(_))) else {
            break;
        };
        f.push(Filter::not(Filter::In(Box::new(sel2))));
        rest = r3;
    }
    Some((Filter::and(f), rest))
}

inventory::submit! { FilterSuffixPattern { name: "relational: with [stat] compared", priority: 100, parse: with_comparison } }
inventory::submit! { FilterSuffixPattern { name: "relational: whose power and toughness", priority: 100, parse: whose_comparison } }
inventory::submit! { FilterSuffixPattern { name: "relational: with the greatest [stat]", priority: 100, parse: with_extreme } }
inventory::submit! { FilterSuffixPattern { name: "relational: that shares a [quality] with", priority: 100, parse: shares_with } }
inventory::submit! { FilterSuffixPattern { name: "relational: names", priority: 100, parse: with_names } }
inventory::submit! { FilterSuffixPattern { name: "relational: other than", priority: 100, parse: other_than } }

/// After a sentence was parsed: a qualifier's "it" the effect parser didn't resolve where
/// it was used (a searched-for or chosen card "with lesser mana value") means what "it"
/// meant as the sentence began, provided the sentence didn't give "it" a new antecedent
/// or add targets (then which object is meant isn't clear, and the placeholder stays, so
/// the ability isn't understood). `before` is "it" and the number of targets before.
pub fn resolve_in_sentence(e: Effect, b: &Builder, before: (Sel, usize)) -> Option<Effect> {
    let e = resolve_they(e);
    if !mentions_referent(&e) {
        return Some(e);
    }
    let (it, n) = before;
    let same_it = format!("{it:?}") == format!("{:?}", b.it);
    if !same_it || b.targets.len() != n {
        return Some(e);
    }
    let it = super::pronoun_groups::singular_it(b);
    Some(substitute(&e, &it).unwrap_or(e))
}

// ---------------------------------------------------------------------------
// "[target] and each other [objects] [related to it]"
// ---------------------------------------------------------------------------

/// "~ deals 1 damage to target creature and each other creature with the same name as
/// that creature", "Return target nonland permanent and each other nonland permanent with
/// the same mana value as that permanent to their owners' hands", "Exile target creature
/// and all other creatures its controller controls with the same name as that creature":
/// the instruction is parsed as if it named only the target, then what it affects is
/// widened to the target and the other objects of the kind that are related to it as the
/// instruction is carried out (CR 608.2c). The others are in the target's zone. If the
/// target is illegal as the spell or ability resolves, nothing is affected (CR 608.2b).
fn target_and_others(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, after, sep) = [" and each other ", " and all other "]
        .iter()
        .find_map(|sep| l.split_once(sep).map(|(h, a)| (h, a, *sep)))?;
    let _ = sep;
    // The target is the last thing the head names: "deals 1 damage to target creature".
    let ti = head.rfind("target ")?;
    let target_text = &head[ti..];
    let (spec, t_rest) = parse_target(target_text)?;
    if !end(t_rest).trim().is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    // The group: an object phrase whose qualifiers mention the target ("that creature",
    // "it"); "its controller controls" is said of the target's controller.
    let (f, _plural, rest) = parse_object_phrase(after)?;
    let mut f = f;
    let mut rest = rest;
    let slot = b.targets.len() as u8;
    let t = rest.trim_start();
    if let Some(r) = t.strip_prefix("its controller controls") {
        let probe = format!("cards{r}");
        let (more, _, tail) = parse_object_phrase(&probe)?;
        let more = match more {
            Filter::Card => Filter::Any,
            Filter::And(v) => Filter::and(v.into_iter().filter(|x| !matches!(x, Filter::Card)).collect()),
            other => other,
        };
        f = Filter::and(vec![
            f,
            Filter::ControlledBy(PlayerRel::TargetOrController(slot)),
            more,
        ]);
        rest = &r[r.len() - tail.len()..];
    }
    if !mentions_referent(&f) {
        return None;
    }
    let target = Sel::Target(slot);
    let f = substitute(&f, &target)?;
    // "other": other than the target (not the source).
    let others = |f: Filter| -> Filter {
        let parts = match f {
            Filter::And(v) => v,
            x => vec![x],
        };
        let mut v: Vec<Filter> = parts
            .into_iter()
            .filter(|p| !matches!(p, Filter::Other))
            .collect();
        v.push(Filter::not(Filter::In(Box::new(Sel::Target(slot)))));
        Filter::and(v)
    };
    let mut group_filter = others(f);
    if crate::relational::has_nested_group(&group_filter)
        || !crate::relational::groups_of(&group_filter).is_empty()
    {
        return None;
    }
    // The rest of the instruction, said of the one target.
    let tail = rest.trim_start();
    let tail = match tail {
        "to their owners' hands" => "to its owner's hand".to_string(),
        t => match t.strip_prefix("get ") {
            Some(r) => format!("gets {r}"),
            None => match t.strip_prefix("gain ") {
                Some(r) => format!("gains {r}"),
                None => t.to_string(),
            },
        },
    };
    // "target creature card and all other cards with the same name as that card from
    // your graveyard": the zone is the target's too.
    let consumed = &after[..after.len() - rest.len()];
    let head = match ["from your graveyard", "from a graveyard", "in your graveyard"]
        .iter()
        .find(|z| consumed.contains(*z))
    {
        Some(z) => format!("{head} {z}"),
        None => head.to_string(),
    };
    let rewritten = if tail.is_empty() {
        head
    } else {
        format!("{head} {tail}")
    };
    let saved_it = b.it.clone();
    let effect = crate::oracle::effects::parse_clause(&rewritten, b);
    let replaced = effect
        .filter(|_| b.targets.len() == slot as usize + 1)
        .and_then(|e| {
            // The others are in the target's zone ("target creature card and all other
            // cards with the same name as that card from your graveyard").
            if let TargetKind::Object(tf) = &b.targets[slot as usize].what {
                if let Some(zone) = tf.zone().filter(|z| *z != ZoneKind::Battlefield) {
                    if group_filter.zone().is_none() {
                        // And whose zone it is ("from your graveyard").
                        let parts = match tf {
                            Filter::And(v) => v.as_slice(),
                            other => std::slice::from_ref(other),
                        };
                        let mut v = vec![group_filter.clone(), Filter::InZone(zone)];
                        v.extend(
                            parts
                                .iter()
                                .filter(|p| matches!(p, Filter::OwnedBy(_)))
                                .cloned(),
                        );
                        group_filter = Filter::and(v);
                    }
                }
            }
            let group = Sel::Union(vec![target.clone(), Sel::All(group_filter.clone())]);
            // The target must be named exactly once, as what the instruction affects.
            let json = serde_json::to_string(&e).ok()?;
            let needle = serde_json::to_string(&target).ok()?;
            if json.matches(&needle).count() != 1 {
                return None;
            }
            let group = serde_json::to_string(&group).ok()?;
            serde_json::from_str::<Effect>(&json.replace(&needle, &group)).ok()
        });
    if replaced.is_none() {
        b.targets.truncate(slot as usize);
        b.it = saved_it;
    }
    replaced
}

inventory::submit! { EffectPattern { name: "relational: [target] and each other [objects] related to it", priority: 90, parse: target_and_others } }

// ---------------------------------------------------------------------------
// Edicts with relational qualifiers
// ---------------------------------------------------------------------------

/// "Each opponent sacrifices a creature they control with the greatest power", "Target
/// opponent exiles a creature or planeswalker they control with the greatest mana value
/// among creatures and planeswalkers they control", "Each opponent returns a nonland
/// permanent they control with the greatest mana value among permanents they control to
/// its owner's hand": each player chooses one of their own objects that has the quality
/// as the instruction is performed (CR 608.2d), among tied ones too.
fn player_edict(l: &str, b: &mut Builder) -> Option<Effect> {
    use crate::oracle::effects::player_ref;
    let l = end(l);
    let before = b.targets.len();
    let saved = (b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let (who, rest) = player_ref(l, b)?;
        let rest = rest.trim_start();
        let (verb, r) = [
            ("sacrifices ", 0),
            ("sacrifice ", 0),
            ("exiles ", 1),
            ("exile ", 1),
            ("returns ", 2),
            ("return ", 2),
        ]
        .iter()
        .find_map(|(p, v)| rest.strip_prefix(p).map(|r| (*v, r)))?;
        let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(r)?;
        if plural {
            return None;
        }
        // "a creature they control with the greatest power": the phrase continues after
        // "they control".
        let t = tail.trim_start();
        let (f, tail) = match t
            .strip_prefix("they control")
            .or_else(|| t.strip_prefix("that player controls"))
        {
            Some(r2) => {
                let so_far = Filter::and(vec![f.clone(), Filter::ControlledBy(THEY)]);
                // "with the greatest power": among the objects described so far, which
                // are the player's.
                let (ext, r2) = match with_extreme(r2.trim_start(), &so_far) {
                    Some((m, r3)) => (m, r3),
                    None => (Filter::Any, r2),
                };
                let probe = format!("cards{r2}");
                let (more, _, tail2) = parse_object_phrase(&probe)?;
                let more = match more {
                    Filter::Card => Filter::Any,
                    Filter::And(v) => Filter::and(
                        v.into_iter()
                            .filter(|x| !matches!(x, Filter::Card))
                            .collect(),
                    ),
                    other => other,
                };
                (
                    Filter::and(vec![so_far, ext, more]),
                    &r2[r2.len() - tail2.len()..],
                )
            }
            None => (f, tail),
        };
        let tail = tail.trim();
        let json = serde_json::to_string(&f).ok()?;
        // Only qualifiers this file adds (other edicts are core patterns).
        if !json.contains("\"Extreme\"") && !json.contains(&they_needle()) {
            return None;
        }
        let many = matches!(
            who,
            PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
        );
        let player_rel = |who: &PlayerRef| -> Option<PlayerRel> {
            Some(match who {
                PlayerRef::Target(n) => PlayerRel::Target(*n),
                PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
                PlayerRef::You => PlayerRel::You,
                PlayerRef::DefendingPlayer => PlayerRel::Defending,
                _ => return None,
            })
        };
        let rel = if many || verb == 0 {
            PlayerRel::Iterated
        } else {
            player_rel(&who)?
        };
        let filter = {
            let s = serde_json::to_string(&f).ok()?;
            let with = serde_json::to_string(&Filter::ControlledBy(rel)).ok()?;
            let f: Filter = serde_json::from_str(&s.replace(&they_needle(), &with)).ok()?;
            Filter::and(vec![f, Filter::ControlledBy(rel)])
        };
        let chooser = if many { PlayerRef::Iterated } else { who.clone() };
        let pick = Sel::Choose {
            chooser,
            filter: filter.clone(),
            count: Value::c(1),
            up_to: false,
            store: None,
        };
        let e = match verb {
            0 => {
                if !tail.is_empty() {
                    return None;
                }
                return Some(Effect::Sacrifice {
                    who,
                    filter,
                    count: Value::c(1),
                });
            }
            1 => {
                if !tail.is_empty() {
                    return None;
                }
                Effect::Exile {
                    what: pick,
                    face_down: false,
                    link: false,
                }
            }
            _ => {
                if tail != "to its owner's hand" {
                    return None;
                }
                Effect::Move {
                    what: pick,
                    to: Destination::zone(ZoneKind::Hand),
                }
            }
        };
        Some(if many {
            Effect::ForEachPlayer {
                who,
                effect: Box::new(e),
            }
        } else {
            e
        })
    })();
    if parsed.is_none() {
        b.targets.truncate(before);
        (b.it, b.it_player) = saved;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "relational: [players] sacrifice/exile/return [an object with the greatest ...]", priority: 30, parse: player_edict } }

// ---------------------------------------------------------------------------
// "the creature with the least power"
// ---------------------------------------------------------------------------

/// "the creature with the least power", "the creature card in your graveyard with the
/// greatest power": the one object with the extreme value, which the controller chooses
/// among tied ones as the instruction is performed ("If two or more creatures are tied
/// for least power, you choose one of them.").
pub fn definite_extreme(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let r = s.strip_prefix("the ")?;
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural || !serde_json::to_string(&f).is_ok_and(|j| j.contains("\"Extreme\"")) {
        return None;
    }
    let f = resolve_referent(f, b)?;
    Some((
        Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        rest.to_string(),
    ))
}

/// "If two or more creatures are tied for least power, you choose one of them." after an
/// instruction about "the creature with the least power": the controller already chooses
/// among tied objects (`definite_extreme`).
fn tied_you_choose(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let s = end(s);
    let Some(r) = s.strip_prefix("if two or more ") else {
        return false;
    };
    let Some((_, r)) = r.split_once(" are tied for ") else {
        return false;
    };
    if !r.ends_with(", you choose one of them") {
        return false;
    }
    serde_json::to_string(prev)
        .is_ok_and(|j| j.contains("\"Extreme\"") && j.contains("\"Choose\""))
}

inventory::submit! { super::FollowupPattern { name: "relational: tied for greatest, you choose", priority: 100, apply: tied_you_choose } }

/// "Return to their owners' hands all creatures with toughness less than or equal to the
/// number of Islands you control", "return to your hand the creature card in your
/// graveyard with the greatest power", "Return to the battlefield target nonland permanent
/// card in your graveyard with ...": the destination before a long object phrase, read as
/// "return [objects] to [destination]".
fn return_to_destination_first(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("return to ")?;
    for dest in [
        "their owners' hands",
        "its owner's hand",
        "your hand",
        "the battlefield tapped under your control",
        "the battlefield under your control",
        "the battlefield tapped",
        "the battlefield",
    ] {
        if let Some(obj) = r.strip_prefix(dest).and_then(|x| x.strip_prefix(' ')) {
            // The object phrase must be the whole rest (a later clause would be ambiguous).
            if obj.contains(", ") || obj.contains(" and ") {
                return None;
            }
            let rewritten = format!("return {obj} to {dest}");
            return crate::oracle::effects::parse_simple(&rewritten, b);
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "relational: return to [destination] [objects]", priority: 100, parse: return_to_destination_first } }

/// "Enchanted creature and other creatures that share a creature type with it get +1/+1
/// until end of turn", "~ and each other creature with the same name as it get +3/+3 until
/// end of turn", "it and other creatures you control that share a creature type with it
/// each get +1/+1 ...": the subject and the other objects related to it as the
/// instruction is carried out (CR 608.2c).
fn subject_and_others(l: &str, b: &mut Builder) -> Option<Effect> {
    use crate::oracle::effects::{object_ref, parse_clause};
    let l = end(l);
    let (head, after) = [" and each other ", " and all other ", " and other "]
        .iter()
        .find_map(|sep| l.split_once(sep))?;
    if head.contains("target") {
        return None;
    }
    let before = b.targets.len();
    let saved_it = b.it.clone();
    let restore = |b: &mut Builder| {
        b.targets.truncate(before);
        b.it = saved_it.clone();
    };
    // The subject ("enchanted creature and other creatures ... get"), or the object an
    // instruction ends with ("put a +1/+1 counter on that creature and each other
    // creature you control that shares a creature type with it").
    let whole = object_ref(head, b).filter(|(_, r)| r.trim().is_empty());
    let subject = whole.is_some();
    let found = whole.or_else(|| {
        ["that creature", "that permanent", "it", "~", "enchanted creature", "equipped creature"]
            .iter()
            .filter(|p| head.ends_with(&format!(" {p}")))
            .find_map(|p| object_ref(p, b))
    });
    let Some((x, hrest)) = found else {
        restore(b);
        return None;
    };
    if !hrest.trim().is_empty()
        || b.targets.len() != before
        || matches!(x, Sel::All(_) | Sel::Choose { .. } | Sel::Union(_))
        || crate::oracle::patterns::oracle_hardening_referents::is_no_referent(&x)
    {
        restore(b);
        return None;
    }
    let parsed = (|| {
        let (f, _plural, rest) = parse_object_phrase(after)?;
        if !mentions_referent(&f) {
            return None;
        }
        let f = substitute(&other_than_it(f, &x), &x)?;
        // "other" relative to it even when written without a qualifier on "other".
        let f = if serde_json::to_string(&f)
            .is_ok_and(|j| j.contains(&serde_json::to_string(&Filter::not(Filter::In(Box::new(x.clone())))).unwrap_or_default()))
        {
            f
        } else {
            Filter::and(vec![f, Filter::not(Filter::In(Box::new(x.clone())))])
        };
        if !crate::relational::groups_of(&f).is_empty() || crate::relational::has_nested_group(&f)
        {
            return None;
        }
        // The rest of the instruction, said of the subject alone.
        let tail = rest.trim_start();
        let rewritten = if subject {
            let tail = tail.strip_prefix("each ").unwrap_or(tail);
            let mut tail = tail.to_string();
            for (p, s) in [("get ", "gets "), ("gain ", "gains "), ("have ", "has ")] {
                if let Some(r) = tail.strip_prefix(p) {
                    tail = format!("{s}{r}");
                }
            }
            let tail = tail.replace(" and gain ", " and gains ");
            format!("{head} {tail}")
        } else if tail.is_empty() {
            head.to_string()
        } else {
            return None;
        };
        let e = parse_clause(&rewritten, b)?;
        if b.targets.len() != before {
            return None;
        }
        let group = Sel::Union(vec![x.clone(), Sel::All(f)]);
        let json = serde_json::to_string(&e).ok()?;
        let needle = serde_json::to_string(&x).ok()?;
        if json.matches(&needle).count() != 1 {
            return None;
        }
        let group = serde_json::to_string(&group).ok()?;
        serde_json::from_str::<Effect>(&json.replace(&needle, &group)).ok()
    })();
    if parsed.is_none() {
        restore(b);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "relational: [subject] and other [objects] related to it", priority: 90, parse: subject_and_others } }

/// "Return any number of permanent cards with different names from your graveyard to the
/// battlefield", "return any number of artifact creature cards with total mana value 6 or
/// less from your graveyard to the battlefield": the controller chooses the cards as the
/// instruction is carried out, as a group meeting the requirement.
fn return_any_number(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("return any number of ")?;
    let (f, plural, rest) = parse_object_phrase(r)?;
    if !plural {
        return None;
    }
    let f = resolve_referent(f, b)?;
    let rest = rest.trim_start();
    let (to, tail) = if let Some(t) = rest.strip_prefix("to the battlefield") {
        (Destination::battlefield().under_your_control(), t)
    } else if let Some(t) = rest.strip_prefix("to your hand") {
        (Destination::zone(ZoneKind::Hand), t)
    } else {
        return None;
    };
    let (to, tail) = match tail.strip_prefix(" tapped") {
        Some(t) => (to.tapped(), t),
        None => (to, tail),
    };
    if !tail.trim().is_empty() || f.zone() != Some(ZoneKind::Graveyard) {
        return None;
    }
    if crate::relational::has_nested_group(&f) {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: Value::c(999),
            up_to: true,
            store: None,
        },
        to,
    })
}

inventory::submit! { EffectPattern { name: "relational: return any number of [cards] to ...", priority: 100, parse: return_any_number } }

// ---------------------------------------------------------------------------
// Values of groups of objects
// ---------------------------------------------------------------------------

/// Objects a value is about: a description ("creatures you control", "other creatures
/// you control"), a reference ("those creatures", "the exiled cards"), or two
/// descriptions joined by "and" ("noncreature permanents you control and noncreature
/// cards in your graveyard").
fn objects_for_value(t: &str, b: &mut Builder) -> Option<(Sel, String)> {
    use crate::oracle::effects::object_ref;
    let t = t.trim_start();
    let t2 = t.strip_prefix("all ").unwrap_or(t);
    if let Some((f, plural, rest)) = parse_object_phrase(t2) {
        // "cards milled this way", "cards revealed this way": a description the phrase
        // parser doesn't finish (the core value parser reads some of these).
        // So is a relative clause ("other spells you've cast this turn").
        let next = rest.split_whitespace().next().unwrap_or("");
        if (next.ends_with("ed") && next != "and")
            || matches!(next, "you" | "you've" | "they" | "they've" | "that" | "which")
        {
            return None;
        }
        if plural {
            let f = resolve_referent(f, b)?;
            // "... and [another description]"
            if let Some(r2) = rest.trim_start().strip_prefix("and ") {
                if let Some((f2, true, rest2)) = parse_object_phrase(r2) {
                    if f2.zone() != f.zone() {
                        return Some((
                            Sel::Union(vec![Sel::All(f), Sel::All(f2)]),
                            rest2.to_string(),
                        ));
                    }
                }
            }
            return Some((Sel::All(f), rest.to_string()));
        }
    }
    for (p, sel) in [
        ("the exiled cards", Sel::All(exiled_with_source())),
        ("cards exiled with ~", Sel::All(exiled_with_source())),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if word_end(r) {
                return Some((sel, r.to_string()));
            }
        }
    }
    let before = b.targets.len();
    let (sel, rest) = object_ref(t, b)?;
    if b.targets.len() != before {
        b.targets.truncate(before);
        return None;
    }
    Some((sel, rest))
}

/// "the total power of those creatures", "their total power", "the total mana value of
/// cards you own in exile", "the greatest mana value among noncreature permanents you
/// control and noncreature cards in your graveyard", "the least power among creatures you
/// control". None for forms the core value parser reads (a plain description after "the
/// total power of" / "the greatest power among").
pub fn value_of_objects(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let s = s.trim_start();
    // "their total power" / "their total toughness" / "their total mana value"
    if let Some(r) = s.strip_prefix("their total ") {
        let (stat, rest) = stat_word(r)?;
        let (sel, _) = crate::oracle::effects::object_ref("them", b)?;
        return Some((stat_of(stat, sel), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("the total ") {
        let (stat, r) = stat_word(r)?;
        let r = r.trim_start().strip_prefix("of ")?;
        // The core reads a plain description for power and toughness.
        if stat != Stat::ManaValue
            && parse_object_phrase(r).is_some_and(|(_, plural, rest)| {
                plural && !rest.trim_start().starts_with("and ")
            })
        {
            return None;
        }
        let (sel, rest) = objects_for_value(r, b)?;
        return Some((stat_of(stat, sel), rest));
    }
    let r = s.strip_prefix("the ")?;
    let (greatest, r) = if let Some(x) = r
        .strip_prefix("greatest ")
        .or_else(|| r.strip_prefix("highest "))
    {
        (true, x)
    } else if let Some(x) = r
        .strip_prefix("least ")
        .or_else(|| r.strip_prefix("lowest "))
    {
        (false, x)
    } else {
        return None;
    };
    let (stat, r) = stat_word(r)?;
    let r = r.trim_start().strip_prefix("among ")?;
    // The core reads "the greatest power/mana value among [a description]".
    if greatest
        && stat != Stat::Toughness
        && parse_object_phrase(r).is_some_and(|(_, plural, rest)| {
            plural && !rest.trim_start().starts_with("and ")
        })
    {
        return None;
    }
    let (sel, rest) = objects_for_value(r, b)?;
    Some((
        Value::Extreme(Box::new(tested(stat)), Box::new(sel), greatest),
        rest,
    ))
}

/// After a clause was parsed: "they" in "among creatures they control" is the player each
/// instruction is performed for, and a qualifier's "it" the effect parser didn't resolve
/// where it was used means what "it" meant as the clause began (`it`). If that has no
/// antecedent the placeholder stays, and the ability isn't understood.
pub fn resolve_clause(e: Effect, it: &Sel) -> Effect {
    let e = resolve_they(e);
    if !mentions_referent(&e) {
        return e;
    }
    substitute(&e, it).unwrap_or(e)
}

/// "Target player sacrifices a creature with the greatest power among creatures they
/// control. You gain life equal to its power.": after one player sacrifices one object
/// with an extreme quality, "it" is the sacrificed object (as `edict_greatest_power.rs`
/// has it).
pub fn note_sacrificed(e: &Effect, b: &mut Builder) {
    if let Effect::Sacrifice { who, filter, count } = e {
        let single = !matches!(
            who,
            PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
        );
        if single
            && matches!(count, Value::Const(1))
            && serde_json::to_string(filter).is_ok_and(|j| j.contains("\"Extreme\""))
        {
            b.it = Sel::Var(vars::SACRIFICED);
        }
    }
}

/// "each other attacking creature that shares a creature type with it", "another target
/// creature card with lesser mana value": in a phrase related to "it", "other" means
/// other than it (not the source), so the phrase's top-level [`Filter::Other`] becomes
/// "not it" (the placeholder, resolved with it).
pub fn other_than_referent(f: Filter) -> Filter {
    if !matches!(&f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::Other)))
        || !mentions_referent(&f)
    {
        return f;
    }
    other_than_it(f, &referent())
}

/// "that each have mana value X", "that each have mana value X or less" (after a plural
/// description: Uncage the Menagerie, Ecological Appreciation): each object's own value.
fn that_each_have<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that each have ")?;
    let (stat, r) = stat_word(r)?;
    let r = r.trim_start();
    if let Some((cmp, v, rest)) = bound(r) {
        return Some((stat_filter(stat, cmp, v), rest));
    }
    let (n, rest) = parse_number(r)?;
    Some((stat_filter(stat, Cmp::Eq, n), rest))
}

inventory::submit! { FilterSuffixPattern { name: "relational: that each have [stat] N", priority: 100, parse: that_each_have } }

/// "that dealt damage to you this turn" (Reciprocate, Spear of Heliod): see
/// `kw/dealt_damage_to_you.rs`.
fn dealt_damage_to_you<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that dealt damage to you this turn")?;
    word_end(r).then(|| {
        (
            Filter::Custom(crate::kw::dealt_damage_to_you::DEALT_DAMAGE_TO_YOU_THIS_TURN.into()),
            r,
        )
    })
}

inventory::submit! { FilterSuffixPattern { name: "relational: that dealt damage to you this turn", priority: 100, parse: dealt_damage_to_you } }
