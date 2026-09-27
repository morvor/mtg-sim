//! "Conjure a card named [name] into your hand", "conjure four cards named [name] into
//! your library, then shuffle", "conjure a card named [name] onto the battlefield",
//! "conjure the Power Nine into your library" (see `kw/conjure.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::kw::conjure::{effect_name, ConjureNames, ConjureZone};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "The Power Nine".
const POWER_NINE: [&str; 9] = [
    "Ancestral Recall",
    "Black Lotus",
    "Mox Emerald",
    "Mox Jet",
    "Mox Pearl",
    "Mox Ruby",
    "Mox Sapphire",
    "Time Walk",
    "Timetwister",
];

fn conjure(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (l, shuffle) = match l.strip_suffix(", then shuffle") {
        Some(r) => (r, true),
        None => (l, false),
    };
    let r = l.strip_prefix("conjure ")?;
    let (zone, r) = [
        (" into your hand", ConjureZone::Hand),
        (" into your library", ConjureZone::Library),
        (" into your graveyard", ConjureZone::Graveyard),
        (" onto the battlefield", ConjureZone::Battlefield),
    ]
    .into_iter()
    .find_map(|(s, z)| r.strip_suffix(s).map(|r| (z, r)))?;
    let (count, names, how) = if r == "the power nine" {
        (
            POWER_NINE.len() as u32,
            POWER_NINE.iter().map(|s| s.to_string()).collect(),
            ConjureNames::All,
        )
    } else {
        let (n, rest) = parse_number(r)?;
        let count = u32::try_from(n.as_const()?).ok()?;
        let name = rest
            .strip_prefix("card named ")
            .or_else(|| rest.strip_prefix("cards named "))?;
        let name = if name == "~" {
            b.ctx.card_name.to_string()
        } else {
            name.to_string()
        };
        (count, vec![name], ConjureNames::Each)
    };
    // Only real cards can be conjured.
    if count == 0 || names.iter().any(|n| mtg_data::cards().by_name(n).is_none()) {
        return None;
    }
    let e = Effect::Custom(effect_name(count, zone, how, &names).into());
    Some(if shuffle {
        Effect::Seq(vec![
            e,
            Effect::Shuffle {
                who: PlayerRef::You,
            },
        ])
    } else {
        e
    })
}

inventory::submit! { EffectPattern { name: "conjure", priority: 100, parse: conjure } }
