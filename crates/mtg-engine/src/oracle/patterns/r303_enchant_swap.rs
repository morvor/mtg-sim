//! "It loses "enchant creature card in a graveyard" and gains "enchant creature put onto the
//! battlefield with ~."" (Animate Dead, Dance of the Dead): an indefinite effect on the
//! Aura (CR 611.2c, until it leaves the battlefield, CR 400.7) that removes one enchant
//! ability and adds another in layer 6 (CR 613.1f). From then on, what it can legally
//! enchant is defined by the new enchant ability (CR 303.4c, 702.5a), checked as state-based
//! actions (CR 704.5m).

use super::EffectPattern;
use crate::ability::*;
use crate::keywords::Keyword;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// A quoted keyword ability: "\"enchant creature card in a graveyard\"".
fn quoted_keyword(s: &str, b: &Builder) -> Option<Keyword> {
    let s = s.trim().strip_prefix('"')?.strip_suffix('"')?;
    let s = s.trim().trim_end_matches('.');
    let abilities = crate::oracle::keywords::parse_keyword_line(s, b.ctx)?;
    match abilities.as_slice() {
        [a] => match &a.kind {
            AbilityKind::Keyword(k) => Some(k.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn loses_and_gains_keyword(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = match l.strip_prefix("it loses ") {
        Some(r) if matches!(b.it, Sel::This) => r,
        _ => l.strip_prefix("~ loses ")?,
    };
    let (lost, gained) = r.split_once(" and gains ")?;
    let lost = quoted_keyword(lost, b)?;
    let gained = quoted_keyword(gained, b)?;
    if lost.kind != gained.kind {
        return None;
    }
    b.it = Sel::This;
    Some(Effect::Modify {
        what: Sel::This,
        mods: vec![
            Modification::LoseKeyword(lost),
            Modification::AddKeyword(gained),
        ],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "r303 it loses \"[keyword]\" and gains \"[keyword]\"", priority: 100, parse: loses_and_gains_keyword } }
