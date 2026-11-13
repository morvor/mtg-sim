//! "As long as an opponent controls a creature with flying not named ~, ~ has flying. The
//! same is true for first strike, trample, and protection from any color." (Escaped
//! Shapeshifter): one printed ability, compiled as a static ability for the keywords and
//! one for each color of protection.

use super::nouns::Det;
use super::*;

/// The color of a static "~ has protection from [color] as long as [objects with
/// protection from that color exist]" and the objects, if `a` is one.
fn protection_as_long_as(a: &Ability) -> Option<(crate::types::Color, Vec<Filter>)> {
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let Some(Condition::Exists(Filter::And(parts))) = &s.condition else {
        return None;
    };
    let StaticEffect::Continuous {
        affected: Filter::Source,
        mods,
    } = &s.effect
    else {
        return None;
    };
    let [Modification::AddKeyword(k)] = mods.as_slice() else {
        return None;
    };
    let Some(Filter::Color(c)) = &k.filter else {
        return None;
    };
    if k.kind != crate::keywords::KeywordKind::Protection {
        return None;
    }
    let mut rest = parts.clone();
    let i = rest.iter().position(|f| {
        matches!(f, Filter::Custom(n)
            if n.strip_prefix(crate::kw::protection_from_color_filter::PREFIX) == Some(c.word()))
    })?;
    rest.remove(i);
    Some((*c, rest))
}

impl Renderer<'_> {
    /// The line and the number of abilities it stands for.
    pub(crate) fn same_is_true_group(&mut self, abilities: &[Ability]) -> Option<(usize, String)> {
        let first = abilities.first()?;
        let AbilityKind::Static(s) = &first.kind else {
            return None;
        };
        let StaticEffect::Continuous {
            affected: Filter::Source,
            mods,
        } = &s.effect
        else {
            return None;
        };
        let [Modification::AddKeywordsOf { kinds, from }] = mods.as_slice() else {
            return None;
        };
        if s.condition.is_some() || kinds.is_empty() {
            return None;
        }
        let Filter::And(from_parts) = from else {
            return None;
        };
        // One protection ability for each color, about the same objects.
        let mut colors = Vec::new();
        for a in abilities.iter().skip(1).take(5) {
            let (c, rest) = protection_as_long_as(a)?;
            if format!("{rest:?}") != format!("{from_parts:?}") {
                return None;
            }
            colors.push(c);
        }
        if colors.len() != 5 {
            return None;
        }
        // "an opponent controls a creature with flying not named ~".
        let mut objs: Vec<Filter> = from_parts
            .iter()
            .filter(|f| {
                !matches!(
                    f,
                    Filter::ControlledBy(PlayerRel::Opponent)
                        | Filter::InZone(ZoneKind::Battlefield)
                )
            })
            .cloned()
            .collect();
        if objs.len() + 2 != from_parts.len() {
            return None;
        }
        // "with flying not named ~": the keyword, then the name.
        let not_named = objs.iter().position(|f| {
            matches!(f, Filter::Not(x) if matches!(x.as_ref(), Filter::SameNameAs(s) if matches!(s.as_ref(), Sel::This))
                || matches!(x.as_ref(), Filter::Named(n) if n.as_str() == self.info.name))
        });
        let k0 = self.keyword_kind_word(kinds[0]);
        let n = match not_named {
            Some(i) => {
                objs.remove(i);
                let base = self.noun_det(&Filter::and(objs), Det::A);
                let m = self.me();
                format!("{base} with {k0} not named {m}")
            }
            None => {
                objs.push(Filter::HasKeyword(kinds[0]));
                self.noun_det(&Filter::and(objs), Det::A)
            }
        };
        let mut others: Vec<String> = kinds[1..]
            .iter()
            .map(|k| self.keyword_kind_word(*k))
            .collect();
        others.push("protection from any color".into());
        let m = self.me();
        Some((
            6,
            format!(
                "As long as an opponent controls {n}, {m} has {k0}. The same is true for {}.",
                join_list(&others, "and")
            ),
        ))
    }
}
