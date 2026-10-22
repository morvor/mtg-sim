//! Static abilities that also affect spells and cards outside the battlefield
//! (`patterns/statics_other_zones.rs`, `choice_grammar_statics.rs`, `becomes_grammar.rs`;
//! CR 611.3a): "Nonland permanents you control are white. The same is true for spells you
//! control and nonland cards you own that aren't on the battlefield." (Celestial Dawn),
//! "Lands you control and land cards you own that aren't on the battlefield are Deserts in
//! addition to their other types." (Dune Chanter). The cards "that aren't on the
//! battlefield" are compiled as one alternative for each zone a card can be in but the
//! battlefield, the stack included (a spell is a card on the stack, CR 112.1).

use super::*;
use crate::oracle::patterns::statics_other_zones::NOT_ON_BATTLEFIELD;

impl Renderer<'_> {
    pub(crate) fn off_battlefield_static(&mut self, s: &StaticAbility) -> Option<String> {
        if s.condition.is_some() {
            return None;
        }
        let StaticEffect::Continuous {
            affected: Filter::Or(alts),
            mods,
        } = &s.effect
        else {
            return None;
        };
        // The cards in each zone but the battlefield: one quality, in every such zone.
        let zone_of = |f: &Filter| match f {
            Filter::And(v) => v.iter().find_map(|x| match x {
                Filter::InZone(z) if NOT_ON_BATTLEFIELD.contains(z) => Some(*z),
                _ => None,
            }),
            _ => None,
        };
        let base_of = |f: &Filter| match f {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|x| !matches!(x, Filter::InZone(_)))
                    .cloned()
                    .collect(),
            ),
            other => other.clone(),
        };
        let carded: Vec<&Filter> = alts
            .iter()
            .filter(|f| {
                zone_of(f).is_some()
                    && matches!(f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::Card))
                        && v.iter().any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You))))
            })
            .collect();
        let base = base_of(carded.first()?);
        let key = format!("{base:?}");
        let zones: Vec<ZoneKind> = carded
            .iter()
            .filter(|f| format!("{:?}", base_of(f)) == key)
            .filter_map(|f| zone_of(f))
            .collect();
        if zones.len() != carded.len() || !NOT_ON_BATTLEFIELD.iter().all(|z| zones.contains(z)) {
            return None;
        }
        let Filter::And(bv) = &base else {
            return None;
        };
        if !bv
            .iter()
            .any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You)))
        {
            return None;
        }
        let kind = Filter::and(
            bv.iter()
                .filter(|x| !matches!(x, Filter::OwnedBy(_)))
                .cloned()
                .collect(),
        );
        let cards = format!(
            "{} you own that aren't on the battlefield",
            self.noun(&kind, Num::Many)
        );
        // Spells (on the stack) and permanents (on the battlefield).
        let is_spell = |f: &Filter| match f {
            Filter::And(v) => v
                .iter()
                .any(|x| matches!(x, Filter::Spell | Filter::InZone(ZoneKind::Stack))),
            Filter::Spell => true,
            _ => false,
        };
        let others: Vec<&Filter> = alts
            .iter()
            .filter(|f| !carded.iter().any(|c| std::ptr::eq(*c, *f)))
            .collect();
        let spells: Vec<&Filter> = others.iter().copied().filter(|f| is_spell(f)).collect();
        let permanents: Vec<Filter> = others
            .iter()
            .copied()
            .filter(|f| !is_spell(f))
            .cloned()
            .collect();
        if permanents.is_empty() || spells.len() > 1 {
            return None;
        }
        let mut first = s.clone();
        first.effect = StaticEffect::Continuous {
            affected: if permanents.len() == 1 {
                permanents[0].clone()
            } else {
                Filter::Or(permanents.clone())
            },
            mods: mods.clone(),
        };
        let line = self.static_ability_conditioned(&first);
        match spells.first() {
            // "The same is true for spells you control and nonland cards you own that
            // aren't on the battlefield."
            Some(sp) => {
                let k = match sp {
                    Filter::And(v) => Filter::and(
                        v.iter()
                            .filter(|x| {
                                !matches!(
                                    x,
                                    Filter::Spell
                                        | Filter::Card
                                        | Filter::InZone(_)
                                        | Filter::ControlledBy(PlayerRel::You)
                                )
                            })
                            // A permanent spell is one of a permanent card type (CR 110.4b).
                            .map(|x| match x {
                                Filter::PermanentCard => Filter::Permanent,
                                other => other.clone(),
                            })
                            .collect(),
                    ),
                    _ => Filter::Any,
                };
                let sn = if matches!(k, Filter::Any) {
                    "spells".to_string()
                } else if matches!(k, Filter::Permanent) {
                    "permanent spells".to_string()
                } else {
                    self.spell_noun_plural(&k)
                };
                let line = line.trim_end_matches('.');
                Some(format!(
                    "{line}. The same is true for {sn} you control and {cards}"
                ))
            }
            // "Lands you control and land cards you own that aren't on the battlefield
            // are Deserts ...": the battlefield's objects named first.
            None => {
                let subj = self.noun(&first_affected(&first), Num::Many);
                let rest = line.strip_prefix(&capitalize(&subj))?;
                Some(format!("{} and {cards}{rest}", capitalize(&subj)))
            }
        }
    }
}

fn first_affected(s: &StaticAbility) -> Filter {
    match &s.effect {
        StaticEffect::Continuous { affected, .. } => affected.clone(),
        _ => Filter::Any,
    }
}
