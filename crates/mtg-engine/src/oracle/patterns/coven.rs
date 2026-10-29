//! Coven (an ability word): "you control three or more creatures with different powers",
//! in intervening-if clauses, "as long as" statics, "activate only if" restrictions and
//! "then if" follow-ups (see `kw/coven.rs`).

use crate::ability::*;
use crate::kw::coven::DIFFERENT_POWERS;
use crate::oracle::patterns::ConditionPattern;

fn coven(c: &str) -> Option<Condition> {
    let c = c.trim().trim_end_matches(['.', ',']);
    let n = match c {
        "you control three or more creatures with different powers" => 3,
        _ => return None,
    };
    Some(Condition::Compare(
        Value::Custom(DIFFERENT_POWERS.into()),
        Cmp::Ge,
        Value::c(n),
    ))
}

inventory::submit! {
    ConditionPattern {
        name: "coven: creatures with different powers",
        priority: 100,
        parse: coven,
    }
}
