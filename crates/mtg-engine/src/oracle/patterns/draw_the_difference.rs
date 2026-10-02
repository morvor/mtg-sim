//! "At the beginning of your upkeep, if you have fewer than seven cards in hand, draw
//! cards equal to the difference", "Then if you have fewer than three cards in hand, draw
//! cards equal to the difference": the difference between that number and the number of
//! cards in your hand, determined as the instruction is performed (CR 608.2h).

use super::AbilityPattern;
use crate::oracle::phrases::parse_number;
use crate::oracle::CompileContext;

const DIFFERENCE: &str = "draw cards equal to the difference";

fn draw_the_difference(block: &str, ctx: &CompileContext) -> Option<Vec<crate::ability::Ability>> {
    let lower = block.to_lowercase();
    let at = lower.find(DIFFERENCE)?;
    // The comparison it refers to comes just before, in the same sentence.
    let head = &lower[..at];
    let i = head.rfind("if you have fewer than ")?;
    let after = &head[i + "if you have fewer than ".len()..];
    let (n, rest) = parse_number(after)?;
    let n = n.as_const()?;
    if rest.trim_start() != "cards in hand, " && rest.trim_start() != "cards in hand, then " {
        return None;
    }
    let text = format!(
        "{}draw cards equal to {n} minus the number of cards in your hand{}",
        &block[..at],
        &block[at + DIFFERENCE.len()..]
    );
    let mut abilities = crate::oracle::parse_ability(&text, ctx)?;
    for a in &mut abilities {
        std::sync::Arc::make_mut(a).text = block.to_string();
    }
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "draw cards equal to the difference", priority: 100, parse: draw_the_difference } }
