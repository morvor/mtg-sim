//! "Return [card] to the battlefield and attach ~ to it." (Pre-War Formalwear): the
//! source is attached to the permanent the card became (CR 400.7: "it" is the new
//! object, followed through `vars::IT`). If it can't be attached to that permanent (it
//! isn't a creature, for an Equipment), the source doesn't move and stays where it is,
//! unattached if it was (CR 701.3b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn return_and_attach(l: &str, b: &mut Builder) -> Option<Effect> {
    let first = end(l).strip_suffix(" and attach ~ to it")?;
    if !first.ends_with(" to the battlefield") {
        return None;
    }
    let e = parse_clause(first, b)?;
    let Effect::Move { to, .. } = &e else {
        return None;
    };
    if to.zone != ZoneKind::Battlefield {
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Seq(vec![
        e,
        Effect::Attach {
            what: Sel::This,
            to: Sel::Var(vars::IT),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "return [card] to the battlefield and attach ~ to it", priority: 90, parse: return_and_attach } }
