//! Secret votes (secret council, CR 701.38): "Each player secretly votes for a creature you
//! don't control, then those votes are revealed." and what follows them: "For each
//! creature with one or more votes, put that many stun counters on it, then tap it." (Trap
//! the Trespassers). See `kwa/vote.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kwa::vote::{GOT_VOTES, VOTED_FOR, VOTES_FOR_IT};
use crate::kwa::Spec;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// An object phrase for what may be voted for, on the battlefield unless it names a zone.
fn votable(s: &str) -> Option<Filter> {
    let x = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(x)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some(if f.zone().is_some() {
        f
    } else {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
    })
}

/// "each player secretly votes for [a object], then those votes are revealed".
fn secret_vote(l: &str, b: &mut Builder) -> Option<Effect> {
    let _ = b;
    let r = end(l).strip_prefix("each player secretly votes for ")?;
    let what = r.strip_suffix(", then those votes are revealed")?;
    let mut spec = Spec::new(
        KeywordAction::Vote,
        PlayerRef::You,
        Sel::All(votable(what)?),
        Value::c(1),
    );
    spec.secret = true;
    Some(spec.effect())
}

inventory::submit! { EffectPattern { name: "a701 secret vote", priority: 60, parse: secret_vote } }

/// "for each [object] with one or more votes, [effect]": the effect is performed for each
/// object voted for in the vote earlier in the spell or ability, with "it" that object
/// and "that many" the number of votes it got.
fn for_each_voted(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (what, rest) = r.split_once(" with one or more votes, ")?;
    let (f, _, tail) = parse_object_phrase(what)?;
    if !end(tail).is_empty() {
        return None;
    }
    let rest = rest.replace("that many", "x");
    let saved = std::mem::replace(&mut b.it, Sel::Var(VOTED_FOR));
    let e = parse_clause(&rest, b);
    b.it = saved;
    Some(Effect::ForEach {
        sel: Sel::All(Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Battlefield),
            Filter::Custom(SmolStr::new(GOT_VOTES)),
        ])),
        var: VOTED_FOR,
        effect: Box::new(Effect::seq(vec![
            Effect::SetX {
                value: Value::Custom(SmolStr::new(VOTES_FOR_IT)),
            },
            e?,
        ])),
    })
}

inventory::submit! { EffectPattern { name: "a701 for each object with votes", priority: 60, parse: for_each_voted } }
