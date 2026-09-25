//! CR 702.34 Flashback: "You may cast this card from your graveyard if the resulting spell
//! is an instant or sorcery spell by paying [cost] rather than paying its mana cost" and
//! "If the flashback cost was paid, exile this card instead of putting it anywhere else any
//! time it would leave the stack." (CR 702.34a). Casting it follows the rules for
//! alternative costs (CR 601.2b, 601.2f–h); timing restrictions still apply.
//!
//! A flashback ability granted with "The flashback cost is equal to its mana cost" has no
//! cost of its own ([`Keyword::cost`] is `None`): the card's mana cost is paid.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast with flashback.
pub const FLASHBACK: &str = "flashback";

pub struct Flashback;

/// Whether the spell was cast with its flashback ability (and so is exiled instead of
/// leaving the stack any other way).
pub fn cast_with_flashback(g: &Game, spell: ObjectId) -> bool {
    matches!(
        g.obj(spell).stack.as_deref().map(|s| &s.cast.method),
        Some(CastMethod::Keyword(KeywordKind::Flashback))
    )
}

impl KeywordRules for Flashback {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Flashback]
    }

    /// The replacement effect "exile this card instead of putting it anywhere else any
    /// time it would leave the stack", which applies only if the flashback cost was paid.
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::Source,
                from: Some(ZoneKind::Stack),
                to: None,
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        }));
        s.zone = FunctionZone::Stack;
        s.condition = Some(Condition::CostPaid(FLASHBACK.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Flashback.name(),
        )])
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Flashback);
        // CR 702.34a: only if the resulting spell is an instant or sorcery spell.
        let chars = g.option_characteristics(card, &opt);
        if !chars.is(CardType::Instant) && !chars.is(CardType::Sorcery) {
            return vec![];
        }
        opt.alt_cost = match &kw.cost {
            Some(c) => Some(c.clone()),
            None => Some(Cost::mana(chars.mana_cost.clone().unwrap_or_default())),
        };
        opt.tag = Some(FLASHBACK);
        vec![opt]
    }

    fn resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_with_flashback(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn countered_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_with_flashback(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }
}

inventory::submit! { KeywordRegistration(&Flashback) }
