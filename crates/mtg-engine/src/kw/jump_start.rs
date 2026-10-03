//! CR 702.133 Jump-start: two static abilities. "You may cast this card from your
//! graveyard if the resulting spell is an instant or sorcery spell by discarding a card as
//! an additional cost to cast it" and "If this spell was cast using its jump-start
//! ability, exile this card instead of putting it anywhere else any time it would leave
//! the stack." (CR 702.133a). Casting it follows the rules for additional costs
//! (CR 601.2b, 601.2f–h): its mana cost is paid as usual, and normal timing applies.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

/// The name recorded in `CastInfo::paid` when a spell is cast with jump-start.
pub const JUMP_START: &str = "jump-start";

pub struct JumpStart;

/// Whether the spell was cast using its jump-start ability.
pub fn cast_with_jump_start(g: &Game, spell: ObjectId) -> bool {
    matches!(
        g.obj(spell).stack.as_deref().map(|s| &s.cast.method),
        Some(CastMethod::Keyword(KeywordKind::JumpStart))
    )
}

/// The additional cost of casting a spell with jump-start: discarding a card.
fn discard_a_card() -> Cost {
    Cost {
        mana: None,
        parts: vec![CostPart::Discard {
            filter: Filter::Any,
            count: Value::c(1),
            random: false,
        }],
    }
}

impl KeywordRules for JumpStart {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::JumpStart]
    }

    /// The replacement effect that exiles the card whenever it would leave the stack,
    /// which applies only if it was cast with jump-start.
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
        s.condition = Some(Condition::CostPaid(JUMP_START.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::JumpStart.name(),
        )])
    }

    fn cast_options(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        _kw: &Keyword,
    ) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::JumpStart);
        // Only if the resulting spell is an instant or sorcery spell.
        let chars = g.option_characteristics(card, &opt);
        if !chars.is(CardType::Instant) && !chars.is(CardType::Sorcery) {
            return vec![];
        }
        opt.extra_cost = Some(discard_a_card());
        opt.tag = Some(JUMP_START);
        vec![opt]
    }

    /// "Exile it instead of putting it anywhere else" applies wherever the card would go,
    /// after any other replacement effect: its controller has nothing to choose.
    fn resolved_destination_replaces(&self) -> bool {
        false
    }

    fn resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_with_jump_start(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn countered_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_with_jump_start(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }
}

inventory::submit! { KeywordRegistration(&JumpStart) }
