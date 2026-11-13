//! "Whenever you attack, you may sacrifice X Foods. When you do, up to X target attacking
//! creatures each get +3/+3 ..." (Campsite Cuisine): nothing defines X, so the player
//! chooses it as they sacrifice (CR 107.3f) — they choose any number of those permanents
//! they control and sacrifice them, and X is that number for the rest of the resolution,
//! including the reflexive triggered ability it causes (CR 603.12a).
//!
//! Only a literal X in a triggered ability's text that nothing else defines is this X;
//! other patterns rewrite "that many" to "x" (Phyrexian Negator: "sacrifice that many
//! permanents"), so the ability pattern marks the literal one before the effect is parsed.

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The permanents chosen to be sacrificed.
const CHOSEN: Var = vars::USER + 9133;
/// Marks "sacrifice x" whose X the player chooses (a control character never in text).
const MARKED: &str = "sacrifice \u{2}x ";

fn sacrifice_x(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix(MARKED)?;
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

/// A permanent's triggered ability (not an entering one, whose X may be the spell's, CR
/// 107.3m) with "you may sacrifice X ..." and nothing defining X.
fn trigger_with_chosen_x(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    if ctx.is_spell()
        || !(lower.starts_with("whenever ") || lower.starts_with("at "))
        || lower.contains(" enters")
        || lower.contains("where x is")
        || lower.contains("{x}")
    {
        return None;
    }
    let i = lower.find("you may sacrifice x ")?;
    let at = i + "you may ".len();
    if lower.len() != text.len() {
        return None;
    }
    let marked = format!(
        "{}{}{}",
        text.get(..at)?,
        MARKED,
        text.get(at + "sacrifice x ".len()..)?
    );
    let mut abilities = crate::oracle::parse_ability(&marked, ctx)?;
    for a in &mut abilities {
        std::sync::Arc::make_mut(a).text = block.to_string();
    }
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "sacrifice x: chosen in a triggered ability", priority: 104, parse: trigger_with_chosen_x } }
