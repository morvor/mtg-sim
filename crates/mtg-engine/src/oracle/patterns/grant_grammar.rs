//! Ability-granting grammar that the core static and effect grammars don't cover.
//!
//! - A list of predicates about the source, each with its own condition (CR 611.3a):
//!   "~ gets +0/+2 as long as you control a Plains, has flying as long as you control an
//!   Island, ..., and has trample as long as you control a Forest" (Tek); "~ has trample
//!   as long as you control a Beast, haste as long as you control a Goblin, ..., and
//!   \"{B}: Regenerate ~\" as long as you control a Zombie" (Tribal Golem). A predicate
//!   without a verb shares the previous one's. Each predicate applies exactly when its own
//!   condition holds, so the line is compiled as one conditional static ability per
//!   predicate.

use super::statics::mask_quotes;
use super::StaticPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

const VERBS: &[&str] = &["gets ", "has ", "have ", "get "];

fn per_predicate_conditions(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l.matches(" as long as ").count() < 2 || !l.starts_with("~ ") {
        return None;
    }
    let orig = text.trim().trim_end_matches('.');
    let (masked, quotes) = mask_quotes(orig)?;
    let masked = masked.strip_prefix("~ ")?;
    let mut segs: Vec<&str> = Vec::new();
    for part in masked.split(", and ") {
        segs.extend(part.split(", "));
    }
    if segs.len() < 2 || !segs.iter().all(|s| s.contains(" as long as ")) {
        return None;
    }
    let mut verb = "";
    let mut out = Vec::new();
    for seg in segs {
        let seg = seg.trim();
        let lower = seg.to_lowercase();
        let body = match VERBS.iter().find(|v| lower.starts_with(**v)) {
            Some(v) => {
                verb = v;
                seg.to_string()
            }
            None if !verb.is_empty() => format!("{verb}{seg}"),
            None => return None,
        };
        // Restore the quoted abilities.
        let mut line = format!("~ {body}.");
        for (k, q) in quotes.iter().enumerate() {
            line = line.replace(&format!("\"#{k}\""), &format!("\"{q}\""));
        }
        let v = crate::oracle::statics::parse_static(&line, ctx)?;
        if v.iter()
            .any(|a| !matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
        {
            return None;
        }
        out.extend(v);
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "grants: per-predicate conditions", priority: 200, parse: per_predicate_conditions } }
