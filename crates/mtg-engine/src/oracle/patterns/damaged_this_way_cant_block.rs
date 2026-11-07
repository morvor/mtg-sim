//! "Creatures dealt damage this way can't block this turn." (Huatli, Warrior Poet;
//! Ballista Watcher): the creatures the preceding instruction dealt damage to, fixed as
//! the effect begins (CR 608.2c, 611.2c). A creature whose damage was prevented wasn't
//! dealt damage (CR 615.1).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;

fn p_damaged_cant_block(l: &str, _b: &mut Builder) -> Option<Effect> {
    let who = l.strip_suffix(" can't block this turn")?;
    if !matches!(
        who,
        "a creature dealt damage this way" | "creatures dealt damage this way"
    ) {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::CantBlock(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::DAMAGED))),
            Filter::creature(),
        ])),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "creatures dealt damage this way can't block this turn", priority: 50, parse: p_damaged_cant_block } }
