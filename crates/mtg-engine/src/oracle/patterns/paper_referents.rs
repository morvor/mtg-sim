//! Referents of older Oracle wordings:
//!
//! - a subtype noun as the anaphor of a trigger's subject: "Whenever a Sliver becomes
//!   blocked, that Sliver gets +1/+1 until end of turn for each creature blocking it."
//!   (Spined Sliver): "that Sliver" is the object the trigger is about, as "that
//!   creature" would be;
//! - "the creature" as the subject of a replacement for the object the text just
//!   mentioned: "That creature gains \"Cumulative upkeep {2}.\" If the creature would leave
//!   the battlefield, exile it instead of putting it anywhere else." (Dreams of the Dead);
//! - a qualifier on a group the text named: "Tap X target creatures. ~ deals 2 damage to
//!   each of those creatures with flying." (Winter Blast: the creatures among them that
//!   have flying as the spell resolves, its ruling).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "that [subtype]" in a triggered ability whose "it" is the trigger's object: "that
/// creature".
fn that_subtype(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger || !matches!(b.it, Sel::TriggerObject | Sel::TriggerLki) {
        return None;
    }
    let l = end(l);
    let mut out = String::new();
    let mut rest = l;
    let mut changed = false;
    while let Some(i) = rest.find("that ") {
        let at_word = i == 0 || rest[..i].ends_with(' ');
        let after = &rest[i + "that ".len()..];
        let w_len = after
            .find(|c: char| !c.is_alphanumeric() && c != '-')
            .unwrap_or(after.len());
        let w = &after[..w_len];
        let tail = &after[w_len..];
        let word_ends = tail.is_empty() || tail.starts_with(' ') || tail.starts_with('\'');
        // A creature type noun only ("that Sliver"), not "that creature" or a card type.
        if at_word
            && word_ends
            && !w.is_empty()
            && subtype_word(w).is_some()
            && !w.ends_with('s')
        {
            out.push_str(&rest[..i]);
            out.push_str("that creature");
            rest = tail;
            changed = true;
        } else {
            out.push_str(&rest[..i + "that ".len()]);
            rest = after;
        }
    }
    out.push_str(rest);
    if !changed {
        return None;
    }
    parse_clause(&out, b)
}

inventory::submit! { EffectPattern { name: "that [subtype] (the trigger's object)", priority: 300, parse: that_subtype } }

/// "If the creature would [event], [replacement]": "the creature" is what "it" refers to
/// (an object the text just put onto the battlefield or named), never the source.
fn if_the_creature_would(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("if the creature would ")?;
    if matches!(b.it, Sel::This | Sel::None)
        || super::oracle_hardening_referents::is_no_referent(&b.it)
    {
        return None;
    }
    parse_clause(&format!("if it would {r}"), b)
}

inventory::submit! { EffectPattern { name: "if the creature would [event], [replacement]", priority: 300, parse: if_the_creature_would } }

/// The plural object of a simple instruction, to which a qualifier applies.
fn group_object(e: &mut Effect) -> Option<&mut Sel> {
    match e {
        Effect::DealDamage { to, .. } => Some(to),
        Effect::Tap { what }
        | Effect::Untap { what }
        | Effect::Destroy { what, .. }
        | Effect::Exile { what, .. }
        | Effect::Modify { what, .. }
        | Effect::AddCounters { what, .. } => Some(what),
        _ => None,
    }
}

/// "... each of those creatures with flying": the members of the named group that match
/// the qualifier as the instruction is performed.
fn group_with_qualifier(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (noun, i) = ["creatures", "permanents", "lands", "artifacts"]
        .iter()
        .find_map(|n| l.find(&format!("each of those {n} with ")).map(|i| (*n, i)))?;
    let head = format!("{}each of those {noun}", &l[..i]);
    let qual = &l[head.len()..];
    // The qualifier ends the clause.
    let phrase = format!("{noun}{qual}");
    let (f, plural, tail) = parse_object_phrase(&phrase)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    let base = parse_object_phrase(noun)?.0;
    if format!("{f:?}") == format!("{base:?}") {
        return None;
    }
    let mut e = parse_clause(&head, b)?;
    let what = group_object(&mut e)?;
    if !matches!(what, Sel::Target(_) | Sel::Var(_) | Sel::AllTargets) {
        return None;
    }
    *what = Sel::Matching(Box::new(what.clone()), f);
    Some(e)
}

inventory::submit! { EffectPattern { name: "each of those [objects] with [qualifier]", priority: 300, parse: group_with_qualifier } }
