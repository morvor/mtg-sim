//! A spell's own alternative costs (CR 118.9, 601.2b): "You may pay {W}{U}{B}{R}{G}
//! rather than pay this spell's mana cost.", "You may pay 1 life and exile a blue card
//! from your hand rather than pay this spell's mana cost.", "If you control a Swamp, you
//! may pay 4 life rather than pay this spell's mana cost.", "You may return two Islands
//! you control to their owner's hand rather than pay this spell's mana cost."
//!
//! The alternative cost is offered as a way of casting the spell only while its
//! condition, if any, is true (see [`crate::spell_costs::alternative_cost_allowed`]).

use super::costs_casting_self::{cost_condition, this_spell_cost_ability};
use super::{AbilityPattern, ConditionPattern};
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::costs::parse_cost;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// One action of an alternative cost ("pay {1}", "pay 4 life", "sacrifice a Mountain",
/// "exile a blue card from your hand", "return an Island you control to its owner's
/// hand", "tap an untapped creature you control", "discard a Forest card").
fn cost_action(s: &str) -> Option<Cost> {
    let s = end(s);
    // "reveal your hand" (Land Grant): revealing is the whole cost (CR 701.20).
    if s == "reveal your hand" {
        return Some(Cost::free().with(CostPart::Effect(Box::new(Effect::RevealHand {
            who: PlayerRef::You,
        }))));
    }
    if let Some(m) = s.strip_prefix("pay {") {
        let m = ManaCost::parse(&format!("{{{m}"))?;
        if m.has_x() || format!("{m}").to_lowercase() != format!("{{{}", &s[5..]) {
            return None;
        }
        return Some(Cost::mana(m));
    }
    // "discard another card" after "discard an Island card": one more card.
    let another;
    let s = match s.strip_prefix("discard another ") {
        Some(r) => {
            another = format!("discard a {r}");
            another.as_str()
        }
        None => s,
    };
    let (c, false) = parse_cost(s)? else {
        return None;
    };
    let ok = c.mana.is_none()
        && c.parts.len() == 1
        && match c.parts[0] {
            // Nothing may follow "life" (the cost parser tolerates trailing words).
            CostPart::PayLife(Value::Const(_)) => s.ends_with(" life"),
            _ => true,
        }
        && matches!(
            c.parts[0],
            CostPart::PayLife(Value::Const(_))
                | CostPart::Sacrifice { .. }
                | CostPart::Exile { .. }
                | CostPart::ReturnToHand { .. }
                | CostPart::TapUntapped { .. }
                | CostPart::Discard { random: false, .. }
        );
    ok.then_some(c)
}

/// "pay {1} and return a basic land you control to its owner's hand": the actions joined
/// by "and", paid together.
pub(crate) fn plain_cost(s: &str) -> Option<Cost> {
    // A value defined by the card paid ("a black card with mana value X") isn't a cost
    // this compiler can express.
    if s.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    let mut cost = Cost::free();
    let mut verb = String::new();
    for part in s.split(" and ") {
        // "discard an Island card and another card": the verb carries over.
        let carried;
        let part = match cost_action(part) {
            Some(_) => part,
            None if !verb.is_empty() => {
                carried = format!("{verb} {part}");
                carried.as_str()
            }
            None => return None,
        };
        verb = part.split(' ').next().unwrap_or("").to_string();
        let c = cost_action(part)?;
        if let Some(m) = c.mana {
            match cost.mana.as_mut() {
                Some(t) => t.add(&m),
                None => cost.mana = Some(m),
            }
        }
        cost.parts.extend(c.parts);
    }
    Some(cost)
}

/// "[If <condition>, ]you may <cost> rather than pay ~'s mana cost[ if <condition>]."
fn own_alternative_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let l = end(&lower);
    if l.contains(". ") {
        return None;
    }
    let (pre, body) = match l.split_once(", you may ") {
        // A condition about the card itself ("If ~ is in your graveyard, ...") would be
        // a permission to cast it from elsewhere, not only a cost.
        Some((head, _)) if head.contains('~') => return None,
        Some((head, body)) => (Some(cost_condition(head, ctx)?), body),
        None => (None, l.strip_prefix("you may ")?),
    };
    // "you may cast ~ without paying its mana cost" (CR 118.9): an alternative cost of
    // nothing.
    let (cost, tail) = match body.strip_prefix("cast ~ without paying its mana cost") {
        Some(tail) => (Cost::free(), tail),
        None => match body.split_once(" rather than pay ~'s mana cost") {
            Some((cost_s, tail)) => (plain_cost(cost_s)?, tail),
            // "you may pay {B/R} to cast ~" (an alternative cost, CR 118.9).
            None => (plain_cost(body.strip_suffix(" to cast ~")?)?, ""),
        },
    };
    let post = match tail.trim() {
        "" => None,
        t => Some(cost_condition(t, ctx)?),
    };
    let cond = match (pre, post) {
        (Some(a), Some(b)) => Some(Condition::And(vec![a, b])),
        (a, b) => a.or(b),
    };
    Some(vec![this_spell_cost_ability(
        CostChange::AlternativeCost(cost),
        cond,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "costs_casting: rather than pay this spell's mana cost", priority: 80, parse: own_alternative_cost } }

/// "you control a commander" (CR 903.3): a commander, anyone's, on the battlefield under
/// your control.
fn control_a_commander(c: &str) -> Option<Condition> {
    (end(c) == "you control a commander").then(|| {
        Condition::Exists(Filter::and(vec![
            Filter::Commander,
            Filter::ControlledBy(PlayerRel::You),
        ]))
    })
}

inventory::submit! { ConditionPattern { name: "costs_casting: you control a commander", priority: 80, parse: control_a_commander } }
