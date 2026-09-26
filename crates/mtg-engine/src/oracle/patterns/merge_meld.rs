//! Oracle patterns for merged and melded permanents: "Whenever this creature mutates"
//! (CR 702.140d) and "if you both own and control [this] and a creature named [partner],
//! exile them, then meld them into [result]" (CR 701.42a).

use super::{AbilityPattern, ConditionPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::merge::{MELD_EFFECT, MELD_PAIR_CONDITION, MUTATES_SELF};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use smol_str::SmolStr;
use std::sync::Arc;

fn mutates_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    match end(r) {
        "~ mutates" | "this creature mutates" => Some((
            TriggerCond::Custom(SmolStr::new(MUTATES_SELF)),
            Sel::This,
            PlayerRef::You,
        )),
        _ => None,
    }
}

/// "exile them, then meld them into [its meld result]".
fn meld_into(l: &str, _b: &mut Builder) -> Option<Effect> {
    let result = end(l).strip_prefix("exile them, then meld them into ")?;
    let result = match result {
        "" => return None,
        MELD_RESULT => "",
        r => r,
    };
    Some(Effect::Custom(format!("{MELD_EFFECT}{result}").into()))
}

/// "you both own and control this and its meld partner" (after [`meld_block`]).
fn own_and_control_pair(c: &str) -> Option<Condition> {
    (end(c) == format!("you both own and control this and {MELD_PARTNER}"))
        .then(|| Condition::Custom(SmolStr::new(MELD_PAIR_CONDITION)))
}

const MELD_PARTNER: &str = "its meld partner";
const MELD_RESULT: &str = "its meld result";

/// "…if [condition and] you both own and control [this] and a creature named [partner],
/// [you may pay … If you do,] exile them, then meld them into [result]." The names may
/// contain commas, so they're replaced by references to the meld pair and its result
/// before the ability is parsed; a condition joined to it with "and" is parsed apart.
fn meld_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    // ASCII lowercasing keeps byte offsets the same as in `text`.
    let lower = text.to_ascii_lowercase();
    let meld = ", exile them, then meld them into ";
    let m = lower.find(meld)?;
    let own = "you both own and control ";
    let mut i = lower[..m].find(own)?;
    let named = lower[i..m].find(" named ")? + i;
    // The partner's name ends where the rest of the ability continues.
    let j = [", exile them", ", you may pay "]
        .iter()
        .filter_map(|p| lower[named..].find(p).map(|k| k + named))
        .min()?;
    // The result's name ends its sentence.
    let after = m + meld.len();
    let k = lower[after..]
        .find(". ")
        .map_or(text.len(), |k| k + after + 1);
    // "if [condition] and you both own and control …".
    let mut extra = None;
    if lower[..i].ends_with(" and ") {
        let if_at = lower[..i].rfind("if ")?;
        let cond = &lower[if_at + 3..i - 5];
        extra = Some(crate::oracle::statics::parse_condition(cond, ctx)?);
        i = if_at + 3;
    }
    let rewritten = format!(
        "{}{own}this and {MELD_PARTNER}{}{meld}{MELD_RESULT}{}",
        &text[..i],
        &text[j..m],
        if k < text.len() { &text[k..] } else { "." },
    );
    let mut out = crate::oracle::parse_ability(&rewritten, ctx)?;
    for a in out.iter_mut() {
        let a = Arc::make_mut(a);
        a.text = text.to_string();
        if let (Some(extra), AbilityKind::Triggered(t)) = (&extra, &mut a.kind) {
            let c = t.intervening_if.take()?;
            t.intervening_if = Some(Condition::And(vec![extra.clone(), c]));
        }
    }
    Some(out)
}

inventory::submit! { TriggerPattern { name: "this creature mutates", priority: 0, parse: mutates_trigger } }
inventory::submit! { EffectPattern { name: "exile them, then meld them", priority: 0, parse: meld_into } }
inventory::submit! { ConditionPattern { name: "you both own and control this and its meld partner", priority: 0, parse: own_and_control_pair } }
inventory::submit! { AbilityPattern { name: "meld pair abilities", priority: 0, parse: meld_block } }
