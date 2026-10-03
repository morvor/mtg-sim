//! "When ~ enters, it becomes an Aura with enchant creature. Manifest the top card of your
//! library and attach ~ to it." (the "Forms" of Fate Reforged: Cloudform, Lightform,
//! Rageform).
//!
//! The enchantment becomes an Aura (an enchantment subtype, CR 205.3h) with the enchant
//! ability (CR 303.4a, 702.5) from then on (CR 611.2): an indefinite effect on the object
//! that lasts until it leaves the battlefield (CR 400.7). The next instruction attaches it;
//! an Aura that isn't attached to a legal object is put into its owner's graveyard as a
//! state-based action (CR 303.4d, 704.5m) — e.g. if there was no card to manifest.

use super::EffectPattern;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::*;
use smol_str::SmolStr;

/// "it becomes an Aura with enchant [quality]" / "~ becomes an Aura with enchant
/// [quality]", said of an enchantment with no subtypes (so adding the Aura subtype is all
/// its type change does, CR 205.1a).
fn becomes_aura(l: &str, b: &mut Builder) -> Option<Effect> {
    let (subj, enchant) = end(l).split_once(" becomes an aura with ")?;
    match subj {
        "~" => {}
        "it" if matches!(b.it, Sel::This) => {}
        _ => return None,
    }
    let tl = b.ctx.type_line;
    if tl.card_types != CardTypeSet::single(CardType::Enchantment) || !tl.subtypes.is_empty() {
        return None;
    }
    // Quoted: "it becomes an Aura with \"enchant creature put onto the battlefield with
    // ~.\"" (Necromancy).
    let enchant = match enchant.strip_prefix('"').and_then(|e| e.strip_suffix('"')) {
        Some(e) => e.trim_end_matches('.'),
        None => enchant,
    };
    if !enchant.starts_with("enchant ") {
        return None;
    }
    let abilities = crate::oracle::keywords::parse_keyword_line(enchant, b.ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Keyword(k) = &a.kind else {
        return None;
    };
    if k.kind != KeywordKind::Enchant {
        return None;
    }
    b.it = Sel::This;
    Some(Effect::Modify {
        what: Sel::This,
        mods: vec![
            Modification::AddSubtypes(vec![SmolStr::new("Aura")]),
            Modification::AddKeyword(k.clone()),
        ],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "r303 ~ becomes an Aura with enchant", priority: 90, parse: becomes_aura } }
