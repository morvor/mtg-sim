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
/// (CR 719.3c, 702.169a). A static ability applies "as long as this Case is solved"
/// (CR 702.169b), in whatever layer its effect belongs to ("This Case is a 4/4 Gorgon
/// creature ...", Case of the Gorgon's Kiss); the Case has a triggered or activated one as
/// long as it's solved (CR 702.169c–d).
fn solved(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.trim().strip_prefix("Solved — ")?;
    let inner = crate::oracle::parse_ability(r, ctx)?;
    if inner.is_empty() {
        return None;
    }
    let is_solved = || Condition::Custom(SOLVED.into());
    let mut out = Vec::new();
    let mut granted = Vec::new();
    for a in inner {
        match &a.kind {
            AbilityKind::Static(st) => {
                let mut st = st.clone();
                st.condition = Some(match st.condition.take() {
                    Some(c) => Condition::And(vec![is_solved(), c]),
                    None => is_solved(),
                });
                out.push(AbilityDef::new(AbilityKind::Static(st), block));
            }
            _ => granted.push(Modification::AddAbility(a)),
        }
    }
    if !granted.is_empty() {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: granted,
        });
        s.condition = Some(is_solved());
        out.push(AbilityDef::new(AbilityKind::Static(s), block));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "r719 to solve", priority: 70, parse: to_solve } }
inventory::submit! { AbilityPattern { name: "r719 solved", priority: 70, parse: solved } }
