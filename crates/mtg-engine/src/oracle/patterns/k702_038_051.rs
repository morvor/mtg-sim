//! Keyword lines of CR 702.38–702.51 that the generic keyword parser doesn't handle:
//! "Modular—Sunburst" (CR 702.44c), "Affinity for outlaws" (CR 702.41a).

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::keywords::compile_keyword;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::oracle::CompileContext;
use smol_str::SmolStr;

fn keyword_line(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    // CR 702.44c: "Modular—Sunburst".
    if lower == "modular—sunburst" {
        let kw = Keyword::new(KeywordKind::Modular).text(SmolStr::new(
            crate::kw::modular::SUNBURST_TEXT,
        ));
        return Some(compile_keyword(kw, t));
    }
    // CR 702.41a, 700.12: "Affinity for outlaws" — Assassins, Mercenaries, Pirates,
    // Rogues, and/or Warlocks.
    if lower == "affinity for outlaws" {
        let kw = Keyword::with_filter(KeywordKind::Affinity, outlaw()).text(t);
        return Some(compile_keyword(kw, t));
    }
    None
}

/// An outlaw (CR 700.12): an object with the Assassin, Mercenary, Pirate, Rogue, and/or
/// Warlock creature types.
fn outlaw() -> Filter {
    Filter::Or(
        ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
            .into_iter()
            .map(|s| Filter::Subtype(SmolStr::new(s)))
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "k702_038_051 keywords", priority: 100, parse: keyword_line } }

/// "An opponent gains N life" (e.g. a splice cost, CR 702.47a): the player chooses one of
/// their opponents, who gains the life.
fn an_opponent_gains_life(l: &str, _b: &mut Builder) -> Option<Effect> {
    let n: i32 = l
        .strip_prefix("an opponent gains ")?
        .strip_suffix(" life")?
        .trim()
        .parse()
        .ok()?;
    Some(Effect::Seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        },
        Effect::GainLife {
            who: PlayerRef::ChosenOpponent,
            n: Value::c(n),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "an opponent gains N life", priority: 100, parse: an_opponent_gains_life } }
