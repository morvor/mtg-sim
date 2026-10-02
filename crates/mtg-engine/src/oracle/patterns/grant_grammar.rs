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
use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
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

// ---------------------------------------------------------------------------
// Quoted abilities in one-shot effects
// ---------------------------------------------------------------------------

/// The index of a masked quote item (`"#3"`).
fn quote_index(item: &str) -> Option<usize> {
    item.trim()
        .strip_prefix("\"#")?
        .strip_suffix('"')?
        .parse()
        .ok()
}

/// Splits a grant list ("flying, hexproof, and \"#0\"", "\"#0\" and \"#1\"") into its
/// keyword items and its quoted abilities (indices). Quotes are masked.
fn split_grant_list(list: &str) -> (Vec<String>, Vec<usize>) {
    let mut v: Vec<&str> = vec![list];
    for sep in [", and ", ", ", " and "] {
        v = v
            .into_iter()
            .flat_map(|p| p.split(sep))
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
    }
    let mut kws: Vec<String> = Vec::new();
    let mut qs = Vec::new();
    for item in v {
        match quote_index(item) {
            Some(k) => qs.push(k),
            None => match kws.last_mut() {
                // "protection from black and from red" is one keyword.
                Some(last) if item.starts_with("from ") && last.starts_with("protection from ") => {
                    last.push_str(" and ");
                    last.push_str(item);
                }
                _ => kws.push(item.to_string()),
            },
        }
    }
    (kws, qs)
}

/// Joins keyword items as Oracle text lists them ("a", "a and b", "a, b, and c").
fn join_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} and {}", items[0], items[1]),
        n => format!("{}, and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

/// Compiles the quoted abilities `idx` of the sentence (masked as `"#k"`), granted to an
/// object of kind `hint`; `to_source`: granted to the card itself.
pub(crate) fn compile_quotes(
    idx: &[usize],
    quotes: &[String],
    hint: crate::types::CardType,
    to_source: bool,
    b: &Builder,
) -> Option<Vec<Ability>> {
    let text = crate::oracle::normalize(&crate::oracle::raw_text(), b.ctx);
    let mut out = Vec::new();
    for k in idx {
        let q = quotes.get(*k)?;
        out.extend(super::statics::granted_abilities_to(
            q, &text, hint, b.ctx, to_source,
        )?);
    }
    Some(out)
}

const DURATION_SUFFIXES: &[&str] = &[
    " until end of turn",
    " until your next turn",
    " until end of combat",
    " for as long as ~ remains on the battlefield",
    ". it's still a land",
];

/// "[subject] becomes [type words] with [keywords and] \"[ability]\" [duration]",
/// "[subject] becomes [type words] and gains \"[ability]\"" (CR 611.2, 613.1f): the
/// sentence without the quoted abilities is a "becomes" effect, and the abilities are
/// granted by the same effect, for the same duration. "~" in a quoted ability means the
/// object that has it.
fn becomes_with_quotes(l: &str, b: &mut Builder) -> Option<Effect> {
    if !l.contains('"') || !(l.contains(" becomes ") || l.contains(" become ")) {
        return None;
    }
    let (masked, quotes) = mask_quotes(l)?;
    // Peel a trailing duration.
    let mut core = masked.as_str();
    let mut suffix = "";
    for s in DURATION_SUFFIXES {
        if let Some(r) = core.strip_suffix(s) {
            core = r;
            suffix = s;
            break;
        }
    }
    // The grant list: the last " with " or " and gains " clause holding a quote.
    let mut best: Option<(usize, &str)> = None;
    for sep in [" with ", " and gains ", " and gain "] {
        if let Some(i) = core.rfind(sep) {
            if core[i..].contains("\"#") && best.is_none_or(|(j, _)| i > j) {
                best = Some((i, sep));
            }
        }
    }
    let (i, sep) = best?;
    let head = &core[..i];
    if head.contains("\"#") || !(head.contains(" becomes ") || head.contains(" become ")) {
        return None;
    }
    let (kws, qs) = split_grant_list(&core[i + sep.len()..]);
    if qs.is_empty() || kws.iter().any(|k| k.contains('"')) {
        return None;
    }
    let rebuilt = if kws.is_empty() {
        format!("{head}{suffix}")
    } else {
        format!("{head}{sep}{}{suffix}", join_list(&kws))
    };
    // "becomes a Construct artifact creature with \"This creature's power and toughness
    // are each equal to ...\"": the granted characteristic-defining ability gives the
    // creature its power and toughness (CR 604.3, 613.4a).
    let cda_pt = {
        let gctx_hint = crate::types::CardType::Creature;
        compile_quotes(&qs, &quotes, gctx_hint, false, b)
            .or_else(|| compile_quotes(&qs, &quotes, gctx_hint, true, b))
            .is_some_and(|v| v.iter().any(is_cda_pt))
    };
    let saved = (b.targets.len(), b.it.clone());
    let mut e = match crate::oracle::effects::parse_sentence(&rebuilt, b) {
        Some(e) => e,
        None if cda_pt => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            let with_pt = [" becomes a ", " becomes an ", " become "]
                .iter()
                .find_map(|p| {
                    let i = rebuilt.find(p)?;
                    let article = if p.ends_with("an ") { "a " } else { "" };
                    Some(format!(
                        "{}{}{article}0/0 {}",
                        &rebuilt[..i],
                        if p.ends_with("an ") { " becomes " } else { p },
                        &rebuilt[i + p.len()..]
                    ))
                })?;
            let mut e = crate::oracle::effects::parse_sentence(&with_pt, b)?;
            let Effect::Modify { mods, .. } = &mut e else {
                return None;
            };
            let zero = |v: &Option<Value>| matches!(v, Some(Value::Const(0)));
            let n = mods.len();
            mods.retain(|m| !matches!(m, Modification::SetPT(p, t) if zero(p) && zero(t)));
            if mods.len() + 1 != n {
                return None;
            }
            e
        }
        None => return None,
    };
    let (what, mods) = match &mut e {
        Effect::Modify { what, mods, .. } => (what.clone(), mods),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => match &mut **then {
            Effect::Modify { what, mods, .. } => (what.clone(), mods),
            _ => return None,
        },
        _ => return None,
    };
    // Only effects that make the objects something (types, colors, P/T).
    if !mods.iter().any(|m| {
        matches!(
            m,
            Modification::SetTypes { .. }
                | Modification::AddTypes(_)
                | Modification::AddSubtypes(_)
                | Modification::SetColors(_)
                | Modification::SetPT(..)
        )
    }) {
        return None;
    }
    let hint = if mods.iter().any(|m| match m {
        Modification::SetTypes { types, .. } | Modification::AddTypes(types) => {
            types.contains(&crate::types::CardType::Creature)
        }
        _ => false,
    }) {
        crate::types::CardType::Creature
    } else {
        crate::types::CardType::Artifact
    };
    let to_source = matches!(what, Sel::This);
    let granted = compile_quotes(&qs, &quotes, hint, to_source, b)?;
    mods.extend(granted.into_iter().map(Modification::AddAbility));
    Some(e)
}

/// An ability that sets the power and toughness of the object that has it ("This
/// creature's power and toughness are each equal to ...": granted, it isn't
/// characteristic-defining, CR 604.3a).
fn is_cda_pt(a: &Ability) -> bool {
    matches!(&a.kind, AbilityKind::Static(st)
        if matches!(&st.effect, StaticEffect::Continuous { affected: Filter::Source, mods }
            if mods.iter().any(|m| matches!(m, Modification::SetPT(Some(_), Some(_))))))
}

inventory::submit! { EffectPattern { name: "grants: becomes ... with \"[ability]\"", priority: 90, parse: becomes_with_quotes } }

/// Splits "[a], gets [b], and gains [c]" at the commas and "and"s that start a "gets",
/// "gains" or "has" predicate.
fn split_predicates(s: &str) -> Vec<&str> {
    let mut cuts = vec![];
    for verb in ["gets ", "gains ", "has "] {
        for sep in [", and ", ", ", " and "] {
            let pat = format!("{sep}{verb}");
            for (i, _) in s.match_indices(&pat) {
                cuts.push((i, i + sep.len()));
            }
        }
    }
    cuts.sort();
    // ", and gets" also matches ", gets"-less forms; keep the earliest cut at each place.
    let mut out = Vec::new();
    let mut from = 0;
    let mut last_end = 0;
    for (start, body) in cuts {
        if start < last_end {
            continue;
        }
        out.push(&s[from..start]);
        from = body;
        last_end = body;
    }
    out.push(&s[from..]);
    out
}

/// "[subject] becomes [type words or a color], gets +N/+N, and gains [keywords and
/// quoted abilities] [duration]" (Defiling Tears, Mizzium Tank, Kellan): one continuous
/// effect on the subject, each part read as the sentence it would be on its own.
fn becomes_and_more(l: &str, b: &mut Builder) -> Option<Effect> {
    let (masked, quotes) = mask_quotes(l)?;
    let (lead, core) = match masked.strip_prefix("until end of turn, ") {
        Some(r) => (" until end of turn", r),
        None => ("", masked.as_str()),
    };
    let mut core = core;
    let mut suffix = lead;
    if lead.is_empty() {
        for s in DURATION_SUFFIXES[..4].iter() {
            if let Some(r) = core.strip_suffix(s) {
                core = r;
                suffix = s;
                break;
            }
        }
    }
    let (subject, rest) = core.split_once(" becomes ")?;
    if subject.contains('"') {
        return None;
    }
    let parts = split_predicates(rest);
    if parts.len() < 2 {
        return None;
    }
    let first = format!("{subject} becomes {}{suffix}", parts[0]);
    if first.contains("\"#") {
        return None;
    }
    let Effect::Modify {
        what,
        mut mods,
        duration,
    } = crate::oracle::effects::parse_sentence(&first, b)?
    else {
        return None;
    };
    let w = format!("{what:?}");
    // The later predicates are about the same object ("it").
    b.it = what.clone();
    for p in &parts[1..] {
        let (kws, qs) = match p.strip_prefix("gains ") {
            Some(list) => split_grant_list(list),
            None => (vec![], vec![]),
        };
        if !qs.is_empty() {
            let hint = crate::types::CardType::Creature;
            let to_source = matches!(what, Sel::This);
            mods.extend(
                compile_quotes(&qs, &quotes, hint, to_source, b)?
                    .into_iter()
                    .map(Modification::AddAbility),
            );
            if kws.is_empty() {
                continue;
            }
        }
        let text = if qs.is_empty() {
            format!("it {p}{suffix}")
        } else {
            format!("it gains {}{suffix}", join_list(&kws))
        };
        if text.contains("\"#") {
            return None;
        }
        let Effect::Modify {
            what: w2,
            mods: m2,
            duration: d2,
        } = crate::oracle::effects::parse_sentence(&text, b)?
        else {
            return None;
        };
        if format!("{w2:?}") != w || format!("{d2:?}") != format!("{duration:?}") {
            return None;
        }
        mods.extend(m2);
    }
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "grants: becomes ..., gets ..., and gains ...", priority: 89, parse: becomes_and_more } }
