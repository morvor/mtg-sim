//! Game terms (CR 700): "if you have a full party" (CR 700.8c), "Each player chooses a
//! party from among creatures they control, then sacrifices the rest." (CR 700.8d).

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::game_terms::{CHOOSE_PARTY_SACRIFICE_REST, PARTY_SIZE, PARTY_TYPES};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "you have a full party": four creatures in your party (CR 700.8c).
fn full_party(c: &str) -> Option<Condition> {
    (end(c) == "you have a full party").then(|| {
        Condition::Compare(
            Value::Custom(PARTY_SIZE.into()),
            Cmp::Ge,
            Value::c(PARTY_TYPES.len() as i32),
        )
    })
}

inventory::submit! { ConditionPattern { name: "r700 full party", priority: 100, parse: full_party } }

/// "each player chooses a party from among creatures they control, then sacrifices the
/// rest" (CR 700.8d).
fn choose_party(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l)
        == "each player chooses a party from among creatures they control, then sacrifices the rest")
        .then(|| Effect::Custom(CHOOSE_PARTY_SACRIFICE_REST.into()))
}

inventory::submit! { EffectPattern { name: "r700 choose a party", priority: 100, parse: choose_party } }
