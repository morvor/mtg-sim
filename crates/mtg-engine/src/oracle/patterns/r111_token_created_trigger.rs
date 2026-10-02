//! "Whenever you create a token" / "Whenever you create a Blood token" (CR 111.1): one
//! trigger for each token you create of that kind, which you control as it enters
//! (CR 111.2). ("Whenever you create one or more tokens" triggers once for a batch and
//! isn't parsed here.)

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn you_create_a_token(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let noun = r
        .strip_prefix("you create a ")
        .or_else(|| r.strip_prefix("you create an "))?;
    if !noun.ends_with("token") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::TokenCreated(Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)])),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "r111 whenever you create a token", priority: 100, parse: you_create_a_token } }
