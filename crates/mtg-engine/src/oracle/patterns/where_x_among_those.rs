//! "Ally creatures you control get +X/+X until end of turn, where X is the number of colors
//! among those creatures." (General Tazri): "those creatures" are the objects the sentence
//! is about, so X is read about the same set, as the effect begins (CR 107.3c, 608.2h,
//! 611.2c).

use super::EffectPattern;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;

fn where_x_among_those(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, value) = l.rsplit_once(", where x is ")?;
    if !value.contains(" those creatures") {
        return None;
    }
    let (subject, _) = clause.split_once(" get ")?;
    let (_, plural, tail) = parse_object_phrase(subject)?;
    if !plural || !end(tail).is_empty() || subject.contains("target") {
        return None;
    }
    let value = value.replace(" those creatures", &format!(" {subject}"));
    parse_sentence(&format!("{clause}, where x is {value}"), b)
}

use crate::ability::Effect;

inventory::submit! { EffectPattern { name: "[objects] get +X/+X, where X is ... among those creatures", priority: 100, parse: where_x_among_those } }
