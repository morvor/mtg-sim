//! Oracle patterns for CR 702.166 bargain: the linked abilities that check whether the
//! spell (or the permanent it became) was bargained (CR 702.166b–c) — "If this spell was
//! bargained, ...", "When this creature enters, if it was bargained, ...", and "This spell
//! costs {2} less to cast if it's bargained." They check the bargain cost recorded in the
//! spell's `CastInfo::paid` (see `kw/bargain.rs`); a permanent's abilities see how its
//! spell was cast (CR 607.2i).

use super::{AbilityPattern, ConditionPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

/// "~ was bargained", "it was bargained", "it's bargained".
fn was_bargained(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "~ was bargained" | "it was bargained" | "it's bargained" | "~ is bargained"
    )
    .then(|| Condition::CostPaid(crate::kw::bargain::BARGAIN.into()))
}

inventory::submit! { ConditionPattern { name: "k702.166 was bargained", priority: 100, parse: was_bargained } }

/// "Look at the top four cards of your library. If this spell was bargained, look at the
/// top eight cards of your library instead. Put two of them into your hand ..." (Farsight
/// Ritual): the instruction without the middle sentence, looking at a number of cards
/// that depends on whether the spell was bargained (CR 702.166c).
fn bargained_look_at_more(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const IF: &str = "if ~ was bargained, look at the top ";
    const INSTEAD: &str = " cards of your library instead.";
    let lower = block.to_lowercase();
    let start = lower.find(IF)?;
    let after = &lower[start + IF.len()..];
    let n_end = after.find(INSTEAD)?;
    let (n, rest) = parse_number(&after[..n_end])?;
    if !rest.trim().is_empty() {
        return None;
    }
    let end_of_sentence = start + IF.len() + n_end + INSTEAD.len();
    let text = format!(
        "{}{}",
        &block[..start],
        block[end_of_sentence..].trim_start()
    );
    let abilities = crate::oracle::parse_ability(&text, ctx)?;
    let bargained = was_bargained("~ was bargained")?;
    let mut changed = false;
    let out = abilities
        .into_iter()
        .map(|a| {
            let mut kind = a.kind.clone();
            if let AbilityKind::Spell(s) = &mut kind {
                changed |= with_dig_count(&mut s.body.effect, &mut |old| {
                    Value::If(
                        Box::new(bargained.clone()),
                        Box::new(n.clone()),
                        Box::new(old),
                    )
                });
            }
            AbilityDef::with_link(kind, block.trim(), a.link)
        })
        .collect();
    changed.then_some(out)
}

/// Replaces the number of cards the (first) "look at the top N cards" instruction looks
/// at.
fn with_dig_count(e: &mut Effect, f: &mut dyn FnMut(Value) -> Value) -> bool {
    match e {
        Effect::Dig { n, .. } => {
            *n = f(std::mem::replace(n, Value::c(0)));
            true
        }
        Effect::Seq(v) => v.iter_mut().any(|e| with_dig_count(e, f)),
        _ => false,
    }
}

inventory::submit! { AbilityPattern { name: "k702.166 if bargained, look at more cards instead", priority: 100, parse: bargained_look_at_more } }
