//! Comparing a rendering with the card's Oracle text.
//!
//! Both sides go through the same normalization ([`normalize_unit`]): reminder text
//! removed, the card's own name and "this creature/permanent/spell" shorthand replaced by
//! `~`, case, punctuation, whitespace and numbers in words vs digits made uniform.
//!
//! Wordings that differ but mean the same thing under the Comprehensive Rules are listed
//! in ONE place, [`EQUIVALENCES`] (string rewrites applied to both sides, each with its
//! justification) and [`IGNORED_WORDS`]. Keep the list small: an equivalence must never
//! hide a real difference in meaning.

use super::{render_card, RenderedFace};
use crate::card::CardDef;
use crate::keywords::KeywordKind;
use regex::Regex;
use std::sync::OnceLock;

/// An allowed equivalence between two wordings: every match of `pattern` (a regex over
/// the lowercased, reminder-free text with `~` for self-references) is replaced by
/// `replacement` on both sides.
pub struct Equivalence {
    pub pattern: &'static str,
    pub replacement: &'static str,
    /// Why the two wordings mean the same thing.
    pub why: &'static str,
}

/// The allowed equivalences, applied in order.
pub const EQUIVALENCES: &[Equivalence] = &[
    Equivalence {
        pattern: r"\bwhenever\b",
        replacement: "when",
        why: "\"When\" and \"whenever\" both introduce a trigger condition (CR 603.1); the \
              choice of word has no rules meaning.",
    },
    Equivalence {
        pattern: r"\bis put into a graveyard from the battlefield\b",
        replacement: "dies",
        why: "CR 700.4: \"dies\" means \"is put into a graveyard from the battlefield\".",
    },
    Equivalence {
        pattern: r"\bare put into a graveyard from the battlefield\b",
        replacement: "die",
        why: "CR 700.4 (plural).",
    },
    Equivalence {
        pattern: r"\b(to|into|on top of|on the bottom of|onto) (your|its owner's|their owners'|their owner's|its owners') (hand|library|graveyard)",
        replacement: "$1 owner's $3",
        why: "A card always goes to its owner's hand, library, or graveyard (CR 400.3); \
              \"your hand\" on a card you own is its owner's hand.",
    },
    Equivalence {
        pattern: r"\bthat (creature|permanent|card|spell|land|artifact|enchantment|planeswalker|token|aura|equipment|vehicle|battle|ability|object|source)s?'s\b",
        replacement: "its",
        why: "Anaphora: \"that creature's\" and \"its\" both refer back to the object the \
              text already named; the renderer always uses the pronoun.",
    },
    Equivalence {
        pattern: r"\b(that|the) (creature|permanent|card|spell|land|artifact|enchantment|planeswalker|token|aura|equipment|vehicle|battle|ability|object|source)\b",
        replacement: "it",
        why: "Anaphora: \"that creature\" and \"it\" refer back to the object already named.",
    },
    Equivalence {
        pattern: r"\b(those|the) (creatures|permanents|cards|spells|lands|artifacts|tokens|objects)\b",
        replacement: "them",
        why: "Anaphora (plural).",
    },
    Equivalence {
        pattern: r"\bthey\b",
        replacement: "them",
        why: "Pronoun case: \"they\"/\"them\" refer to the same objects.",
    },
    Equivalence {
        pattern: r"\btheir\b",
        replacement: "its",
        why: "Pronoun number: \"their\" and \"its\" (grammatical number is ignored, see \
              singularization).",
    },
];

/// Words dropped from both sides before comparing.
pub const IGNORED_WORDS: &[(&str, &str)] = &[
    (
        "and",
        "Joins clauses and list items; instructions are followed in order (CR 608.2c) \
         whether they're joined by \"and\", \"then\", or a period. A list of alternatives \
         still says \"or\".",
    ),
    (
        "then",
        "Sequencing word: CR 608.2c, instructions are followed in the order written.",
    ),
    (
        "each",
        "Universal quantification is written \"each X\", \"all Xs\", or a bare plural \
         (\"creatures you control get +1/+1\"); all mean every object that matches.",
    ),
    ("all", "See \"each\"."),
    (
        "a",
        "Indefinite article: \"a\"/\"an\" vs none (\"put a +1/+1 counter\" vs \"put \
         +1/+1 counters\"); counts are always explicit (one, two, X).",
    ),
];

/// Self-reference shorthand used on cards (CR 201.5a).
const SELF_REFS: &[&str] = &[
    "creature",
    "artifact",
    "enchantment",
    "land",
    "permanent",
    "spell",
    "card",
    "aura",
    "equipment",
    "vehicle",
    "token",
    "planeswalker",
    "saga",
    "battle",
    "class",
    "case",
    "room",
    "fortification",
    "spacecraft",
    "siege",
    "mount",
    "object",
    "scheme",
    "plane",
    "phenomenon",
    "conspiracy",
    "attraction",
    "contraption",
];

/// Removes parenthesized reminder text.
pub fn strip_reminder(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Replaces the card's names and "this [object]" with `~`.
pub fn self_refs(text: &str, names: &[String]) -> String {
    let mut s = text.to_string();
    let mut names: Vec<&String> = names.iter().filter(|n| !n.is_empty()).collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for n in names {
        s = s.replace(n.as_str(), "~");
    }
    static RE: OnceLock<Regex> = OnceLock::new();
    let re =
        RE.get_or_init(|| Regex::new(&format!(r"(?i)\bthis ({})\b", SELF_REFS.join("|"))).unwrap());
    re.replace_all(&s, "~").to_string()
}

/// The names a card's text may use for itself.
pub fn self_names(def: &CardDef, face: usize) -> Vec<String> {
    let mut v = vec![def.name.to_string()];
    let fname = def.faces[face].chars.name.to_string();
    v.push(fname.clone());
    let legendary = def.faces[face]
        .chars
        .supertypes
        .contains(crate::types::Supertype::Legendary);
    if legendary {
        for n in [&fname] {
            if let Some((short, _)) = n.split_once(',') {
                v.push(short.to_string());
            }
            if let Some((short, _)) = n.split_once(" of ") {
                v.push(short.to_string());
            }
            if let Some((short, _)) = n.split_once(" the ") {
                v.push(short.to_string());
            }
            let words: Vec<&str> = n.split(' ').collect();
            if words.len() > 1 && words[0].len() >= 3 && words[0] != "The" {
                v.push(words[0].to_string());
            }
        }
    }
    v
}

fn number_words() -> &'static [(&'static str, &'static str)] {
    &[
        ("zero", "0"),
        ("one", "1"),
        ("two", "2"),
        ("three", "3"),
        ("four", "4"),
        ("five", "5"),
        ("six", "6"),
        ("seven", "7"),
        ("eight", "8"),
        ("nine", "9"),
        ("ten", "10"),
        ("eleven", "11"),
        ("twelve", "12"),
        ("thirteen", "13"),
        ("fourteen", "14"),
        ("fifteen", "15"),
        ("sixteen", "16"),
        ("seventeen", "17"),
        ("eighteen", "18"),
        ("nineteen", "19"),
        ("twenty", "20"),
        ("once", "1 time"),
        ("twice", "2 times"),
    ]
}

/// Singular form of a token (grammatical number is ignored).
fn singular(w: &str) -> String {
    if w.len() <= 3 || w.starts_with('{') || w.contains('/') {
        return match w {
            "has" => "have".into(),
            "is" | "are" => "is".into(),
            "was" | "were" => "was".into(),
            "its" => "it".into(),
            "does" => "do".into(),
            other => other.to_string(),
        };
    }
    match w {
        "doesn't" | "don't" => return "don't".into(),
        "isn't" | "aren't" => return "isn't".into(),
        "wasn't" | "weren't" => return "wasn't".into(),
        _ => {}
    }
    if let Some(stem) = w.strip_suffix("ies") {
        return format!("{stem}y");
    }
    if let Some(stem) = w.strip_suffix("ves") {
        if ["el", "dwar", "wol", "werewol", "sel", "hal"]
            .iter()
            .any(|s| stem.ends_with(s))
        {
            return format!("{stem}f");
        }
    }
    for suf in ["ches", "shes", "sses", "xes"] {
        if w.ends_with(suf) {
            return w[..w.len() - 2].to_string();
        }
    }
    if w.ends_with("ss") || w.ends_with("us") || w.ends_with("is") {
        return w.to_string();
    }
    if let Some(stem) = w.strip_suffix('s') {
        return stem.to_string();
    }
    w.to_string()
}

/// Normalizes one unit (an ability line or keyword) into comparable tokens.
pub fn normalize_unit(text: &str) -> Vec<String> {
    let mut s = text
        .replace(['\u{2212}', '\u{2013}'], "-")
        .replace('\u{2019}', "'")
        .replace(['\u{201C}', '\u{201D}'], "\"")
        .to_lowercase();
    s = s.replace('\n', " ").replace('•', " ");
    s = sentence_rewrites(&s);
    for e in EQUIVALENCES {
        let re = equivalence_regex(e.pattern);
        s = re.replace_all(&s, e.replacement).to_string();
    }
    // Tokenize: keep {..} symbols, +1/+1, ~, words with apostrophes and hyphens.
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_brace = false;
    for ch in s.chars() {
        if in_brace {
            cur.push(ch);
            if ch == '}' {
                in_brace = false;
                tokens.push(std::mem::take(&mut cur));
            }
            continue;
        }
        match ch {
            '{' => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
                cur.push(ch);
                in_brace = true;
            }
            c if c.is_alphanumeric() || matches!(c, '\'' | '+' | '-' | '/' | '~' | '*') => {
                cur.push(c)
            }
            _ => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    let mut out = Vec::new();
    for t in tokens {
        let t = t.trim_matches('\'').to_string();
        let t = t
            .strip_suffix("'s")
            .or_else(|| t.strip_suffix("s'"))
            .map(|x| x.to_string())
            .unwrap_or(t);
        let t = t.trim_end_matches('-').to_string();
        if t.is_empty() || t == "-" {
            continue;
        }
        let t = match number_words().iter().find(|(w, _)| *w == t) {
            Some((_, d)) => d.to_string(),
            None => t,
        };
        for part in t.split(' ') {
            let p = if part == "an" {
                "a".to_string()
            } else {
                singular(part)
            };
            if IGNORED_WORDS.iter().any(|(w, _)| *w == p) {
                continue;
            }
            out.push(p);
        }
    }
    out
}

fn equivalence_regex(p: &'static str) -> &'static Regex {
    static CACHE: OnceLock<
        std::sync::Mutex<std::collections::HashMap<&'static str, &'static Regex>>,
    > = OnceLock::new();
    let m = CACHE.get_or_init(Default::default);
    let mut g = m.lock().unwrap();
    g.entry(p)
        .or_insert_with(|| Box::leak(Box::new(Regex::new(p).expect("equivalence regex"))))
}

/// Sentence-level rewrites (word order): a leading "Until end of turn, ..." or "As long
/// as ..., ..." clause moves to the end of its sentence; "X ..., where X is V" and
/// "equal to V" forms are made uniform.
fn sentence_rewrites(s: &str) -> String {
    static LEAD: OnceLock<Regex> = OnceLock::new();
    let lead = LEAD.get_or_init(|| {
        Regex::new(r"(^|[.:—•] |\n)(until end of turn|until your next turn|this turn|as long as [^,]+), ([^.]+)\.")
            .unwrap()
    });
    let mut s = s.to_string();
    for _ in 0..3 {
        let n = lead.replace_all(&s, "$1$3 $2.").to_string();
        if n == s {
            break;
        }
        s = n;
    }
    s
}

/// Splits a face's normalized Oracle text into comparison units: one per ability line,
/// keyword lines split into one unit per keyword, bullets joined to their modal line.
pub fn oracle_units(text: &str, names: &[String]) -> Vec<String> {
    let text = strip_reminder(text);
    let text = self_refs(&text, names);
    let mut lines: Vec<String> = Vec::new();
    for l in text.lines() {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        if l.starts_with('•') || l.starts_with(|c: char| c.is_ascii_digit()) && l.contains('|') {
            if let Some(last) = lines.last_mut() {
                last.push('\n');
                last.push_str(l);
                continue;
            }
        }
        lines.push(l.to_string());
    }
    let mut out = Vec::new();
    for l in lines {
        match keyword_items(&l) {
            Some(items) => out.extend(items),
            None => out.push(l),
        }
    }
    out
}

/// If a line is a keyword line, its keywords (one per protection quality).
fn keyword_items(line: &str) -> Option<Vec<String>> {
    let l = line.trim().trim_end_matches('.');
    if l.contains('"') || l.contains(':') {
        return None;
    }
    let lower = l.to_lowercase();
    let starts_kw = |s: &str| -> bool {
        let s = s.trim();
        s.ends_with("walk")
            || s.contains("cycling")
            || s.starts_with("partner with")
            || s.starts_with("bands with other")
            || s.starts_with("megamorph")
            || s.starts_with("multikicker")
            || s.starts_with("daybound")
            || s.starts_with("nightbound")
            || s.starts_with("totem armor")
            || KeywordKind::ALL.iter().any(|k| {
                let n = k.name().to_lowercase();
                s.starts_with(&n)
                    && s[n.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !c.is_alphanumeric())
            })
    };
    if !starts_kw(&lower) {
        return None;
    }
    if l.contains('—') {
        return Some(vec![l.to_string()]);
    }
    // Split on top-level commas/semicolons.
    let mut items: Vec<String> = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for ch in l.chars() {
        match ch {
            '{' => {
                depth += 1;
                cur.push(ch)
            }
            '}' => {
                depth -= 1;
                cur.push(ch)
            }
            ',' | ';' if depth == 0 => items.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    items.push(cur);
    let mut merged: Vec<String> = Vec::new();
    for it in items {
        let t = it.trim().to_string();
        if t.is_empty() {
            continue;
        }
        let tl = t.to_lowercase();
        let continues = merged.last().is_some_and(|p| {
            let p = p.to_lowercase();
            (p.starts_with("protection from") || p.starts_with("hexproof from"))
                && (tl.starts_with("from ") || tl.starts_with("and from ") || !starts_kw(&tl))
        });
        if continues {
            let last = merged.last_mut().unwrap();
            last.push_str(", ");
            last.push_str(&t);
        } else {
            if !starts_kw(&tl) {
                return None;
            }
            merged.push(t);
        }
    }
    // "Protection from black and from red" stands for one ability per quality
    // (CR 702.16g, 702.11f); "from each color" for one per color (CR 702.16h).
    let mut out = Vec::new();
    for m in merged {
        let ml = m.to_lowercase();
        let head = if ml.starts_with("protection from ") {
            Some("protection from ")
        } else if ml.starts_with("hexproof from ") {
            Some("hexproof from ")
        } else {
            None
        };
        match head {
            Some(h) => {
                let rest = &m[h.len()..];
                let rl = rest.to_lowercase();
                if rl == "each color" || rl == "all colors" {
                    for c in ["white", "blue", "black", "red", "green"] {
                        out.push(format!("{h}{c}"));
                    }
                    continue;
                }
                let parts: Vec<String> = rest
                    .split(", and from ")
                    .flat_map(|p| p.split(", from "))
                    .flat_map(|p| p.split(" and from "))
                    .map(|p| p.trim().to_string())
                    .collect();
                for p in parts {
                    out.push(format!("{h}{p}"));
                }
            }
            None => out.push(m),
        }
    }
    Some(out)
}

/// The result of comparing one card.
#[derive(Clone, Debug)]
pub struct CardCheck {
    pub name: String,
    pub pass: bool,
    /// Per face: (Oracle units, rendered units).
    pub faces: Vec<(Vec<String>, Vec<String>)>,
    /// Gaps found while rendering.
    pub gaps: Vec<String>,
    /// Oracle units with no match, and rendered units with no match.
    pub unmatched_oracle: Vec<String>,
    pub unmatched_rendered: Vec<String>,
}

/// Renders a card and compares it with its Oracle text.
pub fn check_card(def: &CardDef) -> CardCheck {
    let rendered: Vec<RenderedFace> = render_card(def);
    let mut faces = Vec::new();
    let mut gaps = Vec::new();
    let mut unmatched_oracle = Vec::new();
    let mut unmatched_rendered = Vec::new();
    let mut pass = true;
    for (i, (face, r)) in def.faces.iter().zip(rendered.iter()).enumerate() {
        let names = self_names(def, i);
        let oracle = oracle_units(&face.chars.rules_text, &names);
        let mine: Vec<String> = r
            .lines
            .iter()
            .flat_map(|l| {
                let l = self_refs(l, &names);
                match keyword_items(&l) {
                    Some(items) => items,
                    None => vec![l],
                }
            })
            .collect();
        gaps.extend(r.gaps.iter().cloned());
        let (uo, ur) = diff_units(&oracle, &mine);
        if !uo.is_empty() || !ur.is_empty() {
            // Fall back to comparing the whole face in order (one line may compile to
            // several abilities, or several lines to one).
            let all_o: Vec<String> = oracle.iter().flat_map(|u| normalize_unit(u)).collect();
            let all_m: Vec<String> = mine.iter().flat_map(|u| normalize_unit(u)).collect();
            if all_o != all_m {
                pass = false;
                unmatched_oracle.extend(uo);
                unmatched_rendered.extend(ur);
            }
        }
        faces.push((oracle, mine));
    }
    if !gaps.is_empty() {
        pass = false;
    }
    CardCheck {
        name: def.name.to_string(),
        pass,
        faces,
        gaps,
        unmatched_oracle,
        unmatched_rendered,
    }
}

/// Multiset difference of units by normalized tokens.
fn diff_units(oracle: &[String], mine: &[String]) -> (Vec<String>, Vec<String>) {
    let mut mine_left: Vec<(Vec<String>, &String)> =
        mine.iter().map(|m| (normalize_unit(m), m)).collect();
    let mut uo = Vec::new();
    for o in oracle {
        let n = normalize_unit(o);
        if let Some(pos) = mine_left.iter().position(|(m, _)| *m == n) {
            mine_left.remove(pos);
        } else {
            uo.push(o.clone());
        }
    }
    let ur = mine_left.into_iter().map(|(_, m)| m.clone()).collect();
    (uo, ur)
}
