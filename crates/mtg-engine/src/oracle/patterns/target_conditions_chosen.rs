//! Conditions on a target checked as the spell or ability resolves (CR 608.2b: the target
//! is still legal, the condition is checked then), often about a value chosen for the
//! source (CR 607.2d):
//!
//! - "[instruction with one target] if it's controlled by the chosen player [and if it
//!   targets you or a permanent you control]" (Stalking Leonin, Emissary of Grudges);
//! - "If target attacking creature token is the chosen type, [instruction with 'it']"
//!   (A Killer Among Us);
//! - the object qualifier "controlled by the chosen player".

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "controlled by the chosen player" / "that's controlled by the chosen player".
fn controlled_by_chosen<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s
        .strip_prefix("controlled by the chosen player")
        .or_else(|| s.strip_prefix("that's controlled by the chosen player"))?;
    (r.is_empty() || r.starts_with([' ', ',', '.']))
        .then_some((Filter::ControlledBy(PlayerRel::Chosen), r))
}

inventory::submit! { FilterSuffixPattern { name: "controlled by the chosen player", priority: 120, parse: controlled_by_chosen } }

/// What "it's [quality]" / "it [verb] ..." / "[target] is [quality]" says of an object:
/// "controlled by the chosen player", "the chosen type", "a Goblin", "targets you or a
/// permanent you control".
fn quality(q: &str) -> Option<Filter> {
    let q = end(q);
    if let Some(r) = q.strip_prefix("it targets ") {
        let text = format!("that targets {r}");
        let (fs, cond, rest) =
            super::trigger_grammar_combat_cast::spell_qualifier(&text, &Filter::Any)?;
        if cond.is_some() || !end(rest).is_empty() || fs.is_empty() {
            return None;
        }
        return Some(Filter::and(fs));
    }
    let r = q
        .strip_prefix("it's ")
        .or_else(|| q.strip_prefix("it is "))
        .unwrap_or(q);
    match r {
        "controlled by the chosen player" => return Some(Filter::ControlledBy(PlayerRel::Chosen)),
        "the chosen type" | "the chosen creature type" => return Some(Filter::ChosenType),
        "the chosen color" => return Some(Filter::ChosenColor),
        _ => {}
    }
    let x = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, tail) = parse_object_phrase(x)? else {
        return None;
    };
    end(tail).is_empty().then_some(f)
}

/// "[instruction] if it's controlled by the chosen player and if it targets you or a
/// permanent you control": the instruction's one target is checked as it resolves.
fn instruction_if_its(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let at = l.find(" if it's ").or_else(|| l.find(" if it is "))?;
    let (x, conds) = (&l[..at], &l[at + " if ".len()..]);
    let mut filters = Vec::new();
    for c in conds.split(" and if ") {
        filters.push(quality(c)?);
    }
    let first = b.targets.len();
    let e = parse_clause(x, b);
    let ok = e.is_some() && b.targets.len() == first + 1;
    if !ok {
        b.targets.truncate(first);
        return None;
    }
    Some(Effect::If {
        cond: Condition::SelMatches(Sel::Target(first as u8), Filter::and(filters)),
        then: Box::new(e?),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "[instruction] if it's [quality] (its target)", priority: 130, parse: instruction_if_its } }

/// "If target attacking creature token is the chosen type, [instruction]": the target is
/// chosen as the ability is put on the stack; the instruction ("it") happens if it has
/// the quality as the ability resolves.
fn if_target_is(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if target ")?;
    let (c, rest) = r.split_once(", ")?;
    let (what, q) = c.split_once(" is ")?;
    let f = quality(q)?;
    let text = format!("target {what}");
    let (spec, tail) = parse_target(&text)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    let first = b.targets.len();
    let slot = b.add_target(spec, "target");
    let saved = std::mem::replace(&mut b.it, Sel::Target(slot));
    let e = parse_clause(rest, b);
    b.it = saved;
    let Some(e) = e else {
        b.targets.truncate(first);
        return None;
    };
    Some(Effect::If {
        cond: Condition::SelMatches(Sel::Target(slot), f),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if target [object] is [quality], [instruction]", priority: 130, parse: if_target_is } }
