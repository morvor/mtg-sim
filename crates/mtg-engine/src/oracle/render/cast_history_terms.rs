//! How a spell was cast and paid for, and where a permanent came from, as conditions and
//! amounts (see `kw/cast_history.rs` and `kw/played_from_zone.rs`): "if it wasn't cast",
//! "if you cast it from your graveyard", "if it entered from a graveyard", "if it was
//! kicked", "if mana from a Treasure was spent to cast it", "if no colored mana was spent
//! to cast it", "if this ability has been activated four or more times this turn", "if
//! it's the second creature spell you cast this turn".

use super::players::Case;
use super::*;
use crate::kw::cast_history as ch;

/// A zone a spell was cast from or a permanent came from, with its owner if the filter
/// says one ("your graveyard"; a card in a graveyard, hand or library is its owner's, CR
/// 400.3).
fn zone_from(z: &str, owner: Option<PlayerRel>) -> Option<&'static str> {
    Some(match (z, owner) {
        ("Exile", _) => "exile",
        ("Graveyard", Some(PlayerRel::You)) => "your graveyard",
        ("Graveyard", None) => "a graveyard",
        ("Hand", Some(PlayerRel::You) | None) => "your hand",
        ("Library", Some(PlayerRel::You)) => "your library",
        ("Library", None) => "a library",
        _ => return None,
    })
}

/// The quality of a cast-history or came-from filter and its owner, if `f` is one
/// (maybe with the kind of object it implies).
fn history_atoms(f: &Filter) -> Option<(&str, Option<PlayerRel>)> {
    let atoms: Vec<&Filter> = match f {
        Filter::And(v) => v.iter().collect(),
        other => vec![other],
    };
    let mut name = None;
    let mut owner = None;
    for a in atoms {
        match a {
            Filter::Custom(n)
                if name.is_none()
                    && (n.starts_with("cast history:") || n.starts_with("came from:")) =>
            {
                name = Some(n.as_str())
            }
            Filter::OwnedBy(r) if owner.is_none() => owner = Some(*r),
            Filter::Permanent | Filter::Card | Filter::Any => {}
            _ => return None,
        }
    }
    Some((name?, owner))
}

/// "was cast from a graveyard" / "entered from your graveyard" (and their negations):
/// (subject-less predicate, negated predicate, and the "you cast [it]" form if there is
/// one).
fn history_predicate(
    name: &str,
    owner: Option<PlayerRel>,
) -> Option<(String, String, Option<(String, String)>)> {
    if let Some(z) = name.strip_prefix("came from:") {
        let z = zone_from(z, owner)?;
        return Some((
            format!("entered from {z}"),
            format!("didn't enter from {z}"),
            None,
        ));
    }
    if owner.is_some() && !name.starts_with(ch::CAST_FROM) {
        return None;
    }
    Some(match name {
        ch::WAS_CAST => ("was cast".into(), "wasn't cast".into(), None),
        ch::CAST_THIS_TURN => (
            "was cast this turn".into(),
            "wasn't cast this turn".into(),
            None,
        ),
        ch::KICKED => ("was kicked".into(), "wasn't kicked".into(), None),
        n => {
            let z = zone_from(n.strip_prefix(ch::CAST_FROM)?, owner)?;
            (
                format!("was cast from {z}"),
                format!("wasn't cast from {z}"),
                Some((format!(" from {z}"), format!(" from {z}"))),
            )
        }
    })
}

impl Renderer<'_> {
    /// "it wasn't cast", "you cast it from your graveyard", "it entered from a graveyard",
    /// "it was kicked".
    pub(crate) fn cast_history_clause(
        &mut self,
        s: &Sel,
        f: &Filter,
        negated: bool,
    ) -> Option<String> {
        let (name, owner) = history_atoms(f)?;
        let (pos, neg, you_cast) = history_predicate(name, owner)?;
        let subj = self.sel(s, Case::Subj);
        Some(match (negated, you_cast) {
            (false, Some((from, _))) => {
                format!("{{alt:you cast {subj}{from}|{subj} {pos}}}")
            }
            (true, Some((from, _))) => {
                format!("{{alt:you didn't cast {subj}{from}|{subj} {neg}}}")
            }
            (false, None) => format!("{subj} {pos}"),
            (true, None) => format!("{subj} {neg}"),
        })
    }

    /// "you cast it from anywhere other than your hand": it was cast, and not from your
    /// hand.
    pub(crate) fn cast_from_elsewhere(&mut self, v: &[Condition]) -> Option<String> {
        let [Condition::SelMatches(a, Filter::Custom(w)), Condition::Not(n)] = v else {
            return None;
        };
        let Condition::SelMatches(b, Filter::Custom(z)) = n.as_ref() else {
            return None;
        };
        if w != ch::WAS_CAST || format!("{a:?}") != format!("{b:?}") {
            return None;
        }
        let z = zone_from(z.strip_prefix(ch::CAST_FROM)?, None)?;
        let s = self.sel(a, Case::Obj);
        Some(format!("you cast {s} from anywhere other than {z}"))
    }

    /// "one or more of them entered from a graveyard or was cast from a graveyard": some
    /// of the objects have one of the qualities.
    pub(crate) fn some_of_them_history(&mut self, v: &[Condition]) -> Option<String> {
        let mut sel = None;
        let mut preds = Vec::new();
        for c in v {
            let Condition::SelNonEmpty(m) = c else {
                return None;
            };
            let Sel::Matching(s, f) = m else {
                return None;
            };
            if sel.is_some_and(|x: &Sel| format!("{x:?}") != format!("{s:?}")) {
                return None;
            }
            sel = Some(s.as_ref());
            let (name, owner) = history_atoms(f)?;
            preds.push(history_predicate(name, owner)?.0);
        }
        let s = self.sel(sel?, Case::Obj);
        let s = if s == "them" || s.contains("them") {
            "them".to_string()
        } else {
            return None;
        };
        Some(format!("one or more of {s} {}", join_list(&preds, "or")))
    }

    /// "it entered from your library or was cast from your library": the object itself,
    /// said once.
    pub(crate) fn own_entered_or_cast(&mut self, v: &[Condition]) -> Option<String> {
        let [Condition::SelMatches(Sel::This, f), Condition::CastFrom(z)] = v else {
            return None;
        };
        let (name, owner) = history_atoms(f)?;
        if !name.starts_with("came from:") {
            return None;
        }
        let (entered, _, _) = history_predicate(name, owner)?;
        let zw = zone_word(*z);
        let m = self.me();
        // Your own card is in your zone (CR 400.3).
        Some(format!(
            "{m} {entered} or {{alt:you cast it from your {zw}|was cast from your {zw}}}"
        ))
    }

    /// What the mana spent was counted on: "cast ~", "cast that spell", "activate this
    /// ability".
    fn spent_on(&mut self, whose: &str) -> Option<String> {
        Some(match whose {
            "self" => {
                let m = self.me();
                format!("cast {m}")
            }
            "that spell" => "cast {alt:it|that spell}".into(),
            "this ability" => "activate this ability".into(),
            w => {
                let slot: u8 = w.strip_prefix("target ")?.parse().ok()?;
                let t = self.sel(&Sel::Target(slot), Case::Obj);
                format!("cast {t}")
            }
        })
    }

    /// The mana counted: "mana", "colored mana", "mana from a Treasure" (`one`) or "mana
    /// from Treasures".
    fn spent_what(what: &str, one: bool) -> Option<String> {
        Some(match what {
            "total" => "mana".into(),
            "colored" => "colored mana".into(),
            w => {
                if let Some(st) = w.strip_prefix("source subtype:") {
                    if one {
                        format!("mana from {}", with_article(st))
                    } else {
                        format!("mana from {}s", st)
                    }
                } else if let Some(t) = w.strip_prefix("source type:") {
                    let t = t.to_lowercase();
                    if one {
                        format!("mana from {}", with_article(&t))
                    } else {
                        format!("mana from {t}s")
                    }
                } else {
                    return None;
                }
            }
        })
    }

    /// An amount of a spell's or ability's history, as a number: "the number of mana from
    /// a Treasure spent to cast it".
    pub(crate) fn cast_history_value(&mut self, name: &str) -> Option<String> {
        if name == ch::TIMES_ACTIVATED {
            return Some("the number of times this ability has been activated this turn".into());
        }
        let (whose, what) = name.strip_prefix(ch::SPENT)?.split_once(':')?;
        let m = if what == "colors" {
            "colors of mana".to_string()
        } else {
            Self::spent_what(what, true)?
        };
        let on = self.spent_on(whose)?;
        Some(format!("the {{alt:number|amount}} of {m} spent to {on}"))
    }

    /// A comparison of a spell's or ability's history with a number: "if this ability has
    /// been activated four or more times this turn", "if no colored mana was spent to cast
    /// it", "if three or more mana from creatures was spent to cast it", "if exactly three
    /// colors of mana were spent to activate this ability", "if the amount of mana spent
    /// to cast that spell was less than its mana value".
    pub(crate) fn cast_history_compare(
        &mut self,
        name: &str,
        cmp: Cmp,
        b: &Value,
    ) -> Option<String> {
        if name == ch::TIMES_ACTIVATED {
            let Value::Const(n) = b else { return None };
            let min = match cmp {
                Cmp::Ge => *n,
                Cmp::Gt => n + 1,
                _ => return None,
            };
            return Some(format!(
                "this ability has been activated {} or more times this turn",
                number_word(min)
            ));
        }
        let (whose, what) = name.strip_prefix(ch::SPENT)?.split_once(':')?;
        // "the amount of mana spent to cast that spell was less than its mana value".
        if let Value::ManaValueOf(s) = b {
            if what != "total" || !matches!(cmp, Cmp::Lt) {
                return None;
            }
            let on = self.spent_on(whose)?;
            let whose_mv = match s.as_ref() {
                Sel::Target(_) if whose.starts_with("target ") => "its".to_string(),
                other => {
                    let o = self.sel(other, Case::Poss);
                    o
                }
            };
            return Some(format!(
                "the amount of mana spent to {on} was less than {whose_mv} mana value"
            ));
        }
        let Value::Const(n) = b else { return None };
        if what == "colors" {
            let on = self.spent_on(whose)?;
            return match cmp {
                Cmp::Eq => Some(format!(
                    "exactly {} colors of mana were spent to {on}",
                    number_word(*n)
                )),
                _ => None,
            };
        }
        let on_text = |r: &mut Self| r.spent_on(whose);
        match (cmp, *n) {
            (Cmp::Eq, 0) | (Cmp::Le, 0) | (Cmp::Lt, 1) => {
                let m = Self::spent_what(what, true)?;
                let on = on_text(self)?;
                Some(format!("no {m} was spent to {on}"))
            }
            (Cmp::Ge, 1) | (Cmp::Gt, 0) => {
                let m = Self::spent_what(what, true)?;
                let on = on_text(self)?;
                Some(format!("{m} was spent to {on}"))
            }
            (Cmp::Ge, k) => {
                let m = Self::spent_what(what, false)?;
                let on = on_text(self)?;
                Some(format!("{} or more {m} was spent to {on}", number_word(k)))
            }
            _ => None,
        }
    }

    /// "if it's the second creature spell you cast this turn", "if it was the second spell
    /// you cast this turn": the spells cast this turn up to that one (see
    /// [`ch::CAST_BY_THEN`]).
    pub(crate) fn nth_spell_cast(
        &mut self,
        a: &Value,
        cmp: Cmp,
        b: &Value,
        kind: Option<&Filter>,
    ) -> Option<String> {
        let (Value::EventsThisTurn(c, Tally::Events), Cmp::Eq, Value::Const(n)) = (a, cmp, b)
        else {
            return None;
        };
        let TriggerCond::CastSpell { who, filter } = c.as_ref() else {
            return None;
        };
        let Filter::And(v) = filter else { return None };
        let whose = v.iter().find_map(|x| match x {
            Filter::Custom(n) => n.strip_prefix(ch::CAST_BY_THEN),
            _ => None,
        })?;
        let rest: Vec<Filter> = v
            .iter()
            .filter(|x| !matches!(x, Filter::Custom(_)))
            .cloned()
            .collect();
        let rest = Filter::and(rest);
        // The spell itself has the quality counted: "the second creature spell" is a
        // creature spell.
        if let Some(k) = kind {
            if format!("{k:?}") != format!("{rest:?}") {
                return None;
            }
        } else if !matches!(rest, Filter::Spell) {
            return None;
        }
        let noun = self.noun(&rest, Num::One);
        let noun = if noun.contains("spell") {
            noun
        } else {
            format!("{noun} spell")
        };
        let who = self.rel_subject(*who);
        let nth = ordinal_word(*n as u32);
        Some(match whose {
            "self" => {
                let m = self.me();
                format!("{m} was the {nth} {noun} {who} cast this turn")
            }
            "that spell" => format!("it's the {nth} {noun} {who} cast this turn"),
            _ => return None,
        })
    }
}
