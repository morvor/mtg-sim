//! Oracle patterns for collecting evidence (CR 701.59) as an optional additional cost:
//! "As an additional cost to cast this spell, you may collect evidence N." and the linked
//! "if evidence was collected" (CR 701.59c). "Collect evidence N" / "forage" as costs and
//! instructions are handled by `oracle/costs.rs` and `a701_actions.rs`; "whenever you
//! collect evidence" / "whenever you forage" by `a701_action_triggers.rs`.

use super::{AbilityPattern, ConditionPattern};
use crate::ability::*;
use crate::kwa::evidence_forage::EVIDENCE_COST;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "As an additional cost to cast ~, you may collect evidence N."
fn optional_evidence_cost(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower)
        .strip_prefix("as an additional cost to cast ~, you may collect evidence ")?
        .trim();
    let n: u32 = r.parse().ok()?;
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::OptionalAdditionalCost {
            name: SmolStr::new(EVIDENCE_COST),
            cost: Cost::free().with(CostPart::CollectEvidence(n)),
        },
    }));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text.trim())])
}

inventory::submit! { AbilityPattern { name: "a701 optional collect evidence cost", priority: 90, parse: optional_evidence_cost } }

/// "evidence was collected" (CR 701.59c): the linked optional additional cost was paid.
fn evidence_was_collected(c: &str) -> Option<Condition> {
    (end(c) == "evidence was collected").then(|| Condition::CostPaid(EVIDENCE_COST.into()))
}

inventory::submit! { ConditionPattern { name: "a701 evidence was collected", priority: 60, parse: evidence_was_collected } }
