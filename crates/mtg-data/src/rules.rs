//! Parser for the Magic: The Gathering Comprehensive Rules text file.
//!
//! The document has a table of contents, then numbered rules (`100.` … `905.x`),
//! then a glossary and credits. Rule identifiers have three shapes:
//!
//! * section headers — `704.` ("704. State-Based Actions")
//! * rules — `704.5.`
//! * subrules — `704.5a` (letters skip `l` and `o`)
//!
//! Lines starting with `Example:` attach to the preceding rule.

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    /// Top-level chapter ("1. Game Concepts").
    Chapter,
    /// Three-digit section ("704. State-Based Actions").
    Section,
    /// Numbered rule ("704.5.").
    Rule,
    /// Lettered subrule ("704.5a").
    Subrule,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub kind: RuleKind,
    pub text: String,
    pub examples: Vec<String>,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GlossaryEntry {
    pub term: String,
    pub definition: String,
}

#[derive(Debug, Clone)]
pub struct ComprehensiveRules {
    /// e.g. "September 25, 2026"
    pub effective_date: String,
    /// Rules in document order.
    pub order: Vec<String>,
    rules: BTreeMap<String, Rule>,
    pub glossary: Vec<GlossaryEntry>,
}

/// Sort key that orders rule ids the way the document does
/// ("702.9" < "702.10", "704.5z" < "704.5aa").
pub fn rule_sort_key(id: &str) -> (u32, u32, u32, String) {
    let mut parts = id.splitn(2, '.');
    let section: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let rest = parts.next().unwrap_or("");
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    let letters: String = rest.chars().skip(digits.len()).collect();
    let num: u32 = digits.parse().unwrap_or(0);
    (section, num, letters.len() as u32, letters)
}

impl ComprehensiveRules {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::parse(&text))
    }

    pub fn parse(text: &str) -> Self {
        let text = text
            .trim_start_matches('\u{feff}')
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        let effective_date = text
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("These rules are effective as of ")
                    .map(|s| s.trim_end_matches('.').to_string())
            })
            .unwrap_or_default();

        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        // The body starts at the *second* "100. General" (the first is the contents).
        let starts: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| **l == "100. General")
            .map(|(i, _)| i)
            .collect();
        let body_start = *starts.get(1).or(starts.first()).unwrap_or(&0);
        let glossary_start = lines
            .iter()
            .enumerate()
            .skip(body_start)
            .find(|(_, l)| **l == "Glossary")
            .map(|(i, _)| i)
            .unwrap_or(lines.len());
        let credits_start = lines
            .iter()
            .enumerate()
            .skip(glossary_start)
            .find(|(_, l)| **l == "Credits")
            .map(|(i, _)| i)
            .unwrap_or(lines.len());

        let mut rules: BTreeMap<String, Rule> = BTreeMap::new();
        let mut order: Vec<String> = Vec::new();
        let mut current: Option<String> = None;
        let mut current_chapter: Option<String> = None;

        // Chapter headers appear just before their first section ("1. Game Concepts").
        // They occur in the contents; we synthesize them from the section numbers.
        for line in &lines[body_start..glossary_start] {
            if line.is_empty() {
                continue;
            }
            if let Some((id, rest, kind)) = parse_rule_line(line) {
                if kind == RuleKind::Chapter {
                    // e.g. "7. Additional Rules"
                    let r = Rule {
                        id: id.clone(),
                        kind,
                        text: rest.to_string(),
                        examples: vec![],
                        parent: None,
                        children: vec![],
                    };
                    current_chapter = Some(id.clone());
                    if !rules.contains_key(&id) {
                        order.push(id.clone());
                        rules.insert(id.clone(), r);
                    }
                    current = Some(id);
                    continue;
                }
                let parent = match kind {
                    RuleKind::Section => {
                        let ch = id[..1].to_string();
                        if !rules.contains_key(&ch) {
                            rules.insert(
                                ch.clone(),
                                Rule {
                                    id: ch.clone(),
                                    kind: RuleKind::Chapter,
                                    text: String::new(),
                                    examples: vec![],
                                    parent: None,
                                    children: vec![],
                                },
                            );
                            order.push(ch.clone());
                        }
                        let _ = &current_chapter;
                        Some(ch)
                    }
                    RuleKind::Rule => Some(id.split('.').next().unwrap().to_string()),
                    RuleKind::Subrule => {
                        let base: String = id
                            .trim_end_matches(|c: char| c.is_ascii_lowercase())
                            .to_string();
                        Some(base)
                    }
                    RuleKind::Chapter => None,
                };
                if let Some(p) = &parent {
                    if let Some(pr) = rules.get_mut(p) {
                        pr.children.push(id.clone());
                    }
                }
                order.push(id.clone());
                rules.insert(
                    id.clone(),
                    Rule {
                        id: id.clone(),
                        kind,
                        text: rest.to_string(),
                        examples: vec![],
                        parent,
                        children: vec![],
                    },
                );
                current = Some(id);
            } else if let Some(cur) = &current {
                let r = rules.get_mut(cur).unwrap();
                if line.starts_with("Example:") {
                    r.examples.push(line.to_string());
                } else if let Some(last) = r.examples.last_mut() {
                    last.push('\n');
                    last.push_str(line);
                } else {
                    r.text.push('\n');
                    r.text.push_str(line);
                }
            }
        }

        let mut glossary = Vec::new();
        let gl = &lines[(glossary_start + 1).min(lines.len())..credits_start];
        let mut i = 0;
        while i < gl.len() {
            if gl[i].is_empty() {
                i += 1;
                continue;
            }
            let term = gl[i].to_string();
            let mut def = String::new();
            i += 1;
            while i < gl.len() && !gl[i].is_empty() {
                if !def.is_empty() {
                    def.push('\n');
                }
                def.push_str(gl[i]);
                i += 1;
            }
            glossary.push(GlossaryEntry {
                term,
                definition: def,
            });
        }

        Self {
            effective_date,
            order,
            rules,
            glossary,
        }
    }

    pub fn get(&self, id: &str) -> Option<&Rule> {
        let id = id.trim().trim_end_matches('.');
        self.rules.get(id)
    }

    /// All rules and subrules (excluding chapters and section headers), in document order.
    pub fn numbered_rules(&self) -> impl Iterator<Item = &Rule> {
        self.order
            .iter()
            .filter_map(|id| self.rules.get(id))
            .filter(|r| matches!(r.kind, RuleKind::Rule | RuleKind::Subrule))
    }

    /// Every entry (chapters, sections, rules, subrules) in document order.
    pub fn all(&self) -> impl Iterator<Item = &Rule> {
        self.order.iter().filter_map(|id| self.rules.get(id))
    }

    pub fn glossary_entry(&self, term: &str) -> Option<&GlossaryEntry> {
        self.glossary
            .iter()
            .find(|g| g.term.eq_ignore_ascii_case(term))
    }

    /// Full text of a rule including its subrules, for display.
    pub fn render(&self, id: &str) -> Option<String> {
        let r = self.get(id)?;
        let mut out = format!("{} {}", r.id, r.text);
        for e in &r.examples {
            out.push('\n');
            out.push_str(e);
        }
        for c in &r.children {
            if let Some(s) = self.render(c) {
                out.push('\n');
                out.push_str(&s);
            }
        }
        Some(out)
    }
}

/// Recognizes a rule-number prefix. Returns (id, remaining text, kind).
fn parse_rule_line(line: &str) -> Option<(String, &str, RuleKind)> {
    let bytes = line.as_bytes();
    // Chapter: "1. Game Concepts" — single digit, dot, space, capitalized word.
    if bytes.len() > 3 && bytes[0].is_ascii_digit() && bytes[1] == b'.' && bytes[2] == b' ' {
        return Some((line[..1].to_string(), line[3..].trim(), RuleKind::Chapter));
    }
    if bytes.len() < 5 || !bytes[..3].iter().all(u8::is_ascii_digit) || bytes[3] != b'.' {
        return None;
    }
    // Section: "704. State-Based Actions"
    if bytes[4] == b' ' {
        return Some((line[..3].to_string(), line[5..].trim(), RuleKind::Section));
    }
    let mut j = 4;
    while j < bytes.len() && bytes[j].is_ascii_digit() {
        j += 1;
    }
    if j == 4 {
        return None;
    }
    let num_end = j;
    while j < bytes.len() && bytes[j].is_ascii_lowercase() {
        j += 1;
    }
    let has_letters = j > num_end;
    let id = line[..j].to_string();
    if has_letters {
        // "704.5a text" (some subrules are followed by a period in older docs)
        let rest = line[j..].trim_start_matches('.').trim();
        Some((id, rest, RuleKind::Subrule))
    } else if j < bytes.len() && bytes[j] == b'.' {
        Some((id, line[j + 1..].trim(), RuleKind::Rule))
    } else if j < bytes.len() && bytes[j] == b' ' {
        // Tolerate a missing period after the rule number (e.g. "606.5 If the total…").
        Some((id, line[j..].trim(), RuleKind::Rule))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rule_lines() {
        assert_eq!(
            parse_rule_line("704.5a If a player").map(|x| x.0),
            Some("704.5a".into())
        );
        assert_eq!(
            parse_rule_line("704.5. The state").map(|x| x.2),
            Some(RuleKind::Rule)
        );
        assert_eq!(
            parse_rule_line("704. State-Based Actions").map(|x| x.2),
            Some(RuleKind::Section)
        );
        assert_eq!(
            parse_rule_line("704.5aa If a player").map(|x| x.0),
            Some("704.5aa".into())
        );
        assert!(parse_rule_line("Example: foo").is_none());
    }

    #[test]
    fn sort_keys() {
        assert!(rule_sort_key("702.9") < rule_sort_key("702.10"));
        assert!(rule_sort_key("704.5z") < rule_sort_key("704.5aa"));
    }
}
