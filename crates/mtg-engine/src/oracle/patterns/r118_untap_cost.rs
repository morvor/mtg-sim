//! Untapping permanents as a cost (CR 118.1): "Untap a tapped creature you control",
//! "Untap two tapped creatures you control" (Halo Fountain). The permanents are untapped
//! as the cost is paid; a stun counter's replacement effect can replace an untap (the
//! counter is removed instead and the permanent stays tapped), and the cost is still paid
//! (CR 122.1d).

use super::CostPattern;
use crate::ability::*;
use crate::oracle::phrases::*;

fn untap_tapped_cost(p: &str) -> Option<CostPart> {
    let r = strip(p, "untap")?;
    let (n, r) = parse_number(r)?;
    let r = strip(r, "tapped")?;
    // Your own permanents, or another player's ("Untap a tapped land an opponent
    // controls", Benthic Explorers: part of the cost, not a target; its rulings).
    let controller = if end(r).ends_with(" you control") {
        PlayerRel::You
    } else if end(r).ends_with(" an opponent controls") {
        PlayerRel::Opponent
    } else {
        return None;
    };
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() || !format!("{f:?}").contains(&format!("ControlledBy({controller:?})")) {
        return None;
    }
    Some(CostPart::UntapTapped {
        filter: f,
        count: n,
    })
}

inventory::submit! { CostPattern { name: "r118 untap N tapped [permanents] (cost)", priority: 60, parse: untap_tapped_cost } }
