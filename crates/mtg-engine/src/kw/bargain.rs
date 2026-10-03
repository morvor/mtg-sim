//! CR 702.166 Bargain: "As an additional cost to cast this spell, you may sacrifice an
//! artifact, enchantment, or token." (CR 702.166a). Paying it follows the rules for
//! additional costs (CR 601.2b, 601.2f–h): one permanent is sacrificed.
//!
//! A spell whose controller declared the intention to pay its bargain cost has been
//! "bargained" (CR 702.166b): [`BARGAIN`] is recorded in the spell's `CastInfo::paid`,
//! which the linked "if this spell was bargained" / "if it was bargained" abilities check
//! (`Condition::CostPaid`, CR 702.166c; see `oracle/patterns/k702_166_bargain.rs`). A copy
//! of a bargained spell copies that choice (CR 707.10) and is bargained too; a permanent
//! that enters as a copy of a bargained permanent wasn't cast, so it isn't.
//!
//! Targets of a part of a spell that has its effect only if it was bargained are chosen
//! only if it was (CR 702.166d; see `TargetSpec::condition`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The optional cost name recorded in `CastInfo::paid`: the spell was bargained.
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

    fn optional_costs(
        &self,
        _g: &Game,
        _spell: ObjectId,
        _kw: &Keyword,
    ) -> Vec<(SmolStr, Cost, bool)> {
        vec![(BARGAIN.into(), bargain_cost(), false)]
    }
}

inventory::submit! { KeywordRegistration(&Bargain) }
