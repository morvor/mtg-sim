//! The decisions log, `docs/DECISIONS.md`: every decision of the interpretation register,
//! by who made it, for reviewers.
//!
//! [`render`] builds it from three sources, and `mtg-tools interpretations --write` writes
//! it beside `docs/INTERPRETATIONS.md` (`--check`, and `cargo test -p mtg-data`, fail when
//! it is stale):
//! - the register (`docs/interpretations/*.md`, [`crate::interpretations`]): each decided
//!   entry is attributed ([`attribute`]) from its "Decided by ..." words (looked for in the
//!   `Decision` field, then in `Why`): to the user, with the date ([`parse_date`]
//!   normalizes the formats used), and the user's own words where the entry quotes them
//!   ([`user_quotes`]); to the sources (a ruling, a rule, a dated change); or to the
//!   coordinator or a worker. An entry that names no decider was decided when it was
//!   written; one whose words name a decider this module can't read is listed under
//!   "needs a source line", never guessed;
//! - the "Revisit later" list of `docs/agents/finish-line-plan.md` ([`revisit_later`]);
//! - the ruling discrepancy log, `docs/RULING_DISCREPANCIES.md` ([`crate::discrepancies`]).

use crate::discrepancies::{self, LogEntry};
use crate::interpretations::{self, Entry};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The log, relative to the repository root.
pub const OUTPUT: &str = "docs/DECISIONS.md";

/// The plan whose "Revisit later" section the log lists, relative to the repository root.
pub const PLAN: &str = "docs/agents/finish-line-plan.md";

/// The year of a date written without one (the register's dates are all of the project's
/// year). Used only to order dates; a date is shown as written.
pub const DEFAULT_YEAR: u16 = 2026;

const MONTHS: &[&str] = &[
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
const MONTH_NAMES: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A date as the register writes it ("Nov 5", "on Nov 5", "November 5", "5 Nov",
/// "2026-11-05", "Nov 5, 2026"), normalized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Date {
    /// The year, if the text gives one.
    pub year: Option<u16>,
    /// 1–12.
    pub month: u8,
    /// 1–31.
    pub day: u8,
}

impl Date {
    /// The key dates are ordered by (a date without a year is in [`DEFAULT_YEAR`]).
    pub fn key(&self) -> (u16, u8, u8) {
        (self.year.unwrap_or(DEFAULT_YEAR), self.month, self.day)
    }
}

impl std::fmt::Display for Date {
    /// "Nov 5"; "Nov 5, 2025" for a year other than [`DEFAULT_YEAR`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = MONTH_NAMES[usize::from(self.month - 1)];
        match self.year {
            Some(y) if y != DEFAULT_YEAR => write!(f, "{m} {}, {y}", self.day),
            _ => write!(f, "{m} {}", self.day),
        }
    }
}

/// The month a word names ("Nov", "Nov.", "November", any case), 1–12.
fn month_of(word: &str) -> Option<u8> {
    let w = word.trim_end_matches('.').to_ascii_lowercase();
    if w.len() < 3 {
        return None;
    }
    let i = MONTHS.iter().position(|m| w.starts_with(m))?;
    // A full name must be that month's name ("Nove" is not a month; "Novel" is not).
    const FULL: &[&str] = &[
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    (w.len() == 3 || FULL[i] == w || (i == 8 && w == "sept")).then_some(i as u8 + 1)
}

/// The leading run of ASCII digits of `s` and the rest.
fn digits(s: &str) -> (&str, &str) {
    let n = s.bytes().take_while(u8::is_ascii_digit).count();
    s.split_at(n)
}

/// A day number ("5", "05", "5th", "21st"), then the rest.
fn day_of(s: &str) -> Option<(u8, &str)> {
    let (d, rest) = digits(s);
    let day: u8 = d.parse().ok().filter(|d| (1..=31).contains(d))?;
    let rest = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|suf| rest.strip_prefix(suf))
        .unwrap_or(rest);
    Some((day, rest))
}

/// True if `rest` doesn't continue the word or number before it.
fn boundary(rest: &str) -> bool {
    !rest.starts_with(|c: char| c.is_alphanumeric())
}

/// An optional year after a day: ", 2026" or " 2026".
fn year_after(rest: &str) -> (Option<u16>, &str) {
    let r = rest.strip_prefix(',').unwrap_or(rest);
    let r = r.strip_prefix(' ').unwrap_or(r);
    let (y, after) = digits(r);
    match y.parse::<u16>() {
        Ok(year) if y.len() == 4 && boundary(after) => (Some(year), after),
        _ => (None, rest),
    }
}

/// Parses a date at the start of `s` (after spaces; an "on " before it is skipped):
/// returns the date and the number of bytes of `s` it takes.
pub fn parse_date(s: &str) -> Option<(Date, usize)> {
    let t = s.trim_start();
    let t = t.strip_prefix("on ").map(str::trim_start).unwrap_or(t);
    // 2026-11-05
    let (y, rest) = digits(t);
    if y.len() == 4 {
        let mut it = rest.splitn(3, '-');
        if let (Some(""), Some(m), Some(d_rest)) = (it.next(), it.next(), it.next()) {
            let (d, after) = digits(d_rest);
            let (month, day) = (m.parse::<u8>().ok()?, d.parse::<u8>().ok()?);
            if m.len() == 2
                && d.len() == 2
                && (1..=12).contains(&month)
                && (1..=31).contains(&day)
                && boundary(after)
            {
                let date = Date {
                    year: y.parse().ok(),
                    month,
                    day,
                };
                return Some((date, s.len() - after.len()));
            }
        }
        return None;
    }
    // 5 Nov [2026]
    if !y.is_empty() {
        let (day, rest) = day_of(t)?;
        let rest = rest.strip_prefix(' ')?;
        let word_len = rest
            .find(|c: char| !c.is_ascii_alphabetic() && c != '.')
            .unwrap_or(rest.len());
        let month = month_of(&rest[..word_len])?;
        let (year, after) = year_after(&rest[word_len..]);
        return Some((Date { year, month, day }, s.len() - after.len()));
    }
    // Nov 5 [, 2026]
    let word_len = t
        .find(|c: char| !c.is_ascii_alphabetic() && c != '.')
        .unwrap_or(t.len());
    let month = month_of(&t[..word_len])?;
    let rest = t[word_len..].strip_prefix(' ')?;
    let (day, rest) = day_of(rest)?;
    if !boundary(rest) {
        return None;
    }
    let (year, after) = year_after(rest);
    Some((Date { year, month, day }, s.len() - after.len()))
}

/// Who made a decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decider {
    /// The user, on that date.
    User(Date),
    /// A ruling, a rule, or a dated change: the entry says "Decided by the ruling" (the
    /// rulings, the rule, the sources, the CR, ...).
    Sources,
    /// The coordinator, or a worker or a review ("Decided by the coordinator", "decided by
    /// the rulings-P192 worker", "Decided in the review of ...").
    Coordinator,
    /// The entry names no decider: it was decided as it was written.
    Unstated,
    /// The entry names a decider in words this module can't read (the reason).
    Unparsed(String),
}

/// An entry's attribution: who decided, and the words that say so.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribution {
    pub decider: Decider,
    /// The attribution as written ("Decided by the user, Nov 5", "decided by the coordinator
    /// on Oct 14", "Decided by the ruling (...)"); empty when no decider is named.
    pub clause: String,
    /// The field the attribution is in (`Decision` or `Why`), or "" for none.
    pub field: &'static str,
}

/// The end of the clause starting at `from`: after its first parenthesis, if it has one
/// before the clause ends; else at the first ". ", ": " or "; ", or at a ")" closing a
/// parenthesis the clause is inside ("(option B, decided by the coordinator on Oct 14)").
fn clause_end(s: &str, from: usize) -> usize {
    let mut depth = 0usize;
    for (i, c) in s[from..].char_indices() {
        let at = from + i;
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return at,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return at + 1;
                }
            }
            '.' | ':' | ';' if depth == 0 && s[at + 1..].starts_with(' ') => return at,
            _ => {}
        }
    }
    s.len() - s[from..].len() + s[from..].trim_end_matches('.').len()
}

/// Where the text names a decider: the start of "Decided by"/"Decided in" (a sentence's
/// start), else of a lowercase "decided by"/"decided in" opening a clause ("(option B,
/// decided by the coordinator ..."). Not "not decided by", "would be decided by", ...
fn find_attribution(text: &str) -> Option<usize> {
    if let Some(p) = ["Decided by ", "Decided in "]
        .iter()
        .filter_map(|pat| text.find(pat))
        .min()
    {
        return Some(p);
    }
    let mut from = 0;
    while let Some(p) = text[from..]
        .find("decided by ")
        .or_else(|| text[from..].find("decided in "))
    {
        let at = from + p;
        let before = text[..at].trim_end_matches(' ');
        if before.is_empty() || before.ends_with(['(', ',', ';']) {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// Words that make a decider a source: "the ruling", "the rulings", "the rule", "the CR",
/// "the WotC Seasinger ruling", "the sources", "the Oracle text", "a dated change".
fn names_a_source(phrase: &str) -> bool {
    let p = phrase.to_lowercase();
    [
        "ruling",
        "rule",
        "source",
        "oracle",
        "comprehensive",
        "change",
    ]
    .iter()
    .any(|w| p.contains(w))
        || p.split(|c: char| !c.is_alphanumeric()).any(|w| w == "cr")
}

/// Classifies the attribution starting at `at` in `text` ("Decided by ...").
fn classify(text: &str, at: usize, field: &'static str) -> Attribution {
    let end = clause_end(text, at);
    let clause = text[at..end].trim_end_matches(['.', ',']).to_string();
    let rest = &text[at + "decided by ".len()..];
    let in_review = text[at..].to_lowercase().starts_with("decided in ");
    // The decider's words, up to a parenthesis or the clause's end.
    let phrase_end = [" (", ",", ": ", ". ", ";"]
        .iter()
        .filter_map(|p| rest.find(p))
        .min()
        .unwrap_or(rest.len());
    let phrase = rest[..phrase_end].trim().trim_end_matches(['.', ':']);
    let lower = phrase.to_lowercase();
    let decider = if in_review {
        if lower.contains("review") {
            Decider::Coordinator
        } else {
            Decider::Unparsed(format!("\"decided in {phrase}\" names no decider"))
        }
    } else if let Some(after) = lower.strip_prefix("the user") {
        let after_user = &rest["the user".len()..];
        let date_text = after_user.trim_start_matches([',', ' ', '(']);
        match parse_date(date_text) {
            Some((d, _)) if after.is_empty() || after.starts_with(' ') => Decider::User(d),
            _ => Decider::Unparsed("\"decided by the user\" with no date after it".to_string()),
        }
    } else if lower.starts_with("the coordinator")
        || lower.contains("worker")
        || lower.contains("review")
    {
        Decider::Coordinator
    } else if names_a_source(phrase) {
        Decider::Sources
    } else {
        Decider::Unparsed(format!("\"decided by {phrase}\" names no known decider"))
    };
    Attribution {
        decider,
        clause,
        field,
    }
}

/// "the user, Nov 12" or "the user on Nov 12" opening a clause, without "decided by" (as
/// in "(to be fixed later; the user, Nov 12)"): the start of "the user".
fn bare_user_date(text: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(p) = text[from..].find("the user") {
        let at = from + p;
        let before = text[..at].trim_end_matches(' ');
        let after = &text[at + "the user".len()..];
        let dated = after
            .strip_prefix(',')
            .or_else(|| after.strip_prefix(" on"))
            .and_then(parse_date)
            .is_some();
        if dated && (before.is_empty() || before.ends_with(['(', ';'])) {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// Who decided an entry: the first attribution in its `Decision` field, else in its `Why`
/// field. A field that dates the user without "Decided by" makes it [`Decider::Unparsed`].
pub fn attribute(e: &Entry) -> Attribution {
    for field in ["Decision", "Why"] {
        let text = e.fields.get(field).map(String::as_str).unwrap_or("");
        if let Some(at) = find_attribution(text) {
            return classify(text, at, field);
        }
    }
    let text = e.fields.get("Decision").map(String::as_str).unwrap_or("");
    if let Some(at) = bare_user_date(text) {
        let end = clause_end(text, at);
        return Attribution {
            decider: Decider::Unparsed(
                "names the user and a date without \"Decided by the user, <date>\"".to_string(),
            ),
            clause: text[at..end].trim_end_matches([')', '.']).to_string(),
            field: "Decision",
        };
    }
    Attribution {
        decider: Decider::Unstated,
        clause: String::new(),
        field: "",
    }
}

fn is_open_quote_at(s: &str, i: usize) -> bool {
    let prev = s[..i].chars().next_back();
    let next = s[i + 1..].chars().next();
    prev.is_none_or(|c| c.is_whitespace() || "([{/-—".contains(c))
        && next.is_some_and(|c| !c.is_whitespace())
}

/// The top-level quotations of `s`: (start of the opening quote, end after the closing
/// quote, the text inside). Straight quotes nest ("when "an opponent chooses" the ..."): a
/// quote after a space or an opening bracket that a non-space follows opens one, any
/// other closes one; curly quotes pair as written.
pub fn quotations(s: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i..].chars().next().unwrap();
        let opens = (c == '"' && is_open_quote_at(s, i)) || c == '“';
        if !opens {
            i += c.len_utf8();
            continue;
        }
        let start = i;
        let mut depth = 1usize;
        let mut j = i + c.len_utf8();
        let mut end = None;
        let mut close_len = 1;
        while j < s.len() {
            let d = s[j..].chars().next().unwrap();
            close_len = d.len_utf8();
            match d {
                '“' => depth += 1,
                '”' => depth -= 1,
                '"' if is_open_quote_at(s, j) => depth += 1,
                '"' => depth -= 1,
                _ => {}
            }
            j += d.len_utf8();
            if depth == 0 {
                end = Some(j);
                break;
            }
        }
        let Some(end) = end else { break };
        let inner = &s[start + c.len_utf8()..end - close_len];
        out.push((start, end, inner.to_string()));
        i = end;
    }
    out
}

/// True if `lead` (the text of a sentence up to a quotation) introduces the user's words:
/// it ends with ":" or "(" and names the user as the speaker ("the user: ", "Decided by the
/// user, Nov 19: ", "(the user, Nov 19: ", "Decided by the user, Nov 19 (") or opens with
/// an acceptance ("Accepted tentatively (", "As recommended: ").
fn introduces_user_words(lead: &str) -> bool {
    let t = lead.trim_end();
    if !t.ends_with([':', '(']) {
        return false;
    }
    let lower = t.to_lowercase();
    let speaker = lower.match_indices("the user").any(|(p, _)| {
        // Not inside a parenthesis that closes before the quotation ("(decided by the
        // user, Nov 19: ...) ... rulings (").
        let mut depth = 0i32;
        for c in t[p..].chars() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth < 0 {
                return false;
            }
        }
        let after = &t[p + "the user".len()..];
        after.starts_with([':', ',', '('])
            || after.starts_with(" (")
            || after.starts_with(" on ")
            || ((after.starts_with("'s decision") || after.starts_with("’s decision"))
                && t.ends_with(':'))
            || parse_date(after).is_some()
    });
    speaker
        || ["accepted", "accepting", "as recommended"]
            .iter()
            .any(|w| lower.trim_start().starts_with(w))
}

/// The user's own words that a field quotes: each quotation that a sentence introduces as
/// the user's ([`introduces_user_words`]), in order.
pub fn user_quotes(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut prev_end = 0;
    for (start, end, inner) in quotations(text) {
        let between = &text[prev_end..start];
        let lead = between.rfind(". ").map_or(between, |p| &between[p + 2..]);
        if introduces_user_words(lead) {
            out.push(inner);
        }
        prev_end = end;
    }
    out
}

/// The user's words an entry quotes: those of `first` (the field its attribution is in),
/// else of its other fields (Decision, Why, Rules), with the field each came from.
pub fn entry_user_quotes(e: &Entry, first: &str) -> Vec<(String, String)> {
    let mut fields = vec![if first.is_empty() { "Decision" } else { first }];
    for f in ["Decision", "Why", "Rules"] {
        if !fields.contains(&f) {
            fields.push(f);
        }
    }
    for f in fields {
        let quotes = user_quotes(e.fields.get(f).map(String::as_str).unwrap_or(""));
        if !quotes.is_empty() {
            return quotes.into_iter().map(|q| (f.to_string(), q)).collect();
        }
    }
    Vec::new()
}

/// Words that mark a decision the user or the entry holds with low confidence.
pub const LOW_CONFIDENCE: &[&str] = &[
    "for now",
    "tentative",
    "not sure",
    "unsure",
    "low confidence",
    "low-confidence",
    "to be fixed later",
];

/// The low-confidence marks ([`LOW_CONFIDENCE`]) in an entry's `Decision` and `Why`, each
/// with the words around its first occurrence (a reviewer judges whether it is about this
/// decision).
pub fn low_confidence(e: &Entry) -> Vec<(&'static str, String)> {
    let mut found: Vec<(&'static str, String)> = Vec::new();
    for m in LOW_CONFIDENCE {
        if found.iter().any(|(f, _)| f.contains(m) || m.contains(f)) {
            continue;
        }
        for field in ["Decision", "Why"] {
            let text = e.fields.get(field).map(String::as_str).unwrap_or("");
            // ASCII lowercasing keeps byte offsets.
            if let Some(p) = text.to_ascii_lowercase().find(m) {
                found.push((m, context(text, p, p + m.len())));
                break;
            }
        }
    }
    found
}

/// The words of `text` around `from..to`: about 60 bytes each side, cut at spaces.
fn context(text: &str, from: usize, to: usize) -> String {
    let mut start = from.saturating_sub(60);
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    if start > 0 {
        start = text[start..from].find(' ').map_or(from, |p| start + p + 1);
    }
    let mut end = (to + 60).min(text.len());
    while !text.is_char_boundary(end) {
        end += 1;
    }
    if end < text.len() {
        end = text[to..end].rfind(' ').map_or(to, |p| to + p);
    }
    format!(
        "{}{}{}",
        if start > 0 { "..." } else { "" },
        &text[start..end],
        if end < text.len() { "..." } else { "" }
    )
}

/// True if an entry's `Affects` line names a single case: one card or thing, with no list
/// ("," "and" "or") and no class ("every", "any", "all", "other", "each").
pub fn affects_a_single_case(affects: &str) -> bool {
    let a = affects.trim().trim_end_matches('.').to_lowercase();
    !a.is_empty()
        && !a.contains([',', ';'])
        && !a
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| ["and", "or", "every", "any", "all", "other", "each"].contains(&w))
}

/// The "Revisit later" section of the plan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RevisitLater {
    /// The heading's text ("Revisit later (user, Nov 12)").
    pub heading: String,
    /// The text before the first bullet.
    pub intro: Vec<String>,
    /// Each top-level bullet with its nested lines, as written.
    pub bullets: Vec<String>,
}

/// The section of `plan` whose heading starts with "Revisit later" (any level, any case),
/// up to the next heading.
pub fn revisit_later(plan: &str) -> Option<RevisitLater> {
    let plan = plan.strip_prefix('\u{feff}').unwrap_or(plan);
    let mut out: Option<RevisitLater> = None;
    for line in plan.lines() {
        let line = line.trim_end();
        if line.starts_with('#') {
            if out.is_some() {
                break;
            }
            let h = line.trim_start_matches('#').trim();
            if h.to_lowercase().starts_with("revisit later") {
                out = Some(RevisitLater {
                    heading: h.to_string(),
                    ..Default::default()
                });
            }
            continue;
        }
        let Some(r) = out.as_mut() else { continue };
        if line.starts_with("- ") || line.starts_with("* ") {
            r.bullets.push(format!("- {}", &line[2..]));
        } else if line.trim().is_empty() {
            continue;
        } else if let Some(last) = r.bullets.last_mut() {
            last.push('\n');
            last.push_str(line);
        } else {
            r.intro.push(line.to_string());
        }
    }
    out
}

/// The backticked words of `text` (`` `id` ``).
fn backticked(text: &str) -> Vec<&str> {
    text.split('`').skip(1).step_by(2).collect()
}

/// The anchor GitHub gives a heading: lowercase, spaces to hyphens, punctuation dropped.
pub fn anchor(heading: &str) -> String {
    heading
        .trim()
        .chars()
        .filter_map(|c| {
            if c == ' ' {
                Some('-')
            } else if c.is_alphanumeric() || c == '-' || c == '_' {
                Some(c)
            } else {
                None
            }
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn entry_link(e: &Entry) -> String {
    format!(
        "[{}](interpretations/{}.md#{})",
        e.area,
        e.area,
        anchor(&format!("{}: {}", e.id, e.title))
    )
}

/// The first sentence of `text` (up to ". " before a capital letter or a backtick).
fn first_sentence(text: &str) -> &str {
    let mut from = 0;
    while let Some(p) = text[from..].find(". ") {
        let at = from + p;
        let next = text[at + 2..].chars().next();
        let prev = text[..at].chars().next_back();
        let abbreviation = text[..at].ends_with(" e.g") || text[..at].ends_with(" i.e");
        if next.is_some_and(|c| c.is_uppercase() || c == '`' || c == '(')
            && prev.is_some_and(|c| c != '.')
            && !abbreviation
        {
            return &text[..at + 1];
        }
        from = at + 2;
    }
    text
}

fn field<'a>(e: &'a Entry, name: &str) -> &'a str {
    e.fields.get(name).map(String::as_str).unwrap_or("")
}

/// "The user: "..." / "..."", naming the field when it isn't the Decision line.
fn user_words_line(quotes: &[(String, String)]) -> String {
    if quotes.is_empty() {
        return String::new();
    }
    let field = &quotes[0].0;
    let words = quotes
        .iter()
        .map(|(_, q)| format!("\"{q}\""))
        .collect::<Vec<_>>()
        .join(" / ");
    if field == "Decision" {
        format!("  - The user: {words}\n")
    } else {
        format!("  - The user (quoted in the {field} line): {words}\n")
    }
}

/// The register's entries sorted by who decided them (indices into the entries).
struct Groups {
    attrs: Vec<Attribution>,
    user: Vec<usize>,
    sources: Vec<usize>,
    coordinator: Vec<usize>,
    unstated: Vec<usize>,
    unparsed: Vec<usize>,
    provisional: Vec<usize>,
    policies: Vec<usize>,
}

fn groups(entries: &[Entry]) -> Groups {
    let attrs: Vec<Attribution> = entries.iter().map(attribute).collect();
    let decided = |i: usize| entries[i].status != "provisional";
    let by = |pred: &dyn Fn(&Decider) -> bool| -> Vec<usize> {
        (0..entries.len())
            .filter(|&i| decided(i) && pred(&attrs[i].decider))
            .collect()
    };
    let user = by(&|d| matches!(d, Decider::User(_)));
    let policies = user
        .iter()
        .copied()
        .filter(|&i| {
            entries[i].kind == "policy" && !affects_a_single_case(field(&entries[i], "Affects"))
        })
        .collect();
    Groups {
        sources: by(&|d| matches!(d, Decider::Sources)),
        coordinator: by(&|d| matches!(d, Decider::Coordinator)),
        unstated: by(&|d| matches!(d, Decider::Unstated)),
        unparsed: by(&|d| matches!(d, Decider::Unparsed(_))),
        provisional: (0..entries.len()).filter(|&i| !decided(i)).collect(),
        user,
        policies,
        attrs,
    }
}

/// The log's sections, in order, with the number of items in each (for
/// `mtg-tools interpretations`).
pub fn section_counts(entries: &[Entry], plan: &str, log: &str) -> Vec<(&'static str, usize)> {
    let g = groups(entries);
    vec![
        ("Decided by the user", g.user.len()),
        ("Decided by the sources", g.sources.len()),
        (
            "Decided by the coordinator or a worker",
            g.coordinator.len() + g.unstated.len(),
        ),
        ("Provisional", g.provisional.len()),
        (
            "Revisit later",
            revisit_later(plan).map_or(0, |r| r.bullets.len()),
        ),
        ("Ruling discrepancies", discrepancies::parse(log).0.len()),
        ("Policies the user set", g.policies.len()),
        ("Needs a source line", g.unparsed.len()),
    ]
}

/// Renders `docs/DECISIONS.md` from the register, the plan and the discrepancy log.
pub fn render_from(entries: &[Entry], plan: &str, log: &str) -> String {
    let Groups {
        attrs,
        user,
        sources,
        coordinator,
        unstated,
        unparsed,
        provisional,
        policies,
    } = groups(entries);
    let ids: BTreeSet<&str> = entries.iter().map(|e| e.id.as_str()).collect();
    let revisit = revisit_later(plan);
    let revisit_ids: BTreeSet<&str> = revisit
        .iter()
        .flat_map(|r| r.bullets.iter())
        .flat_map(|b| backticked(b))
        .filter(|w| ids.contains(w))
        .collect();
    let (log_entries, _) = discrepancies::parse(log);

    let mut s = String::new();
    s.push_str("# Decisions log\n\n");
    s.push_str(
        "Generated by `cargo run --release -p mtg-tools -- interpretations --write` from the \
         interpretation register (`docs/interpretations/*.md`), the \"Revisit later\" list of \
         `docs/agents/finish-line-plan.md` and `docs/RULING_DISCREPANCIES.md`; don't edit by \
         hand (`cargo test` fails when this file is out of date). To change what it says, \
         change those files and regenerate.\n\n",
    );
    s.push_str(
        "Every judgment call the engine makes about the rules (a reading of an ambiguous or \
         silent rule, a knowing deviation from the Comprehensive Rules, or a policy for a \
         choice the rules leave to players) is an entry of the register. This file sorts them \
         by who made the call, so a reviewer can see who decided what, and links each to its \
         full entry (the rules quoted, the question, the alternatives and the reasoning). Who \
         decided is read from the entry's own words, \"Decided by the user, <date>\", \
         \"Decided by the ruling (...)\" and the like, in its Decision line (or else its Why \
         line).\n\n",
    );
    s.push_str("Sections:\n\n");
    let sections: [(&str, usize, &str); 8] = [
        (
            "Decided by the user",
            user.len(),
            "the user chose the reading, newest first, by date: the user's own words where the \
             entry quotes them, the reading chosen, the alternatives not chosen, and marks of \
             low confidence (\"for now\", \"not sure\", \"tentatively\", ...) or of being on the \
             \"Revisit later\" list.",
        ),
        (
            "Decided by the sources",
            sources.len(),
            "a Wizards ruling, a rule of the Comprehensive Rules or a dated change answers the \
             question, and the entry records how; one line each, with that basis.",
        ),
        (
            "Decided by the coordinator or a worker",
            coordinator.len() + unstated.len(),
            "everything else decided: calls the coordinator, a worker or a review made, most of \
             them defaults for choices an agent leaves unanswered. Entries that name no decider \
             were decided as they were written and are listed apart; one line each, with the \
             entry's reasoning.",
        ),
        (
            "Provisional",
            provisional.len(),
            "calls a reasonable judge could make the other way, awaiting the user.",
        ),
        (
            "Revisit later",
            revisit.as_ref().map_or(0, |r| r.bullets.len()),
            "readings the user accepted for now but wants to look at again before the project \
             is called done, copied from the finish-line plan.",
        ),
        (
            "Ruling discrepancies",
            log_entries.len(),
            "Wizards rulings the engine doesn't follow in full, or tests on another card than \
             the one the data attaches them to, by kind and review status; a human checks each \
             and reports the erroneous ones to Wizards.",
        ),
        (
            "Policies the user set",
            policies.len(),
            "standing rules the user set for whole classes of cases (which source wins, who \
             chooses), drawn from their register entries: the user-decided entries of kind \
             policy whose Affects line names more than one case.",
        ),
        (
            "Needs a source line",
            unparsed.len(),
            "decided entries whose words about who decided this file can't read; they need a \
             \"Decided by ...\" line in the usual form, and are never guessed.",
        ),
    ];
    for (h, n, what) in sections {
        s.push_str(&format!("- [{h}](#{}) ({n}): {what}\n", anchor(h)));
    }
    s.push_str(&format!(
        "\n{} register entries: {} decided by the user, {} by the sources, {} by the \
         coordinator or a worker ({} of them naming no decider), {} provisional, {} needing a \
         source line.\n",
        entries.len(),
        user.len(),
        sources.len(),
        coordinator.len() + unstated.len(),
        unstated.len(),
        provisional.len(),
        unparsed.len()
    ));

    // 1. Decided by the user.
    s.push_str("\n## Decided by the user\n\n");
    s.push_str(
        "Newest first. \"The user\" quotes the user's words as the entry records them; \
         \"Decision\" is the entry's Decision line in full.\n",
    );
    let mut by_date: BTreeMap<(u16, u8, u8), (Date, Vec<usize>)> = BTreeMap::new();
    for &i in &user {
        if let Decider::User(d) = attrs[i].decider {
            by_date.entry(d.key()).or_insert((d, Vec::new())).1.push(i);
        }
    }
    for (_, (date, list)) in by_date.iter().rev() {
        s.push_str(&format!("\n### {date}\n\n"));
        for &i in list {
            let e = &entries[i];
            s.push_str(&format!(
                "- **`{}`**: {} ({}, {})\n",
                e.id,
                e.title,
                e.kind,
                entry_link(e)
            ));
            s.push_str(&user_words_line(&entry_user_quotes(e, attrs[i].field)));
            let mut marks: Vec<String> = low_confidence(e)
                .iter()
                .map(|(m, ctx)| format!("\"{m}\" ({ctx})"))
                .collect();
            if revisit_ids.contains(e.id.as_str()) {
                marks.push("on the \"Revisit later\" list".to_string());
            }
            if !marks.is_empty() {
                s.push_str(&format!("  - Low confidence: {}\n", marks.join("; ")));
            }
            if attrs[i].field != "Decision" {
                s.push_str(&format!(
                    "  - Who decided: {} (in the entry's {} line)\n",
                    attrs[i].clause, attrs[i].field
                ));
            }
            s.push_str(&format!("  - Decision: {}\n", field(e, "Decision")));
            s.push_str(&format!("  - Alternatives: {}\n", field(e, "Alternatives")));
        }
    }

    // 2. Decided by the sources.
    s.push_str("\n## Decided by the sources\n\n");
    s.push_str("The basis is the entry's own attribution.\n\n");
    for &i in &sources {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}). Basis: {}.\n",
            e.id,
            e.title,
            entry_link(e),
            attrs[i].clause
        ));
    }
    if sources.is_empty() {
        s.push_str("None.\n");
    }

    // 2b. Decided by the coordinator or a worker.
    s.push_str("\n## Decided by the coordinator or a worker\n\n");
    s.push_str(
        "Who decided is the entry's attribution where it names one; the basis is the first \
         sentence of its Why line.\n\n### Named in the entry\n\n",
    );
    for &i in &coordinator {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}). Decided: {}. Basis: {}\n",
            e.id,
            e.title,
            entry_link(e),
            attrs[i].clause,
            first_sentence(field(e, "Why"))
        ));
    }
    if coordinator.is_empty() {
        s.push_str("None.\n");
    }
    s.push_str(
        "\n### Naming no decider\n\nDecided as they were written, by the worker who wrote \
         them and the review that landed them.\n\n",
    );
    for &i in &unstated {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}, {}). Basis: {}\n",
            e.id,
            e.title,
            e.kind,
            entry_link(e),
            first_sentence(field(e, "Why"))
        ));
    }
    if unstated.is_empty() {
        s.push_str("None.\n");
    }

    // 3. Provisional.
    s.push_str("\n## Provisional\n\nAwaiting the user.\n\n");
    for &i in &provisional {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}, {})\n",
            e.id,
            e.title,
            e.kind,
            entry_link(e)
        ));
        s.push_str(&user_words_line(&entry_user_quotes(e, "Decision")));
        s.push_str(&format!("  - Question: {}\n", field(e, "Question")));
        s.push_str(&format!("  - Reading proposed: {}\n", field(e, "Decision")));
    }
    if provisional.is_empty() {
        s.push_str("None.\n");
    }

    // 4. Revisit later.
    s.push_str("\n## Revisit later\n\n");
    match &revisit {
        Some(r) => {
            s.push_str(&format!(
                "From [the finish-line plan](agents/finish-line-plan.md#{}), \"{}\".",
                anchor(&r.heading),
                r.heading
            ));
            for line in &r.intro {
                s.push(' ');
                s.push_str(line.trim());
            }
            s.push_str("\n\n");
            for b in &r.bullets {
                s.push_str(b);
                s.push('\n');
            }
        }
        None => s.push_str(&format!(
            "No \"Revisit later\" section found in `{PLAN}`.\n"
        )),
    }

    // 5. Ruling discrepancies.
    s.push_str("\n## Ruling discrepancies\n\n");
    s.push_str(
        "From [`docs/RULING_DISCREPANCIES.md`](RULING_DISCREPANCIES.md), where each entry \
         quotes the ruling and the source it conflicts with, the evidence and what the \
         engine does. Statuses: *open for review* (not yet checked by a human), *decided* \
         (checked by the user), *reported* (to Wizards), *confirmed* (Wizards changed or \
         confirmed the ruling). The dates are the rulings' publication dates.\n\n",
    );
    let anchors = heading_anchors(log);
    let kind_of = |e: &LogEntry| e.field("Kind").unwrap_or("").trim().to_lowercase();
    let status_of = |e: &LogEntry| e.field("Status").unwrap_or("").trim().to_string();
    let mut kinds: Vec<String> = discrepancies::KINDS.iter().map(|k| k.to_string()).collect();
    for e in &log_entries {
        let k = kind_of(e);
        if !kinds.contains(&k) {
            kinds.push(k);
        }
    }
    let mut statuses: Vec<String> = discrepancies::STATUSES
        .iter()
        .map(|k| k.to_string())
        .collect();
    for e in &log_entries {
        let st = status_of(e).to_lowercase();
        if !statuses.contains(&st) {
            statuses.push(st);
        }
    }
    s.push_str("| Kind |");
    for st in &statuses {
        s.push_str(&format!(" {st} |"));
    }
    s.push_str(" Total |\n|---|");
    for _ in &statuses {
        s.push_str("---:|");
    }
    s.push_str("---:|\n");
    for k in &kinds {
        let of_kind: Vec<&LogEntry> = log_entries.iter().filter(|e| &kind_of(e) == k).collect();
        if of_kind.is_empty() {
            continue;
        }
        s.push_str(&format!("| {k} |"));
        for st in &statuses {
            let n = of_kind
                .iter()
                .filter(|e| &status_of(e).to_lowercase() == st)
                .count();
            s.push_str(&format!(" {n} |"));
        }
        s.push_str(&format!(" {} |\n", of_kind.len()));
    }
    s.push_str(&format!("| all |{}", ""));
    for st in &statuses {
        let n = log_entries
            .iter()
            .filter(|e| &status_of(e).to_lowercase() == st)
            .count();
        s.push_str(&format!(" {n} |"));
    }
    s.push_str(&format!(" {} |\n", log_entries.len()));
    for k in &kinds {
        let of_kind: Vec<&LogEntry> = log_entries.iter().filter(|e| &kind_of(e) == k).collect();
        if of_kind.is_empty() {
            continue;
        }
        s.push_str(&format!("\n### {} ({})\n\n", capitalize(k), of_kind.len()));
        s.push_str("| Entry | Status | Card | Ruling date |\n|---|---|---|---|\n");
        for e in of_kind {
            let mut cards: Vec<&str> = Vec::new();
            for r in &e.rulings {
                if !cards.contains(&r.card.as_str()) {
                    cards.push(&r.card);
                }
            }
            let mut dates: Vec<&str> = e.rulings.iter().map(|r| r.date.as_str()).collect();
            dates.sort();
            dates.dedup();
            let cards = if cards.len() > 3 {
                format!("{} and {} more", cards[..3].join(", "), cards.len() - 3)
            } else {
                cards.join(", ")
            };
            let dates = match dates.len() {
                0 => String::new(),
                1..=3 => dates.join(", "),
                n => format!("{} to {} ({n} dates)", dates[0], dates[n - 1]),
            };
            let link = anchors
                .get(&e.line)
                .map(|a| format!("RULING_DISCREPANCIES.md#{a}"))
                .unwrap_or_else(|| "RULING_DISCREPANCIES.md".to_string());
            s.push_str(&format!(
                "| [{}]({link}) | {} | {} | {} |\n",
                cell(&e.title),
                cell(&status_of(e)),
                cell(&cards),
                dates
            ));
        }
    }

    // 6. Policies the user set.
    s.push_str("\n## Policies the user set\n\n");
    s.push_str(
        "The user-decided entries of kind policy whose Affects line names more than one case \
         (a list or a class of cards, not a single card): standing rules for whole classes of \
         cases, each also listed under its date above.\n\n",
    );
    for &i in &policies {
        let e = &entries[i];
        let Decider::User(d) = attrs[i].decider else {
            continue;
        };
        s.push_str(&format!(
            "- **`{}`** ({d}, {}): {}\n",
            e.id,
            entry_link(e),
            e.title
        ));
        s.push_str(&user_words_line(&entry_user_quotes(e, attrs[i].field)));
    }
    if policies.is_empty() {
        s.push_str("None.\n");
    }

    // 7. Needs a source line.
    s.push_str("\n## Needs a source line\n\n");
    s.push_str(
        "Decided entries whose words about who decided can't be read. Give each a \
         \"Decided by the user, <date>.\", \"Decided by the ruling (...).\" or \"Decided by the \
         coordinator, <date>.\" sentence in its Decision line.\n\n",
    );
    for &i in &unparsed {
        let e = &entries[i];
        let Decider::Unparsed(why) = &attrs[i].decider else {
            continue;
        };
        s.push_str(&format!(
            "- **`{}`**: {} ({}). {}: \"{}\" ({} line).\n",
            e.id,
            e.title,
            entry_link(e),
            capitalize(why),
            attrs[i].clause,
            attrs[i].field
        ));
    }
    if unparsed.is_empty() {
        s.push_str("None.\n");
    }
    s
}

/// Renders `docs/DECISIONS.md` from the files under `root`.
pub fn render(root: &Path) -> String {
    let entries = interpretations::load(&root.join("docs/interpretations"));
    let plan = std::fs::read_to_string(root.join(PLAN)).unwrap_or_default();
    let log = std::fs::read_to_string(root.join(discrepancies::LOG)).unwrap_or_default();
    render_from(&entries, &plan, &log)
}

/// The GitHub anchor of every heading of a Markdown text, by 1-based line (a repeated
/// heading's anchor gets "-1", "-2", ... as GitHub gives it).
fn heading_anchors(text: &str) -> BTreeMap<usize, String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if !line.starts_with('#') {
            continue;
        }
        let a = anchor(line.trim_start_matches('#'));
        let n = seen.entry(a.clone()).or_insert(0);
        let a = if *n == 0 { a } else { format!("{a}-{n}") };
        *n += 1;
        out.insert(i + 1, a);
    }
    out
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// A table cell: pipes escaped.
fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}
