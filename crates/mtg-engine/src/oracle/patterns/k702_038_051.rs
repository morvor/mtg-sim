//! Keyword lines of CR 702.38–702.51 that the generic keyword parser doesn't handle:
//! "Modular—Sunburst" (CR 702.44c), "Affinity for outlaws" (CR 702.41a).

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::keywords::compile_keyword;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::types::CardType;
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

/// "each creature that convoked it" (CR 702.51c).
fn convoked_it() -> Filter {
    Filter::And(vec![
        Filter::Type(CardType::Creature),
        Filter::Custom(SmolStr::new(crate::kw::convoke::CONVOKED_IT)),
    ])
}

/// "put a +1/+1 counter on each creature that convoked it", "put a flying counter on each
/// creature that convoked it" (CR 702.51c).
fn counters_on_convokers(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("put a ").or_else(|| l.strip_prefix("put an "))?;
    let kind = r
        .strip_suffix(" counter on each creature that convoked it")
        .or_else(|| r.strip_suffix(" counter on each creature that convoked ~"))?
        .trim();
    if kind.is_empty() || kind.contains(' ') {
        return None;
    }
    Some(Effect::AddCounters {
        what: Sel::All(convoked_it()),
        kind: SmolStr::new(kind),
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "counters on creatures that convoked it", priority: 100, parse: counters_on_convokers } }

/// "~ enters with two +1/+1 counters on it for each creature that convoked it"
/// (CR 702.51c).
fn enters_with_counters_per_convoker(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.trim_end_matches('.').to_lowercase();
    let r = lower
        .strip_prefix("~ enters with ")?
        .strip_suffix(" +1/+1 counters on it for each creature that convoked it")?;
    let n = crate::oracle::phrases::parse_number(r)
        .filter(|(_, rest)| rest.trim().is_empty())
        .map(|(n, _)| n)?;
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::EnterWithCounters(
            SmolStr::new(crate::types::counters::PLUS1),
            Value::Mul(
                Box::new(n),
                Box::new(Value::Custom(SmolStr::new(
                    crate::kw::convoke::CONVOKED_COUNT,
                ))),
            ),
        ),
        self_replacement: false,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "enters with counters for each creature that convoked it", priority: 100, parse: enters_with_counters_per_convoker } }
