//! Numbers that are the greatest or least of several: "the greatest power among creatures
//! you control" ([`Value::Extreme`]), "the highest life total among all players", "the
//! greatest number of creatures a player controls" ([`Value::OverPlayers`]).

use super::players::Case;
use super::*;

/// The characteristic a measure of [`vars::TESTED`] names: "power", "mana value".
fn tested_stat(v: &Value) -> Option<&'static str> {
    let tested = |s: &Sel| matches!(s, Sel::Var(vars::TESTED));
    Some(match v {
        Value::PowerOf(s) if tested(s) => "power",
        Value::ToughnessOf(s) if tested(s) => "toughness",
        Value::ManaValueOf(s) if tested(s) => "mana value",
        _ => return None,
    })
}

/// The qualities of a filter, as a sorted list (the battlefield, where permanents are,
/// left out).
fn atoms(f: &Filter, out: &mut Vec<String>) {
    match f {
        Filter::And(v) => v.iter().for_each(|x| atoms(x, out)),
        Filter::Any | Filter::InZone(ZoneKind::Battlefield) => {}
        other => {
            let k = format!("{other:?}");
            if !out.contains(&k) {
                out.push(k);
            }
        }
    }
}

impl Renderer<'_> {
    /// A quality "with the greatest power among [objects]": the object's power is at least
    /// the greatest among them (`siblings`: the other qualities of the object; when the
    /// objects compared are the ones the object is one of, cards may leave them out:
    /// "target creature you control with the greatest power").
    pub(crate) fn extreme_quality(
        &mut self,
        a: &Value,
        c: Cmp,
        b: &Value,
        siblings: &[Filter],
    ) -> Option<String> {
        let Value::Extreme(m, sel, greatest) = b else {
            return None;
        };
        let x = tested_stat(a)?;
        if tested_stat(m)? != x || !matches!((c, greatest), (Cmp::Ge, true) | (Cmp::Le, false)) {
            return None;
        }
        let Sel::All(g) = sel.as_ref() else {
            return None;
        };
        let rest: Vec<Filter> = siblings
            .iter()
            .filter(|s| !matches!(s, Filter::ValueCmp(..)))
            .cloned()
            .collect();
        let (mut r, mut o) = (Vec::new(), Vec::new());
        atoms(&Filter::And(rest), &mut r);
        atoms(g, &mut o);
        r.sort();
        o.sort();
        let word = match (greatest, x) {
            (true, "mana value") => "{alt:greatest|highest}",
            (false, "mana value") => "{alt:least|lowest}",
            (true, _) => "greatest",
            (false, _) => "least",
        };
        let n = self.noun(g, Num::Many);
        let field = if Self::counts_all_permanents(g, &n) {
            " {opt:on the battlefield}"
        } else {
            ""
        };
        // The objects compared are the ones the object is one of: cards may leave them
        // out ("target nonland permanent with the lowest mana value").
        if r == o && !n.contains(['{', '|', '}']) {
            let words: &[&str] = match (greatest, x) {
                (true, "mana value") => &["greatest", "highest"],
                (false, "mana value") => &["least", "lowest"],
                (true, _) => &["greatest"],
                (false, _) => &["least"],
            };
            let mut forms = Vec::new();
            for w in words {
                forms.push(format!("with the {w} {x}"));
                forms.push(format!("with the {w} {x} among {n}"));
                if !field.is_empty() {
                    forms.push(format!("with the {w} {x} among {n} on the battlefield"));
                }
            }
            return Some(format!("{{alt:{}}}", forms.join("|")));
        }
        Some(format!("with the {word} {x} among {n}{field}"))
    }

    /// An object "with the greatest power" among the objects of its own kind: cards say
    /// "target creature you control with the greatest power" or "a creature with the
    /// greatest power among creatures you control" (the same objects compared, whether
    /// or not they're named again).
    pub(crate) fn extreme_noun(&mut self, f: &Filter, det: &super::nouns::Det) -> Option<String> {
        let Filter::And(v) = f else {
            return None;
        };
        let (a, c, b) = v.iter().find_map(|x| match x {
            Filter::ValueCmp(a, c, b) => Some((a, *c, b)),
            _ => None,
        })?;
        let Value::Extreme(m, sel, greatest) = b.as_ref() else {
            return None;
        };
        let x = tested_stat(a)?;
        if tested_stat(m)? != x || !matches!((c, greatest), (Cmp::Ge, true) | (Cmp::Le, false)) {
            return None;
        }
        let Sel::All(g) = sel.as_ref() else {
            return None;
        };
        let rest: Vec<Filter> = v
            .iter()
            .filter(|s| !matches!(s, Filter::ValueCmp(..)))
            .cloned()
            .collect();
        let (mut r, mut o) = (Vec::new(), Vec::new());
        atoms(&Filter::And(rest.clone()), &mut r);
        atoms(g, &mut o);
        r.sort();
        o.sort();
        if r != o {
            return None;
        }
        let bare: Vec<Filter> = rest
            .iter()
            .filter(|s| {
                !matches!(
                    s,
                    Filter::ControlledBy(_) | Filter::InZone(ZoneKind::Battlefield)
                )
            })
            .cloned()
            .collect();
        let plain = self.noun_det(&Filter::and(rest), det.clone());
        let bare = self.noun_det(&Filter::and(bare), det.clone());
        let n = self.noun(g, Num::Many);
        let words: &[&str] = match (greatest, x) {
            (true, "mana value") => &["greatest", "highest"],
            (false, "mana value") => &["least", "lowest"],
            (true, _) => &["greatest"],
            (false, _) => &["least"],
        };
        let all_permanents = Self::counts_all_permanents(g, &n);
        let mut forms = Vec::new();
        for w in words {
            forms.push(format!("{plain} with the {w} {x}"));
            forms.push(format!("{bare} with the {w} {x} among {n}"));
            if all_permanents {
                forms.push(format!(
                    "{bare} with the {w} {x} among {n} on the battlefield"
                ));
            }
        }
        if forms.iter().any(|f| f.contains(['{', '|', '}'])) {
            let w = if words.len() > 1 {
                format!("{{alt:{}}}", words.join("|"))
            } else {
                words[0].to_string()
            };
            let field = if all_permanents {
                " {opt:on the battlefield}"
            } else {
                ""
            };
            return Some(format!("{bare} with the {w} {x} among {n}{field}"));
        }
        Some(format!("{{alt:{}}}", forms.join("|")))
    }

    /// [`Value::Extreme`]: "the greatest power among creatures you control".
    pub(crate) fn extreme_value(&mut self, v: &Value, sel: &Sel, greatest: bool) -> String {
        let Some(stat) = tested_stat(v) else {
            return self.gap("Value::Extreme");
        };
        let s = match sel {
            Sel::All(f) => self.noun(f, Num::Many),
            other => self.sel(other, Case::Obj),
        };
        let word = match (greatest, stat) {
            (true, "mana value") => "{alt:greatest|highest}",
            (false, "mana value") => "{alt:least|lowest}",
            (true, _) => "greatest",
            (false, _) => "least",
        };
        format!("the {word} {stat} among {s}")
    }

    /// [`Value::OverPlayers`]: each matching player's value, combined.
    pub(crate) fn over_players_value(&mut self, op: AggOp, pf: &PlayerFilter, v: &Value) -> String {
        let iterated = |p: &PlayerRef| matches!(p, PlayerRef::Iterated);
        let opponents = matches!(pf, PlayerFilter::Opponent);
        if !opponents && !matches!(pf, PlayerFilter::Any) {
            return self.gap("Value::OverPlayers");
        }
        let among = if opponents {
            "your opponents"
        } else {
            "all players"
        };
        let a_player = if opponents { "an opponent" } else { "a player" };
        match (op, v) {
            // "the highest life total among all players".
            (AggOp::Max | AggOp::Min, Value::LifeTotal(p)) if iterated(p) => {
                let w = if op == AggOp::Max {
                    "highest"
                } else {
                    "lowest"
                };
                format!("the {w} life total among {among}")
            }
            // "the number of cards in the hand of the opponent with the most cards in hand".
            (AggOp::Max, Value::HandSize(p)) if iterated(p) => {
                let who = if opponents { "opponent" } else { "player" };
                format!(
                    "{{alt:the number of cards in the hand of the {who} with the most cards in hand|the greatest number of cards in {a_player}'s hand}}"
                )
            }
            // "the greatest number of artifacts an opponent controls".
            (AggOp::Max | AggOp::Min, Value::Count(f)) if matches!(f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::ControlledBy(PlayerRel::Iterated)))) =>
            {
                let n = self.noun(&effects::strip_controller(f), Num::Many);
                let w = if op == AggOp::Max {
                    "greatest"
                } else {
                    "least"
                };
                format!("the {w} number of {n} {a_player} controls")
            }
            // "the total number of rad counters among players".
            (AggOp::Sum, Value::CountersOn(s, k)) if matches!(s.as_ref(), Sel::Players(p) if iterated(p)) =>
            {
                let c = match k {
                    Some(k) => format!("{k} counters"),
                    None => "counters".into(),
                };
                let among = if opponents {
                    "your opponents"
                } else {
                    "players"
                };
                format!("the total number of {c} among {among}")
            }
            _ => self.gap("Value::OverPlayers"),
        }
    }

    /// "a graveyard has twenty or more cards in it": some player's value is at least `n`.
    pub(crate) fn over_players_at_least(
        &mut self,
        pf: &PlayerFilter,
        v: &Value,
        n: &Value,
    ) -> Option<String> {
        let Value::Const(n) = n else {
            return None;
        };
        match (pf, v) {
            (PlayerFilter::Any, Value::CardsInGraveyard(PlayerRef::Iterated, Filter::Card)) => {
                Some(format!(
                    "a graveyard has {} or more cards in it",
                    number_word(*n)
                ))
            }
            _ => None,
        }
    }
}
