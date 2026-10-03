//! CR 702.194 Teamwork.
//!
//! * "Teamwork N" means "As an additional cost to cast this spell, you may tap any number
//!   of creatures you control with total power N or more." (CR 702.194a): an optional
//!   additional cost (CR 601.2b, 601.2f–h), a [`CostPart::TapTotalPower`] for teamwork
//!   paid as crew's is (`kw/crew.rs`).
//! * A spell was cast "using teamwork" if its caster declared the intention to pay its
//!   teamwork cost (CR 702.194b): [`TEAMWORK`] is recorded in its `CastInfo::paid`
//!   (`Condition::CostPaid`).
//! * Targets of a part of the spell that has its effect only if teamwork was used are
//!   chosen only if it was (CR 702.194c; `TargetSpec::condition`, set by the oracle
//!   compiler for "if this spell was cast using teamwork, [effect]").
//! * "Choose one. If this spell was cast using teamwork, choose both instead." is a modal
//!   header whose mode count depends on that condition (the modes are chosen after the
//!   cost is announced, CR 601.2b).
//! * "You may cast this spell as though it had flash if it's cast using teamwork."
//!   ([`FLASH_WITH_TEAMWORK`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when a spell is cast using teamwork.
pub const TEAMWORK: &str = "teamwork";
/// `StaticEffect::Custom` functioning on the stack: "You may cast this spell as though it
/// had flash if it's cast using teamwork."
pub const FLASH_WITH_TEAMWORK: &str = "teamwork:flash if cast using teamwork";
/// Whether the spell was cast using teamwork.
pub fn used(g: &Game, spell: ObjectId) -> bool {
    g.obj(spell)
        .stack
        .as_deref()
        .is_some_and(|s| s.cast.paid.iter().any(|x| x == TEAMWORK))
}

fn has_custom(chars: &Characteristics, name: &str) -> bool {
    chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if matches!(&s.effect, StaticEffect::Custom(n) if n == name))
    })
}

/// The teamwork cost: tap any number of untapped creatures you control with total power N
/// or more.
pub fn teamwork_cost(n: i32) -> Cost {
    Cost::free().with(CostPart::TapTotalPower {
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::ControlledBy(PlayerRel::You),
        ]),
        power: Value::c(n),
        keyword: KeywordKind::Teamwork,
    })
}

pub struct Teamwork;

impl KeywordRules for Teamwork {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Teamwork]
    }

    fn optional_costs(&self, g: &Game, spell: ObjectId, kw: &Keyword) -> Vec<(SmolStr, Cost, bool)> {
        // Already paid as the cost of casting it with flash.
        if g.obj(spell).stack.as_deref().is_some_and(|s| {
            s.cast.method == CastMethod::Keyword(KeywordKind::Teamwork)
        }) {
            return vec![];
        }
        vec![(
            SmolStr::new(TEAMWORK),
            teamwork_cost(kw.n.unwrap_or(0)),
            false,
        )]
    }

    /// "You may cast this spell as though it had flash if it's cast using teamwork": a
    /// way to cast it with flash that requires the teamwork cost.
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Hand(p) || !has_custom(&o.chars, FLASH_WITH_TEAMWORK) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Teamwork);
        opt.flash = true;
        opt.extra_cost = Some(teamwork_cost(kw.n.unwrap_or(0)));
        opt.tag = Some(TEAMWORK);
        vec![opt]
    }
}

inventory::submit! { KeywordRegistration(&Teamwork) }
