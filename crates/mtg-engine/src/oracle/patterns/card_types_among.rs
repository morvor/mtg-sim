//! Counting card types among objects (CR 205.2a): "the number of card types among other
//! nonland permanents you control" (Loot, the Key to Everything), "for each card type
//! among permanents you control" (`Value::CardTypesAmong`). Each object counts every
//! card type it has; the count is of distinct card types.

use crate::ability::*;
use crate::oracle::phrases::parse_object_phrase;

/// "card type(s) among [objects]" → the value and the text after the objects.
pub(crate) fn value(s: &str) -> Option<(Value, &str)> {
    let s = s.trim_start();
    let r = s
        .strip_prefix("card types among ")
        .or_else(|| s.strip_prefix("card type among "))?;
    let (f, _, rest) = parse_object_phrase(r)?;
    Some((Value::CardTypesAmong(f), rest))
}
