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

/// Whether the spell's additional cost exiles a permanent ("As an additional cost to cast
/// this spell, exile a creature you control.").
fn additional_cost_exiles() -> bool {
    crate::oracle::raw_text()
        .to_lowercase()
        .contains("as an additional cost to cast this spell, exile a ")
}

/// "[instruction] if the exiled creature was a Thrull" / "If the exiled creature was a
/// Thrull, [instruction]" (Soul Exchange): the creature the additional cost exiled, as it
/// last existed on the battlefield (CR 400.7j, 608.2h).
fn if_the_exiled_creature_was(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !b.ctx.is_spell() || !additional_cost_exiles() {
        return None;
    }
    let (x, q) = if let Some(r) = l.strip_prefix("if the exiled creature was ") {
        let (q, x) = r.split_once(", ")?;
        (x, q)
    } else {
        let (x, q) = l.split_once(" if the exiled creature was ")?;
        (x, q)
    };
    let q = q.strip_prefix("a ").or_else(|| q.strip_prefix("an "))?;
    let (f, false, tail) = parse_object_phrase(q)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let e = parse_clause(x, b)?;
    Some(Effect::If {
        cond: Condition::SelMatches(
            Sel::Before(Box::new(Sel::Var(crate::zones::COST_EXILED))),
            f,
        ),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if the exiled creature was a [quality] (additional cost)", priority: 300, parse: if_the_exiled_creature_was } }

/// Whether the card creates exactly one token, with one instruction: "the token" in its
/// other abilities is that token (CR 607.1d: the objects an ability creates are linked to
/// its source, `Game::link_to_creator`; Dance of Many's ruling: "Each Dance of Many is
/// associated only with its token creature").
fn single_token_card() -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    raw.matches("create ").count() == 1 && raw.contains("create a token") && !raw.contains(" tokens")
}

/// The token the card's other ability created, while it's on the battlefield.
fn the_token() -> Filter {
    Filter::and(vec![Filter::Token, Filter::In(Box::new(Sel::Linked))])
}

/// "When ~ leaves the battlefield, exile the token." (Dance of Many).
fn act_on_the_token(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let verb = l.strip_suffix(" the token")?;
    if !b.in_trigger || !single_token_card() {
        return None;
    }
    let what = Sel::All(the_token());
    Some(match verb {
        "exile" => Effect::Exile {
            what,
            face_down: false,
            link: false,
        },
        "sacrifice" => Effect::SacrificeObjects { what },
        "destroy" => Effect::Destroy {
            what,
            no_regen: false,
        },
        _ => return None,
    })
}

inventory::submit! { EffectPattern { name: "exile the token (created by another ability)", priority: 300, parse: act_on_the_token } }

/// "When the token leaves the battlefield, ..." (Dance of Many): the token the card's
/// other ability created.
fn the_token_leaves(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if end(r) != "the token leaves the battlefield" || !single_token_card() {
        return None;
    }
    Some((
        TriggerCond::LeavesBattlefield(Filter::and(vec![
            Filter::Token,
            Filter::Custom(crate::kw::created_by_source::CREATED_BY_SOURCE.into()),
        ])),
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { super::TriggerPattern { name: "the token leaves the battlefield (created by another ability)", priority: 300, parse: the_token_leaves } }

/// "[verb] one of those creatures and [instruction on] the other" (Cannibalize: "Exile one
/// of those creatures and put two +1/+1 counters on the other."): the chosen one, then the
/// rest of the group (chosen as the spell resolves, its ruling).
fn one_and_the_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (a, c) = l.split_once(" and ")?;
    if !a.contains(" one of ") || !(c.ends_with(" the other") || c.contains(" the other ")) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let first = parse_clause(a, b)?;
    // The group the one was chosen from.
    let group = match &first {
        Effect::Seq(v) => v.iter().find_map(|e| match e {
            Effect::Store {
                var,
                sel: Sel::Choose { filter: Filter::In(g), count: Value::Const(1), .. },
            } if *var == super::referents_batches::ONE_OF_THEM => Some((**g).clone()),
            _ => None,
        }),
        _ => None,
    };
    let Some(group) = group else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    // Exactly two: "the other" is one object.
    let two = match &group {
        Sel::Target(k) => b
            .targets
            .get(*k as usize)
            .is_some_and(|t| matches!((&t.min, &t.max), (Value::Const(2), Value::Const(2)))),
        _ => false,
    };
    if !two {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    let other = Sel::Matching(
        Box::new(group),
        Filter::not(Filter::In(Box::new(Sel::Var(
            super::referents_batches::ONE_OF_THEM,
        )))),
    );
    let text = c.replacen(" the other", " it", 1);
    b.it = other;
    let second = parse_clause(&text, b);
    let Some(second) = second else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    b.it = saved.1;
    Some(Effect::seq(vec![first, second]))
}

inventory::submit! { EffectPattern { name: "[verb] one of those creatures and [instruction on] the other", priority: 300, parse: one_and_the_other } }
