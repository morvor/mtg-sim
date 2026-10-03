//! "[Object] becomes the color or colors of your choice [until end of turn]" (CR 105.3,
//! 105.4): as the effect is created, its controller chooses any single color or any
//! combination of colors (never colorless), and the new colors replace the object's
//! colors (layer 5, CR 613.1e). Quickchange, Prismwake Merrow, Scuttlemutt, Dream Coat,
//! Shyft ("you may have ~ become ...").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

fn colors_of_your_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (l, duration) = match l.strip_suffix(" until end of turn") {
        Some(x) => (x, Duration::EndOfTurn),
        None => (l, Duration::Permanent),
    };
    let l = l.strip_prefix("have ").unwrap_or(l);
    let subj = l
        .strip_suffix(" becomes the color or colors of your choice")
        .or_else(|| l.strip_suffix(" become the color or colors of your choice"))?;
    let (what, rest) = object_ref(subj, b)?;
    if !end(&rest).trim().is_empty() || matches!(what, Sel::None) {
        return None;
    }
    // A spell as the subject ("target spell or permanent") isn't a permanent's color.
    if let Sel::Target(i) = what {
        if !matches!(
            b.targets.get(i as usize).map(|t| &t.what),
            Some(TargetKind::Object(_))
        ) {
            return None;
        }
    }
    Some(Effect::seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Colors,
        },
        Effect::Modify {
            what,
            mods: vec![Modification::SetChosenColors],
            duration,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "r105 becomes the color or colors of your choice", priority: 100, parse: colors_of_your_choice } }
