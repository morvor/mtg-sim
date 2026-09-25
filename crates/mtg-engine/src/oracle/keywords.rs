//! Parsing keyword ability lines ("Flying, trample", "Ward {2}", "Protection from red",
//! "Enchant creature", "Equip {1}", "Bushido 2", "Islandwalk", "Swampcycling {2}").

use super::phrases::*;
use super::CompileContext;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::types::*;
use smol_str::SmolStr;

/// Keyword names sorted longest first (for prefix matching).
fn names() -> &'static [(String, KeywordKind)] {
    use std::sync::OnceLock;
    static N: OnceLock<Vec<(String, KeywordKind)>> = OnceLock::new();
    N.get_or_init(|| {
        let mut v: Vec<(String, KeywordKind)> = KeywordKind::ALL
            .iter()
            .map(|k| (k.name().to_lowercase(), *k))
            .collect();
        v.push(("multikicker".into(), KeywordKind::Kicker));
        v.push(("typecycling".into(), KeywordKind::Cycling));
        v.push(("landcycling".into(), KeywordKind::Cycling));
        v.push(("partner with".into(), KeywordKind::Partner));
        v.push(("bands with other".into(), KeywordKind::Banding));
        v.push(("hexproof from".into(), KeywordKind::Hexproof));
        v.sort_by_key(|(n, _)| std::cmp::Reverse(n.len()));
        v
    })
}

/// Parses a line consisting only of keywords. Returns None if any part isn't a keyword.
pub fn parse_keyword_line(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = text.trim().trim_end_matches('.');
    if t.is_empty() {
        return None;
    }
    let parts = split_keyword_list(t);
    let mut out = Vec::new();
    for part in parts {
        let kw = parse_one_keyword(part.trim(), ctx)?;
        out.extend(compile_keyword(kw, part.trim()));
    }
    Some(out)
}

/// Splits "Flying, first strike" but not "Ward—Pay 3 life, then ..." or costs with commas.
fn split_keyword_list(t: &str) -> Vec<&str> {
    // Keyword lines with a cost after an em dash may contain commas in the cost.
    if t.contains('—') {
        return vec![t];
    }
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, ch) in t.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' | ';' if depth == 0 => {
                out.push(&t[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&t[start..]);
    out
}

fn parse_one_keyword(part: &str, ctx: &CompileContext) -> Option<Keyword> {
    let lower = part.to_lowercase();
    // Landwalk variants: "islandwalk", "nonbasic landwalk", "legendary landwalk".
    if let Some(stem) = lower.strip_suffix("walk") {
        if !stem.is_empty() && !stem.contains(' ') || stem.ends_with("land") {
            let filter = landwalk_filter(stem)?;
            return Some(Keyword {
                filter: Some(filter),
                text: Some(SmolStr::new(part)),
                ..Keyword::new(KeywordKind::Landwalk)
            });
        }
    }
    // Typecycling: "swampcycling {2}", "basic landcycling {1}", "wizardcycling {3}".
    if let Some(idx) = lower.find("cycling") {
        if idx > 0 {
            let ty = lower[..idx].trim();
            let rest = lower[idx + "cycling".len()..].trim();
            let cost = parse_keyword_cost(rest)?;
            let filter = if ty == "basic land" {
                Filter::and(vec![
                    Filter::Supertype(Supertype::Basic),
                    Filter::Type(CardType::Land),
                ])
            } else if ty == "land" {
                Filter::Type(CardType::Land)
            } else {
                Filter::Subtype(subtype_word(ty)?)
            };
            return Some(Keyword {
                cost: Some(cost),
                filter: Some(filter),
                text: Some(SmolStr::new(part)),
                ..Keyword::new(KeywordKind::Cycling)
            });
        }
    }
    let (name, kind) = names().iter().find(|(n, _)| {
        lower.starts_with(n.as_str())
            && lower[n.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric())
    })?;
    let rest_raw = part[name.len()..].trim();
    let rest = lower[name.len()..].trim();
    let mut kw = Keyword::new(*kind);
    kw.text = Some(SmolStr::new(part));
    let _ = ctx;
    match kind {
        KeywordKind::Protection => {
            kw.filter = Some(protection_filter(rest.strip_prefix("from")?.trim())?);
        }
        KeywordKind::Hexproof => {
            if let Some(r) = rest.strip_prefix("from") {
                kw.filter = Some(protection_filter(r.trim())?);
            } else if !rest.is_empty() {
                return None;
            }
        }
        KeywordKind::Enchant => {
            // CR 702.5d: "Enchant player" / "Enchant opponent" Auras enchant players only;
            // they have no object filter (see `attach::enchant_player`).
            if rest != "player" && rest != "opponent" {
                kw.filter = Some(quality_phrase(rest)?);
            }
        }
        KeywordKind::Equip => {
            // "Equip {2}", "Equip legendary creature {3}", "Equip—Pay 3 life."
            if let Some(i) = rest.find('{') {
                let pre = rest[..i].trim();
                if !pre.is_empty() {
                    // CR 702.6c: "Equip [quality]" / "Equip [quality] creature".
                    kw.filter = Some(if pre == "commander" {
                        Filter::Commander
                    } else {
                        quality_phrase(pre)?
                    });
                }
                kw.cost = Some(parse_keyword_cost(&rest_raw[rest_raw.find('{')?..])?);
            } else {
                kw.cost = Some(parse_keyword_cost(rest_raw)?);
            }
        }
        KeywordKind::Affinity => {
            let r = rest.strip_prefix("for")?.trim();
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            kw.filter = Some(f);
        }
        KeywordKind::Partner if name.as_str() == "partner with" => {
            kw.text = Some(SmolStr::new(rest_raw));
        }
        KeywordKind::Banding if name.as_str() == "bands with other" => {
            kw.text = Some(SmolStr::new(rest_raw));
        }
        _ => {
            if rest.is_empty() {
                // plain keyword
            } else if let Ok(n) = rest.parse::<i32>() {
                kw.n = Some(n);
            } else if rest == "x" {
                kw.n = Some(-1);
            } else if let Some(c) = parse_keyword_cost(rest_raw) {
                kw.cost = Some(c);
            } else if let Some((c, n)) = cost_then_number(rest_raw) {
                kw.cost = Some(c);
                kw.n = Some(n);
            } else {
                return None;
            }
        }
    }
    Some(kw)
}

/// The object phrase of "Enchant [quality]" / "Equip [quality]": "creature you control",
/// "artifact, creature, or planeswalker", "red or green creature" (adjectives joined by
/// "or" before a shared noun).
pub fn quality_phrase(s: &str) -> Option<Filter> {
    if let Some((f, _, tail)) = parse_object_phrase(s) {
        if end(tail).is_empty() {
            return Some(f);
        }
    }
    // "red or green creature" = "red creature or green creature".
    let (first, rest) = s.split_once(" or ")?;
    let (second, noun) = rest.split_once(' ')?;
    if first.contains(' ') {
        return None;
    }
    let mut fs = Vec::new();
    for adj in [first, second] {
        let phrase = format!("{adj} {noun}");
        let (f, _, tail) = parse_object_phrase(&phrase)?;
        if !end(tail).is_empty() {
            return None;
        }
        fs.push(f);
    }
    Some(Filter::Or(fs))
}

fn cost_then_number(s: &str) -> Option<(Cost, i32)> {
    // e.g. "Suspend 4—{1}{U}" → (cost, 4)
    let (n, c) = s.split_once('—')?;
    Some((parse_keyword_cost(c)?, n.trim().parse().ok()?))
}

/// Parses a keyword's cost: "{2}{R}", "—Sacrifice a creature.", "{1}—Pay 2 life".
pub fn parse_keyword_cost(s: &str) -> Option<Cost> {
    let s = s.trim().trim_end_matches('.');
    let s = s.strip_prefix('—').unwrap_or(s).trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('{')
        && s.chars()
            .all(|c| "{}0123456789WUBRGCSXPHwubrgcsxph/½∞".contains(c))
    {
        return Some(Cost::mana(ManaCost::parse(s)?));
    }
    super::costs::parse_cost(&s.replace('—', ", ")).map(|(c, _)| c)
}

/// "from red", "from everything", "from creatures", "from each color", "from multicolored".
pub fn protection_filter(s: &str) -> Option<Filter> {
    let s = s.trim().trim_end_matches('.');
    if s == "everything" {
        return Some(Filter::Any);
    }
    if s == "each color" || s == "all colors" {
        return Some(Filter::not(Filter::Colorless));
    }
    if s == "multicolored" {
        return Some(Filter::Multicolored);
    }
    if s == "monocolored" {
        return Some(Filter::Monocolored);
    }
    if s == "colorless" {
        return Some(Filter::Colorless);
    }
    // CR 607.2d: "protection from the chosen color" (linked to "choose a color").
    if s == "the chosen color" {
        return Some(Filter::ChosenColor);
    }
    // CR 702.16k: protection from a player is protection from each object that player
    // controls (or owns, outside the battlefield and stack).
    if s == "the chosen player" {
        return Some(Filter::ControlledBy(PlayerRel::Chosen));
    }
    // "red and from white" / "red and white" / "white and from blue"
    let parts: Vec<&str> = s
        .split(" and from ")
        .flat_map(|p| p.split(" and "))
        .collect();
    let mut fs = Vec::new();
    for p in parts {
        let p = p.trim();
        if let Some(c) = Color::from_word(p) {
            fs.push(Filter::Color(c));
        } else if let Some((f, _, tail)) = parse_object_phrase(p) {
            if !end(tail).is_empty() {
                return None;
            }
            fs.push(f);
        } else {
            return None;
        }
    }
    Some(if fs.len() == 1 {
        fs.pop().unwrap()
    } else {
        Filter::Or(fs)
    })
}

fn landwalk_filter(stem: &str) -> Option<Filter> {
    let stem = stem.trim();
    Some(match stem {
        "nonbasic land" => Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::not(Filter::Supertype(Supertype::Basic)),
        ]),
        "legendary land" => Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Supertype(Supertype::Legendary),
        ]),
        "snow land" => Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Supertype(Supertype::Snow),
        ]),
        "artifact land" => Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Type(CardType::Artifact),
        ]),
        "desert" | "island" | "swamp" | "mountain" | "forest" | "plains" => {
            Filter::Subtype(subtype_word(stem)?)
        }
        other => Filter::Subtype(subtype_word(other)?),
    })
}

/// Turns a keyword into its ability entries. CDA keywords (changeling, devoid) also get
/// the characteristic-defining static they imply.
pub fn compile_keyword(kw: Keyword, text: &str) -> Vec<Ability> {
    let mut out = vec![AbilityDef::new(AbilityKind::Keyword(kw.clone()), text)];
    if kw.kind == KeywordKind::Devoid {
        // CR 702.114a: this object is colorless (CDA, layer 5).
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::SetColors(ColorSet::NONE)],
        });
        s.is_cda = true;
        s.zone = FunctionZone::Anywhere;
        out.push(AbilityDef::new(
            AbilityKind::Static(s),
            "Devoid (colorless)",
        ));
    }
    out
}
