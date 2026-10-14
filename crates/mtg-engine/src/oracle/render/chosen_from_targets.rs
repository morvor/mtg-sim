//! "Choose two target creatures controlled by the same opponent. That player chooses one
//! of those creatures. ...": the targets named first, then their controller's choice.

use super::*;

impl Renderer<'_> {
    pub(crate) fn chosen_from_targets_part(
        &mut self,
        v: &[Effect],
        i: usize,
    ) -> Option<(usize, String)> {
        let (
            Effect::Store {
                var: tv,
                sel: Sel::Target(t),
            },
            Some(Effect::Store {
                var: w,
                sel: Sel::Union(empty),
            }),
            Some(Effect::ForEachPlayer {
                who: PlayerRef::ControllerOf(of),
                effect,
            }),
        ) = (&v[i], v.get(i + 1), v.get(i + 2))
        else {
            return None;
        };
        if !empty.is_empty()
            || !matches!(of.as_ref(), Sel::Target(x) if x == t)
            || self.introduced.get(*t as usize) != Some(&false)
        {
            return None;
        }
        let Effect::Store {
            var: w2,
            sel: Sel::Union(parts),
        } = effect.as_ref()
        else {
            return None;
        };
        let [Sel::Var(w3), Sel::Choose {
            chooser: PlayerRef::Iterated,
            filter: Filter::In(from),
            count: Value::Const(1),
            up_to: false,
            ..
        }] = parts.as_slice()
        else {
            return None;
        };
        if w2 != w || w3 != w || !matches!(from.as_ref(), Sel::Var(x) if x == tv) {
            return None;
        }
        let tp = self.target_phrase(*t);
        self.introduced[*t as usize] = true;
        self.plural_vars.push(*tv);
        let who = "{alt:that player|its controller}";
        let them = "{alt:them|those creatures}";
        // "That player sacrifices one of them" / "Its controller chooses and sacrifices one
        // of them".
        if matches!(v.get(i + 3), Some(Effect::SacrificeObjects { what: Sel::Var(x) }) if x == w) {
            return Some((
                4,
                format!(
                    "choose {tp}. {who} {{alt:sacrifices|chooses and sacrifices}} one of {them}"
                ),
            ));
        }
        Some((3, format!("choose {tp}. {who} chooses one of {them}")))
    }
}

impl Renderer<'_> {
    /// "Any opponent may sacrifice a creature. If a player does, ...": each opponent in
    /// turn order may choose a creature they control, and the chosen ones are sacrificed
    /// (Clackbridge Troll's rulings); "if a player does" is whether any was.
    pub(crate) fn any_opponent_sacrifices_part(
        &mut self,
        v: &[Effect],
        i: usize,
    ) -> Option<(usize, String)> {
        let (
            Effect::Store {
                var: u,
                sel: Sel::Union(empty),
            },
            Some(Effect::ForEachPlayer {
                who: PlayerRef::EachOpponent,
                effect,
            }),
            Some(Effect::If {
                cond,
                then,
                otherwise,
            }),
        ) = (&v[i], v.get(i + 1), v.get(i + 2))
        else {
            return None;
        };
        if !empty.is_empty() {
            return None;
        }
        let Effect::Seq(steps) = effect.as_ref() else {
            return None;
        };
        let [Effect::Store {
            var: c,
            sel:
                Sel::Choose {
                    chooser: PlayerRef::Iterated,
                    filter,
                    count: Value::Const(1),
                    up_to: true,
                    ..
                },
        }, Effect::SacrificeObjects { what: Sel::Var(c2) }, Effect::Store {
            var: u2,
            sel: Sel::Union(acc),
        }] = steps.as_slice()
        else {
            return None;
        };
        let [Sel::Var(u3), Sel::Var(crate::ability::vars::SACRIFICED)] = acc.as_slice() else {
            return None;
        };
        if c2 != c || u2 != u || u3 != u {
            return None;
        }
        let parts: Vec<Filter> = match filter {
            Filter::And(p) => p.clone(),
            f => vec![f.clone()],
        };
        if !parts
            .iter()
            .any(|f| matches!(f, Filter::ControlledBy(PlayerRel::Iterated)))
        {
            return None;
        }
        let rest = Filter::and(
            parts
                .into_iter()
                .filter(|f| {
                    !matches!(
                        f,
                        Filter::ControlledBy(PlayerRel::Iterated)
                            | Filter::InZone(ZoneKind::Battlefield)
                    )
                })
                .collect(),
        );
        // Whether any was sacrificed.
        let counted = |c: &Condition| -> Option<bool> {
            let Condition::Compare(Value::CountSel(s), cmp, Value::Const(k)) = c else {
                return None;
            };
            if !matches!(s.as_ref(), Sel::Var(x) if x == u) {
                return None;
            }
            match (cmp, k) {
                (Cmp::Ge, 1) | (Cmp::Gt, 0) => Some(true),
                (Cmp::Eq, 0) | (Cmp::Lt, 1) | (Cmp::Le, 0) => Some(false),
                _ => None,
            }
        };
        let did = counted(cond)?;
        let (did, then) = match (
            matches!(then.as_ref(), Effect::Noop),
            matches!(otherwise.as_ref(), Effect::Noop),
        ) {
            (false, true) => (did, then.as_ref()),
            (true, false) => (!did, otherwise.as_ref()),
            _ => return None,
        };
        let n = self.noun_det(&rest, super::nouns::Det::A);
        let t = self.effect(then);
        let when = if did {
            "{alt:a player does|one or more players do}"
        } else {
            "{alt:no one does|no player does}"
        };
        Some((3, format!("any opponent may sacrifice {n}. If {when}, {t}")))
    }
}
