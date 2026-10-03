//! "Each creature that convoked ~ connives." (Lethal Scheme): each creature that was
//! tapped to pay for the spell with convoke (CR 702.51c) connives (CR 701.50), including
//! one that has left the battlefield since (CR 701.50b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use smol_str::SmolStr;

fn convokers_connive(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l != "each creature that convoked ~ connives" && l != "each creature that convoked it connives"
    {
        return None;
    }
    Some(Effect::Custom(SmolStr::new(
        crate::kw::convoke::CONVOKERS_CONNIVE,
    )))
}

inventory::submit! { EffectPattern { name: "each creature that convoked it connives", priority: 100, parse: convokers_connive } }
