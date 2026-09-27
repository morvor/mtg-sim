//! Eminence (an ability word, CR 207.2c): "[Trigger], if ~ is in the command zone or on
//! the battlefield, [effect]." The ability states the zones it functions in (CR 113.6b),
//! so it's compiled as a triggered ability that functions anywhere with that intervening
//! "if" clause (CR 603.4). See `kw/eminence.rs`.

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::eminence::IN_COMMAND_ZONE_OR_ON_BATTLEFIELD;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

const CONDITION: &str = "if ~ is in the command zone or on the battlefield, ";

fn eminence(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    if !(lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at "))
    {
        return None;
    }
    let i = lower.find(CONDITION)?;
    // The rest of the ability without the clause about where ~ is.
    let rest = format!("{}{}", &t[..i], &t[i + CONDITION.len()..]);
    let mut ability = crate::oracle::triggers::parse_triggered(&rest, ctx)?;
    let a = std::sync::Arc::make_mut(&mut ability);
    let AbilityKind::Triggered(tr) = &mut a.kind else {
        return None;
    };
    let zones = Condition::Custom(SmolStr::new(IN_COMMAND_ZONE_OR_ON_BATTLEFIELD));
    tr.intervening_if = Some(match tr.intervening_if.take() {
        Some(c) => Condition::And(vec![zones, c]),
        None => zones,
    });
    tr.zone = FunctionZone::Anywhere;
    a.text = t.into();
    Some(vec![ability])
}

inventory::submit! { AbilityPattern { name: "eminence", priority: 60, parse: eminence } }
