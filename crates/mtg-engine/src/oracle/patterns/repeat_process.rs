//! "Repeat this process once." (Calamity, Galloping Inferno; Remorseless Punishment): the
//! instructions of the previous sentence (with those that continue it) happen again, with
//! new choices.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn repeat_once(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "repeat this process once" || matches!(prev, Effect::Noop) {
        return false;
    }
    let once = prev.clone();
    *prev = Effect::seq(vec![std::mem::take(prev), once]);
    true
}

inventory::submit! { FollowupPattern { name: "repeat this process once", priority: 100, apply: repeat_once } }
