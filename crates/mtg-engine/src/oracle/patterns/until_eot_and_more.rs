//! "~ has base power and toughness 4/2 until end of turn and gains first strike until end
//! of turn." (the Shadowmoor Mimics), "... and can't be blocked this turn": two
//! instructions for the same object joined by "and", each with its own duration. They're
//! performed in order, as two sentences would be.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn until_eot_and_more(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, rest) = if let Some(r) = l.strip_prefix("~ ") {
        ("~", r)
    } else {
        return None;
    };
    let (first, second) = rest.split_once(" until end of turn and ")?;
    // The second instruction is a verb phrase of its own ("gains flying until end of
    // turn", "can't be blocked this turn").
    if !(second.starts_with("gains ") || second.starts_with("can't ")) {
        return None;
    }
    let e1 = parse_clause(&format!("{subject} {first} until end of turn"), b)?;
    let e2 = parse_clause(&format!("{subject} {second}"), b)?;
    Some(Effect::Seq(vec![e1, e2]))
}

inventory::submit! { EffectPattern { name: "~ [does X] until end of turn and [gains Y / can't Z]", priority: 100, parse: until_eot_and_more } }
