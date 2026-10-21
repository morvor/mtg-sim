//! Choices among listed values made as an instruction is performed:
//!
//! - "[object] gains your choice of flying, vigilance, deathtouch, or haste until end of
//!   turn", "... both gain your choice of first strike or lifelink until end of turn"
//!   (Argivian Avenger, Jodah's Avenger, Ballroom Brawlers): the controller chooses one of
//!   the listed abilities as the instruction is performed (CR 608.2d), and the
//!   instruction gives that one.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// The listed options of "A, B, or C" / "A or B".
fn options(s: &str) -> Option<Vec<String>> {
    let v: Vec<String> = if let Some((a, b)) = s.split_once(", or ") {
        let mut v: Vec<String> = a.split(", ").map(str::to_string).collect();
        v.push(b.to_string());
        v
    } else {
        let (a, b) = s.split_once(" or ")?;
        vec![a.to_string(), b.to_string()]
    };
    (v.len() >= 2 && v.iter().all(|o| !o.is_empty())).then_some(v)
}

/// "... gains your choice of A, B, or C [until end of turn]".
fn gains_your_choice_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (verb, at) = ["gains your choice of ", "gain your choice of "]
        .iter()
        .find_map(|v| l.find(v).map(|i| (*v, i)))?;
    let head = &l[..at];
    let r = &l[at + verb.len()..];
    let (list, tail) = match r.find(" until ") {
        Some(i) => (&r[..i], &r[i..]),
        None => (r, ""),
    };
    let opts = options(list)?;
    let plain_verb = verb.trim_end_matches("your choice of ");
    let first = b.targets.len();
    let mut out = Vec::new();
    for o in &opts {
        b.targets.truncate(first);
        let text = format!("{head}{plain_verb}{o}{tail}");
        let e = parse_clause(&text, b)?;
        // Each option grants only what it names: a keyword ability.
        out.push((o.clone(), e));
    }
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options: out,
    })
}

inventory::submit! { EffectPattern { name: "gains your choice of [abilities]", priority: 60, parse: gains_your_choice_of } }
