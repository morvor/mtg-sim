//! An activated ability with {X} in its cost whose text defines X: "{X}, {T}: Create a
//! token that's a copy of the exiled card. X is the mana value of that card." (Soul
//! Foundry, Prototype Portal), "{X}, {T}: Draw a card. X is the number of cards in an
//! opponent's hand." (Bargaining Table). The controller doesn't choose X: it's the defined
//! value (CR 107.3c), so the only value of X that can be announced as the ability is
//! activated is that one (CR 602.2b, 601.2b) — compiled as a restriction on the announced
//! X.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn x_defined_by_text(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (cost, _) = text.split_once(": ")?;
    if !cost.contains("{X}") || cost.contains('"') {
        return None;
    }
    let (before, def) = text.rsplit_once(". X is ")?;
    let def = end(def).to_lowercase();
    if def.contains('.') || before.contains(" X ") {
        return None;
    }
    let mut abilities = crate::oracle::parse_ability(&format!("{before}."), ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Activated(act) = &mut a.kind else {
        return None;
    };
    // "that card" names the card the instruction did ("the exiled card").
    let lower = before.to_lowercase();
    let def = if def.contains("that card") && lower.contains("the exiled card") {
        def.replace("that card", "the exiled card")
    } else {
        def
    };
    let mut b = Builder::new(ctx);
    let (v, rest) = crate::oracle::statics::parse_value_phrase(&def, &mut b)?;
    if !rest.trim().is_empty() || !b.targets.is_empty() {
        return None;
    }
    let only = Condition::Compare(Value::X, Cmp::Eq, v);
    act.condition = Some(match act.condition.take() {
        Some(c) => Condition::And(vec![c, only]),
        None => only,
    });
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "r107.3c {X} in an activation cost defined by the text", priority: 1100, parse: x_defined_by_text } }
