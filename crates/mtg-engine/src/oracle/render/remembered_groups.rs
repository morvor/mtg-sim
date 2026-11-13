//! Instructions that remember a group first and act on it later, said as one sentence:
//! "Simultaneously untap all tapped creatures and tap all untapped creatures" (both
//! groups are determined before either changes), "If you put a Cave onto the battlefield
//! this way, you gain 4 life" (the permanent the optional instruction put there), "For
//! each blight counter on it, each opponent loses 4 life unless that player sacrifices a
//! nonland permanent of their choice or discards a card" (the number, then each player's
//! choices among their permanents and cards, all made before any is carried out).

use super::nouns::Det;
use super::*;

impl Renderer<'_> {
    /// At `v[i]`: one of these sentences, as (how many instructions, text).
    pub(crate) fn remembered_group_part(
        &mut self,
        v: &[Effect],
        i: usize,
    ) -> Option<(usize, String)> {
        self.simultaneous_tap_untap(v, i)
            .or_else(|| self.put_this_way(v, i))
            .or_else(|| self.punisher_for_each(v, i))
    }

    /// "Simultaneously untap all tapped creatures and tap all untapped creatures": the two
    /// groups are remembered before either instruction changes them.
    fn simultaneous_tap_untap(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let (
            Some(Effect::Store {
                var: a,
                sel: Sel::All(fa),
            }),
            Some(Effect::Store {
                var: b,
                sel: Sel::All(fb),
            }),
            Some(Effect::Untap { what: Sel::Var(x) }),
            Some(Effect::Tap { what: Sel::Var(y) }),
        ) = (v.get(i), v.get(i + 1), v.get(i + 2), v.get(i + 3))
        else {
            return None;
        };
        if x != a || y != b || a == b {
            return None;
        }
        let ua = self.noun_det(fa, Det::Plural);
        let tb = self.noun_det(fb, Det::Plural);
        Some((4, format!("simultaneously untap all {ua} and tap all {tb}")))
    }

    /// "If you put a Cave onto the battlefield this way, [instruction]": after "you may
    /// put a land card from your hand onto the battlefield", the permanent it put there
    /// (none if you didn't), checked for a quality.
    fn put_this_way(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let (
            Some(Effect::If {
                cond: Condition::PrevHappened,
                then,
                otherwise,
            }),
            Some(Effect::Store {
                var: z,
                sel: Sel::All(Filter::And(zf)),
            }),
            Some(Effect::If {
                cond: Condition::And(conds),
                then: effect,
                otherwise: no,
            }),
        ) = (v.get(i), v.get(i + 1), v.get(i + 2))
        else {
            return None;
        };
        let (
            Effect::Store {
                var: y,
                sel: Sel::Var(vars::IT),
            },
            Effect::Store {
                var: y2,
                sel: Sel::None,
            },
        ) = (then.as_ref(), otherwise.as_ref())
        else {
            return None;
        };
        if y != y2 || !matches!(no.as_ref(), Effect::Noop) || z == y {
            return None;
        }
        // The instruction before put it onto the battlefield.
        let put = |e: &Effect| -> bool {
            fn moves_to_battlefield(e: &Effect) -> bool {
                match e {
                    Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield,
                    Effect::May { effect, .. } => moves_to_battlefield(effect),
                    Effect::Seq(s) => s.iter().any(moves_to_battlefield),
                    _ => false,
                }
            }
            moves_to_battlefield(e)
        };
        if !i.checked_sub(1).and_then(|p| v.get(p)).is_some_and(put) {
            return None;
        }
        // The remembered permanent is of the quality: "a Cave".
        let [Condition::SelNonEmpty(s1), Condition::Not(n)] = conds.as_slice() else {
            return None;
        };
        let Condition::SelMatches(s2, Filter::Not(f)) = n.as_ref() else {
            return None;
        };
        let is_y = |s: &Sel| matches!(s, Sel::Var(x) if x == y);
        if !is_y(s1) || !is_y(s2) {
            return None;
        }
        // The group of those of the quality, which nothing names, is the same quality.
        let in_y = |x: &Filter| matches!(x, Filter::In(s) if is_y(s));
        let zrest: Vec<Filter> = zf.iter().filter(|x| !in_y(x)).cloned().collect();
        if zrest.len() + 1 != zf.len() || format!("{:?}", Filter::and(zrest)) != format!("{f:?}") {
            return None;
        }
        let n = self.noun_det(f, Det::A);
        let e = self.effect(effect);
        Some((
            3,
            format!("if you put {n} onto the battlefield this way, {e}"),
        ))
    }

    /// "For each blight counter on it, each opponent loses 4 life unless that player
    /// sacrifices a nonland permanent of their choice or discards a card": the number
    /// remembered, then each player chooses up to that many of their permanents and cards
    /// in hand (each choice once, Rottenmouth Viper's rulings), sacrifices and discards
    /// them, and loses life once for each choice they didn't make.
    fn punisher_for_each(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let (
            Some(Effect::StoreValue { var: n, value }),
            Some(Effect::ForEachPlayer { who, effect }),
        ) = (v.get(i), v.get(i + 1))
        else {
            return None;
        };
        let Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect,
        } = effect.as_ref()
        else {
            return None;
        };
        let Effect::Seq(steps) = effect.as_ref() else {
            return None;
        };
        let [Effect::Store {
            var: c,
            sel:
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::Or(options),
                    count: Value::Var(n2),
                    up_to: true,
                    ..
                },
        }, Effect::StoreValue {
            var: d,
            value: Value::Diff(a, b),
        }, Effect::SacrificeObjects { what: sac }, Effect::Discard {
            who: PlayerRef::You,
            ..
        }, Effect::Repeat {
            times: Value::Var(d2),
            effect: punish,
        }] = steps.as_slice()
        else {
            return None;
        };
        let is_c = |s: &Sel| matches!(s, Sel::Var(x) if x == c);
        if n2 != n
            || d2 != d
            || !matches!(a.as_ref(), Value::Var(x) if x == n)
            || !matches!(b.as_ref(), Value::CountSel(s) if is_c(s))
            || !matches!(sac, Sel::Matching(s, Filter::InZone(ZoneKind::Battlefield)) if is_c(s))
        {
            return None;
        }
        // What's counted: "blight counter on ~".
        let each = match value {
            Value::CountersOn(s, Some(k)) if matches!(s.as_ref(), Sel::This) => {
                let m = self.me();
                format!("{} on {m}", counter_name(k))
            }
            _ => return None,
        };
        // The choices: a permanent they control, a card in their hand.
        let mut alts = Vec::new();
        for o in options {
            let Filter::And(parts) = o else {
                return None;
            };
            if parts
                .iter()
                .any(|x| matches!(x, Filter::InZone(ZoneKind::Hand)))
            {
                alts.push("discards a card".to_string());
            } else if parts
                .iter()
                .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You)))
            {
                let rest = Filter::and(
                    parts
                        .iter()
                        .filter(|x| !matches!(x, Filter::ControlledBy(_)))
                        .cloned()
                        .collect(),
                );
                let n = self.noun_det(&rest, Det::A);
                alts.push(format!("sacrifices {n} of their choice"));
            } else {
                return None;
            }
        }
        let w = self.player(who, super::players::Case::Subj);
        let inner = self.with_subject_as(punish)?;
        Some((
            2,
            format!(
                "for each {each}, {w} {inner} unless that player {}",
                join_list(&alts, "or")
            ),
        ))
    }

    /// An instruction a player performs as "you", in the third person: "loses 4 life".
    fn with_subject_as(&mut self, e: &Effect) -> Option<String> {
        let vp = self.as_each_player(&Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e.clone()),
        });
        Some(super::effects::third_person(&vp))
    }
}
