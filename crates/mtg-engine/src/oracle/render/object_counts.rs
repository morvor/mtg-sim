//! Qualities of an object stated as a count about it (`Filter::ValueCmp` with the tested
//! object, `vars::TESTED`): "with a legendary Equipment attached to it", "with one or
//! more blue mana symbols in its mana cost", "that doesn't have the same name as another
//! permanent you control", "that doesn't share a land type with a land you control";
//! and choices among dug cards one for each kind ("For each card type among noncreature
//! spells you've cast this turn, you may put a card of that type ...").

use super::players::Case;
use super::*;

fn tested(s: &Sel) -> bool {
    matches!(s, Sel::Var(vars::TESTED))
}

/// The parts of an `And` (or the filter itself).
fn parts(f: &Filter) -> Vec<Filter> {
    match f {
        Filter::And(v) => v.clone(),
        other => vec![other.clone()],
    }
}

impl Renderer<'_> {
    /// The quality a count about the tested object says, after the noun, or its negation.
    pub(crate) fn count_quality(
        &mut self,
        a: &Value,
        c: Cmp,
        b: &Value,
        negated: bool,
    ) -> Option<String> {
        if !matches!((c, b), (Cmp::Ge, Value::Const(1))) {
            return None;
        }
        match a {
            // "with one or more blue mana symbols in its mana cost".
            Value::Aggregate(AggOp::Sum, Stat::ManaSymbols(col), s) if tested(s) && !negated => {
                Some(format!(
                    "with one or more {} mana symbols in its mana cost",
                    col.word()
                ))
            }
            Value::Count(_) | Value::CountSel(_) => {
                let f = match a {
                    Value::Count(f) => f.clone(),
                    Value::CountSel(s) => match s.as_ref() {
                        Sel::All(f) => f.clone(),
                        _ => return None,
                    },
                    _ => return None,
                };
                let ps = parts(&f);
                // "with a legendary Equipment attached to it".
                if let Some(i) = ps
                    .iter()
                    .position(|p| matches!(p, Filter::AttachedTo(s) if tested(s)))
                {
                    if negated {
                        return None;
                    }
                    let mut rest = ps.clone();
                    rest.remove(i);
                    let n = self.noun_det(&Filter::and(rest), super::nouns::Det::A);
                    return Some(format!("with {n} attached to it"));
                }
                // "that doesn't have the same name as another permanent you control".
                let same = ps
                    .iter()
                    .position(|p| matches!(p, Filter::SameNameAs(s) if tested(s)))?;
                let mut rest: Vec<Filter> = ps
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != same)
                    .map(|(_, p)| p.clone())
                    .collect();
                let other = rest.iter().position(|p| {
                    matches!(p, Filter::Not(x) if matches!(x.as_ref(), Filter::In(s) if tested(s)))
                });
                let another = if let Some(o) = other {
                    rest.remove(o);
                    true
                } else {
                    false
                };
                let n = self.noun(&Filter::and(rest), Num::One);
                let n = if another {
                    format!("another {n}")
                } else {
                    with_article(&n)
                };
                Some(if negated {
                    format!("that doesn't have the same name as {n}")
                } else {
                    format!("that has the same name as {n}")
                })
            }
            _ => None,
        }
    }

    /// "that shares a land type with a land you control" (CR 205.3i): for some land type,
    /// the object has it and one of those objects has it.
    pub(crate) fn shares_land_type(&mut self, alts: &[Filter], negated: bool) -> Option<String> {
        let mut objs: Option<Filter> = None;
        let mut n = 0;
        for a in alts {
            let pa = parts(a);
            let [Filter::Subtype(t), Filter::ValueCmp(v, Cmp::Ge, one)] = pa.as_slice() else {
                return None;
            };
            if !matches!(one.as_ref(), Value::Const(1)) {
                return None;
            }
            let Value::CountSel(s) = v.as_ref() else {
                return None;
            };
            let Sel::All(Filter::And(fs)) = s.as_ref() else {
                return None;
            };
            let [rest @ .., Filter::Subtype(t2)] = fs.as_slice() else {
                return None;
            };
            if t != t2 || rest.is_empty() {
                return None;
            }
            let base = &Filter::and(rest.to_vec());
            match &objs {
                Some(b) if format!("{b:?}") != format!("{base:?}") => return None,
                Some(_) => {}
                None => objs = Some(base.clone()),
            }
            n += 1;
        }
        if n < crate::types::land_types().len() {
            return None;
        }
        let o = self.noun_det(&objs?, super::nouns::Det::A);
        Some(if negated {
            format!("that doesn't share a land type with {o}")
        } else {
            format!("that shares a land type with {o}")
        })
    }

    /// "For each card type among noncreature spells you've cast this turn, you may put a
    /// card of that type from among the revealed cards into your hand", "For each of those
    /// colors, you may exile a card of that color from among the revealed cards": a card
    /// of each kind some count says there is.
    pub(crate) fn take_for_each_kind(
        &mut self,
        each_of: &[Filter],
        up_to: bool,
        to: &Destination,
    ) -> Option<String> {
        if each_of.len() < 2 || !up_to {
            return None;
        }
        enum Kind {
            Type,
            Color,
        }
        let mut kind = None;
        let mut base: Option<String> = None;
        for f in each_of {
            let pf = parts(f);
            let [k, Filter::ValueCmp(v, Cmp::Ge, one)] = pf.as_slice() else {
                return None;
            };
            if !matches!(one.as_ref(), Value::Const(1)) {
                return None;
            }
            let (this_kind, rest) = match (k, v.as_ref()) {
                // Spells you've cast this turn with that type.
                (Filter::Type(t), Value::SpellsCastThisTurn(PlayerRef::You, sf)) => {
                    let mut ps = parts(sf);
                    let i = ps
                        .iter()
                        .position(|p| matches!(p, Filter::Type(x) if x == t))?;
                    ps.remove(i);
                    let mut sp = ps;
                    sp.push(Filter::Spell);
                    let n = self.noun(&Filter::and(sp), Num::Many);
                    (Kind::Type, format!("among {n} you've cast this turn"))
                }
                // Permanents you control of that color.
                (Filter::Color(c), Value::CountSel(s)) => {
                    let Sel::All(pf) = s.as_ref() else {
                        return None;
                    };
                    let mut ps = parts(pf);
                    let i = ps
                        .iter()
                        .position(|p| matches!(p, Filter::Color(x) if x == c))?;
                    ps.remove(i);
                    let n = self.noun(&Filter::and(ps), Num::Many);
                    (Kind::Color, format!("among {n}"))
                }
                _ => return None,
            };
            match (&kind, this_kind) {
                (None, k) => kind = Some(k),
                (Some(Kind::Type), Kind::Type) | (Some(Kind::Color), Kind::Color) => {}
                _ => return None,
            }
            match &base {
                Some(b) if *b != rest => return None,
                Some(_) => {}
                None => base = Some(rest),
            }
        }
        let base = base?;
        let (each, card) = match kind? {
            Kind::Type => (format!("for each card type {base}"), "a card of that type"),
            Kind::Color => (
                format!("{{alt:for each of those colors|for each color {base}}}"),
                "a card of that color",
            ),
        };
        let among = "from among {alt:the revealed cards|them}";
        let act = match to.zone {
            ZoneKind::Exile => format!("exile {card} {among}"),
            _ => {
                let d = self.dig_destination(to, false);
                format!("put {card} {among} {d}")
            }
        };
        let _ = Case::Obj;
        Some(format!("{each}, you may {act}"))
    }
}

/// The kinds of a choice stored as a number ("Choose land or nonland": 1 or 0 in a
/// variable), as the filter that tests it says them: (var, [(number, kind)]).
pub(crate) fn chosen_kinds(f: &Filter) -> Option<(Var, Vec<(i32, Filter)>)> {
    let Filter::Or(alts) = f else {
        return None;
    };
    let mut var = None;
    let mut out = Vec::new();
    for a in alts {
        let pa = parts(a);
        let [Filter::ValueCmp(v, Cmp::Eq, k), kind] = pa.as_slice() else {
            return None;
        };
        let (Value::Var(x), Value::Const(k)) = (v.as_ref(), k.as_ref()) else {
            return None;
        };
        if var.is_some_and(|y| y != *x) {
            return None;
        }
        var = Some(*x);
        out.push((*k, kind.clone()));
    }
    (out.len() >= 2).then_some((var?, out))
}

impl Renderer<'_> {
    /// "Choose land or nonland. Reveal cards from the top of your library until you reveal
    /// a card of the chosen kind." (`v[0]` is the choice).
    pub(crate) fn choose_kind_then_reveal(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        }, next, ..] = v
        else {
            return None;
        };
        let filter = match next {
            Effect::RevealUntil { filter, .. } => filter,
            _ => return None,
        };
        let kinds = parts(filter).into_iter().find_map(|p| chosen_kinds(&p))?;
        let mut words = Vec::new();
        for (_, eff) in options {
            let Effect::StoreValue {
                var,
                value: Value::Const(k),
            } = eff
            else {
                return None;
            };
            if *var != kinds.0 {
                return None;
            }
            let (_, kf) = kinds.1.iter().find(|(n, _)| n == k)?;
            let n = self.card_noun(kf);
            words.push(n.strip_suffix(" card").unwrap_or(&n).to_string());
        }
        let r = self.effect(next);
        Some((format!("choose {}. {r}", join_list(&words, "or")), 2))
    }
}
