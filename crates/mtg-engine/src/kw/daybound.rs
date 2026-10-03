//! CR 702.145 Daybound and Nightbound, on opposite faces of some double-faced cards
//! (CR 702.145a); see CR 731 for day and night.
//!
//! * Daybound means "If it is night and this permanent is represented by a double-faced
//!   card, it enters transformed," "As it becomes night, if this permanent is front face
//!   up, transform it," and "This permanent can't transform except due to its daybound
//!   ability" (CR 702.145b). The first is a static replacement effect of the card that
//!   applies wherever it enters from (a spell cast at night is front face up on the stack
//!   and enters with its back face up); only a double-faced card or token can enter
//!   transformed (see `ReplacementAction::EnterTransformed`).
//! * Nightbound means "As it becomes day, if this permanent is back face up, transform
//!   it" and "This permanent can't transform except due to its nightbound ability"
//!   (CR 702.145e). Other effects that transform permanents don't affect them
//!   (`transform_rules::can_transform_by`).
//! * A front-face-up permanent with daybound at night, or a back-face-up permanent with
//!   nightbound during the day, is transformed immediately (CR 702.145c, 702.145f), and a
//!   permanent with daybound (or with nightbound, when there's no daybound permanent)
//!   makes it day (or night) if it's neither (CR 702.145d, 702.145g). These aren't
//!   state-based actions, but they're checked whenever state-based actions are, and right
//!   as it becomes day or night.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

pub struct Daybound;

/// Which of the two abilities a keyword instance is (its printed word).
fn is_daybound(kw: &Keyword) -> bool {
    kw.kind == KeywordKind::DayboundAndNightbound
        && kw
            .text
            .as_ref()
            .is_some_and(|t| t.trim().eq_ignore_ascii_case("daybound"))
}

fn is_nightbound(kw: &Keyword) -> bool {
    kw.kind == KeywordKind::DayboundAndNightbound
        && kw
            .text
            .as_ref()
            .is_some_and(|t| t.trim().eq_ignore_ascii_case("nightbound"))
}

fn has_daybound(o: &GameObject) -> bool {
    o.chars.keywords().any(is_daybound)
}

fn has_nightbound(o: &GameObject) -> bool {
    o.chars.keywords().any(is_nightbound)
}

/// Whether `id` has daybound or nightbound, so it can transform only due to those
/// abilities (CR 702.145b, 702.145e).
pub fn transforms_only_by_day_night(g: &Game, id: ObjectId) -> bool {
    g.obj(id)
        .chars
        .has_keyword(KeywordKind::DayboundAndNightbound)
}

/// CR 702.145b–c, 702.145e–f: transforms each front-face-up permanent with daybound when
/// it's night and each back-face-up permanent with nightbound when it's day. Returns
/// whether any permanent transformed.
fn transform_to_match(g: &mut Game) -> bool {
    if g.dirty {
        g.recompute();
    }
    let Some(day) = g.day else {
        return false;
    };
    let ids: Vec<ObjectId> = g
        .permanents()
        .filter(|o| {
            (!day && o.face == FaceState::Front && has_daybound(o))
                || (day && o.face == FaceState::Back && has_nightbound(o))
        })
        .map(|o| o.id)
        .collect();
    let mut any = false;
    for id in ids {
        any |= crate::dfc::transform_by_day_night(g, id);
    }
    any
}

impl KeywordRules for Daybound {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::DayboundAndNightbound]
    }

    /// CR 702.145b: "If it is night and this permanent is represented by a double-faced
    /// card, it enters transformed."
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        if !is_daybound(kw) {
            return None;
        }
        let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::Source,
                from: None,
                to: Some(ZoneKind::Battlefield),
            },
            action: ReplacementAction::EnterTransformed,
            self_replacement: false,
            optional: false,
        }));
        s.condition = Some(Condition::IsNight);
        s.zone = FunctionZone::Anywhere;
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::DayboundAndNightbound.name(),
        )])
    }

    /// CR 702.145b, 702.145e: "As it becomes night/day, ... transform it."
    fn day_night_changed(&self, g: &mut Game) {
        transform_to_match(g);
    }

    /// CR 702.145c–d, 702.145f–g.
    fn state_based_actions(&self, g: &mut Game) -> bool {
        if g.dirty {
            g.recompute();
        }
        let mut did = false;
        if g.day.is_none() {
            let day = g.permanents().any(has_daybound);
            let night = !day && g.permanents().any(has_nightbound);
            if day || night {
                g.set_day(day);
                did = true;
            }
        }
        did | transform_to_match(g)
    }
}

inventory::submit! { KeywordRegistration(&Daybound) }
