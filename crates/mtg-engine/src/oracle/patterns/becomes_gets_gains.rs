//! "Until end of turn, ~ becomes a Dragon, gets +5/+3, and gains flying and trample."
//! (Dragonsoul Knight, Paragon of the Amesha): one continuous effect on one object that
//! changes its creature types (layer 4, CR 205.1a), its power and toughness (layer 7c) and
//! gives it keywords (layer 6), each part composed from the sentence it would be on its own.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_pt_mod, parse_sentence, Builder};
use crate::oracle::phrases::end;

fn becomes_gets_gains(s: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(s);
    // The duration may lead or (as the sentence is rewritten) trail.
    let r = l
        .strip_prefix("until end of turn, ")
        .or_else(|| l.strip_suffix(" until end of turn"))?;
    let (subject, rest) = r.split_once(" becomes ")?;
    let (becomes, rest) = rest.split_once(", gets ")?;
    let (pt, gains) = rest.split_once(", and gains ")?;
    let (p, t, tail) = parse_pt_mod(pt)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let parts = [
        parse_sentence(&format!("{subject} becomes {becomes} until end of turn"), b)?,
        parse_sentence(&format!("{subject} gains {gains} until end of turn"), b)?,
    ];
    let mut what_seen: Option<String> = None;
    let mut all = Vec::new();
    for (i, part) in parts.into_iter().enumerate() {
        let Effect::Modify {
            what,
            mods,
            duration: Duration::EndOfTurn,
        } = part
        else {
            return None;
        };
        // Both parts must be about the same object.
        let w = format!("{what:?}");
        if what_seen.as_ref().is_some_and(|x| *x != w) {
            return None;
        }
        what_seen = Some(w);
        all.push((what, mods));
        if i == 0 {
            all.last_mut()?.1.push(Modification::ModifyPT(p.clone(), t.clone()));
        }
    }
    let mut it = all.into_iter();
    let (what, mut mods) = it.next()?;
    for (_, m) in it {
        mods.extend(m);
    }
    Some(Effect::Modify {
        what,
        mods,
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "until end of turn, [object] becomes ..., gets +N/+N, and gains ...", priority: 100, parse: becomes_gets_gains } }
