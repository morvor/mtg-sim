//! Counting colors among objects (Vivid, an ability word, CR 207.2c): "the number of
//! colors among permanents you control" and "for each color among permanents you control"
//! (`Value::ColorsAmong`, CR 105.2 — colorless isn't a color, CR 105.2c), and the
//! condition "there are five colors among permanents you control". Also "draw cards equal
//! to [value]", which several of these cards use.

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

/// "draw cards equal to [value]", "target player draws cards equal to [value]": the
/// player draws that many cards (CR 121.2), the value read as the ability resolves
/// (CR 107.3c-style definition of the number, never negative).
fn draw_cards_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, value) = l.split_once(" cards equal to ")?;
    let verb_ok = head == "draw" || head.ends_with(" draws") || head.ends_with(" draw");
    if !verb_ok || value.contains(" this way") {
        return None;
    }
    let it = b.it.clone();
    super::r107_numbers::where_x_is_parts(&format!("{head} x cards"), value, b, it)
}

inventory::submit! { EffectPattern { name: "misc: draw cards equal to [value]", priority: 70, parse: draw_cards_equal_to } }

/// "there are five colors among permanents you control", "there are two or more colors
/// among creatures you control".
fn colors_among_condition(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("there are ")?;
    let (n, rest) = parse_number(r)?;
    let (cmp, rest) = match rest.trim_start().strip_prefix("or more ") {
        Some(r) => (Cmp::Ge, r),
        // Five is the most there can be: "five colors" means all five.
        None if matches!(n, Value::Const(5)) => (Cmp::Ge, rest.trim_start()),
        None => return None,
    };
    let r = rest.strip_prefix("colors among ")?;
    let (f, true, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(Value::ColorsAmong(f), cmp, n))
}

inventory::submit! { ConditionPattern { name: "misc: there are N colors among [objects]", priority: 100, parse: colors_among_condition } }
