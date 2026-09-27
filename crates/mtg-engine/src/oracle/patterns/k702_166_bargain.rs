//! Oracle patterns for CR 702.166 bargain: the linked abilities that check whether the
//! spell (or the permanent it became) was bargained (CR 702.166b–c) — "If this spell was
//! bargained, ...", "When this creature enters, if it was bargained, ...", and "This spell
//! costs {2} less to cast if it's bargained." They check the bargain cost recorded in the
//! spell's `CastInfo::paid` (see `kw/bargain.rs`); a permanent's abilities see how its
//! spell was cast (CR 607.2i).

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

/// "~ was bargained", "it was bargained", "it's bargained".
fn was_bargained(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "~ was bargained" | "it was bargained" | "it's bargained" | "~ is bargained"
    )
    .then(|| Condition::CostPaid(crate::kw::bargain::BARGAIN.into()))
}

inventory::submit! { ConditionPattern { name: "k702.166 was bargained", priority: 100, parse: was_bargained } }
