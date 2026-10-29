//! "[Gain control of target artifact ...]. If that artifact is an Equipment, attach it to
//! ~." (Thieving Skydiver): after the previous instruction, the object it affected is
//! attached to the source if it's an Equipment then. Attaching does nothing if it can't
//! be attached (the source left the battlefield, CR 701.3b) — the Equipment stays where it
//! was.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn if_equipment_attach_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(noun) = end(l)
        .strip_prefix("if that ")
        .and_then(|r| r.strip_suffix(" is an equipment, attach it to ~"))
    else {
        return false;
    };
    // "That artifact" names what the previous instruction (gaining control of it) took.
    if !matches!(noun, "artifact" | "permanent")
        || !matches!(prev, Effect::GainControl { what, .. } if format!("{what:?}") == format!("{:?}", b.it))
    {
        return false;
    }
    let it = b.it.clone();
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::If {
            cond: Condition::SelMatches(it.clone(), Filter::Subtype("Equipment".into())),
            then: Box::new(Effect::Attach {
                what: it,
                to: Sel::This,
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if that artifact is an Equipment, attach it to ~", priority: 60, apply: if_equipment_attach_it } }
