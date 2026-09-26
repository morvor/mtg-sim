//! A choice between additional costs (CR 601.2b): "As an additional cost to cast this
//! spell, sacrifice a creature or pay {3}{B}.", "... discard a card or pay 3 life.",
//! "... pay {2} or sacrifice an artifact or creature." The player chooses one as the spell
//! is cast; the chosen cost is paid with the rest of the total cost (see
//! `cost_choices.rs`).

use super::costs_casting_alt::plain_cost;
use super::costs_casting_self::this_spell_cost_ability;
use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::costs::parse_cost;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// One option: a plain cost ("pay {2}", "sacrifice a creature", "discard a card", "pay 3
/// life", "exile two cards from your graveyard"), "reveal a Goblin card from your hand"
/// (CR 701.20) or "forage" (CR 701.61).
fn option_cost(s: &str) -> Option<Cost> {
    let s = end(s);
    // Beholding records its own name for "if a Dragon was beheld" (see `a701_behold`).
    if s.starts_with("behold ") {
        return None;
    }
    if let Some(c) = plain_cost(s) {
        return Some(c);
    }
    let (c, false) = parse_cost(s)? else {
        return None;
    };
    let ok = c.mana.is_none()
        && matches!(
            c.parts.as_slice(),
            [CostPart::Forage] | [CostPart::RevealFromHand { .. }]
        );
    ok.then_some(c)
}

/// "[cost] or [cost]": the only way to split the text into two costs.
fn two_options(s: &str) -> Option<[(SmolStr, Cost); 2]> {
    let mut found = None;
    let mut from = 0;
    while let Some(i) = s[from..].find(" or ") {
        let at = from + i;
        let (a, b) = (&s[..at], &s[at + 4..]);
        if let (Some(ca), Some(cb)) = (option_cost(a), option_cost(b)) {
            if found.is_some() {
                return None;
            }
            found = Some([(SmolStr::new(a), ca), (SmolStr::new(end(b)), cb)]);
        }
        from = at + 4;
    }
    found
}

/// "[cost], [cost], or [cost]": three options.
fn three_options(s: &str) -> Option<Vec<(SmolStr, Cost)>> {
    let (a, rest) = s.split_once(", ")?;
    let (b, c) = rest.split_once(", or ")?;
    [a, b, c]
        .into_iter()
        .map(|o| Some((SmolStr::new(end(o)), option_cost(o)?)))
        .collect()
}

/// "As an additional cost to cast ~, [cost] or [cost]." / "..., [cost], [cost], or
/// [cost]."
fn additional_cost_choice(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, ")?;
    if r.contains(". ") || r.starts_with("you may ") {
        return None;
    }
    let options = match three_options(r) {
        Some(v) => v,
        None => two_options(r)?.to_vec(),
    };
    Some(vec![this_spell_cost_ability(
        CostChange::AdditionalCostChoice(options),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "costs_casting: additional cost choice", priority: 80, parse: additional_cost_choice } }
