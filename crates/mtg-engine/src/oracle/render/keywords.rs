//! Keyword abilities (CR 702) with their parameters.

use super::nouns::Det;
use super::*;
use crate::keywords::{Keyword, KeywordKind};

impl Renderer<'_> {
    /// A keyword line item, capitalized ("Flying", "Equip {2}", "Protection from red").
    pub(crate) fn keyword(&mut self, k: &Keyword) -> String {
        let mut s = self.keyword_lower(k);
        // "Equip {0}. Activate only once each turn." (kept with the keyword, CR 602.5b).
        if matches!(k.kind, KeywordKind::Equip | KeywordKind::Crew)
            && k.text
                .as_deref()
                .is_some_and(|t| t.to_lowercase().contains("activate only once each turn"))
        {
            s.push_str(". Activate only once each turn");
        }
        capitalize(&s)
    }

    /// The variant of a keyword marked by its printed name (CR 702.37b megamorph,
    /// 702.33 multikicker, 702.19c trample over planeswalkers, 702.145 daybound and
    /// nightbound). The engine tells these apart by the keyword's name text.
    fn variant(k: &Keyword) -> Option<&'static str> {
        let t = k.text.as_deref().unwrap_or("").to_lowercase();
        let pairs: [(&str, &str); 8] = [
            ("megamorph", "megamorph"),
            ("multikicker", "multikicker"),
            ("sticker kicker", "sticker kicker"),
            ("trample over planeswalkers", "trample over planeswalkers"),
            ("daybound", "daybound"),
            ("nightbound", "nightbound"),
            ("totem armor", "umbra armor"),
            ("partner with", "partner with"),
        ];
        pairs
            .iter()
            .find(|(p, _)| t.starts_with(p))
            .map(|(_, v)| *v)
    }

    /// A keyword in running text, lowercase ("flying", "protection from red").
    pub(crate) fn keyword_lower(&mut self, k: &Keyword) -> String {
        let variant = Self::variant(k);
        let name = match variant {
            Some(v) if v != "partner with" => v.to_string(),
            _ => k.kind.name().to_lowercase(),
        };
        let cost = |r: &mut Self, c: &Cost| -> String {
            let s = r.cost(c);
            if c.mana.is_some() && c.parts.is_empty() {
                format!(" {s}")
            } else if c.mana.is_none() {
                format!("—{s}")
            } else {
                format!(" {s}")
            }
        };
        match k.kind {
            KeywordKind::Enchant => match &k.filter {
                // "enchant creature put onto the battlefield with ~" (CR 607.2c).
                Some(Filter::And(v))
                    if v.len() == 2
                        && matches!(&v[1], Filter::In(s) if matches!(**s, Sel::Linked)) =>
                {
                    let n = self.noun_det(&v[0], Det::Bare);
                    format!("enchant {n} put onto the battlefield with ~")
                }
                Some(f) => {
                    let n = self.noun_det(f, Det::Bare);
                    format!("enchant {n}")
                }
                None => {
                    let t = k.text.as_deref().unwrap_or("").to_lowercase();
                    if t.contains("opponent") {
                        "enchant opponent".into()
                    } else {
                        "enchant player".into()
                    }
                }
            },
            KeywordKind::Protection => match &k.filter {
                Some(f) => {
                    let q = self.quality(f);
                    format!("protection from {q}")
                }
                None => self.gap("protection without quality"),
            },
            KeywordKind::Hexproof => match &k.filter {
                Some(f) => {
                    let q = self.quality(f);
                    format!("hexproof from {q}")
                }
                None => "hexproof".into(),
            },
            KeywordKind::Landwalk => match &k.filter {
                Some(f) => self.landwalk(f),
                None => "landwalk".into(),
            },
            KeywordKind::Affinity => match &k.filter {
                Some(f) => {
                    let n = self.noun(f, Num::Many);
                    format!("affinity for {n}")
                }
                None => self.gap("affinity without filter"),
            },
            KeywordKind::Splice => match &k.filter {
                Some(f) => {
                    let n = self.noun(f, Num::One);
                    let c = k.cost.as_ref().map(|c| cost(self, c)).unwrap_or_default();
                    format!("splice onto {n}{c}")
                }
                None => self.gap("splice without quality"),
            },
            KeywordKind::Banding => match &k.filter {
                Some(f) => {
                    let n = self.noun(f, Num::Many);
                    format!("bands with other {n}")
                }
                None => "banding".into(),
            },
            KeywordKind::Partner if variant == Some("partner with") => {
                let raw = k.text.as_deref().unwrap_or("");
                let name = raw
                    .strip_prefix("Partner with ")
                    .or_else(|| raw.strip_prefix("partner with "))
                    .unwrap_or(raw);
                format!("partner with {name}")
            }
            KeywordKind::Cycling if k.filter.is_some() => {
                let ty = match &k.filter {
                    Some(f) => self.cycling_type(f),
                    None => String::new(),
                };
                let c = k.cost.as_ref().map(|c| cost(self, c)).unwrap_or_default();
                format!("{ty}cycling{c}")
            }
            KeywordKind::Equip => {
                let q = match &k.filter {
                    Some(f) => format!(" {}", self.noun_det(f, Det::Bare)),
                    None => String::new(),
                };
                let c = k.cost.as_ref().map(|c| cost(self, c)).unwrap_or_default();
                format!("equip{q}{c}")
            }
            KeywordKind::Kicker if !k.costs.is_empty() => {
                let a = k.cost.as_ref().map(|c| self.cost(c)).unwrap_or_default();
                let b = self.cost(&k.costs[0]);
                format!("{name} {a} and/or {b}")
            }
            // "Prototype {1}{W} — 2/2": the prototype's power and toughness are the
            // keyword's parameter (kept as "P/T").
            KeywordKind::Prototype
                if k.cost.is_some()
                    && k.text.as_deref().is_some_and(|t| {
                        t.split_once('/').is_some_and(|(p, q)| {
                            p.parse::<i32>().is_ok() && q.parse::<i32>().is_ok()
                        })
                    }) =>
            {
                let c = k.cost.as_ref().map(|c| self.cost(c)).unwrap_or_default();
                let pt = k.text.clone().unwrap_or_default();
                format!("prototype {c} — {pt}")
            }
            KeywordKind::Ward if k.x.is_some() => {
                let v = match &k.x {
                    Some(x) => self.value(x),
                    None => String::new(),
                };
                format!("ward {{X}}, where X is {v}")
            }
            KeywordKind::Suspend if k.n.is_some() && k.cost.is_some() => {
                let n = match k.n {
                    Some(-1) => "X".to_string(),
                    Some(n) => n.to_string(),
                    None => String::new(),
                };
                let c = k.cost.as_ref().map(|c| self.cost(c)).unwrap_or_default();
                // CR 702.62a: "Suspend X" comes with "X can't be 0".
                if k.n == Some(-1) {
                    format!("suspend {n}—{c}. X can't be 0")
                } else {
                    format!("suspend {n}—{c}")
                }
            }
            KeywordKind::Reinforce if k.n == Some(-1) => {
                let c = k.cost.as_ref().map(|c| self.cost(c)).unwrap_or_default();
                format!("reinforce X—{c}")
            }
            _ => {
                let mut s = name;
                if let Some(f) = &k.filter {
                    let n = self.noun(f, Num::One);
                    s.push_str(&format!(" {n}"));
                }
                match (k.n, &k.x) {
                    (_, Some(v)) => {
                        let v = self.value(v);
                        s.push_str(&format!(" X, where X is {v}"));
                    }
                    (Some(-1), None) => s.push_str(" X"),
                    (Some(n), None) => s.push_str(&format!(" {n}")),
                    (None, None) => {}
                }
                if let Some(c) = &k.cost {
                    if k.n.is_some() {
                        let c = self.cost(c);
                        s.push_str(&format!("—{c}"));
                    } else {
                        let c = cost(self, c);
                        s.push_str(&c);
                    }
                }
                s
            }
        }
    }

    /// The quality of protection/hexproof ("red", "creatures", "everything").
    pub(crate) fn quality(&mut self, f: &Filter) -> String {
        match f {
            Filter::Any => "everything".into(),
            Filter::Color(c) => c.word().into(),
            Filter::Multicolored => "multicolored".into(),
            Filter::Monocolored => "monocolored".into(),
            Filter::Colorless => "colorless".into(),
            Filter::ChosenColor => "the chosen color".into(),
            Filter::ChosenName => "the chosen name".into(),
            Filter::ControlledBy(PlayerRel::Chosen) => "the chosen player".into(),
            // CR 702.16i, 702.16k: protection from each opponent.
            Filter::ControlledBy(PlayerRel::Opponent) => "each of your opponents".into(),
            Filter::ManaValueOfChosenQuality => "each mana value of the chosen quality".into(),
            Filter::Supertype(s) => nouns::supertype_word(*s).into(),
            Filter::ManaValue(c, v) => {
                let v = self.value(v);
                format!("mana value {}", nouns::cmp_phrase(*c, &v))
            }
            Filter::Or(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.quality(x)).collect();
                join_list(&parts, "and")
            }
            other => self.noun(other, Num::Many),
        }
    }

    fn landwalk(&mut self, f: &Filter) -> String {
        match f {
            Filter::Subtype(s) => format!("{}walk", s.to_lowercase()),
            Filter::ChosenType => "landwalk of the chosen type".into(),
            other => {
                let n = self.noun(other, Num::One);
                format!("{n}walk")
            }
        }
    }

    fn cycling_type(&mut self, f: &Filter) -> String {
        match f {
            Filter::Subtype(s) => s.to_lowercase(),
            other => self.noun(other, Num::One),
        }
    }
}
