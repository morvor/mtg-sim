//! "[Effect] unless that player pays [cost][, where X is ...]" (CR 118.12a): "Whenever a
//! player casts an instant spell, counter it unless that player pays {X}, where X is its
//! mana value." (In the Eye of Chaos), "Whenever a player casts a spell, counter it unless
//! that player pays {1}." (Nether Void), "Whenever an opponent casts a spell, you may draw
//! a card unless that player pays {1}." (Rhystic Study). The player the trigger is about
//! may pay; if they don't, the effect happens. X is determined as the ability resolves.

use super::counters_resources_pay::resolution_cost;
use super::EffectPattern;
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::end;
use crate::oracle::statics::parse_value_phrase;

fn unless_that_player_pays(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (main, where_x) = match l.split_once(", where x is ") {
        Some((m, w)) => (m, Some(w)),
        None => (l, None),
    };
    // "unless they pay": read here only for a target spell's controller (below); other
    // "they"s are read where "they" names a player mentioned before.
    let (eff, cost_text, they) = match main.rsplit_once(" unless that player pays ") {
        Some((e, c)) => (e, c, false),
        None => {
            let (e, c) = main.rsplit_once(" unless they pay ")?;
            if !super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
                return None;
            }
            (e, c, true)
        }
    };
    // The effect first: "counter target spell an opponent controls unless they pay {1}"
    // (Hope-Ender Coatl): "they" is the player its target names.
    let effect = match eff.strip_prefix("you may ") {
        Some(r) => Effect::May {
            who: PlayerRef::You,
            effect: Box::new(parse_clause(r, b)?),
        },
        None => parse_clause(eff, b)?,
    };
    let named = if they { None } else { player_ref("that player", b) };
    let who = match named {
        Some((who, _)) => who,
        // No player named before: the controller of the target spell the effect names.
        None if matches!(effect, Effect::CounterSpell { .. }) && matches!(b.it, Sel::Target(_)) => {
            PlayerRef::ControllerOf(Box::new(b.it.clone()))
        }
        None => return None,
    };
    let (cost, x) = match where_x {
        Some(w) => {
            if cost_text != "{x}" {
                return None;
            }
            let (v, rest) = parse_value_phrase(w, b)?;
            if !end(&rest).trim().is_empty() {
                return None;
            }
            (Cost::mana(ManaCost::parse("{X}")?), Some(v))
        }
        // "unless they pay X life" (Killing Wave): the X of the spell (CR 107.3a).
        None if cost_text == "x life"
            && ((b.ctx.is_spell() && !b.in_trigger)
                || super::value_grammar::x_defined()) =>
        {
            (Cost::free().with(CostPart::PayLife(Value::X)), None)
        }
        None => (resolution_cost(cost_text)?, None),
    };
    let pay = Effect::PayOptional {
        who,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    };
    Some(match x {
        Some(value) => Effect::seq(vec![Effect::SetX { value }, pay]),
        None => pay,
    })
}

inventory::submit! { EffectPattern { name: "r118 effect unless that player pays", priority: 100, parse: unless_that_player_pays } }
