//! Case cards (CR 719.3): "To solve — [Condition]" and "Solved — [Ability]".

use super::AbilityPattern;
use crate::ability::*;
use crate::cases::{SOLVE, SOLVED};
use crate::oracle::CompileContext;

/// "To solve — [Condition]": "At the beginning of your end step, if [condition] and this
/// Case is not solved, this Case becomes solved." (CR 719.3a).
fn to_solve(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.trim().strip_prefix("To solve — ")?;
    let cond = crate::oracle::statics::parse_condition(&r.to_lowercase(), ctx)?;
    let mut tr = TriggeredAbility::new(
        TriggerCond::BeginningOf {
            step: TriggerStep::End,
            whose: PlayerRel::You,
        },
        Body::effect(Effect::Custom(SOLVE.into())),
    );
    tr.intervening_if = Some(Condition::And(vec![
        cond,
        Condition::Not(Box::new(Condition::Custom(SOLVED.into()))),
    ]));
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

/// "Solved — [Ability]": the ability functions only while this Case is solved
/// (CR 719.3c, 702.169a): the Case has it as long as it's solved.
fn solved(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.trim().strip_prefix("Solved — ")?;
    let inner = crate::oracle::parse_ability(r, ctx)?;
    if inner.is_empty() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: inner.into_iter().map(Modification::AddAbility).collect(),
    });
    s.condition = Some(Condition::Custom(SOLVED.into()));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block)])
}

inventory::submit! { AbilityPattern { name: "r719 to solve", priority: 70, parse: to_solve } }
inventory::submit! { AbilityPattern { name: "r719 solved", priority: 70, parse: solved } }
