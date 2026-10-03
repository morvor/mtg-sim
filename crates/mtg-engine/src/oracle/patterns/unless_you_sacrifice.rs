//! "[effect] unless you sacrifice [~ / a permanent]": "you lose 1 life unless you
//! sacrifice this creature" (Withercrown's granted ability). The player chooses whether to
//! pay the sacrifice as the ability resolves (CR 118.12); if they don't (or can't), the
//! effect happens.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn unless_you_sacrifice(l: &str, b: &mut Builder) -> Option<Effect> {
    let (effect, cost) = end(l).rsplit_once(" unless you sacrifice ")?;
    // "For each card drawn this way, discard a card unless you sacrifice a permanent": a
    // choice for each of them (`hand_graveyard_grammar::scale`), not one for all.
    if super::unless_grammar::leading_for_each_this_way(effect) {
        return None;
    }
    let cost = crate::oracle::keywords::parse_keyword_cost(&format!("sacrifice {cost}"))?;
    let otherwise = parse_clause(effect, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(otherwise),
    })
}

inventory::submit! { EffectPattern { name: "[effect] unless you sacrifice [cost]", priority: 300, parse: unless_you_sacrifice } }
