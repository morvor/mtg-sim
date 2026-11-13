//! "Choose any number of target creatures with equal toughness. Destroy the chosen
//! creatures." (V.A.T.S.), "Choose up to three target creature cards with total mana
//! value 8 or less in your graveyard." (The War in Heaven): several targets that must
//! have a relationship with each other (see `src/target_groups.rs`), chosen as the spell
//! or ability is put on the stack (CR 601.2c). Choosing them does nothing by itself; the
//! sentences that follow name them ("the chosen creatures", "those creatures").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_target};

fn choose_target_group(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || spec.together.is_none() {
        return None;
    }
    let TargetKind::Object(_) = &spec.what else {
        return None;
    };
    // The plural noun the later sentences use: "creatures", "creature cards".
    let after = r.split_once("target ")?.1;
    let noun = after
        .split(" with ")
        .next()?
        .split(" that ")
        .next()?
        .split(" in ")
        .next()?
        .split(" from ")
        .next()?
        .trim()
        .to_string();
    let slot = b.add_target(spec, r);
    for name in [format!("the chosen {noun}"), format!("those {noun}")] {
        b.named.push((name, Sel::Target(slot)));
    }
    b.it = Sel::Target(slot);
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choose [targets that must be related]", priority: 80, parse: choose_target_group } }
