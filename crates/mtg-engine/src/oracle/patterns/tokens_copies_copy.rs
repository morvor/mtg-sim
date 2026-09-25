//! Token copies with exceptions (CR 707.9) and in other forms (CR 111.10, 707.2):
//!
//! ```text
//! [you] create COUNT [tapped [and attacking]] token(s) that's a copy of / that are
//!     copies of OBJECT [, except EXCEPTION ([,] and EXCEPTION)*] [and that's tapped and
//!     attacking]
//! EXCEPTION := it isn't legendary | it has ABILITIES | it's N/N | it's a N/N COLORS SUBTYPES
//!            | it's a [N/N] TYPES in addition to its other types | its name is NAME
//! ```
//!
//! The exceptions become part of the token's copiable values (CR 707.9a, 707.9b); an
//! exception that provides specific values for a characteristic also keeps the copied
//! object's characteristic-defining abilities for it from being copied (CR 707.9d, see
//! `copy::drop_overridden_cdas`).

use super::tokens_copies_create::ability_list;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;
use smol_str::SmolStr;

/// The subjects an exception clause can start with ("it", "the token", "they").
const SUBJECTS: [&str; 12] = [
    "it's ",
    "it isn't ",
    "it is ",
    "it has ",
    "its ",
    "they're ",
    "they aren't ",
    "they have ",
    "their ",
    "the token ",
    "the tokens ",
    "each of them ",
];

/// Splits "it isn't legendary and it has haste" into its clauses (quotes masked).
fn exception_clauses(masked: &str) -> Vec<String> {
    let mut s = masked.to_string();
    for subj in SUBJECTS {
        for sep in [", and ", " and ", ", "] {
            s = s.replace(&format!("{sep}{subj}"), &format!("|{subj}"));
        }
    }
    s.split('|').map(|x| x.trim().to_string()).collect()
}

/// "4/4" → (4, 4).
fn pt(w: &str) -> Option<(i32, i32)> {
    let (p, t) = w.split_once('/')?;
    Some((p.parse().ok()?, t.parse().ok()?))
}

/// "a 1/1 Fractal creature", "an artifact", "a Spirit" (in addition to its other types).
fn added_types(s: &str) -> Option<Vec<Modification>> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let mut out = Vec::new();
    let mut card_types = Vec::new();
    let mut subtypes = Vec::new();
    for (i, w) in s.split_whitespace().enumerate() {
        if i == 0 {
            if let Some((p, t)) = pt(w) {
                out.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
                continue;
            }
        }
        if let Some(t) = CardType::from_word(w) {
            card_types.push(t);
        } else {
            let sub = subtype_word(w)?;
            subtypes.push(sub);
        }
    }
    if card_types.is_empty() && subtypes.is_empty() {
        return None;
    }
    if !card_types.is_empty() {
        out.push(Modification::AddTypes(card_types));
    }
    if !subtypes.is_empty() {
        out.push(Modification::AddSubtypes(subtypes));
    }
    Some(out)
}

/// "a 4/4 black Zombie", "a 1/1 green Frog": power and toughness, colors, and creature
/// types instead of the copied ones.
fn replaced_characteristics(s: &str) -> Option<Vec<Modification>> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let mut words = s.split_whitespace();
    let (p, t) = pt(words.next()?)?;
    let mut out = vec![Modification::SetPT(Some(Value::c(p)), Some(Value::c(t)))];
    let mut colors = ColorSet::NONE;
    let mut subtypes = Vec::new();
    for w in words {
        if w == "and" && subtypes.is_empty() {
            continue;
        }
        if let Some(c) = Color::from_word(w) {
            if !subtypes.is_empty() {
                return None;
            }
            colors.insert(c);
        } else {
            let sub = subtype_word(w)?;
            if subtype_kind(sub.as_str()) != Some(SubtypeKind::Creature) {
                return None;
            }
            subtypes.push(sub);
        }
    }
    if !colors.is_colorless() {
        out.push(Modification::SetColors(colors));
    }
    if !subtypes.is_empty() {
        out.push(Modification::RemoveAllCreatureTypes);
        out.push(Modification::AddSubtypes(subtypes));
    }
    Some(out)
}

/// The original-case name after "its name is " (lowercase `name`).
fn original_name(name: &str) -> Option<String> {
    let raw = crate::oracle::raw_text();
    let rl = raw.to_lowercase();
    if rl.len() != raw.len() {
        return None;
    }
    let needle = format!("name is {name}");
    let i = rl.find(&needle)? + "name is ".len();
    raw.get(i..i + name.len()).map(str::to_string)
}

/// Parses the exceptions of a copy effect ("it isn't legendary and it has haste").
pub(crate) fn copy_exceptions(
    masked: &str,
    quotes: &[String],
    ctx: &CompileContext,
) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    for c in exception_clauses(masked) {
        let c = c.as_str();
        if matches!(
            c,
            "it isn't legendary"
                | "it's not legendary"
                | "it is not legendary"
                | "they're not legendary"
                | "they aren't legendary"
                | "the token isn't legendary"
                | "the token is not legendary"
                | "the tokens aren't legendary"
                | "the tokens are not legendary"
        ) {
            out.push(Modification::RemoveSupertypes(vec![Supertype::Legendary]));
        } else if let Some(r) = ["it has ", "they have ", "the token has ", "the tokens have ", "each of them has "]
            .iter()
            .find_map(|p| c.strip_prefix(p))
        {
            if r.starts_with("this ability") || r.contains(" this ability") {
                return None;
            }
            for a in ability_list(r, quotes, &[CardType::Creature], ctx)? {
                match &a.kind {
                    AbilityKind::Keyword(k) => out.push(Modification::AddKeyword(k.clone())),
                    _ => out.push(Modification::AddAbility(a)),
                }
            }
        } else if let Some(r) = ["it's ", "it is ", "they're ", "the token is ", "the tokens are "]
            .iter()
            .find_map(|p| c.strip_prefix(p))
        {
            if let Some(types) = r
                .strip_suffix(" in addition to its other types")
                .or_else(|| r.strip_suffix(" in addition to their other types"))
            {
                out.extend(added_types(types)?);
            } else if let Some((p, t)) = pt(r) {
                out.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
            } else {
                out.extend(replaced_characteristics(r)?);
            }
        } else if let Some(n) = c.strip_prefix("its name is ") {
            if n.is_empty() || n.contains('~') || n.contains('"') || n.split(' ').count() > 4 {
                return None;
            }
            out.push(Modification::SetName(SmolStr::new(original_name(n)?)));
        } else {
            return None;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// The object a token copies: "~", "it", "that creature", "enchanted creature", a target.
fn copied_object(r: &str, b: &mut Builder) -> Option<(Sel, String)> {
    if let Some(rest) = r.strip_prefix('~') {
        return Some((Sel::This, rest.to_string()));
    }
    for p in [
        "enchanted creature",
        "equipped creature",
        "enchanted artifact",
        "enchanted permanent",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((Sel::AttachedTo, rest.to_string()));
        }
    }
    for p in [
        "it",
        "that creature",
        "that permanent",
        "that card",
        "that artifact",
        "that token",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with(',') {
                return Some((b.it.clone(), rest.to_string()));
            }
        }
    }
    let (spec, rest) = parse_target(r)?;
    let text = r[..r.len() - rest.len()].trim().to_string();
    let slot = b.add_target(spec, &text);
    Some((Sel::Target(slot), rest.to_string()))
}

/// "create a token that's a copy of target creature you control, except it isn't
/// legendary", "create a tapped and attacking token that's a copy of it", "create two
/// tokens that are copies of ~, except they're not legendary".
fn token_copy_with_exceptions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you create ")
        .or_else(|| l.strip_prefix("create "))?;
    let (count, r) = parse_number(r)?;
    let r = r.trim_start();
    let (mut tapped, mut attacking, r) = if let Some(x) = r.strip_prefix("tapped and attacking ") {
        (true, true, x)
    } else if let Some(x) = r.strip_prefix("tapped ") {
        (true, false, x)
    } else {
        (false, false, r)
    };
    let r = r
        .strip_prefix("token that's a copy of ")
        .or_else(|| r.strip_prefix("tokens that are copies of "))?;
    let (masked, quotes) = super::statics::mask_quotes(r)?;
    let (obj_part, exc_part) = match masked.split_once(", except ") {
        Some((a, e)) => (a.to_string(), Some(e.to_string())),
        None => (masked.clone(), None),
    };
    // "... and that's tapped and attacking" (before or after the exceptions).
    let strip_attacking = |s: &str| -> Option<String> {
        [
            " and that's tapped and attacking",
            " and that are tapped and attacking",
            " that's tapped and attacking",
            " that are tapped and attacking",
        ]
        .iter()
        .find_map(|p| s.strip_suffix(p))
        .map(str::to_string)
    };
    let (obj_part, exc_part) = match (strip_attacking(&obj_part), &exc_part) {
        (Some(o), None) => {
            tapped = true;
            attacking = true;
            (o, None)
        }
        (_, Some(e)) => match strip_attacking(e) {
            Some(e2) => {
                tapped = true;
                attacking = true;
                (obj_part, Some(e2))
            }
            None => (obj_part, exc_part.clone()),
        },
        (None, None) => (obj_part, None),
    };
    if obj_part.contains('"') {
        return None;
    }
    let (of, tail) = copied_object(&obj_part, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let mods = match exc_part {
        Some(e) => copy_exceptions(&e, &quotes, b.ctx)?,
        None => {
            if !quotes.is_empty() {
                return None;
            }
            vec![]
        }
    };
    Some(Effect::CreateTokenCopy {
        of,
        count,
        controller: PlayerRef::You,
        tapped,
        attacking,
        mods,
    })
}

inventory::submit! { EffectPattern { name: "tokens_copies: token copy with exceptions", priority: 105, parse: token_copy_with_exceptions } }
