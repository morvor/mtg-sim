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
        pattern: r"\bfrom (your|a|an|their|its owner's|that player's|target player's|target opponent's|an opponent's|each|all|any) ((?:opponent's |player's )?)(graveyards?|hands?|library|libraries)\b",
        replacement: "in $1 $2$3",
        why: "An object description says where the object is: \"a creature card from your \
              graveyard\" and \"a creature card in your graveyard\" describe the same cards.",
    },
    Equivalence {
        pattern: r"\bfrom exile\b",
        replacement: "in exile",
        why: "See \"from your graveyard\".",
    },
    Equivalence {
        pattern: r"\byou (draw|discard|mill|scry|surveil|sacrifice|create|put|return|exile|search|reveal|look|shuffle|tap|untap|destroy|investigate|proliferate|seek|conjure|venture|explore|amass|populate|manifest|cloak|choose|add|counter|attach|transform|gain|lose|get|become|take|may|cast|pay|play|win|skip)\b",
        replacement: "$1",
        why: "An instruction without a subject is performed by the ability's controller \
              (CR 608.2c, 113.8): \"draw a card\" and \"you draw a card\" mean the same.",
    },
    Equivalence {
        pattern: r"(sacrifices? [^.]*?) of (their|his or her|your) choice\b",
        replacement: "$1",
        why: "The player who sacrifices chooses what to sacrifice (CR 701.21a); \"of their \
              choice\" restates it.",
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
        pattern: r"\buntil end of turn\b",
        replacement: "this turn",
        why: "\"Until end of turn\" and \"this turn\" effects both end in the cleanup step \
              (CR 514.2).",
    },
    Equivalence {
        pattern: r#"\. (it has|they have|it gains|they gain) ""#,
        replacement: " with \"",
        why: "A token created \"with\" an ability and one that \"has\" it (a following \
              sentence) are the same token (CR 111.1).",
    },
    Equivalence {
        pattern: r"\breturn(s?)\b",
        replacement: "put$1",
        why: "\"Return\" has no rules meaning of its own: it's a zone change like \
              \"put\" (CR 400.6), described by its destination.",
    },
    Equivalence {
        pattern: r"\b(into|onto)\b",
        replacement: "to",
        why: "Prepositions of a destination zone (\"into your hand\", \"onto the \
              battlefield\").",
    },
    Equivalence {
        pattern: r"\bwith (x|\d+|an?) additional\b",
        replacement: "with $1",
        why: "Counters an object enters with are put on it in addition to any others it \
              would enter with (CR 614.1c, 122.6).",
    },
    Equivalence {
        pattern: r"\balso\b ",
        replacement: "",
        why: "\"Also\" has no rules meaning.",
    },
    Equivalence {
        pattern: r"\beach player's (upkeep|draw step|end step)\b",
        replacement: "each $1",
        why: "Each player has one upkeep (draw step, end step) per turn: \"each upkeep\" \
              and \"each player's upkeep\" are the same steps (CR 501–503, 513).",
    },
    Equivalence {
        pattern: r"\bnumber of of\b",
        replacement: "number of",
        why: "\"For each of its colors\" rewritten to the \"where X is the number of\" form.",
    },
    Equivalence {
        pattern: r"\bat the beginning of the end step\b",
        replacement: "at the beginning of each end step",
        why: "Older wording: \"the end step\" in a trigger condition means every end step \
              (CR 513.1a).",
    },
    Equivalence {
        pattern: r"\b(gets?) an additional ([+-])",
        replacement: "$1 $2",
        why: "P/T modifications add up (CR 613.4c); \"an additional +2/-2\" is +2/-2.",
    },
    Equivalence {
        pattern: r"\b(is|are|was|were|do|does|has|have)n't\b",
        replacement: "$1 not",
        why: "Contraction.",
    },
    Equivalence {
        pattern: r"\band/or\b",
        replacement: "and",
        why: "In a list of object kinds, \"artifacts and/or enchantments\" and \"artifacts \
              and enchantments\" both mean objects that are either.",
    },
    Equivalence {
        pattern: r"(^|[^~\w])(it|that|there|what|he|she)'s\b",
        replacement: "$1$2 is",
        why: "Contraction.",
    },
    Equivalence {
        pattern: r"\b(adds?) an additional\b",
        replacement: "$1",
        why: "A triggered mana ability's mana is added in addition to the mana the \
              permanent produced (CR 605.1b, 106.12a); \"additional\" restates it.",
    },
    Equivalence {
        pattern: r"\bthem\b",
        replacement: "it",
        why: "Pronoun number (grammatical number is ignored).",
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
        Regex::new(r"(^|[.:—•] |\n)(until end of turn|until your next turn|this turn|as long as [^,]+|at the beginning of the next end step|until the end of your next turn|during your turn|during turns other than yours|during each of your turns|at the beginning of the next turn's upkeep|at the beginning of the next cleanup step|at the beginning of your next upkeep|at end of combat), ([^.]+)\.")
            .unwrap()
    });
    let mut s = s.to_string();
    // "If C, Y. Otherwise, X." and "X. If C, Y instead." state the same choice.
    static OTHERWISE: OnceLock<[Regex; 2]> = OnceLock::new();
    let [instead, otherwise] = OTHERWISE.get_or_init(|| {
        [
            Regex::new(r"(^|[.:—•] |\n)if ([^,.]+), instead ([^.]+)\.").unwrap(),
            Regex::new(r"(^|[.:—•] |\n)if ([^,.]+), ([^.]+)\. otherwise, ([^.]+)\.").unwrap(),
        ]
    });
    s = instead.replace_all(&s, "${1}if $2, $3 instead.").to_string();
    s = otherwise.replace_all(&s, "$1$4. if $2, $3 instead.").to_string();
    // "X if C." and "If C, X." state the same condition (a trailing "if able" or "only
    // if" is something else).
    static TRAILING_IF: OnceLock<Regex> = OnceLock::new();
    let trailing = TRAILING_IF.get_or_init(|| {
        Regex::new(r"(^|[.:—•] |\n)([^.:—•\n]+?) if ([^.,:\n]+)\.").unwrap()
    });
    s = trailing
        .replace_all(&s, |c: &regex::Captures| {
            let (lead, body, cond) = (&c[1], &c[2], &c[3]);
            let keep = body.starts_with("if ")
                || body.ends_with(" only")
                || body.ends_with(" as though")
                || cond == "able"
                || cond.starts_with("able ")
                || body.contains(" unless ")
                || body.contains("\"");
            if keep {
                c[0].to_string()
            } else {
                format!("{lead}if {cond}, {body}.")
            }
        })
        .to_string();
    for (re, rep) in where_x_rewrites() {
        s = re.replace_all(&s, *rep).to_string();
    }
    for _ in 0..3 {
        let n = lead.replace_all(&s, "$1$3 $2.").to_string();
        if n == s {
            break;
        }
        s = n;
    }
    s
}

/// Amounts stated as "equal to V" or "for each F" are rewritten to the "X ..., where X is
/// V" form (CR 107.3: X is defined by the text): both describe the same number.
fn where_x_rewrites() -> &'static [(Regex, &'static str)] {
    static R: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    R.get_or_init(|| {
        [
            (r"\bdeals? damage equal to ([^.]+?) to ([^.]+?)(\.|$)", "deals x damage to $2, where x is $1$3"),
            (r"\bdeals? damage to ([^.]+?) equal to ([^.]+?)(\.|$)", "deals x damage to $1, where x is $2$3"),
            (r"\b(gains?|loses?) life equal to ([^.]+?)(\.|$)", "$1 x life, where x is $2$3"),
            (r"\b(gains?|loses?) 1 life for each ([^.]+?)(\.|$)", "$1 x life, where x is the number of $2$3"),
            (r"\b(gets?) ([+-])1/([+-])1 ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)", "$1 ${2}x/${3}x $4, where x is the number of $5$6"),
            (r"\b(gets?) ([+-])1/([+-])0 ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)", "$1 ${2}x/${3}0 $4, where x is the number of $5$6"),
            (r"\b(gets?) ([+-])0/([+-])1 ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)", "$1 ${2}0/${3}x $4, where x is the number of $5$6"),
            (r"\b(draws?) cards equal to ([^.]+?)(\.|$)", "$1 x cards, where x is $2$3"),
            (r"\b(mills?) cards equal to ([^.]+?)(\.|$)", "$1 x cards, where x is $2$3"),
            (r"\bputs? an? (\S+) counter on ([^.]+?) for each ([^.]+?)(\.|$)", "put x $1 counters on $2, where x is the number of $3$4"),
            (r"\benters? with an? (\S+) counter on it for each ([^.]+?)(\.|$)", "enters with x $1 counters on it, where x is the number of $2$3"),
            (r"\b(draws?) a card for each ([^.]+?)(\.|$)", "$1 x cards, where x is the number of $2$3"),
            (r"\b(creates?) an? ([^.]+?) tokens? for each ([^.]+?)(\.|$)", "$1 x $2 tokens, where x is the number of $3$4"),
            (r"\b(mills?) a card for each ([^.]+?)(\.|$)", "$1 x cards, where x is the number of $2$3"),
        ]
        .into_iter()
        .map(|(p, r)| (Regex::new(p).unwrap(), r))
        .collect()
    })
}

/// Ability words (CR 207.2c): they have no rules meaning.
const ABILITY_WORDS: &[&str] = &[
    "adamant", "addendum", "alliance", "battalion", "bloodrush", "celebration", "channel",
    "chroma", "cohort", "constellation", "converge", "council's dilemma", "coven", "delirium",
    "descend 4", "descend 8", "disappear", "domain", "eerie", "eminence", "enrage",
    "fateful hour", "fathomless descent", "ferocious", "flurry", "formidable", "grandeur",
    "hellbent", "heroic", "imprint", "infusion", "inspired", "join forces", "kinship",
    "landfall", "lieutenant", "magecraft", "metalcraft", "morbid", "opus", "pack tactics",
    "paradox", "parley", "radiance", "raid", "rally", "renew", "repartee", "revolt",
    "secret council", "spell mastery", "strive", "survival", "sweep", "tempting offer",
    "threshold", "undergrowth", "valiant", "vivid", "void", "will of the council",
];

/// Labels before an em dash that aren't ability or flavor words.
const NOT_FLAVOR: &[&str] = &[
    "companion", "boast", "exhaust", "forecast", "max speed", "power-up", "to solve",
    "solved", "choose", "level up", "ward", "equip", "cumulative upkeep", "echo",
];

/// Strips a leading ability word (CR 207.2c) or flavor word (CR 207.2d): "Landfall — ".
fn strip_ability_word(line: &str) -> String {
    let Some((head, rest)) = line.split_once(" — ") else {
        return line.to_string();
    };
    // Saga chapters with a flavor word: "I — Aerial Blast — effect" (CR 714.2b, 207.2d).
    if head.split(", ").all(|n| !n.is_empty() && n.chars().all(|c| matches!(c, 'I' | 'V' | 'X'))) {
        let inner = strip_ability_word(rest);
        return format!("{head} — {inner}");
    }
    let h = head.trim().to_lowercase().replace('\u{2019}', "'");
    if ABILITY_WORDS.contains(&h.as_str()) {
        return rest.to_string();
    }
    let words: Vec<&str> = head.split_whitespace().collect();
    let flavor = !words.is_empty()
        && words.len() <= 7
        && head.chars().next().is_some_and(|c| c.is_uppercase())
        && !head.contains(['{', ':', '"', ',', '~', '\n', '•', '|'])
        && !head.chars().any(|c| c.is_ascii_digit())
        && !NOT_FLAVOR.iter().any(|n| h.starts_with(n))
        && !KeywordKind::ALL
            .iter()
            .any(|k| h.starts_with(&k.name().to_lowercase()))
        && !head.split(", ").all(|n| {
            n.chars().all(|c| matches!(c, 'I' | 'V' | 'X'))
        });
    if flavor {
        rest.to_string()
    } else {
        line.to_string()
    }
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
        lines.push(strip_ability_word(l));
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
            let units_m: Vec<Vec<String>> = mine.iter().map(|u| normalize_unit(u)).collect();
            if !tokens_match(&all_o, &all_m) && !shared_subject_match(&all_o, &units_m) {
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

/// Whether two normalized token sequences are the same. The renderer's `~it` (the object
/// itself, just mentioned) matches "~" or "it": cards refer to an object that a trigger
/// condition just named either by its name or by "it".
pub fn tokens_match(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| token_eq(x, y))
}

/// Token equality for [`tokens_match`].
pub fn token_eq(x: &str, y: &str) -> bool {
    x == y || (x == "~it" && (y == "~" || y == "it")) || (y == "~it" && (x == "~" || x == "it"))
}

/// Whether the rendered units, in order, spell the Oracle tokens when a unit may drop
/// the subject it shares with the previous unit: two abilities printed on one line with
/// one subject ("Enchanted creature gets +1/+0 and can't be blocked.").
fn shared_subject_match(oracle: &[String], units: &[Vec<String>]) -> bool {
    let mut pos = 0;
    for (i, u) in units.iter().enumerate() {
        let fits = |t: &[String], pos: usize| {
            pos + t.len() <= oracle.len() && tokens_match(&oracle[pos..pos + t.len()], t)
        };
        if fits(u, pos) {
            pos += u.len();
            continue;
        }
        let prev = if i > 0 { &units[i - 1] } else { return false };
        let lcp = prev.iter().zip(u).take_while(|(a, b)| a == b).count();
        let mut ok = false;
        for k in (1..=lcp.min(6)).rev() {
            if fits(&u[k..], pos) {
                pos += u.len() - k;
                ok = true;
                break;
            }
        }
        if !ok {
            return false;
        }
    }
    pos == oracle.len()
}

/// Multiset difference of units by normalized tokens.
fn diff_units(oracle: &[String], mine: &[String]) -> (Vec<String>, Vec<String>) {
    let mut mine_left: Vec<(Vec<String>, &String)> =
        mine.iter().map(|m| (normalize_unit(m), m)).collect();
    let mut uo = Vec::new();
    for o in oracle {
        let n = normalize_unit(o);
        if let Some(pos) = mine_left.iter().position(|(m, _)| tokens_match(&n, m)) {
            mine_left.remove(pos);
        } else {
            uo.push(o.clone());
        }
    }
    let ur = mine_left.into_iter().map(|(_, m)| m.clone()).collect();
    (uo, ur)
}
