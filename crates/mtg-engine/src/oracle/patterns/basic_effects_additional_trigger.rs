//! More triggers-an-additional-time statics (CR 603.2d), beside
//! `triggers_additional_time*.rs`:
//!
//! - an event other than entering or dying causing the ability to trigger: "If a creature
//!   attacking causes a triggered ability of a permanent you control to trigger, ..."
//!   (Isshin), "a creature you control being dealt damage" (Wayta), "a creature you
//!   control dealing combat damage to a player" (Felix Five-Boots), "a legendary permanent
//!   or an artifact entering or leaving the battlefield" (Gandalf the White), "a player
//!   drawing a card" (Krang), "you casting or copying an instant or sorcery spell"
//!   (Veyran): the event phrase is read as the trigger condition it describes, and an
//!   ability triggers an additional time only for such an event (CR 603.2d, 603.6a);
//! - sources described by alternatives: "a triggered ability of a Shaman or another Wizard
//!   you control" (Harmonic Prodigy), "of a colorless spell you control or another
//!   colorless permanent you control" (Echoes of Eternity);
//! - a condition: "... of another Shrine you control triggers while you control six or
//!   more Shrines, ..." (Sanctum of All).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

const TAIL: &str =
    " causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time";

/// The trigger condition a gerund event phrase describes ("a creature you control
/// attacking" → "whenever a creature you control attacks").
fn event_condition(ev: &str) -> Option<TriggerCond> {
    // The event itself, not a batch of such events (the cause is matched against the
    // single event that made the ability trigger).
    let parse = |s: String| {
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever {s}")).map(|t| {
            match t.0 {
                TriggerCond::Batched { trigger, .. } => *trigger,
                c => c,
            }
        })
    };
    if let Some(np) = ev.strip_suffix(" entering or leaving the battlefield") {
        // "a legendary permanent or an artifact": each of them.
        let mut conds = Vec::new();
        for item in np.split(" or ") {
            conds.push(parse(format!("{item} enters"))?);
            conds.push(parse(format!("{item} leaves the battlefield"))?);
        }
        return Some(TriggerCond::AnyOf(conds));
    }
    let forms: [(&str, &str); 6] = [
        (" attacking", " attacks"),
        (" being dealt damage", " is dealt damage"),
        (" dealing combat damage to a player", " deals combat damage to a player"),
        (" dealing damage", " deals damage"),
        (" leaving the battlefield", " leaves the battlefield"),
        (" drawing a card", " draws a card"),
    ];
    for (g, v) in forms {
        if let Some(np) = ev.strip_suffix(g) {
            // A player draws; an object does the others.
            if g == " drawing a card" && !matches!(np, "a player" | "you" | "an opponent") {
                return None;
            }
            return parse(format!("{np}{v}"));
        }
    }
    if let Some(spell) = ev.strip_prefix("you casting or copying ") {
        return parse(format!("you cast or copy {spell}"));
    }
    if let Some(spell) = ev.strip_prefix("you casting ") {
        return parse(format!("you cast {spell}"));
    }
    None
}

fn caused_additional_time(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let ev = end(l).strip_prefix("if ")?.strip_suffix(TAIL)?;
    // Entering or dying alone: see `triggers_additional_time*.rs`.
    if ev.ends_with(" dying") || ev.ends_with(" entering") || ev.ends_with(" entering the battlefield")
    {
        return None;
    }
    let cause = event_condition(ev)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalTrigger {
            sources: Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            cause: Some(Box::new(cause)),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "basic effects: [event] causes a triggered ability to trigger an additional time", priority: 110, parse: caused_additional_time } }

/// One description of the objects whose abilities trigger again: "a Shaman", "another
/// Wizard you control", "a colorless spell you control".
fn source_item(s: &str) -> Option<Filter> {
    let s = s.trim();
    let (other, s) = match s.strip_prefix("another ") {
        Some(r) => (true, r),
        None => (
            false,
            s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")).unwrap_or(s),
        ),
    };
    let (f, plural, tail) = parse_object_phrase(s)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let f = if f.zone().is_none() {
        Filter::and(vec![f, Filter::Permanent])
    } else {
        f
    };
    Some(if other {
        Filter::and(vec![Filter::Other, f])
    } else {
        f
    })
}

fn controls(f: &Filter) -> bool {
    match f {
        Filter::ControlledBy(_) => true,
        Filter::And(v) => v.iter().any(controls),
        _ => false,
    }
}

/// "If a triggered ability of a Shaman or another Wizard you control triggers, ..." and
/// "If a triggered ability of another Shrine you control triggers while you control six or
/// more Shrines, ...".
fn sources_additional_time(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if a triggered ability of ")?;
    let (objects, cond) = if let Some(o) = r.strip_suffix(" triggers, that ability triggers an additional time") {
        match o.split_once(" triggers while ") {
            Some(_) => return None,
            None => (o, None),
        }
    } else {
        let (o, rest) = r.split_once(" triggers while ")?;
        let c = rest.strip_suffix(", that ability triggers an additional time")?;
        (o, Some(super::super::statics::parse_condition(c, ctx)?))
    };
    let items: Vec<&str> = objects.split(" or ").collect();
    if items.len() < 2 && cond.is_none() {
        return None;
    }
    let mut fs: Vec<Filter> = items.iter().map(|i| source_item(i)).collect::<Option<_>>()?;
    // "a Shaman or another Wizard you control": the controller describes them all.
    if fs.len() > 1 && controls(fs.last().expect("several")) {
        for f in fs.iter_mut() {
            if !controls(f) {
                *f = Filter::and(vec![f.clone(), Filter::ControlledBy(PlayerRel::You)]);
            }
        }
    }
    let sources = if fs.len() == 1 {
        fs.pop().expect("one")
    } else {
        Filter::Or(fs)
    };
    let mut s = StaticAbility::new(StaticEffect::AdditionalTrigger {
        sources,
        cause: None,
    });
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "basic effects: a triggered ability of [alternatives] triggers an additional time", priority: 110, parse: sources_additional_time } }
