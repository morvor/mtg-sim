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

use super::FilterSuffixPattern;
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
/// means there (`b.it`). Returns None if "it" has no antecedent.
pub fn resolve_referent(f: Filter, b: &Builder) -> Option<Filter> {
    if !mentions_referent(&f) {
        return Some(f);
    }
    substitute(&f, &b.it)
}

/// Whether a compiled ability still contains the placeholder, or a group requirement
/// ([`Filter::Together`]) somewhere other than a search's or a choice's filter, where
/// nothing would check it.
pub fn unresolved(a: &AbilityDef) -> bool {
    let Ok(mut v) = serde_json::to_value(&a.kind) else {
        return false;
    };
    let s = v.to_string();
    if s.contains(&referent_needle()) {
        return true;
    }
    if !s.contains("\"Together\"") {
        return false;
    }
    fn strip_checked(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
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
                Some(r2) if word_end(r2) => (
                    Filter::and(vec![f, Filter::ControlledBy(PlayerRel::Iterated)]),
                    r2,
                ),
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
    Some((Filter::not(Filter::In(Box::new(sel))), rest))
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
    if !mentions_referent(&e) {
        return Some(e);
    }
    let (it, n) = before;
    let same_it = format!("{it:?}") == format!("{:?}", b.it);
    if !same_it || b.targets.len() != n {
        return Some(e);
    }
    Some(substitute(&e, &it).unwrap_or(e))
}
