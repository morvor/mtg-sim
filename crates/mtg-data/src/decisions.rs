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
//!   written; one whose words name a decider this module can't read, or mention the user
//!   deciding in another form ([`unread_user_mention`]), is listed under "needs a source
//!   line", never guessed. Each decision is shown in brief (the opening sentences of its
//!   fields, [`excerpt`]), with a link to the full entry;
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

/// How much of a field the log shows: its first sentence, and the next ones while the
/// excerpt is shorter than the first number of bytes and they keep it within the second.
const BRIEF: (usize, usize) = (200, 500);

/// How much of a field a one-line basis shows: one sentence, at most this many bytes.
const BASIS: (usize, usize) = (1, 400);

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
        let m = MONTH_NAMES[usize::from(self.month.clamp(1, 12) - 1)];
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

/// True if `before` (the text before a period) ends with an abbreviation the period belongs
/// to ("e.g", "i.e", "cf", "vs", a month's: "Nov. 5").
fn ends_with_abbreviation(before: &str) -> bool {
    let word = &before[before
        .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '.')
        .len()..];
    let w = word.to_ascii_lowercase();
    [
        "e.g", "i.e", "cf", "vs", "jan", "feb", "mar", "apr", "jun", "jul", "aug", "sep", "sept",
        "oct", "nov", "dec",
    ]
    .contains(&w.as_str())
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
            '.' if depth == 0 && ends_with_abbreviation(&s[..at]) => {}
            '.' | ':' | ';' if depth == 0 && s[at + 1..].starts_with(' ') => return at,
            _ => {}
        }
    }
    from + s[from..].trim_end_matches('.').len()
}

/// True if the text before a clause opener ends with a backticked word ("`other-id` (" or
/// "`other-id`, "): the clause then describes that entry or thing, not this one.
fn after_backticked_word(before: &str) -> bool {
    before
        .trim_end_matches(['(', ',', ';'])
        .trim_end()
        .ends_with('`')
}

/// Where the text names a decider: the start of "Decided by"/"Decided in" (a sentence's
/// start), else of a lowercase "decided by"/"decided in" opening a clause ("(option B,
/// decided by the coordinator ..."), unless the clause describes a backticked entry before
/// it ("`other-id` (decided by the user, Nov 9)"). Not "not decided by", "would be decided
/// by", ...
fn find_attribution(text: &str) -> Option<usize> {
    if let Some(p) = ["Decided by ", "Decided in "]
        .iter()
        .filter_map(|pat| text.find(pat))
        .min()
    {
        return Some(p);
    }
    let mut from = 0;
    while let Some(p) = ["decided by ", "decided in "]
        .iter()
        .filter_map(|pat| text[from..].find(pat))
        .min()
    {
        let at = from + p;
        let before = text[..at].trim_end_matches(' ');
        if (before.is_empty() || before.ends_with(['(', ',', ';']))
            && !after_backticked_word(before)
        {
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
    let rest = text[at + "decided by ".len()..].trim_start();
    let in_review = text[at..].to_ascii_lowercase().starts_with("decided in ");
    // The decider's words, up to a parenthesis or the clause's end.
    let phrase_end = [" (", ",", ": ", ". ", ";"]
        .iter()
        .filter_map(|p| rest.find(p))
        .min()
        .unwrap_or(rest.len());
    let phrase = rest[..phrase_end].trim_end().trim_end_matches(['.', ':']);
    let lower = phrase.to_ascii_lowercase();
    let decider = if in_review {
        if lower.contains("review") {
            Decider::Coordinator
        } else {
            Decider::Unparsed(format!("\"decided in {phrase}\" names no decider"))
        }
    } else if let Some(after) = lower.strip_prefix("the user") {
        let date_text = rest["the user".len()..].trim_start_matches([',', ' ', '(']);
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

/// Who decided an entry: the first attribution ("Decided by ...", "Decided in the review
/// ...") in its `Decision` field, else in its `Why` field; [`Decider::Unstated`] when neither
/// has one.
pub fn attribute(e: &Entry) -> Attribution {
    for f in ["Decision", "Why"] {
        let text = field(e, f);
        if let Some(at) = find_attribution(text) {
            return classify(text, at, f);
        }
    }
    Attribution {
        decider: Decider::Unstated,
        clause: String::new(),
        field: "",
    }
}

/// Verbs that make "the user <verb>" or "<verb> by the user" a decision of the user's.
const USER_DECISION_VERBS: &[&str] = &[
    "accepted",
    "agreed",
    "approved",
    "chose",
    "chosen",
    "confirmed",
    "decided",
    "rejected",
    "set",
];

/// True if `sentence` names a register entry other than `id` (in backticks).
fn names_another_entry(sentence: &str, id: &str, ids: &BTreeSet<&str>) -> bool {
    backticked(sentence)
        .iter()
        .any(|w| *w != id && ids.contains(w))
}

/// A mention of the user deciding that [`attribute`] doesn't read, in a sentence of the
/// entry's `Decision` or `Why` that names no other register entry: "the user" followed by
/// a date ("(to be fixed later; the user, Nov 12)", "the user on Nov 12"), "the user's
/// decision", "the user <verb>" or "<verb> by the user" (accepted, agreed, approved, chose,
/// confirmed, decided, rejected, set), not within three words after "not" or "never".
/// Returns the field and the sentence. For an entry [`attribute`] finds no decider in, this
/// means the decider is named in a form the log can't read.
pub fn unread_user_mention<'a>(
    e: &'a Entry,
    ids: &BTreeSet<&str>,
) -> Option<(&'static str, &'a str)> {
    for f in ["Decision", "Why"] {
        let text = field(e, f);
        for (s, t) in sentence_spans(text) {
            let sentence = &text[s..t];
            if names_another_entry(sentence, &e.id, ids) {
                continue;
            }
            // ASCII lowercasing keeps byte offsets.
            let lower = sentence.to_ascii_lowercase();
            for (p, _) in lower.match_indices("the user") {
                let after = &lower[p + "the user".len()..];
                // The three words before "the user", nearest first.
                let before_words: Vec<&str> = lower[..p]
                    .split_whitespace()
                    .rev()
                    .take(3)
                    .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\''))
                    .collect();
                if before_words
                    .iter()
                    .any(|w| ["not", "never", "wasn't", "isn't", "hasn't"].contains(w))
                {
                    continue;
                }
                let dated = [",", " (", " on", ""]
                    .iter()
                    .any(|sep| after.strip_prefix(sep).and_then(parse_date).is_some());
                let decision = ["'s decision", "’s decision", "'s answer", "’s answer"]
                    .iter()
                    .any(|w| after.starts_with(w));
                let verb_after = after
                    .strip_prefix(' ')
                    .and_then(|a| a.split(|c: char| !c.is_alphanumeric()).next())
                    .is_some_and(|w| USER_DECISION_VERBS.contains(&w));
                let verb_before = before_words.first() == Some(&"by")
                    && before_words
                        .get(1)
                        .is_some_and(|w| USER_DECISION_VERBS.contains(w));
                if dated || decision || verb_after || verb_before {
                    return Some((f, sentence));
                }
            }
        }
    }
    None
}

/// The attribution of an entry, as the log files it: [`attribute`]'s, except that an entry
/// naming no decider but mentioning the user deciding ([`unread_user_mention`]) is
/// [`Decider::Unparsed`].
pub fn attribution_for_log(e: &Entry, ids: &BTreeSet<&str>) -> Attribution {
    let a = attribute(e);
    if a.decider != Decider::Unstated {
        return a;
    }
    match unread_user_mention(e, ids) {
        Some((f, sentence)) => Attribution {
            decider: Decider::Unparsed(
                "mentions the user deciding without \"Decided by the user, <date>\"".to_string(),
            ),
            clause: excerpt(sentence, BASIS.0, BASIS.1),
            field: f,
        },
        None => a,
    }
}

/// True if the straight quote at byte `i` of `s` opens a quotation: it follows the start,
/// a space or an opening bracket, and a non-space follows it.
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

/// True if `rest` (the text after a sentence's terminator and closing marks) starts a new
/// sentence: the end of the text, or a space and then a capital letter, a backtick, a quote
/// or an opening bracket, or a digit (unless the terminator follows a digit: "is 2. 3").
fn starts_a_sentence(before: &str, rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    let Some(after) = rest.strip_prefix(' ') else {
        return false;
    };
    match after.trim_start().chars().next() {
        None => true,
        Some(n) if n.is_uppercase() || "`(\"“[*".contains(n) => true,
        Some(n) if n.is_ascii_digit() => !before.ends_with(|c: char| c.is_ascii_digit()),
        _ => false,
    }
}

/// The sentences of `text`, as byte spans (start, end): a sentence ends at a ".", "!" or "?"
/// (with the closing quotes and brackets after it) outside quotations and brackets, where
/// [`starts_a_sentence`] holds after it; not at "..." or after an abbreviation ("e.g.",
/// "Nov. 5").
pub fn sentence_spans(text: &str) -> Vec<(usize, usize)> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let skip_spaces = |from: usize| from + (text[from..].len() - text[from..].trim_start().len());
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = skip_spaces(0);
    let mut k = 0;
    while k < chars.len() {
        let (i, c) = chars[k];
        match c {
            '(' | '[' | '“' => depth += 1,
            ')' | ']' | '”' => depth = depth.saturating_sub(1),
            '"' if is_open_quote_at(text, i) => depth += 1,
            '"' => depth = depth.saturating_sub(1),
            '.' | '!' | '?' => {
                // The terminator and the closing marks after it.
                let mut m = k + 1;
                let mut d = depth;
                while let Some(&(j, ch)) = chars.get(m) {
                    match ch {
                        ')' | ']' | '”' => d = d.saturating_sub(1),
                        '"' if !is_open_quote_at(text, j) => d = d.saturating_sub(1),
                        _ => break,
                    }
                    m += 1;
                }
                let end = chars.get(m).map_or(text.len(), |&(j, _)| j);
                let before = &text[..i];
                let ellipsis = before.ends_with('.') || text[i + 1..].starts_with('.');
                let abbreviation = c == '.' && ends_with_abbreviation(before);
                if d == 0 && !ellipsis && !abbreviation && starts_a_sentence(before, &text[end..]) {
                    if end > start {
                        out.push((start, end));
                    }
                    start = skip_spaces(end);
                    depth = 0;
                    k = m;
                    continue;
                }
            }
            _ => {}
        }
        k += 1;
    }
    let last = text.trim_end().len();
    if last > start {
        out.push((start, last));
    }
    out
}

/// A byte offset at most `max` into `text` to cut it at: the last space outside quotations
/// and brackets if it's past the middle, else the last space, else `max`.
fn cut_at_space(text: &str, max: usize) -> usize {
    let mut m = max.min(text.len());
    while !text.is_char_boundary(m) {
        m -= 1;
    }
    let mut depth = 0usize;
    let (mut outside, mut any) = (None, None);
    for (i, c) in text[..m].char_indices() {
        match c {
            '(' | '[' | '“' => depth += 1,
            ')' | ']' | '”' => depth = depth.saturating_sub(1),
            '"' if is_open_quote_at(text, i) => depth += 1,
            '"' => depth = depth.saturating_sub(1),
            ' ' => {
                any = Some(i);
                if depth == 0 {
                    outside = Some(i);
                }
            }
            _ => {}
        }
    }
    match (outside, any) {
        (Some(o), _) if o >= m / 2 => o,
        (_, Some(a)) => a,
        _ => m,
    }
}

/// The opening of `text`, in brief: its first sentence, then the next ones while the excerpt
/// is shorter than `min` bytes and they keep it within `max`; a sentence that would take it
/// past `max` is cut at a space instead if the excerpt is still shorter than `min / 2` (a
/// first sentence longer than `max` always is). A trailing " …" marks that the text goes
/// on.
pub fn excerpt(text: &str, min: usize, max: usize) -> String {
    let text = text.trim();
    let spans = sentence_spans(text);
    let Some(&(_, first)) = spans.first() else {
        return String::new();
    };
    let mut end = first;
    for &(_, e) in &spans[1..] {
        if end >= min {
            break;
        }
        if e > max {
            if end < min / 2 {
                end = e;
            }
            break;
        }
        end = e;
    }
    if end > max {
        let cut = cut_at_space(text, max);
        return format!(
            "{} …",
            text[..cut].trim_end_matches([',', ';', ':', ' ', '(', '—'])
        );
    }
    let shown = text[..end].trim_end();
    if end < text.len() {
        format!("{shown} …")
    } else {
        shown.to_string()
    }
}

/// A quotation a field gives as the user's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserQuote {
    /// The quotation's byte span in the field, quotes included.
    pub start: usize,
    pub end: usize,
    /// The words inside the quotes.
    pub text: String,
    /// The words that introduce it, from the start of their sentence ("As recommended; the
    /// user", "The user's decision of Nov 12"), without the ":" or "(" before the quote.
    pub lead: String,
}

/// True if `lead` (the text of a sentence up to a quotation) introduces the user's words:
/// it ends with ":" or "(" and names the user as the speaker ("the user: ", "Decided by the
/// user, Nov 19: ", "(the user, Nov 19: ", "Decided by the user, Nov 19 (", "The user's
/// reading (Nov 19): ") or is an acceptance ("Accepted tentatively (", "As recommended: ").
fn introduces_user_words(lead: &str) -> bool {
    let t = lead.trim_end();
    if !t.ends_with([':', '(']) {
        return false;
    }
    // ASCII lowercasing keeps byte offsets.
    let lower = t.to_ascii_lowercase();
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
            || ((after.starts_with("'s ") || after.starts_with("’s ")) && t.ends_with(':'))
            || parse_date(after).is_some()
    });
    let acceptance = ["accepted", "accepting", "as recommended"]
        .iter()
        .any(|w| lower.trim_start().starts_with(w))
        && lower.split_whitespace().count() <= 4;
    speaker || acceptance
}

/// The quotations of a field that a sentence introduces as the user's words
/// ([`introduces_user_words`]), in order.
pub fn user_quote_spans(text: &str) -> Vec<UserQuote> {
    let mut out = Vec::new();
    let mut prev_end = 0;
    for (start, end, inner) in quotations(text) {
        let between = &text[prev_end..start];
        // The lead starts after the sentence (or the Rules line's " / " part) before it.
        let from = [
            between.rfind(". ").map(|p| p + 2),
            between.rfind(" / ").map(|p| p + 3),
        ]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(0);
        let lead = &between[from..];
        if introduces_user_words(lead) {
            out.push(UserQuote {
                start,
                end,
                text: inner,
                lead: lead
                    .trim()
                    .trim_end_matches([':', '('])
                    .trim_end()
                    .to_string(),
            });
        }
        prev_end = end;
    }
    out
}

/// The user's own words that a field quotes ([`user_quote_spans`]), in order.
pub fn user_quotes(text: &str) -> Vec<String> {
    user_quote_spans(text).into_iter().map(|q| q.text).collect()
}

/// The user's words an entry quotes: those of `first` (the field its attribution is in),
/// else of its other fields (Decision, Why, Rules), with the field they came from.
pub fn entry_user_quotes(e: &Entry, first: &str) -> (String, Vec<UserQuote>) {
    let mut fields = vec![if first.is_empty() { "Decision" } else { first }];
    for f in ["Decision", "Why", "Rules"] {
        if !fields.contains(&f) {
            fields.push(f);
        }
    }
    for f in fields {
        let quotes = user_quote_spans(field(e, f));
        if !quotes.is_empty() {
            return (f.to_string(), quotes);
        }
    }
    (String::new(), Vec::new())
}

/// Words that mark a decision the user or the entry holds with low confidence.
pub const LOW_CONFIDENCE: &[&str] = &[
    "for now",
    "tentative",
    "not sure",
    "not very sure",
    "not entirely sure",
    "unsure",
    "low confidence",
    "low-confidence",
    "lower confidence",
    "lower-confidence",
    "to be fixed later",
];

/// The low-confidence marks ([`LOW_CONFIDENCE`]) in an entry's `Decision` and `Why`, each
/// with the words around its first occurrence (a reviewer judges whether it is about this
/// decision). An occurrence in a sentence naming another register entry of `ids` is about
/// that entry, and isn't counted, unless the sentence also says "this decision" (or "this
/// entry", "this reading", "this call").
pub fn low_confidence(e: &Entry, ids: &BTreeSet<&str>) -> Vec<(&'static str, String)> {
    let mut found: Vec<(&'static str, String)> = Vec::new();
    // The sentences (field, start) a mark was found in: another mark there is the same one.
    let mut seen: Vec<(&str, usize)> = Vec::new();
    for m in LOW_CONFIDENCE {
        if found.iter().any(|(f, _)| f.contains(m) || m.contains(f)) {
            continue;
        }
        'fields: for f in ["Decision", "Why"] {
            let text = field(e, f);
            // ASCII lowercasing keeps byte offsets.
            let lower = text.to_ascii_lowercase();
            let sentences = sentence_spans(text);
            for (p, _) in lower.match_indices(m) {
                let Some(&(s, t)) = sentences.iter().find(|&&(s, t)| s <= p && p < t) else {
                    continue;
                };
                let sentence = &text[s..t];
                let l = sentence.to_ascii_lowercase();
                let about_another = names_another_entry(sentence, &e.id, ids)
                    && !["this decision", "this entry", "this reading", "this call"]
                        .iter()
                        .any(|w| l.contains(w));
                if about_another {
                    continue;
                }
                if !seen.contains(&(f, s)) {
                    seen.push((f, s));
                    found.push((m, context(text, p, p + m.len())));
                }
                break 'fields;
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

/// True if `line` starts a top-level list item ("- ", "* ", "+ " or "1. ").
fn top_level_item(line: &str) -> bool {
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return true;
    }
    let (n, rest) = digits(line);
    !n.is_empty() && (rest.starts_with(". ") || rest.starts_with(") "))
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
        if top_level_item(line) {
            let item = match line.strip_prefix("* ").or_else(|| line.strip_prefix("+ ")) {
                Some(rest) => format!("- {rest}"),
                None => line.to_string(),
            };
            r.bullets.push(item);
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

fn field<'a>(e: &'a Entry, name: &str) -> &'a str {
    e.fields.get(name).map(String::as_str).unwrap_or("")
}

/// The sentence of `text` that holds the attribution `clause` (one line of basis), or the
/// clause if it can't be found.
fn attribution_sentence(text: &str, clause: &str) -> String {
    let Some(p) = text.find(clause) else {
        return clause.to_string();
    };
    sentence_spans(text)
        .into_iter()
        .find(|&(s, t)| s <= p && p < t)
        .map_or_else(
            || clause.to_string(),
            |(s, t)| excerpt(&text[s..t], BASIS.0, BASIS.1),
        )
}

/// True if a sentence only records how the decision was taken, not the reading: an
/// acceptance naming the user ("As recommended: the user agreed to ...") or a note about the
/// "Revisit later" list.
fn is_acceptance_note(sentence: &str) -> bool {
    let l = sentence.to_ascii_lowercase();
    l.contains("revisit later")
        || (["as recommended", "accepted", "accepting"]
            .iter()
            .any(|w| l.starts_with(w))
            && l.contains("the user"))
}

/// Where the reading of a field begins: after an attribution that opens it ("Decided by the
/// user, Nov 20.") and after the opening sentences that only give the user's words ("As
/// recommended; the user: "...".", at most 12 other words) or record the acceptance
/// ([`is_acceptance_note`]); at its start if that leaves nothing.
fn reading_start(text: &str, attr: &Attribution, name: &str) -> usize {
    let mut start = 0;
    let named = matches!(
        attr.decider,
        Decider::User(_) | Decider::Sources | Decider::Coordinator
    );
    if named && attr.field == name && !attr.clause.is_empty() {
        if let Some(p) = text.find(attr.clause.as_str()) {
            if text[..p].trim().is_empty() {
                start = p + attr.clause.len();
            }
        }
    }
    let quotes = user_quote_spans(text);
    loop {
        let rest = &text[start..];
        start += rest.len()
            - rest
                .trim_start_matches(|c: char| c.is_whitespace() || ".,:;)".contains(c))
                .len();
        let Some(&(s, t)) = sentence_spans(&text[start..]).first() else {
            break;
        };
        let (s, t) = (start + s, start + t);
        let inside: Vec<&UserQuote> = quotes
            .iter()
            .filter(|q| q.start >= s && q.end <= t)
            .collect();
        let only_quotes = !inside.is_empty() && {
            let mut other_words = 0;
            let mut at = s;
            for q in &inside {
                other_words += text[at..q.start].split_whitespace().count();
                at = q.end;
            }
            other_words += text[at..t].split_whitespace().count();
            other_words <= 12
        };
        if !only_quotes && !is_acceptance_note(&text[s..t]) {
            break;
        }
        start = t;
    }
    if text[start..].trim().is_empty() {
        0
    } else {
        start
    }
}

/// A field's reading, in brief ([`reading_start`], [`excerpt`]), with a capital letter.
fn brief_reading(e: &Entry, attr: &Attribution, name: &str) -> String {
    let text = field(e, name);
    let brief = excerpt(&text[reading_start(text, attr, name)..], BRIEF.0, BRIEF.1);
    let mut c = brief.chars();
    match c.next() {
        Some(f) if f.is_ascii_lowercase() => f.to_ascii_uppercase().to_string() + c.as_str(),
        _ => brief,
    }
}

/// "The user: "..." / "..."", naming the field and the words introducing them when it isn't
/// the Decision line.
fn user_words_line((field, quotes): &(String, Vec<UserQuote>)) -> String {
    if quotes.is_empty() {
        return String::new();
    }
    let words = quotes
        .iter()
        .map(|q| format!("\"{}\"", q.text))
        .collect::<Vec<_>>()
        .join(" / ");
    if field == "Decision" {
        format!("  - The user: {words}\n")
    } else {
        format!(
            "  - The user (quoted in the entry's {field} line, after \"{}\"): {words}\n",
            quotes[0].lead
        )
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
    let ids: BTreeSet<&str> = entries.iter().map(|e| e.id.as_str()).collect();
    let attrs: Vec<Attribution> = entries
        .iter()
        .map(|e| attribution_for_log(e, &ids))
        .collect();
    let decided = |i: usize| entries[i].status != "provisional";
    let by = |pred: &dyn Fn(&Decider) -> bool| -> Vec<usize> {
        (0..entries.len())
            .filter(|&i| decided(i) && pred(&attrs[i].decider))
            .collect()
    };
    let user = by(&|d| matches!(d, Decider::User(_)));
    let mut policies: Vec<usize> = user
        .iter()
        .copied()
        .filter(|&i| {
            entries[i].kind == "policy" && !affects_a_single_case(field(&entries[i], "Affects"))
        })
        .collect();
    // Newest first, as the log's dates are (a stable sort keeps the register's order).
    policies.sort_by_key(|&i| match attrs[i].decider {
        Decider::User(d) => std::cmp::Reverse(d.key()),
        _ => std::cmp::Reverse((0, 0, 0)),
    });
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
    let on_revisit_list = |e: &Entry| {
        if revisit_ids.contains(e.id.as_str()) {
            " On the \"Revisit later\" list."
        } else {
            ""
        }
    };
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
         by who made the call, so a reviewer can see who decided what. Who decided is read \
         from the entry's own words, \"Decided by the user, <date>\", \"Decided by the ruling \
         (...)\" and the like, in its Decision line (or else its Why line). Each decision is \
         shown in brief: the opening sentences of the entry's lines, cut at a sentence's end \
         (\" …\" marks a cut), and a link to the full entry (the rules quoted, the question, \
         the alternatives and the reasoning).\n\n",
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
             question, and the entry says so; one line each, with that basis.",
        ),
        (
            "Decided by the coordinator or a worker",
            coordinator.len() + unstated.len(),
            "everything else decided: calls the coordinator, a worker or a review made, most of \
             them defaults for choices an agent leaves unanswered. Those whose entry names no \
             decider are listed apart; one line each, with the entry's reasoning.",
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
         \"Reading chosen\" and \"Alternatives not chosen\" open the entry's Decision and \
         Alternatives lines (after the attribution and the user's words).\n",
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
            let mut marks: Vec<String> = low_confidence(e, &ids)
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
                    "  - Who decided (the entry's {} line): {}\n",
                    attrs[i].field,
                    attribution_sentence(field(e, attrs[i].field), &attrs[i].clause)
                ));
            }
            s.push_str(&format!(
                "  - Reading chosen: {}\n",
                brief_reading(e, &attrs[i], "Decision")
            ));
            s.push_str(&format!(
                "  - Alternatives not chosen: {}\n",
                excerpt(field(e, "Alternatives"), BRIEF.0, BRIEF.1)
            ));
        }
    }

    // 2. Decided by the sources.
    s.push_str("\n## Decided by the sources\n\n");
    s.push_str("The basis is the entry's sentence that names the source.\n\n");
    for &i in &sources {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}). Basis: {}{}\n",
            e.id,
            e.title,
            entry_link(e),
            attribution_sentence(field(e, attrs[i].field), &attrs[i].clause),
            on_revisit_list(e)
        ));
    }
    if sources.is_empty() {
        s.push_str("None.\n");
    }

    // 2b. Decided by the coordinator or a worker.
    s.push_str("\n## Decided by the coordinator or a worker\n\n");
    s.push_str(
        "\"Decided\" is the entry's sentence that names who decided, where it names one; the \
         basis is the first sentence of its Why line.\n\n### Named in the entry\n\n",
    );
    for &i in &coordinator {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}). Decided: {} Basis: {}{}\n",
            e.id,
            e.title,
            entry_link(e),
            attribution_sentence(field(e, attrs[i].field), &attrs[i].clause),
            excerpt(field(e, "Why"), BASIS.0, BASIS.1),
            on_revisit_list(e)
        ));
    }
    if coordinator.is_empty() {
        s.push_str("None.\n");
    }
    s.push_str(
        "\n### Naming no decider\n\nThese entries don't say who decided them. The register \
         records the user's decisions (\"Decided by the user, <date>\") and the sources' \
         (\"Decided by the ruling ...\"), so these were decided as they were written: by the \
         worker who wrote the entry, accepted by the review that landed it.\n\n",
    );
    for &i in &unstated {
        let e = &entries[i];
        s.push_str(&format!(
            "- **`{}`**: {} ({}, {}). Basis: {}{}\n",
            e.id,
            e.title,
            e.kind,
            entry_link(e),
            excerpt(field(e, "Why"), BASIS.0, BASIS.1),
            on_revisit_list(e)
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
            "- **`{}`**: {} ({}, {}){}\n",
            e.id,
            e.title,
            e.kind,
            entry_link(e),
            on_revisit_list(e)
        ));
        s.push_str(&user_words_line(&entry_user_quotes(e, "Decision")));
        if !matches!(attrs[i].decider, Decider::Unstated | Decider::Unparsed(_)) {
            s.push_str(&format!(
                "  - Its Status is provisional, but its {} line says \"{}\".\n",
                attrs[i].field, attrs[i].clause
            ));
        }
        s.push_str(&format!(
            "  - Question: {}\n",
            excerpt(field(e, "Question"), BRIEF.0, BRIEF.1)
        ));
        s.push_str(&format!(
            "  - Reading proposed: {}\n",
            brief_reading(e, &attrs[i], "Decision")
        ));
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
            if r.bullets.is_empty() {
                s.push_str("None.\n");
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
         confirmed the ruling). The dates are the rulings' publication dates.",
    );
    let anchors = heading_anchors(log);
    if let Some(a) = anchors
        .values()
        .find(|a| a.starts_with("not-yet-investigated"))
    {
        s.push_str(&format!(
            " Possible further instances that haven't been through the conflict process yet \
             are listed in the log's [\"Not yet investigated\"](RULING_DISCREPANCIES.md#{a}) \
             section; they aren't entries and aren't counted here."
        ));
    }
    s.push_str("\n\n");
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
        s.push_str(&format!(" {} |", cell(st)));
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
        s.push_str(&format!("| {} |", cell(k)));
        for st in &statuses {
            let n = of_kind
                .iter()
                .filter(|e| &status_of(e).to_lowercase() == st)
                .count();
            s.push_str(&format!(" {n} |"));
        }
        s.push_str(&format!(" {} |\n", of_kind.len()));
    }
    s.push_str("| all |");
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
        s.push_str(&format!(
            "\n### {} ({})\n\n",
            capitalize(if k.is_empty() { "no kind" } else { k }),
            of_kind.len()
        ));
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
                cell(&dates)
            ));
        }
    }

    // 6. Policies the user set.
    s.push_str("\n## Policies the user set\n\n");
    s.push_str(
        "The user-decided entries of kind policy whose Affects line names more than one case \
         (a list or a class of cards, not a single card): standing rules for whole classes of \
         cases, newest first, each also listed under its date above.\n\n",
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
