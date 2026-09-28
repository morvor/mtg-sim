//! Parley (an ability word, see `kw/parley.rs`): "Each player reveals the top card of
//! their library." and "For each nonland card revealed this way, [effect]." (also "land
//! card"). The effect happens that many times: a count it has is multiplied (one event,
//! "you create three 3/3 Elephant tokens"), otherwise it's repeated.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::parley::{LAND_REVEALED, NONLAND_REVEALED, REVEAL_TOPS};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn each_player_reveals_top(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "each player reveals the top card of their library")
        .then(|| Effect::Custom(REVEAL_TOPS.into()))
}

inventory::submit! { EffectPattern { name: "parley: each player reveals the top card of their library", priority: 60, parse: each_player_reveals_top } }

fn for_each_revealed(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (count, rest) = if let Some(r) = l.strip_prefix("for each nonland card revealed this way, ")
    {
        (NONLAND_REVEALED, r)
    } else if let Some(r) = l.strip_prefix("for each land card revealed this way, ") {
        (LAND_REVEALED, r)
    } else {
        return None;
    };
    let targets = b.targets.len();
    // "you investigate": the ability's controller does.
    let e = parse_clause(rest, b).or_else(|| {
        b.targets.truncate(targets);
        parse_clause(rest.strip_prefix("you ")?, b)
    })?;
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    let n = Value::Custom(count.into());
    Some(
        super::damage_removal_foreach::multiply(e.clone(), n.clone()).unwrap_or(
            Effect::Repeat {
                times: n,
                effect: Box::new(e),
            },
        ),
    )
}

inventory::submit! { EffectPattern { name: "parley: for each nonland card revealed this way, [effect]", priority: 60, parse: for_each_revealed } }
