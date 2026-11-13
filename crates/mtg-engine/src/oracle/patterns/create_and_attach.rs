//! "Create [token] and attach ~ to it." (Auxiliary Boosters, Field-Tested Frying Pan) and
//! "Gain control of target Equipment, then create a 0/0 black Phyrexian Germ creature
//! token and attach that Equipment to it." (Grip of Phyresis): the token is created, then
//! the Equipment is attached to it. Those are two actions: abilities that trigger on the
//! token entering see it as it entered, before the Equipment is attached (CR 603.2,
//! 608.2c; see `trigger_timing`), and state-based actions aren't checked in between, so
//! a 0/0 token survives if the Equipment raises its toughness.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn create_and_attach(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, what) = if let Some(f) = l.strip_suffix(" and attach ~ to it") {
        (f, Sel::This)
    } else if let Some(f) = l.strip_suffix(" and attach that equipment to it") {
        // The Equipment an earlier instruction targeted.
        let Sel::Target(slot) = b.it else {
            return None;
        };
        (f, Sel::Target(slot))
    } else {
        return None;
    };
    if !first.starts_with("create ") {
        return None;
    }
    let e = parse_clause(first, b)?;
    if !matches!(e, Effect::CreateToken { .. }) {
        return None;
    }
    b.it = Sel::Var(vars::CREATED);
    Some(Effect::Seq(vec![
        e,
        Effect::Attach {
            what,
            to: Sel::Var(vars::CREATED),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "create [token] and attach ~ to it", priority: 90, parse: create_and_attach } }
