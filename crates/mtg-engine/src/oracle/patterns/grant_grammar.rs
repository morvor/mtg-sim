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
use super::{EffectPattern, FilterSuffixPattern, FollowupPattern, StaticPattern};
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
    // "... and loses all other card types and abilities" (Vraska, Betrayal's Sting): the
    // new card types replace the old ones anyway (CR 205.1a); the abilities it had are
    // removed before the granted ones are added (layer 6, in order, CR 613.1f).
    let mut masked = masked.as_str();
    let mut other_types = false;
    let mut lose_abilities = false;
    for (tail, t, a) in [
        (" and loses all other card types and abilities", true, true),
        (", and it loses all other card types and abilities", true, true),
        (" and loses all other card types", true, false),
        (", and it loses all other card types", true, false),
        (" and loses all other abilities", false, true),
        (", and it loses all other abilities", false, true),
    ] {
        if let Some(r) = masked.strip_suffix(tail) {
            masked = r;
            other_types = t;
            lose_abilities = a;
            break;
        }
    }
    // Peel a trailing duration.
    let mut core = masked;
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
    if other_types && !mods.iter().any(|m| matches!(m, Modification::SetTypes { .. })) {
        return None;
    }
    if lose_abilities {
        mods.insert(0, Modification::RemoveAllAbilities);
    }
    mods.extend(granted.into_iter().map(Modification::AddAbility));
    Some(e)
}

/// "If you do, return that card to the battlefield tapped under your control. It's a
/// Treasure artifact with \"{T}, Sacrifice this artifact: Add one mana of any color,\"
/// and it loses all other card types." (Vraska, the Silencer): what the permanent is as
/// it enters (CR 614.1c), read as "becomes" would be.
fn its_a_with_quotes(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = l.strip_prefix("it's ") else {
        return false;
    };
    if !rest.contains('"') {
        return false;
    }
    {
        let Some(Effect::Move { to, .. }) = last_move(prev) else {
            return false;
        };
        if to.zone != ZoneKind::Battlefield || !to.with_mods.is_empty() {
            return false;
        }
    }
    let saved = b.it.clone();
    b.it = Sel::This;
    let e = becomes_with_quotes(&format!("~ becomes {rest}"), b);
    b.it = saved;
    let Some(Effect::Modify {
        what: Sel::This,
        mods,
        duration: Duration::Permanent,
    }) = e
    else {
        return false;
    };
    // An Aura put onto the battlefield is attached as it enters (CR 303.4f), which
    // "enters as" modifications don't do.
    if mods.iter().any(|m| match m {
        Modification::SetTypes { subtypes, .. } | Modification::AddSubtypes(subtypes) => {
            subtypes.iter().any(|s| s == "Aura")
        }
        _ => false,
    }) {
        return false;
    }
    let Some(Effect::Move { to, .. }) = last_move(prev) else {
        return false;
    };
    to.with_mods = mods;
    true
}

/// The last instruction of `e`, if it's a zone change.
fn last_move(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Move { .. } => Some(e),
        Effect::Seq(v) => v.last_mut().and_then(last_move),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_move(then),
        Effect::May { effect, .. } => last_move(effect),
        _ => None,
    }
}

inventory::submit! { FollowupPattern { name: "grants: it's a [type] with \"[ability]\"", priority: 90, apply: its_a_with_quotes } }

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

// ---------------------------------------------------------------------------
// Compound subjects
// ---------------------------------------------------------------------------

/// Splits a subject list ("you, planeswalkers you control, and other creatures you
/// control", "green creatures and white creatures") into its parts. A controller written
/// once after the last part applies to each part ("Auras, Equipment, and modified
/// creatures you control", "Treefolk and Forests you control").
fn subject_parts(s: &str) -> Vec<String> {
    let mut v: Vec<&str> = vec![s];
    for sep in [", and ", ", ", " and "] {
        v = v
            .into_iter()
            .flat_map(|p| p.split(sep))
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
    }
    let mut out: Vec<String> = v.iter().map(|p| p.to_string()).collect();
    const OWNERS: &[&str] = &[" you control", " your opponents control"];
    if let Some(suffix) = v.last().and_then(|l| OWNERS.iter().find(|o| l.ends_with(**o))) {
        for p in out.iter_mut() {
            let lone = matches!(p.as_str(), "you" | "~" | "it" | "they" | "them");
            if !lone && !p.contains("control") && !p.contains("target ") {
                p.push_str(suffix);
            }
        }
    }
    out
}

const STATIC_VERBS: &[(&str, &str, &str)] = &[
    // (verb found, plural form, singular form)
    (" have ", "have", "has"),
    (" has ", "have", "has"),
    (" get ", "get", "gets"),
    (" gets ", "get", "gets"),
];

/// The earliest grant verb outside quotes: (index, plural, singular, length).
fn find_verb(masked: &str, verbs: &[(&str, &'static str, &'static str)]) -> Option<(usize, &'static str, &'static str, usize)> {
    verbs
        .iter()
        .filter_map(|(v, pl, sg)| masked.find(v).map(|i| (i, *pl, *sg, v.len())))
        .min_by_key(|x| x.0)
}

/// "You and Humans you control have hexproof." (Sigarda, Heron's Grace), "Green creatures
/// and white creatures have protection from Gorgons.", "Saproling creatures and other
/// Treefolk creatures get +1/+1.", "During your turn, you and ~ have hexproof.": the
/// predicate applies to each part of the subject. The objects are one group, so an object
/// that fits two parts is affected once; a player part is a player ability.
fn compound_subject_static(_l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = text.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    if lower.len() != t.len() {
        return None;
    }
    let body_start = if lower.starts_with("during your turn, ") {
        "during your turn, ".len()
    } else {
        0
    };
    let body = &lower[body_start..];
    let (masked, _) = mask_quotes(body)?;
    let (i, plural, singular, vlen) = find_verb(&masked, STATIC_VERBS)?;
    // The subject comes before any quote, so the masked and plain texts agree there.
    if masked[..i].contains('"') {
        return None;
    }
    let parts = subject_parts(&body[..i]);
    if parts.len() < 2 {
        return None;
    }
    let prefix = &t[..body_start];
    let pred = &t[body_start + i + vlen..];
    let mut out: Vec<Ability> = Vec::new();
    let mut group: Option<(StaticAbility, Vec<Filter>, String)> = None;
    for part in parts {
        // The parts are lowercase: subjects have no quotes, and case doesn't matter
        // to the subject grammar.
        let part = part.as_str();
        let orig_part = part;
        let forms: &[&str] = match part {
            "you" => &[plural],
            "~" => &[singular],
            _ => &[plural, singular],
        };
        let v = forms.iter().find_map(|verb| {
            crate::oracle::statics::parse_static(&format!("{prefix}{orig_part} {verb} {pred}."), ctx)
        })?;
        let [a] = v.as_slice() else {
            return None;
        };
        let AbilityKind::Static(st) = &a.kind else {
            return None;
        };
        match &st.effect {
            StaticEffect::PlayerEffect { .. } if part == "you" => out.push(a.clone()),
            StaticEffect::Continuous { affected, mods } if part != "you" => {
                let key = format!("{mods:?}{:?}{:?}", st.condition, st.zone);
                match &mut group {
                    None => group = Some((st.clone(), vec![affected.clone()], key)),
                    Some((_, filters, k)) if *k == key => filters.push(affected.clone()),
                    Some(_) => return None,
                }
            }
            _ => return None,
        }
    }
    if let Some((mut st, filters, _)) = group {
        if let StaticEffect::Continuous { affected, .. } = &mut st.effect {
            *affected = if filters.len() == 1 {
                filters.into_iter().next()?
            } else {
                Filter::Or(filters)
            };
        }
        out.push(AbilityDef::new(AbilityKind::Static(st), t));
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "grants: compound subjects", priority: 210, parse: compound_subject_static } }

const EFFECT_VERBS: &[(&str, &str, &str)] = &[
    (" each gain ", "gain", "gains"),
    (" each get ", "get", "gets"),
    (" both gain ", "gain", "gains"),
    (" both get ", "get", "gets"),
    (" gain ", "gain", "gains"),
    (" gains ", "gain", "gains"),
    (" get ", "get", "gets"),
    (" gets ", "get", "gets"),
];

/// "Auras, Equipment, and modified creatures you control gain hexproof until end of
/// turn." (Silkguard), "Target creature you control and target creature an opponent
/// controls each gain indestructible until end of turn." (Fated Clash), "~ and up to one
/// other target creature each get +3/+3 until end of turn.", "it and Zombies you control
/// gain deathtouch until end of turn.": one effect on the union of the parts, each read
/// as the sentence it would be on its own (an object in two parts is affected once).
fn compound_subject_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let (lead, body) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (" until end of turn", r),
        None => match l.strip_prefix("until your next turn, ") {
            Some(r) => (" until your next turn", r),
            None => ("", l),
        },
    };
    let (masked, _) = mask_quotes(body)?;
    let (i, plural, singular, vlen) = find_verb(&masked, EFFECT_VERBS)?;
    if masked[..i].contains('"') {
        return None;
    }
    let parts = subject_parts(&body[..i]);
    if parts.len() < 2 {
        return None;
    }
    let pred = &body[i + vlen..];
    let saved = (b.targets.len(), b.it.clone());
    let mut sels = Vec::new();
    let mut players = Vec::new();
    let mut key: Option<(String, Duration, Vec<Modification>)> = None;
    for part in parts {
        let part = part.as_str();
        // "you and planeswalkers you control gain protection from that player": the
        // player gains it too (CR 702.11c, 702.16b).
        if part == "you" {
            let e = crate::oracle::effects::parse_sentence(&format!("you {plural} {pred}{lead}"), b);
            match e {
                Some(e @ Effect::AddPlayerEffect { .. }) => players.push(e),
                Some(Effect::Seq(v)) if v.iter().all(|x| matches!(x, Effect::AddPlayerEffect { .. })) => {
                    players.extend(v)
                }
                _ => {
                    b.targets.truncate(saved.0);
                    b.it = saved.1;
                    return None;
                }
            }
            continue;
        }
        let forms: &[&str] = if part == "~" || part.starts_with("target ") || part == "it" {
            &[singular, plural]
        } else {
            &[plural, singular]
        };
        let e = forms.iter().find_map(|verb| {
            let n = b.targets.len();
            let r = crate::oracle::effects::parse_sentence(&format!("{part} {verb} {pred}{lead}"), b);
            if r.is_none() {
                b.targets.truncate(n);
            }
            r
        });
        let Some(Effect::Modify {
            what,
            mods,
            duration,
        }) = e
        else {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        };
        let k = format!("{mods:?}{duration:?}");
        match &key {
            None => key = Some((k, duration, mods)),
            Some((k0, _, _)) if *k0 == k => {}
            Some(_) => {
                b.targets.truncate(saved.0);
                b.it = saved.1;
                return None;
            }
        }
        sels.push(what);
    }
    let modify = match key {
        Some((_, duration, mods)) => Effect::Modify {
            what: if sels.len() == 1 {
                sels.pop()?
            } else {
                Sel::Union(sels)
            },
            mods,
            duration,
        },
        None => return None,
    };
    players.push(modify);
    Some(Effect::seq(players))
}

inventory::submit! { EffectPattern { name: "grants: compound subjects", priority: 210, parse: compound_subject_effect } }

/// "Each creature you control that's a Fungus or a Saproling gets +1/+1 until end of
/// turn.", "Each creature you control with flying, deathtouch, and/or lifelink gets +1/+0
/// until end of turn.", "Equipment you control gain hexproof until end of turn.": a group
/// subject read by the static subject grammar (relative clauses, nouns whose plural is
/// the singular), and the predicate read as it would be about one object of the group.
/// The group is determined as the effect begins (CR 611.2c).
fn group_subject_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let (lead, body) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (" until end of turn", r),
        None => ("", l),
    };
    let (masked, _) = mask_quotes(body)?;
    let (i, plural, singular, vlen) = find_verb(&masked, &EFFECT_VERBS[4..])?;
    let subject = &body[..i];
    if subject.contains('"') {
        return None;
    }
    let verb = body[i..i + vlen].trim();
    let phrase = match subject
        .strip_prefix("each ")
        .or_else(|| subject.strip_prefix("all "))
    {
        Some(p) => p,
        // A bare noun phrase with a plural verb: "Equipment you control gain ...".
        None if verb == plural => subject,
        None => return None,
    };
    if phrase.starts_with("other ") && !phrase.contains(' ') {
        return None;
    }
    let (f, _, rest) = super::statics::object_phrase(phrase)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let pred = &body[i + vlen..];
    let saved = b.it.clone();
    b.it = Sel::All(f);
    let e = crate::oracle::effects::parse_sentence(&format!("it {singular} {pred}{lead}"), b);
    b.it = saved;
    match e? {
        e @ Effect::Modify { what: Sel::All(_), .. } => Some(e),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "grants: group subjects", priority: 220, parse: group_subject_effect } }

/// "that dealt damage this turn" (Executioner's Swing): an object that was the source of
/// damage this turn.
fn dealt_damage_suffix<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that dealt damage this turn")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((
        Filter::Custom(crate::kw::grant_filters::DEALT_DAMAGE_THIS_TURN.into()),
        r,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "grants: that dealt damage this turn", priority: 100, parse: dealt_damage_suffix } }

/// "~ deals 2 damage to target player and gains indestructible until end of turn." (Ellie,
/// Vengeful Hunter): two predicates of the same object, the second read as its own
/// sentence about it.
fn shared_object_subject(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let rest = l.strip_prefix("~ ")?;
    if l.contains('"') {
        return None;
    }
    let (first, verb, second) = [" and gains ", " and gets "]
        .iter()
        .find_map(|v| rest.rsplit_once(v).map(|(a, c)| (a, v.trim_start_matches(" and "), c)))?;
    if !first.starts_with("deals ") {
        return None;
    }
    let e1 = crate::oracle::effects::parse_sentence(&format!("~ {first}"), b)?;
    let e2 = crate::oracle::effects::parse_sentence(&format!("~ {verb}{second}"), b)?;
    if !matches!(e2, Effect::Modify { what: Sel::This, .. }) {
        return None;
    }
    Some(Effect::seq(vec![e1, e2]))
}

inventory::submit! { EffectPattern { name: "grants: ~ deals ... and gains ...", priority: 220, parse: shared_object_subject } }

// ---------------------------------------------------------------------------
// Durations of grants
// ---------------------------------------------------------------------------

/// "For as long as that creature has a bounty counter on it, it has \"When this
/// creature dies, ...\"" (Mathas, Fiend Seeker; Makeshift Mannequin; Obsidian Fireheart;
/// Ultima): the effect lasts while the object it applies to has a counter of that kind
/// (CR 611.2b); once the counter is gone, it ends for good.
fn while_it_has_counter(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let r = l.strip_prefix("for as long as ")?;
    let (cond, clause) = r.split_once(", ")?;
    let (subject, kind) = cond.split_once(" has a ")?;
    let kind = kind.strip_suffix(" counter on it")?;
    if kind.contains(' ') {
        return None;
    }
    let (sel, rest) = crate::oracle::effects::object_ref(subject, b)?;
    if !rest.trim().is_empty() || !matches!(sel, Sel::Target(_) | Sel::Var(_)) {
        return None;
    }
    // A one-shot effect giving an object an ability: "it has" is "it gains".
    let clause = match clause.strip_prefix("it has ") {
        Some(x) => format!("it gains {x}"),
        None => clause.to_string(),
    };
    let Effect::Modify {
        what,
        mods,
        duration: Duration::Permanent,
    } = crate::oracle::effects::parse_sentence(&clause, b)?
    else {
        return None;
    };
    if format!("{what:?}") != format!("{sel:?}") {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods,
        duration: Duration::WhileCondition(Condition::SelMatches(
            Sel::Var(vars::AFFECTED),
            Filter::HasCounter(Some(kind.into())),
        )),
    })
}

inventory::submit! { EffectPattern { name: "grants: for as long as it has a counter", priority: 120, parse: while_it_has_counter } }

/// "{2}, Exile ~ from your hand: Target land gains \"{T}: Add {B}, {R}, or {G}\" until ~
/// is cast from exile." (Masked Bandits and its cycle): the effect lasts until the card
/// (followed through its zone changes, CR 400.7) is cast from exile (CR 601.2a); if it
/// never is, the effect never ends.
fn until_cast_from_exile(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let r = l.strip_suffix(" until ~ is cast from exile")?;
    let Effect::Modify {
        what,
        mods,
        duration: Duration::Permanent,
    } = crate::oracle::effects::parse_sentence(r, b)?
    else {
        return None;
    };
    Some(Effect::Modify {
        what,
        mods,
        duration: Duration::WhileCondition(Condition::Not(Box::new(Condition::Custom(
            super::grant_conditions::SOURCE_CAST_FROM_EXILE.into(),
        )))),
    })
}

inventory::submit! { EffectPattern { name: "grants: until ~ is cast from exile", priority: 120, parse: until_cast_from_exile } }

/// "You may cast ~ for as long as it remains exiled." after an activation cost "Exile ~
/// from your hand" (Masked Bandits and its cycle): a permission to cast the card the cost
/// exiled (the new object it became, CR 400.7j), which ends when it leaves exile.
fn may_cast_self_while_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if crate::oracle::phrases::end(l) != "you may cast ~ for as long as it remains exiled" {
        return false;
    }
    const CARD: Var = vars::USER + 1773;
    let card = Sel::Var(CARD);
    let grant = Effect::ForEach {
        sel: Sel::Var(crate::zones::COST_MOVED),
        var: CARD,
        effect: Box::new(Effect::If {
            cond: Condition::SelMatches(card.clone(), Filter::InZone(ZoneKind::Exile)),
            then: Box::new(Effect::WithPlayTerms {
                terms: PlayTerms {
                    spells_only: true,
                    ..Default::default()
                },
                effect: Box::new(Effect::GrantPlayPermission {
                    who: PlayerRef::You,
                    what: card,
                    duration: Duration::Permanent,
                    free: false,
                }),
            }),
            otherwise: Box::new(Effect::Noop),
        }),
    };
    *prev = Effect::seq(vec![std::mem::take(prev), grant]);
    true
}

inventory::submit! { FollowupPattern { name: "grants: you may cast ~ for as long as it remains exiled", priority: 120, apply: may_cast_self_while_exiled } }

/// "Until end of turn, target creature gets +3/+3, up to one other target creature gets
/// +2/+2, and up to one other target creature gets +1/+1." (Arm the Cathars), "Until end
/// of turn, double target creature's power and it gains first strike." (Legion
/// Leadership): a leading duration over a list of clauses, each with its own subject;
/// each clause is read as its own sentence with that duration.
fn leading_duration_clauses(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let (suffix, body) = [
        ("until end of turn, ", " until end of turn"),
        ("until your next turn, ", " until your next turn"),
    ]
    .into_iter()
    .find_map(|(p, s)| l.strip_prefix(p).map(|r| (s, r)))?;
    if body.contains('"') {
        return None;
    }
    const SUBJECTS: &[&str] = &["it ", "target ", "up to ", "another target ", "~ "];
    let mut cuts = vec![];
    for sep in [", and ", ", ", " and "] {
        for (i, _) in body.match_indices(sep) {
            let next = &body[i + sep.len()..];
            if SUBJECTS.iter().any(|s| next.starts_with(s)) {
                cuts.push((i, i + sep.len()));
            }
        }
    }
    cuts.sort();
    cuts.dedup_by_key(|c| c.0);
    let mut clauses = Vec::new();
    let mut from = 0;
    let mut last_end = 0;
    for (start, next) in cuts {
        if start < last_end {
            continue;
        }
        clauses.push(&body[from..start]);
        from = next;
        last_end = next;
    }
    clauses.push(&body[from..]);
    if clauses.len() < 2 {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut out = Vec::new();
    for c in clauses {
        match crate::oracle::effects::parse_sentence(&format!("{c}{suffix}"), b) {
            Some(e) => out.push(e),
            None => {
                b.targets.truncate(saved.0);
                b.it = saved.1;
                return None;
            }
        }
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "grants: leading duration over several clauses", priority: 95, parse: leading_duration_clauses } }
