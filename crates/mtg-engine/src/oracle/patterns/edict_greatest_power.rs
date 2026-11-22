//! "Target opponent sacrifices a creature with the greatest power among creatures they
//! control." (Consumed by Greed, Crackling Doom): an edict whose candidates are the
//! sacrificing player's creatures with the greatest power among their creatures (that
//! player chooses among those tied, CR 701.21a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::*;

fn edict_greatest_power(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, rest) = player_ref(end(l), b)?;
    let rest = rest.trim();
    let what = rest
        .strip_prefix("sacrifices a creature with the greatest power among creatures ")
        .or_else(|| rest.strip_prefix("sacrifice a creature with the greatest power among creatures "))?;
    if !matches!(
        what,
        "they control" | "that player controls" | "you control" | "they controls"
    ) {
        return None;
    }
    let theirs = Filter::and(vec![
        Filter::creature(),
        Filter::ControlledBy(PlayerRel::Iterated),
    ]);
    if matches!(
        who,
        PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
    ) {
        // "Each opponent sacrifices ...": no single permanent is "that creature" afterward.
    } else {
        b.it = Sel::Var(vars::SACRIFICED);
    }
    Some(Effect::Sacrifice {
        who,
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::Power(Cmp::Ge, Box::new(Value::GreatestPower(theirs))),
        ]),
        count: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "edict: a creature with the greatest power among creatures they control", priority: 40, parse: edict_greatest_power } }
