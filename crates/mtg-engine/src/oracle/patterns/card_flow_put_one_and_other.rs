//! "Look at the top two cards of your library. Put one into your hand and the other into
//! your graveyard." (Faerie Snoop): "one" is "one of them" (see `card_flow_dig.rs`).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn put_one_and_the_other(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("put one into ") else {
        return false;
    };
    if !r.contains(" and the other ") {
        return false;
    }
    crate::oracle_ext::apply_followup_ext(&format!("put one of them into {r}"), prev, b)
}

inventory::submit! { FollowupPattern { name: "card_flow: put one into ... and the other ...", priority: 95, apply: put_one_and_the_other } }
