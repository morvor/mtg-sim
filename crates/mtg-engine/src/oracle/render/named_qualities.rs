//! Object qualities implemented in code (`Filter::Custom`), and the permissions and
//! waiting effects they qualify, put into words from what each one checks.

use super::custom_filters::CustomQuality;
use super::*;

/// The wording of a custom quality after the noun, if this module knows it.
pub(crate) fn custom_rel_named(name: &str) -> Option<CustomQuality> {
    use crate::oracle::patterns::filter_qualities as q;
    use CustomQuality::*;
    let rel = |s: &str| Some(Rel(s.to_string()));
    // "that you've put one or more +1/+1 counters on this turn" (counters you put).
    if let Some(k) = name.strip_prefix(q::COUNTERS_PUT_BY_YOU_THIS_TURN) {
        return rel(&format!(
            "that you've put one or more {k} counters on this turn"
        ));
    }
    // "with protection from white" (a protection ability from exactly that color).
    if let Some(c) = name.strip_prefix(crate::kw::protection_from_color_filter::PREFIX) {
        crate::types::Color::from_word(c)?;
        return rel(&format!("with protection from {c}"));
    }
    // "of the first type" after "choose an order for artifacts, creatures, and lands".
    if let Some(i) = name.strip_prefix(crate::choices::NTH_CHOSEN_CARD_TYPE) {
        let i: u32 = i.parse().ok()?;
        return rel(&format!("of the {} type", ordinal_word(i + 1)));
    }
    match name {
        q::MOST_COMMON_COLOR => rel(
            "that shares a color with the most common color among all permanents or a color tied for most common",
        ),
        // CR 201.2a, 708.2a.
        q::NO_NAME => rel("that {alt:doesn't|don't} have a name"),
        q::TAP_ABILITY => rel("that has an activated ability with {T} in its cost"),
        crate::choices::CHOSEN_NAMES => {
            rel("{alt:with the chosen names|with a name chosen for ~|with one of the chosen names}")
        }
        crate::choices::FIRST_CHOSEN_TYPE => rel("of the first chosen type"),
        crate::choices::OF_THE_CHOSEN_TYPE_WORD => rel("of {alt:that type|the chosen type}"),
        crate::choices::ONLY_COLOR_ITS_CONTROLLER_CHOSE => {
            rel("that's only the color its controller chose")
        }
        _ => None,
    }
}

/// A clause saying the subject has a custom quality this module knows, for a condition.
pub(crate) fn custom_clause_named(name: &str, subj: &str, negated: bool) -> Option<String> {
    let (yes, no) = match name {
        crate::oracle::patterns::filter_qualities::MOST_COMMON_COLOR => {
            let q = "a color with the most common color among all permanents or a color tied for most common";
            (
                format!("{subj} shares {q}"),
                format!("{subj} doesn't share {q}"),
            )
        }
        // "If the card a player revealed has the name they chose".
        crate::choices::NAME_THEY_CHOSE => (
            format!("{subj} has the name they chose"),
            format!("{subj} doesn't have the name they chose"),
        ),
        _ => return None,
    };
    Some(if negated { no } else { yes })
}

impl Renderer<'_> {
    /// "As you cast spells from your hand this turn, they gain cascade": a waiting effect
    /// for each such spell this turn, not only the next (`next_spell.rs`).
    pub(crate) fn each_spell_this_turn(
        &mut self,
        filter: &Filter,
        mods: &[Modification],
        expires: &Duration,
    ) -> Option<String> {
        let each = crate::next_spell::EACH_SPELL_THIS_TURN;
        let Filter::And(parts) = filter else {
            return None;
        };
        if !parts
            .iter()
            .any(|f| matches!(f, Filter::Custom(n) if n == each))
            || !matches!(expires, Duration::EndOfTurn | Duration::ThisTurn)
        {
            return None;
        }
        let rest: Vec<&Filter> = parts
            .iter()
            .filter(|f| !matches!(f, Filter::Custom(n) if n == each))
            .collect();
        // Spells you cast from your hand: in your hand as you begin to cast them, or cast
        // from there.
        let from_hand = rest.iter().any(|f| {
            matches!(f, Filter::Or(v) if matches!(v.as_slice(),
                [Filter::InZone(ZoneKind::Hand), Filter::CastFrom(ZoneKind::Hand)]))
        });
        let others: Vec<Filter> = rest
            .iter()
            .filter(|f| {
                !matches!(
                    f,
                    Filter::Or(_) | Filter::OwnedBy(PlayerRel::You) | Filter::Spell
                )
            })
            .map(|f| (*f).clone())
            .collect();
        if !from_hand || !others.is_empty() {
            return None;
        }
        let (vp, _) = self.mods_vp_split(mods, true);
        let vp = vp.trim_start_matches("gains ").trim_start_matches("gain ");
        Some(format!(
            "as you cast spells from your hand this turn, they gain {vp}"
        ))
    }

    /// "You may cast the first creature spell you cast each turn as though it had flash",
    /// "You may cast ~ as though it had flash if X is 3 or less" (CR 601.3b, 601.2e).
    pub(crate) fn flash_permission_named(
        &mut self,
        who: PlayerRel,
        what: &Filter,
    ) -> Option<String> {
        if who != PlayerRel::You {
            return None;
        }
        let Filter::And(parts) = what else {
            return None;
        };
        // "~ ... if X is 3 or less".
        if let [Filter::Source, Filter::Or(v)] = parts.as_slice() {
            if let [Filter::Not(z), Filter::Custom(n)] = v.as_slice() {
                if matches!(z.as_ref(), Filter::InZone(ZoneKind::Stack)) {
                    let k = n.strip_prefix(crate::kw::proposal_qualities::X_AT_MOST)?;
                    let k: i32 = k.parse().ok()?;
                    let m = self.me();
                    return Some(format!(
                        "you may cast {m} as though it had flash if X is {k} or less"
                    ));
                }
            }
            return None;
        }
        let first = crate::kw::first_spell_each_turn::FIRST_THIS_TURN;
        if !parts
            .iter()
            .any(|f| matches!(f, Filter::Custom(n) if n == first))
        {
            return None;
        }
        // The spells you cast are the spells you control as you cast them.
        let base: Vec<Filter> = parts
            .iter()
            .filter(|f| {
                !matches!(f, Filter::Custom(n) if n == first)
                    && !matches!(f, Filter::ControlledBy(PlayerRel::You))
            })
            .cloned()
            .collect();
        let n = self.noun(&Filter::and(base), Num::One);
        let n = if n.contains("spell") {
            n
        } else {
            format!("{n} spell")
        };
        Some(format!(
            "you may cast the first {n} you cast each turn as though it had flash"
        ))
    }

    /// "The first spell each player casts during each of their turns may be cast without
    /// paying its mana cost." (the first spell its caster casts this turn, cast by the
    /// active player, for no mana).
    pub(crate) fn first_spell_each_player_free(&mut self, cm: &CostModifier) -> Option<String> {
        let CostTarget::Spells(Filter::And(parts)) = &cm.applies_to else {
            return None;
        };
        let CostChange::AlternativeCost(c) = &cm.change else {
            return None;
        };
        if cm.who != PlayerRel::Any || c.mana.is_some() || !c.parts.is_empty() {
            return None;
        }
        let mut seen = (false, false);
        for p in parts {
            match p {
                Filter::Spell => {}
                Filter::Custom(n) if n == crate::kw::proposal_qualities::CASTER_IS_ACTIVE => {
                    seen.0 = true
                }
                Filter::Custom(n) if n == crate::kw::first_spell_each_turn::FIRST_THIS_TURN => {
                    seen.1 = true
                }
                _ => return None,
            }
        }
        (seen == (true, true)).then(|| {
            "the first spell each player casts during each of their turns may be cast without paying its mana cost".into()
        })
    }
}
