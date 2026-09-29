//! Costs that untap other permanents (CR 118.1): "Untap a tapped creature you control",
//! "Untap two tapped blue creatures you control" (Halo Fountain, Crackleburr). A tapped
//! permanent with a stun counter can be chosen: untapping it is replaced by removing a
//! stun counter (CR 122.1d), and the cost is still paid.

use super::CostPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn untap_tapped(p: &str) -> Option<CostPart> {
    let r = strip(p, "untap")?;
    let (n, r) = parse_number(r)?;
    let r = strip(r, "tapped")?;
    // Only your own permanents ("an opponent controls" is another cost).
    if !end(r).ends_with(" you control") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(CostPart::UntapTapped {
        filter: Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
        count: n,
    })
}

inventory::submit! { CostPattern { name: "untap N tapped [permanents] you control", priority: 100, parse: untap_tapped } }
