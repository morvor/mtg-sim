//! Parts of [`super::normalize`] about how a card refers to itself (CR 201.5: text that
//! refers to the object it's on by name means just that object):
//!
//! - a name that's also a subtype ("Gideon", "Jace", "Assembly-Worker") is the card only
//!   where it isn't used as that subtype ("a Gideon planeswalker", "target Assembly-Worker
//!   creature", "Jaces you control", "empower Jace X");
//! - "Icingdeath, Frost Tongue" is another object's name, not the card's;
//! - legendary characters are "he", "she", "him", "her" and "his" (Fblthp, Impossibly Lost:
//!   "~'s owner shuffles him into their library");
//! - plural names take plural verbs and pronouns ("When The Fantastic Four enter", "if
//!   they're not a token, create two tokens that are copies of them");
//! - numbers with a thousands separator ("1,000 or more time counters").

/// Whether `c` continues a word.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '-'
}

/// "1,000" → "1000" (digits grouped by commas, CR 107.1).
pub fn thousands(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let c = b[i];
        if c == b','
            && i > 0
            && b[i - 1].is_ascii_digit()
            && i + 3 < s.len() + 1
            && b.get(i + 1..i + 4)
                .is_some_and(|d| d.iter().all(u8::is_ascii_digit))
            && !b.get(i + 4).is_some_and(u8::is_ascii_digit)
        {
            // The digits before form a group of one to three ("10,000", not "{2},000").
            let run = b[..i]
                .iter()
                .rev()
                .take_while(|d| d.is_ascii_digit())
                .count();
            let before = i.checked_sub(run + 1).map(|j| b[j]);
            if run <= 3 && !before.is_some_and(|x| x == b'{' || x == b',') {
                i += 1;
                continue;
            }
        }
        // Push the whole UTF-8 character.
        let ch = s[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Words before a subtype that make it a subtype, not a name: "a Gideon planeswalker",
/// "each Garruk you control", "target Assembly-Worker creature", "empower Jace X".
const SUBTYPE_DETERMINERS: [&str; 13] = [
    "a", "an", "each", "target", "another", "other", "empower", "among", "no", "any", "all",
    "every", "non",
];

/// Words after a subtype that make it a subtype: "Gideon planeswalker", "Assembly-Worker
/// creature".
const SUBTYPE_NOUNS: [&str; 8] = [
    "planeswalker",
    "planeswalkers",
    "creature",
    "creatures",
    "token",
    "tokens",
    "card",
    "cards",
];

/// Replaces whole-word occurrences of the card's name `name` by `~`, except where `name`
/// is used as a subtype (`subtype`: the name is one) or is the start of another object's
/// name ("Icingdeath, Frost Tongue").
pub fn replace_self_name(s: &str, name: &str, subtype: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = s[i..].find(name) {
        let start = i + pos;
        let end = start + name.len();
        let before = s[..start].chars().next_back();
        let after = s[end..].chars().next();
        let rest = &s[end..];
        let prev_word = s[..start]
            .trim_end()
            .rsplit(|c: char| !is_word(c))
            .next()
            .unwrap_or("")
            .to_lowercase();
        let next_word = rest
            .trim_start()
            .split(|c: char| !is_word(c))
            .next()
            .unwrap_or("");
        let as_subtype = subtype
            && ((before.is_some_and(|c| c == ' ')
                && SUBTYPE_DETERMINERS.contains(&prev_word.as_str()))
                || (rest.starts_with(' ') && SUBTYPE_NOUNS.contains(&next_word)));
        // "Icingdeath, Frost Tongue": another name that starts with this one.
        let other_name = rest
            .strip_prefix(", ")
            .is_some_and(|r| r.starts_with(|c: char| c.is_uppercase()))
            && !rest.starts_with(", ~");
        out.push_str(&s[i..start]);
        if before.is_some_and(is_word) || after.is_some_and(is_word) || as_subtype || other_name {
            out.push_str(name);
        } else {
            out.push('~');
        }
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

/// A word after "her" that makes it an object ("on her.", "attach her to", "her and"),
/// not a possessive ("her power").
const AFTER_OBJECT_HER: [&str; 22] = [
    "to", "into", "onto", "from", "on", "and", "or", "with", "this", "until", "for", "as", "at",
    "if", "unless", "then", "where", "instead", "gains", "the", "a", "an",
];

/// Replaces a legendary character's gendered pronouns by the neuter ones (CR 201.5:
/// they mean the card itself).
pub fn neuter_pronouns(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let words: [(&str, &str); 13] = [
        ("he's", "it's"),
        ("she's", "it's"),
        ("himself", "itself"),
        ("herself", "itself"),
        ("him", "it"),
        ("his", "its"),
        ("he", "it"),
        ("she", "it"),
        ("He's", "It's"),
        ("She's", "It's"),
        ("He", "It"),
        ("She", "It"),
        ("His", "Its"),
    ];
    while i < s.len() {
        let c = s[i..].chars().next().unwrap_or(' ');
        let at_word_start = s[..i]
            .chars()
            .next_back()
            .is_none_or(|p| !is_word(p) && p != '\'');
        if at_word_start && c.is_alphabetic() {
            let rest = &s[i..];
            let word_len = rest
                .find(|c: char| !(is_word(c) || c == '\''))
                .unwrap_or(rest.len());
            let word = &rest[..word_len];
            if let Some((_, rep)) = words.iter().find(|(w, _)| *w == word) {
                out.push_str(rep);
                i += word_len;
                continue;
            }
            if word == "her" || word == "Her" {
                let next = rest[word_len..]
                    .trim_start()
                    .split(|c: char| !is_word(c))
                    .next()
                    .unwrap_or("");
                let ends = rest[word_len..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric() && c != ' ');
                let object = ends || AFTER_OBJECT_HER.contains(&next);
                let rep = match (object, word == "Her") {
                    (true, false) => "it",
                    (true, true) => "It",
                    (false, false) => "its",
                    (false, true) => "Its",
                };
                out.push_str(rep);
                i += word_len;
                continue;
            }
            out.push_str(word);
            i += word_len;
            continue;
        }
        out.push(c);
        i += c.len_utf8();
    }
    out
}

/// A plural name's verbs and pronouns, on the lines that use them ("When ~ enter, if
/// they're not a token, create two tokens that are copies of them" → "When ~ enters, if
/// it's not a token, create two tokens that are copies of it").
pub fn singular_self(s: &str) -> String {
    const VERBS: [(&str, &str); 6] = [
        ("~ enter ", "~ enters "),
        ("~ enter,", "~ enters,"),
        ("~ die,", "~ dies,"),
        ("~ die ", "~ dies "),
        ("~ deal ", "~ deals "),
        ("~ attack,", "~ attacks,"),
    ];
    s.lines()
        .map(|line| {
            if !VERBS.iter().any(|(p, _)| line.contains(p)) {
                return line.to_string();
            }
            let mut l = line.to_string();
            for (p, r) in VERBS {
                l = l.replace(p, r);
            }
            let mut out = String::new();
            let mut rest = l.as_str();
            while !rest.is_empty() {
                let n = rest
                    .find(|c: char| !(is_word(c) || c == '\''))
                    .unwrap_or(rest.len());
                let n = if n == 0 {
                    rest.chars().next().map_or(1, char::len_utf8)
                } else {
                    n
                };
                let w = &rest[..n];
                let next = rest[n..].trim_start();
                let rep = match w {
                    "they're" => "it's",
                    "They're" => "It's",
                    "them" => "it",
                    "they" if next.starts_with("were ") => "it",
                    "They" if next.starts_with("were ") => "It",
                    "they" | "They" => {
                        out.push_str(if w == "they" { "it" } else { "It" });
                        // "they are" → "it is"
                        if let Some(r) = next.strip_prefix("are ") {
                            out.push_str(" is ");
                            rest = r;
                            continue;
                        }
                        rest = &rest[n..];
                        continue;
                    }
                    "were" if out.ends_with("it ") || out.ends_with("It ") => "was",
                    "their" => "its",
                    _ => w,
                };
                out.push_str(rep);
                rest = &rest[n..];
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands("10,000 Needles"), "10000 Needles");
        assert_eq!(thousands("loses 1,000 life"), "loses 1000 life");
        assert_eq!(thousands("1, 2, or 3"), "1, 2, or 3");
        assert_eq!(thousands("{2},000"), "{2},000");
        assert_eq!(thousands("1,0000"), "1,0000");
    }

    #[test]
    fn subtype_names() {
        assert_eq!(
            replace_self_name(
                "As long as you control a Gideon planeswalker, Gideon wins",
                "Gideon",
                true
            ),
            "As long as you control a Gideon planeswalker, ~ wins"
        );
        assert_eq!(
            replace_self_name(
                "+1: Empower Jace X. Among Jaces you control. Jace deals",
                "Jace",
                true
            ),
            "+1: Empower Jace X. Among Jaces you control. ~ deals"
        );
        assert_eq!(
            replace_self_name(
                "{T}: Target Assembly-Worker creature gets +1/+1",
                "Assembly-Worker",
                true
            ),
            "{T}: Target Assembly-Worker creature gets +1/+1"
        );
        assert_eq!(
            replace_self_name(
                "put a loyalty counter on each Garruk you control",
                "Garruk",
                true
            ),
            "put a loyalty counter on each Garruk you control"
        );
        assert_eq!(
            replace_self_name(
                "When Icingdeath dies, create Icingdeath, Frost Tongue, a token",
                "Icingdeath",
                false
            ),
            "When ~ dies, create Icingdeath, Frost Tongue, a token"
        );
    }

    #[test]
    fn gendered_pronouns() {
        assert_eq!(
            neuter_pronouns("~'s owner shuffles him into their library."),
            "~'s owner shuffles it into their library."
        );
        assert_eq!(
            neuter_pronouns("As long as ~ has seven counters on him, he's a 7/7 Dragon"),
            "As long as ~ has seven counters on it, it's a 7/7 Dragon"
        );
        assert_eq!(
            neuter_pronouns("put a counter on her. Then she deals damage equal to her power"),
            "put a counter on it. Then it deals damage equal to its power"
        );
        assert_eq!(
            neuter_pronouns("Sarkhan deals damage to himself"),
            "Sarkhan deals damage to itself"
        );
        assert_eq!(neuter_pronouns("other Heroes"), "other Heroes");
        assert_eq!(
            neuter_pronouns("She's a land named Moon."),
            "It's a land named Moon."
        );
    }

    #[test]
    fn plural_names() {
        assert_eq!(
            singular_self("When ~ enter, if they're not a token, create two tokens that are copies of them, except the tokens aren't legendary."),
            "When ~ enters, if it's not a token, create two tokens that are copies of it, except the tokens aren't legendary."
        );
        assert_eq!(
            singular_self("When ~ die, if they were a creature, return them to the battlefield. They're an artifact."),
            "When ~ dies, if it was a creature, return it to the battlefield. It's an artifact."
        );
        assert_eq!(
            singular_self("Each player discards their hand."),
            "Each player discards their hand."
        );
    }
}
