//! "Whenever you attack, you may sacrifice X Foods. When you do, up to X target attacking
//! creatures each get +3/+3 ..." (Campsite Cuisine): nothing defines X, so the player
//! chooses it as they sacrifice (CR 107.3f) — they choose any number of those permanents
//! they control and sacrifice them, and X is that number for the rest of the resolution,
//! including the reflexive triggered ability it causes (CR 603.12a).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// The permanents chosen to be sacrificed.
const CHOSEN: Var = vars::USER + 9133;

fn sacrifice_x(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("sacrifice x ")?;
    if super::tokens_x_x::x_defined(b) {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(r)?;
    if !plural || !end(tail).is_empty() || super::statics::mentions_other_zones(&f) {
        return None;
    }
    let f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
    // The following sentences' X is this X.
    b.named
        .push((super::tokens_x_x::X_DEFINED.to_string(), Sel::None));
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: f.clone(),
                count: Value::CountSel(Box::new(Sel::All(f))),
                up_to: true,
                store: None,
            },
        },
        Effect::SetX {
            value: Value::CountSel(Box::new(Sel::Var(CHOSEN))),
        },
        Effect::SacrificeObjects {
            what: Sel::Var(CHOSEN),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "sacrifice x: the player chooses x as they sacrifice", priority: 960, parse: sacrifice_x } }
