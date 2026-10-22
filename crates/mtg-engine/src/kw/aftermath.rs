//! CR 702.127 Aftermath: an ability found on some split cards (CR 709). It represents three
//! static abilities: "You may cast this half of this split card from your graveyard,"
//! "This half of this split card can't be cast from any zone other than a graveyard," and
//! "If this spell was cast from a graveyard, exile it instead of putting it anywhere else
//! any time it would leave the stack." (CR 702.127a)
//!
//! The half with aftermath is cast from the graveyard with the method
//! `CastMethod::Keyword(Aftermath)`. Another effect may let a player cast either half from
//! a graveyard; cast from anywhere else, the half with aftermath is prohibited. A half with
//! aftermath cast from a graveyard (by any permission) is exiled whenever it leaves the
//! stack.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

pub struct Aftermath;

/// The zone `card` is being cast from: its zone, or for a spell already on the stack, the
/// zone it was cast from.
fn cast_from(g: &Game, card: ObjectId) -> Option<ZoneKind> {
    let o = g.obj(card);
    match o.zone {
        Zone::Stack => o.stack.as_deref().and_then(|si| si.cast.from),
        z => z.kind(),
    }
}

/// Whether the spell `spell` has aftermath and was cast from a graveyard.
fn cast_from_graveyard(g: &Game, spell: ObjectId) -> bool {
    let o = g.obj(spell);
    o.chars.has_keyword(KeywordKind::Aftermath)
        && o.stack
            .as_deref()
            .is_some_and(|si| si.cast.was_cast && si.cast.from == Some(ZoneKind::Graveyard))
}

impl KeywordRules for Aftermath {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Aftermath]
    }

    /// "If this spell was cast from a graveyard, exile it instead of putting it anywhere
    /// else any time it would leave the stack."
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
        s.condition = Some(Condition::CastFrom(ZoneKind::Graveyard));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Aftermath.name(),
        )])
    }

    /// "You may cast this half of this split card from your graveyard."
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
        let Some(def) = g.obj(card).card.clone() else {
            return vec![];
        };
        if def.layout != crate::card::Layout::Split {
            return vec![];
        }
        (0..def.faces.len().min(2) as u8)
            .map(FaceState::Half)
            .filter(|f| {
                g.face_characteristics(card, *f)
                    .has_keyword(KeywordKind::Aftermath)
            })
            .map(|f| {
                let mut opt = CastOption::normal(f);
                opt.method = CastMethod::Keyword(KeywordKind::Aftermath);
                opt
            })
            .collect()
    }

    /// "This half of this split card can't be cast from any zone other than a graveyard."
    fn cast_prohibited(
        &self,
        g: &Game,
        _p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
    ) -> bool {
        // Only a half: not the whole split card, whose combined characteristics (named
        // "[half] // [half]", CR 709.4) have the keyword too.
        let half = !chars.name.contains(" // ");
        half && chars.has_keyword(KeywordKind::Aftermath)
            && cast_from(g, card) != Some(ZoneKind::Graveyard)
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
        cast_from_graveyard(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn countered_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        cast_from_graveyard(g, spell).then_some((Zone::Exile, LibraryPosition::Top))
    }
}

inventory::submit! { KeywordRegistration(&Aftermath) }
