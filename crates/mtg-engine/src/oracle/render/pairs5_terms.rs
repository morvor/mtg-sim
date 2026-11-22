//! Wordings from roundtrip-pairs-5 (docs/agents/items/roundtrip-pairs-5.md), each put into
//! words from the AST it is compiled to.

use super::*;

impl Renderer<'_> {
    /// "At the beginning of your upkeep, you lose 1 life unless you control another
    /// colorless creature.": an instruction done if you control no such permanent, said
    /// either way (`s` is the "if" wording).
    pub(crate) fn unless_you_control(
        &mut self,
        cond: &Condition,
        then: &Effect,
        otherwise: &Effect,
        s: String,
    ) -> String {
        let Condition::Not(c) = cond else {
            return s;
        };
        let Condition::Exists(Filter::And(v)) = c.as_ref() else {
            return s;
        };
        let simple = matches!(
            then,
            Effect::LoseLife { .. }
                | Effect::SacrificeObjects { what: Sel::This }
                | Effect::Tap { what: Sel::This }
        );
        if !matches!(otherwise, Effect::Noop)
            || !simple
            || !v
                .iter()
                .any(|f| matches!(f, Filter::ControlledBy(PlayerRel::You)))
        {
            return s;
        }
        let pos = self.condition(c);
        if !pos.starts_with("you control ") {
            return s;
        }
        let t = self.effect(then);
        format!("{{alt:{s}|{t} unless {pos}}}")
    }
}

impl Renderer<'_> {
    /// A player who controls fewer [things] than you (`Controls(f, <, the number of f you
    /// control)`): the plural noun.
    pub(crate) fn fewer_than_yours(&mut self, p: &PlayerFilter) -> Option<String> {
        let PlayerFilter::Controls(f, Cmp::Lt, n) = p else {
            return None;
        };
        let Value::Count(Filter::And(v)) = n.as_ref() else {
            return None;
        };
        let [g, Filter::ControlledBy(PlayerRel::You)] = v.as_slice() else {
            return None;
        };
        if format!("{g:?}") != format!("{f:?}") {
            return None;
        }
        Some(self.noun(f, Num::Many))
    }
}

impl Renderer<'_> {
    /// "Target creature gets -2/-2 until end of turn. It gets an additional -1/-1 until end
    /// of turn for each Desert you control." (and "If ~ was kicked, that creature gets an
    /// additional ..."): a second change to the same object's power and toughness, scaled.
    pub(crate) fn additional_pt(&mut self, v: &[Effect]) -> Option<String> {
        let [first @ Effect::Modify {
            what,
            duration: Duration::EndOfTurn,
            mods,
        }, last] = v
        else {
            return None;
        };
        if !matches!(mods.as_slice(), [Modification::ModifyPT(..)]) {
            return None;
        }
        let (cond, second) = match last {
            Effect::If {
                cond,
                then,
                otherwise,
            } if matches!(otherwise.as_ref(), Effect::Noop) => (Some(cond), then.as_ref()),
            other => (None, other),
        };
        let Effect::Modify {
            what: w2,
            duration: Duration::EndOfTurn,
            mods: m2,
        } = second
        else {
            return None;
        };
        if format!("{what:?}") != format!("{w2:?}") {
            return None;
        }
        let [Modification::ModifyPT(Value::Mul(a, x), Value::Mul(b, y))] = m2.as_slice() else {
            return None;
        };
        let (Value::Const(p), Value::Const(t), Value::Count(f)) =
            (a.as_ref(), b.as_ref(), x.as_ref())
        else {
            return None;
        };
        if format!("{x:?}") != format!("{y:?}") {
            return None;
        }
        let sign = |n: i32| {
            if n < 0 {
                format!("{n}")
            } else {
                format!("+{n}")
            }
        };
        let s1 = self.effect_sentences(first);
        let each = self.for_each_noun(f);
        let gets = format!(
            "gets an additional {}/{} until end of turn for each {each}",
            sign(*p),
            sign(*t)
        );
        Some(match cond {
            Some(c) => {
                let c = self.condition(c);
                format!("{s1} If {c}, {{alt:it|that creature}} {gets}")
            }
            None => format!("{s1} {{alt:It|That creature}} {gets}"),
        })
    }
}

impl Renderer<'_> {
    /// "You and permanents you control gain hexproof until end of turn." (the player's
    /// hexproof and the permanents', CR 702.11), then the rest of the instructions.
    pub(crate) fn you_and_yours_gain_hexproof(&mut self, v: &[Effect]) -> Option<String> {
        let [Effect::AddPlayerEffect {
            who: PlayerRef::You,
            effect: PlayerModification::Hexproof,
            duration: Duration::EndOfTurn,
        }, Effect::Modify {
            what: Sel::All(f),
            mods,
            duration: Duration::EndOfTurn,
        }, rest @ ..] = v
        else {
            return None;
        };
        let [Modification::AddKeyword(k)] = mods.as_slice() else {
            return None;
        };
        if k.kind != crate::keywords::KeywordKind::Hexproof || k.filter.is_some() {
            return None;
        }
        let n = self.noun(f, Num::Many);
        let first = format!("You and {n} gain hexproof until end of turn.");
        if rest.is_empty() {
            return Some(first);
        }
        let r = self.seq(rest);
        Some(format!("{first} {}", r))
    }
}
