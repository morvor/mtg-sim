//! "Once during each of your turns, you may cast a creature spell from your graveyard"
//! (`kw/once_each_turn_cast.rs`): a permission that holds during your turn until it's
//! used once.

use super::nouns::Det;
use super::*;
use crate::kw::once_each_turn_cast::ONCE_UNUSED;

impl Renderer<'_> {
    /// The ability, if `conds` (its condition's parts) and `effect` are such a
    /// permission.
    pub(crate) fn once_each_turn_permission(
        &mut self,
        conds: &[Condition],
        effect: &StaticEffect,
    ) -> Option<String> {
        let (yours, c) = match conds {
            [Condition::YourTurn, Condition::Custom(c)] => (true, c),
            [Condition::Custom(c)] => (false, c),
            _ => return None,
        };
        if !c.starts_with(ONCE_UNUSED) {
            return None;
        }
        let StaticEffect::PlayPermission(pp) = effect else {
            return None;
        };
        if !matches!(pp.who, PlayerRel::You) || pp.cost.is_some() || pp.flash {
            return None;
        }
        // A spell isn't a land, and it's the card cast (CR 305.9).
        let what = match &pp.what {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|x| {
                        !matches!(x, Filter::Card)
                            && !matches!(x, Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)))
                    })
                    .cloned()
                    .collect(),
            ),
            other => other.clone(),
        };
        let zone = match pp.zone {
            ZoneKind::Graveyard => "from your graveyard",
            ZoneKind::Exile => "from exile",
            ZoneKind::Hand => "from your hand",
            ZoneKind::Library if pp.top_only => "from the top of your library",
            _ => return None,
        };
        let thing = match (pp.lands, pp.spells) {
            (true, false) => "play a land".to_string(),
            (false, true) => {
                let n = self.spell_noun(&what, Det::A);
                format!("cast {n}")
            }
            // "play a historic land or cast a historic permanent spell", "play a land from
            // your graveyard or cast a permanent spell with mana value 3 or less from your
            // graveyard": a land or a nonland card.
            (true, true) => {
                let Filter::Or(v) = &pp.what else {
                    return None;
                };
                let is_land = |f: &Filter| match f {
                    Filter::Type(CardType::Land) => true,
                    Filter::And(p) => p.iter().any(|x| matches!(x, Filter::Type(CardType::Land))),
                    _ => false,
                };
                let [land, spell] = v.as_slice() else {
                    return None;
                };
                if !is_land(land) || is_land(spell) {
                    return None;
                }
                let strip = |f: &Filter| {
                    match f {
                    Filter::And(p) => Filter::and(
                        p.iter()
                            .filter(|x| {
                                !matches!(x, Filter::Card)
                                    && !matches!(x, Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)))
                            })
                            .cloned()
                            .collect(),
                    ),
                    other => other.clone(),
                }
                };
                let l = self.noun_det(&strip(land), Det::A);
                let n = self.spell_noun(&strip(spell), Det::A);
                format!("{{alt:play {l} or cast {n}|play {l} {zone} or cast {n}}}")
            }
            _ => return None,
        };
        let zone = match pp.zone {
            ZoneKind::Graveyard => "from your graveyard",
            ZoneKind::Exile => "from exile",
            ZoneKind::Hand => "from your hand",
            ZoneKind::Library if pp.top_only => "from the top of your library",
            _ => return None,
        };
        let terms = self.static_permission_terms(pp);
        let when = if yours {
            "once during each of your turns"
        } else {
            "once each turn"
        };
        Some(format!("{when}, you may {thing} {zone}{terms}"))
    }
}
