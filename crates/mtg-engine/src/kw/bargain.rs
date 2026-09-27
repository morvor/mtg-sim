//! CR 702.166 Bargain: "As an additional cost to cast this spell, you may sacrifice an
//! artifact, enchantment, or token" (CR 702.166a), an optional additional cost
//! (CR 601.2b, 601.2f–h).
//!
//! * A spell whose controller declared the intention to pay its bargain cost has been
//!   "bargained" (CR 702.166b): [`BARGAIN`] is recorded in its `CastInfo::paid`, which the
//!   linked "if this spell was bargained" / "if it was bargained" abilities check
//!   (`Condition::CostPaid`, CR 702.166c); the permanent the spell becomes keeps its cast
//!   information, so its enters abilities see it too.
//! * Targets of a part of the spell that has its effect only if it was bargained are chosen
//!   only if it was (CR 702.166d; `TargetSpec::condition`, see the oracle compiler's
//!   handling of "if [a cost was paid], ...").
//! * Several instances of bargain are redundant: only one bargain cost is offered.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when a spell's bargain cost is paid.
pub const BARGAIN: &str = "bargain";

/// "Sacrifice an artifact, enchantment, or token."
pub fn bargain_cost() -> Cost {
    Cost::free().with(CostPart::Sacrifice {
        filter: Filter::Or(vec![
            Filter::Type(CardType::Artifact),
            Filter::Type(CardType::Enchantment),
            Filter::Token,
        ]),
        count: Value::c(1),
    })
}

pub struct Bargain;

impl KeywordRules for Bargain {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Bargain]
    }

    /// One bargain cost however many instances the spell has.
    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        if !g.obj(spell).chars.has_keyword(KeywordKind::Bargain) {
            return vec![];
        }
        vec![(BARGAIN.into(), bargain_cost(), false)]
    }
}

inventory::submit! { KeywordRegistration(&Bargain) }
