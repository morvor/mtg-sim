//! Class cards (CR 716): "When this Class becomes level N, ..." (in a class level
//! section, CR 716.2a).

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn becomes_level(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let n: u32 = end(r)
        .strip_prefix("~ becomes level ")
        .or_else(|| end(r).strip_prefix("this class becomes level "))?
        .trim()
        .parse()
        .ok()?;
    Some((
        TriggerCond::Custom(format!("{}{n}", crate::classes::BECOMES_LEVEL).into()),
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "r716 class becomes level", priority: 60, parse: becomes_level } }
