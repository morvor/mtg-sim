//! Copying abilities on the stack (CR 707.10): "Copy target activated or triggered ability
//! you control." (Lithoform Engine, Strionic Resonator), optionally limited by the
//! ability's source: "from a colorless source", "from an artifact source", "from another
//! legendary source that's not a commander". The follow-up "You may choose new targets for
//! the copy." is handled by the spell-copy follow-up (see `copy_spells.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "a colorless source", "an artifact source", "another legendary source that's not a
/// commander": a description of an ability's source. Returns the filter for the source
/// and the rest of the text.
fn source_description(s: &str) -> Option<(Filter, &str)> {
    let mut parts = Vec::new();
    let mut s = s.trim_start();
    if let Some(r) = strip(s, "another ") {
        parts.push(Filter::Other);
        s = r;
    } else if let Some(r) = strip(s, "an ").or_else(|| strip(s, "a ")) {
        s = r;
    }
    loop {
        let (w, rest) = split_word(s);
        if w.is_empty() {
            return None;
        }
        s = rest;
        if w == "source" {
            break;
        }
        parts.push(adjective(w).or_else(|| head_noun(w))?);
    }
    if let Some(r) = strip(s, "that's not a commander").or_else(|| strip(s, "that isn't a commander"))
    {
        parts.push(Filter::not(Filter::Commander));
        s = r;
    }
    Some((Filter::and(parts), s))
}

/// "copy target activated or triggered ability you control [from a ... source]",
/// "copy target triggered ability you control".
fn copy_target_ability(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy ")?;
    if !r.starts_with("target ") {
        return None;
    }
    let (mut spec, tail) = parse_target(r)?;
    let TargetKind::Ability(f) = &mut spec.what else {
        return None;
    };
    let mut parts = vec![f.clone()];
    let mut rest = tail;
    if let Some(x) = strip(rest, "you control") {
        parts.push(Filter::ControlledBy(PlayerRel::You));
        rest = x;
    }
    if let Some(x) = strip(rest, "from ") {
        let (source, x) = source_description(x)?;
        parts.push(Filter::AbilityFrom(Box::new(source)));
        rest = x;
    }
    if !end(rest).is_empty() {
        return None;
    }
    *f = Filter::and(parts);
    let text = r[..r.len() - rest.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some(Effect::CopySpell {
        what: Sel::Target(slot),
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { EffectPattern { name: "copy target ability", priority: 100, parse: copy_target_ability } }
