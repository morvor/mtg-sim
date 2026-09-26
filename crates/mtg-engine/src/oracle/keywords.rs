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
        // CR 702.33h: sticker kicker is a kicker ability (marked by its text).
        v.push(("sticker kicker".into(), KeywordKind::Kicker));
        // CR 702.37b: megamorph is a variant of morph (marked by its text).
        v.push(("megamorph".into(), KeywordKind::Morph));
        v.push(("typecycling".into(), KeywordKind::Cycling));
        v.push(("landcycling".into(), KeywordKind::Cycling));
        v.push(("partner with".into(), KeywordKind::Partner));
        v.push(("bands with other".into(), KeywordKind::Banding));
        // CR 702.19c: a variant of trample, marked by its text.
        v.push((
            crate::kw::trample::OVER_PLANESWALKERS.into(),
            KeywordKind::Trample,
        ));
        v.push(("hexproof from".into(), KeywordKind::Hexproof));
        // CR 702.89b: older cards printed "totem armor"; it's now umbra armor.
        v.push(("totem armor".into(), KeywordKind::UmbraArmor));
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
        let part = part.trim();
        for kw in parse_one_keyword(part, ctx)? {
            out.extend(compile_keyword(kw, part));
        }
    }
    Some(out)
}

/// Splits "Flying, first strike" but not "Ward—Pay 3 life, then ..." or costs with commas.
/// The qualities of a protection or hexproof ability stay together: "Protection from
/// blue, from black, and from red", "Hexproof from artifacts, creatures, and
/// enchantments".
fn split_keyword_list(t: &str) -> Vec<String> {
    // Keyword lines with a cost after an em dash may contain commas in the cost.
    if t.contains('—') {
        return vec![t.to_string()];
    }
    let mut raw = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, ch) in t.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' | ';' if depth == 0 => {
                raw.push(&t[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    raw.push(&t[start..]);
    merge_quality_fragments(raw.into_iter().map(str::trim), ", ")
}

/// Rejoins list fragments that continue the qualities of a preceding "protection from" /
/// "hexproof from" phrase ("from black", "and from red", "and enchantments") to it,
/// using `sep` (the separator they were split on).
fn merge_quality_fragments<'a>(parts: impl Iterator<Item = &'a str>, sep: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in parts {
        let lower = p.to_lowercase();
        let continues = out.last().is_some_and(|prev| {
            let prev = prev.to_lowercase();
            (prev.starts_with("protection from") || prev.starts_with("hexproof from"))
                && (lower.starts_with("from ")
                    || lower.starts_with("and from ")
                    || (!lower.is_empty() && !is_keyword_phrase(&lower)))
        });
        match out.last_mut() {
            Some(prev) if continues => {
                prev.push_str(sep);
                prev.push_str(p);
            }
            _ => out.push(p.to_string()),
        }
    }
    out
}

/// Whether a fragment of a keyword list starts a keyword of its own.
fn is_keyword_phrase(lower: &str) -> bool {
    let lower = lower.trim();
    lower.ends_with("walk")
        || names().iter().any(|(n, _)| {
            lower.starts_with(n.as_str())
                && lower[n.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric())
        })
}

/// Splits a list of keywords granted by an effect ("flying and trample", "first strike,
/// vigilance, and lifelink", "flying and protection from black and from red") into one
/// phrase per keyword.
pub fn split_keyword_phrases(s: &str) -> Vec<String> {
    let s = s.trim().trim_end_matches('.');
    let mut parts: Vec<&str> = Vec::new();
    for a in s.split(", and ") {
        for b in a.split(" and ") {
            for c in b.split(", ") {
                parts.push(c.trim());
            }
        }
    }
    // Rejoining "from" fragments with " and " keeps "protection from black and from red"
    // (and comma lists, whose "from ..." parts are equivalent) together.
    merge_quality_fragments(parts.into_iter().filter(|p| !p.is_empty()), " and ")
}

fn parse_one_keyword(part: &str, ctx: &CompileContext) -> Option<Vec<Keyword>> {
    let lower = part.to_lowercase();
    // Landwalk variants (CR 702.14a): "islandwalk", "nonbasic landwalk", "legendary
    // landwalk", "snow swampwalk".
    if let Some(stem) = lower.strip_suffix("walk") {
        let snow_type = stem
            .strip_prefix("snow ")
            .is_some_and(|t| !t.is_empty() && !t.contains(' '));
        if !stem.is_empty() && !stem.contains(' ') || stem.ends_with("land") || snow_type {
            let filter = landwalk_filter(stem)?;
            return Some(vec![Keyword {
                filter: Some(filter),
                text: Some(SmolStr::new(part)),
                ..Keyword::new(KeywordKind::Landwalk)
            }]);
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
            } else if ty == "artifact land" {
                Filter::and(vec![
                    Filter::Type(CardType::Artifact),
                    Filter::Type(CardType::Land),
                ])
            } else {
                Filter::Subtype(subtype_word(ty)?)
            };
            return Some(vec![Keyword {
                cost: Some(cost),
                filter: Some(filter),
                text: Some(SmolStr::new(part)),
                ..Keyword::new(KeywordKind::Cycling)
            }]);
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
        // CR 702.16g–i, 702.11f–g: "from A and from B" and "from each [characteristic]"
        // are shorthand for separate abilities, one per quality.
        KeywordKind::Protection | KeywordKind::Hexproof => {
            let qualities = if name.as_str() == "hexproof from" {
                protection_qualities(rest)?
            } else if let Some(r) = rest.strip_prefix("from ") {
                protection_qualities(r)?
            } else if rest.is_empty() && *kind == KeywordKind::Hexproof {
                return Some(vec![kw]);
            } else {
                return None;
            };
            return Some(
                qualities
                    .into_iter()
                    .map(|f| Keyword {
                        filter: Some(f),
                        ..kw.clone()
                    })
                    .collect(),
            );
        }
        // "landwalk of the chosen type" (a land type chosen as the source entered).
        KeywordKind::Landwalk if rest == "of the chosen type" => {
            kw.filter = Some(Filter::ChosenType);
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
        // CR 702.22b: "bands with other [quality]" is banding with a quality.
        KeywordKind::Banding if name.as_str() == "bands with other" => {
            kw.filter = Some(crate::kw::banding::quality_filter(rest, rest_raw)?);
        }
        // CR 702.77a: "Reinforce X—{X}{G}{G}" puts X counters, X paid in the cost (N is
        // -1, see `kw/reinforce.rs`).
        KeywordKind::Reinforce if rest.starts_with("x—") => {
            let cost = parse_keyword_cost(&rest_raw[rest_raw.find('—')?..])?;
            if !cost.mana.as_ref().is_some_and(|m| m.has_x()) {
                return None;
            }
            kw.cost = Some(cost);
            kw.n = Some(-1);
        }
        // CR 702.33b: "Kicker [cost 1] and/or [cost 2]" means "Kicker [cost 1], kicker
        // [cost 2]": the second cost is kept in `costs`.
        KeywordKind::Kicker if rest_raw.contains(" and/or ") => {
            let (a, b) = rest_raw.split_once(" and/or ")?;
            kw.cost = Some(parse_keyword_cost(a)?);
            kw.costs = vec![parse_keyword_cost(b)?];
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
    Some(vec![kw])
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

/// "from red", "from everything", "from creatures", "from each color", "from multicolored"
/// as a single filter (the union of its qualities).
pub fn protection_filter(s: &str) -> Option<Filter> {
    let mut fs = protection_qualities(s)?;
    Some(if fs.len() == 1 {
        fs.pop().unwrap()
    } else {
        Filter::Or(fs)
    })
}

/// The qualities of a protection or hexproof ability, one per separate ability (the text
/// after "from"): "red" → [red]; "black and from red" and "blue, from black, and from
/// red" → one per color (CR 702.16g, 702.11f); "each color" → one per color (CR 702.16h,
/// 702.11g); "artifacts and enchantments" → [artifact or enchantment].
pub fn protection_qualities(s: &str) -> Option<Vec<Filter>> {
    let s = s.trim().trim_end_matches('.');
    let s = s.strip_prefix("from ").unwrap_or(s);
    let mut out = Vec::new();
    for part in s
        .split(", and from ")
        .flat_map(|p| p.split(", from "))
        .flat_map(|p| p.split(" and from "))
    {
        let part = part.trim();
        // CR 702.16h, 702.11g: "each color" stands for one ability per color.
        if part == "each color" || part == "all colors" {
            out.extend(Color::ALL.iter().map(|c| Filter::Color(*c)));
            continue;
        }
        out.push(single_quality(part)?);
    }
    (!out.is_empty()).then_some(out)
}

/// One quality ("red", "creatures", "artifacts, creatures, and enchantments", "mana
/// value 3 or greater").
fn single_quality(s: &str) -> Option<Filter> {
    let s = s.trim();
    match s {
        // CR 702.16j
        "everything" => return Some(Filter::Any),
        "multicolored" => return Some(Filter::Multicolored),
        "monocolored" => return Some(Filter::Monocolored),
        "colorless" => return Some(Filter::Colorless),
        // CR 607.2d: "protection from the chosen color" (linked to "choose a color").
        "the chosen color" => return Some(Filter::ChosenColor),
        // CR 702.16k: protection from a player is protection from each object that
        // player controls (or owns, outside the battlefield and stack).
        "the chosen player" => return Some(Filter::ControlledBy(PlayerRel::Chosen)),
        // CR 702.16a: a quality is a card name only if the ability says it's a name.
        "the chosen card name" | "the chosen name" => return Some(Filter::ChosenName),
        "each of your opponents" | "your opponents" => {
            return Some(Filter::ControlledBy(PlayerRel::Opponent))
        }
        // CR 702.16a: a supertype quality applies to sources with that supertype.
        "snow" => return Some(Filter::Supertype(Supertype::Snow)),
        "spells that are one or more colors" => {
            return Some(Filter::and(vec![
                Filter::Spell,
                Filter::not(Filter::Colorless),
            ]))
        }
        _ => {}
    }
    if let Some(r) = s.strip_prefix("mana value ") {
        let (n, cmp) = if let Some(n) = r.strip_suffix(" or greater") {
            (n, Cmp::Ge)
        } else if let Some(n) = r.strip_suffix(" or less") {
            (n, Cmp::Le)
        } else {
            (r, Cmp::Eq)
        };
        let n: i32 = n.trim().parse().ok()?;
        return Some(Filter::ManaValue(cmp, Box::new(Value::Const(n))));
    }
    // "red and white", "artifacts and enchantments", "artifacts, creatures, and
    // enchantments": a union of qualities.
    let parts: Vec<&str> = s
        .split(", and ")
        .flat_map(|p| p.split(" and "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let mut fs = Vec::new();
    for p in parts {
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
    Some(match fs.len() {
        0 => return None,
        1 => fs.pop().unwrap(),
        _ => Filter::Or(fs),
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
        // CR 702.14c: "snow swampwalk" — both the supertype and the subtype.
        other if other.starts_with("snow ") => Filter::and(vec![
            Filter::Supertype(Supertype::Snow),
            landwalk_filter(&other["snow ".len()..])?,
        ]),
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
    if kw.kind == KeywordKind::Changeling {
        // CR 702.73a: this object is every creature type (CDA, layer 4).
        out.push(AbilityDef::new(
            AbilityKind::Static(crate::kw::changeling::changeling_cda()),
            "Changeling (every creature type)",
        ));
    }
    out
}
