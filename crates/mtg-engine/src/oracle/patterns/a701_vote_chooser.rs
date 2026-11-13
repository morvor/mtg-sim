//! "You choose how each player votes this turn." (Illusion of Choice, CR 701.38): see
//! `kwa::vote::CHOOSES_VOTES`.

use super::EffectPattern;
use crate::ability::Effect;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn parse(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "you choose how each player votes this turn")
        .then(|| Effect::Custom(crate::kwa::vote::CHOOSES_VOTES.into()))
}

inventory::submit! { EffectPattern { name: "a701.38: you choose how each player votes this turn", priority: 100, parse } }
