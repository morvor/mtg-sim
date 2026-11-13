//! Counts of the spells a player has cast this turn in value phrases (also as "for each
//! ..."): "the number of spells you've cast this turn", "the number of instant and sorcery
//! spells you've cast this turn", and paradox's "spell(s) you've cast this turn from
//! anywhere other than your hand" (Surge of Brilliance) or "spells you've cast from
//! anywhere other than your hand this turn" (Impending Flux).
//!
//! These count spells cast (CR 601.2i), wherever they were cast from; a copy of a spell
//! that wasn't cast isn't counted (CR 707.10). A spell counting spells cast counts itself
//! once it has been cast. "Other spells you've cast" (not counting the source) is handled
//! only in "the total mana value of other ... spells you've cast this turn".

use crate::ability::*;
use crate::oracle::phrases::parse_object_phrase;

/// `r` follows "the number of ". Returns the value and the rest of the text.
pub fn spells_you_cast_value(r: &str) -> Option<(Value, String)> {
    let (kind, rest) = r.split_once(" you've cast ")?;
    if kind.starts_with("other ") || kind.starts_with("another ") {
        return None;
    }
    let kinds = match kind {
        "spell" | "spells" => Filter::Any,
        _ => {
            let (f, _, tail) = parse_object_phrase(kind)?;
            if !tail.trim().is_empty() || !(kind.ends_with(" spell") || kind.ends_with(" spells"))
            {
                return None;
            }
            f
        }
    };
    let not_from_hand = Filter::not(Filter::CastFrom(ZoneKind::Hand));
    let (f, tail) = if let Some(t) = rest
        .strip_prefix("this turn from anywhere other than your hand")
        .or_else(|| rest.strip_prefix("from anywhere other than your hand this turn"))
    {
        (Filter::and(vec![kinds, not_from_hand]), t)
    } else if let Some(t) = rest.strip_prefix("this turn") {
        (kinds, t)
    } else {
        return None;
    };
    Some((
        Value::SpellsCastThisTurn(PlayerRef::You, f),
        tail.to_string(),
    ))
}

/// "the total mana value of [other] [kind] spells you've cast this turn" (Call Forth the
/// Tempest): summed over the spells cast (CR 601.2i), whether or not they resolved; a copy
/// of a spell that wasn't cast isn't counted (CR 707.10). "Other" leaves out the source.
pub fn total_mana_value_of_spells_you_cast(s: &str) -> Option<(Value, String)> {
    let r = s.strip_prefix("the total mana value of ")?;
    let (r, other) = match r.strip_prefix("other ") {
        Some(r) => (r, true),
        None => (r, false),
    };
    let (v, rest) = spells_you_cast_value(r)?;
    let Value::SpellsCastThisTurn(who, f) = v else {
        return None;
    };
    let f = if other {
        Filter::and(vec![f, Filter::Not(Box::new(Filter::In(Box::new(Sel::This))))])
    } else {
        f
    };
    Some((Value::SpellsCastThisTurnManaValue(who, f), rest))
}
