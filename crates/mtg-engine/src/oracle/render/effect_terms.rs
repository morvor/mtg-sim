//! Instructions and ability lines worded as a whole from the shape of their AST:
//! "Each player sacrifices an artifact, a creature, and a land of their choice" (one
//! permanent of each kind, `kw/choice_grammar.rs`), "Unattach all Equipment from target
//! creature" (CR 701.3d), "For each of those tokens, you may attach an Equipment you
//! control to it", "If it's paired with a creature, that creature also gets +2/+2"
//! (CR 702.95), "Choose target opponent who has more life than you do as you activate
//! this ability" (a target nothing else refers to, CR 115.1), "∞ — [ability]"
//! (CR 702.186b).

use super::nouns::Det;
use super::players::Case;
use super::*;
use crate::kw::choice_grammar::{OneOfEach, ONE_OF_EACH};

/// The kinds of a "one permanent of each kind" choice sequence that ends with sacrificing
/// the chosen permanents: `Seq[Store(v0, Choose ..), Store(v1, Union[v0, Choose ..]), ...,
/// SacrificeObjects(vN)]`.
fn one_of_each_sacrifice(e: &Effect) -> Option<Vec<Filter>> {
    let Effect::Seq(v) = e else {
        return None;
    };
    let (last, stores) = v.split_last()?;
    let Effect::SacrificeObjects { what: Sel::Var(sv) } = last else {
        return None;
    };
    let mut kinds = None;
    let mut prev: Option<Var> = None;
    for (i, s) in stores.iter().enumerate() {
        let Effect::Store { var, sel } = s else {
            return None;
        };
        let choose = match (sel, prev) {
            (c @ Sel::Choose { .. }, None) => c,
            (Sel::Union(u), Some(p)) => match u.as_slice() {
                [Sel::Var(a), c @ Sel::Choose { .. }] if *a == p => c,
                _ => return None,
            },
            _ => return None,
        };
        let Sel::Choose {
            chooser: PlayerRef::Iterated,
            filter: Filter::And(parts),
            count: Value::Const(1),
            up_to: false,
            store: None,
        } = choose
        else {
            return None;
        };
        let json = parts.iter().find_map(|p| match p {
            Filter::Custom(n) => n.strip_prefix(ONE_OF_EACH),
            _ => None,
        })?;
        let o: OneOfEach = serde_json::from_str(json).ok()?;
        if o.slot != i {
            return None;
        }
        if kinds.is_none() {
            kinds = Some(o.kinds);
        }
        prev = Some(*var);
    }
    let kinds = kinds?;
    (prev == Some(*sv) && kinds.len() == stores.len()).then_some(kinds)
}

impl Renderer<'_> {
    /// The instruction, if this module words it.
    pub(crate) fn effect_terms(&mut self, e: &Effect) -> Option<String> {
        match e {
            // "Target player sacrifices an artifact and a land of their choice."
            Effect::AsPlayer { who, effect } => {
                let Effect::ForEachPlayer {
                    who: PlayerRef::You,
                    effect: inner,
                } = effect.as_ref()
                else {
                    return None;
                };
                let kinds = one_of_each_sacrifice(inner)?;
                let w = self.player(who, Case::Subj);
                Some(self.sacrifice_one_of_each(&w, &kinds))
            }
            // "Each player sacrifices an artifact, a creature, ... of their choice."
            Effect::ForEachPlayer { who, effect } => {
                let kinds = one_of_each_sacrifice(effect)?;
                let w = self.player(who, Case::Subj);
                Some(self.sacrifice_one_of_each(&w, &kinds))
            }
            // "For each of those tokens, you may attach an Equipment you control to it."
            Effect::ForEach {
                sel: sel @ Sel::Var(vars::CREATED),
                var,
                effect,
            } => {
                self.var_defs.push((*var, sel.clone(), false));
                let saved = self.for_each_subject.replace("it".into());
                let t = self.effect(effect);
                self.for_each_subject = saved;
                Some(format!(
                    "{{opt:for each of {{alt:them|those tokens}},}} {t}"
                ))
            }
            // "Unattach all Equipment from target creature."
            Effect::ForEach { sel, var, effect } => {
                let Effect::Unattach {
                    what: Sel::All(Filter::And(parts)),
                } = effect.as_ref()
                else {
                    return None;
                };
                let affected = |p: &Filter| matches!(p, Filter::Custom(n) if n == "attached_to_affected");
                if !parts.iter().any(affected) {
                    return None;
                }
                let rest = Filter::and(parts.iter().filter(|p| !affected(p)).cloned().collect());
                let _ = var;
                let n = self.noun(&rest, Num::Many);
                let from = self.sel(sel, Case::Obj);
                Some(format!("unattach all {n} from {from}"))
            }
            // "If it's paired with a creature, that creature also gets +2/+2 until end of
            // turn."
            Effect::Modify {
                what: Sel::All(Filter::Custom(n)),
                mods,
                duration,
            } if n == crate::kw::soulbond::PAIRED_WITH_TARGET => {
                let other = Effect::Modify {
                    what: Sel::All(Filter::And(vec![Filter::Custom(n.clone())])),
                    mods: mods.clone(),
                    duration: duration.clone(),
                };
                let s = self.effect(&other);
                Some(format!("if it's paired with a creature, {s}"))
            }
            _ => None,
        }
    }

    fn sacrifice_one_of_each(&mut self, who: &str, kinds: &[Filter]) -> String {
        let saved = std::mem::replace(&mut self.default_head, Some("permanent"));
        let items: Vec<String> = kinds
            .iter()
            .map(|k| self.noun_det(k, Det::A))
            .collect();
        self.default_head = saved;
        let verb = if who == "you" { "sacrifice" } else { "sacrifices" };
        format!(
            "{who} {verb} {} {{opt:of their choice}}",
            join_list(&items, "and")
        )
    }

    /// "Choose target opponent who ... as you activate this ability." before the
    /// instructions, for each target they never refer to: the target is chosen and does
    /// nothing else (CR 115.1, 601.2c).
    pub(crate) fn choose_unmentioned_targets(&mut self, s: String) -> String {
        let missing: Vec<usize> = (0..self.targets.len())
            .filter(|i| !self.introduced[*i])
            .collect();
        if missing.is_empty() {
            return s;
        }
        let mut pre = Vec::new();
        let mut s = s;
        for i in missing {
            let chooser = self.targets[i].chosen_by.take();
            let t = self.target_mention(i as u8, Case::Obj);
            let verb = match &chooser {
                Some(p) => {
                    let w = self.player(p, Case::Subj);
                    format!("{w} chooses")
                }
                None => "choose".to_string(),
            };
            // "That player chooses target player ... The first player may ...": the
            // chooser is the first of the two players named.
            if matches!(chooser, Some(PlayerRef::TriggerPlayer)) {
                s = s.replacen(
                    "{alt:that player|they}",
                    "{alt:that player|they|the first player}",
                    1,
                );
            }
            self.targets[i].chosen_by = chooser;
            pre.push(format!("{verb} {t}."));
        }
        format!("{} {s}", pre.join(" "))
    }

    /// A player quality worded as the comparison with another player: "who has more life
    /// than you do", "who controls more lands than they do and is their opponent".
    pub(crate) fn player_quality_terms(&mut self, pf: &PlayerFilter) -> Option<String> {
        match pf {
            PlayerFilter::Life(Cmp::Gt, v) => {
                let Value::LifeTotal(p) = v.as_ref() else {
                    return None;
                };
                let w = self.player(p, Case::Subj);
                Some(format!("who has more life than {w} do"))
            }
            PlayerFilter::HandSize(Cmp::Ge, v) => {
                let Value::Sum(parts) = v.as_ref() else {
                    return None;
                };
                let [Value::HandSize(p), Value::Const(k)] = parts.as_slice() else {
                    return None;
                };
                let w = self.player(p, Case::Subj);
                Some(format!(
                    "who has at least {} more cards in hand than {w} do",
                    number_word(*k)
                ))
            }
            PlayerFilter::HandSize(Cmp::Gt, v) => {
                let Value::HandSize(p) = v.as_ref() else {
                    return None;
                };
                let w = self.player(p, Case::Subj);
                Some(format!("who has more cards in hand than {w} do"))
            }
            PlayerFilter::Controls(f, Cmp::Gt, v) => {
                let Value::Count(g) = v.as_ref() else {
                    return None;
                };
                let Filter::And(gv) = g else {
                    return None;
                };
                let p = gv.iter().find_map(|x| match x {
                    Filter::ControlledBy(r) => Some(Err(*r)),
                    Filter::ControlledByPlayer(r) => Some(Ok(r.as_ref().clone())),
                    _ => None,
                })?;
                let rest: Vec<Filter> = gv
                    .iter()
                    .filter(|x| !matches!(x, Filter::ControlledBy(_) | Filter::ControlledByPlayer(_)))
                    .cloned()
                    .collect();
                if format!("{:?}", Filter::and(rest)) != format!("{:?}", f.as_ref()) {
                    return None;
                }
                let n = self.noun(f, Num::Many);
                let w = match p {
                    Ok(p) => self.player(&p, Case::Subj),
                    Err(r) => self.rel_subject(r),
                };
                Some(format!("who controls more {n} than {w} do"))
            }
            // "who controls more lands than they do and is their opponent".
            PlayerFilter::And(v) => {
                let [PlayerFilter::OpponentOf(r), q] = v.as_slice() else {
                    return None;
                };
                let q = self.player_quality_terms(q)?;
                let p = self.player(r, Case::Poss);
                Some(format!("{q} and is {p} opponent"))
            }
            _ => None,
        }
    }

    /// "∞ — [ability]": as long as the permanent is harnessed, it has the ability
    /// (CR 702.186b).
    pub(crate) fn infinity_static(&mut self, s: &StaticAbility) -> Option<String> {
        let Some(Condition::SelMatches(Sel::This, Filter::Custom(n))) = &s.condition else {
            return None;
        };
        if n != crate::kwa::harness::HARNESSED {
            return None;
        }
        let StaticEffect::Continuous {
            affected: Filter::Source,
            mods,
        } = &s.effect
        else {
            return None;
        };
        let [Modification::AddAbility(a)] = mods.as_slice() else {
            return None;
        };
        let line = self.ability(a);
        Some(format!("∞ — {line}"))
    }
}
