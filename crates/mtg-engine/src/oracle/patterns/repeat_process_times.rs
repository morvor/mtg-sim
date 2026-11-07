//! "Repeat the following process X times. [instructions]" (Torment of Hailfire): the
//! instructions of the rest of the spell are performed X times, each time with new choices
//! (CR 608.2c). No state-based actions are checked between the repetitions (CR 704.3).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn repeat_following_process(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rest = block
        .trim()
        .strip_prefix("Repeat the following process X times. ")?;
    let mut abilities = crate::oracle::parse_ability(rest, ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Spell(SpellAbility { body }) = &mut a.kind else {
        return None;
    };
    // The process can't target: its targets would be chosen once, not per repetition.
    if !body.targets.is_empty() || body.modal.is_some() {
        return None;
    }
    let effect = std::mem::take(&mut body.effect);
    body.effect = Effect::Repeat {
        times: Value::X,
        effect: Box::new(effect),
    };
    a.text = block.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "repeat the following process X times", priority: 10, parse: repeat_following_process } }
