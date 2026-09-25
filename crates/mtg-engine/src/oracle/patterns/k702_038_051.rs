//! Keyword lines of CR 702.38–702.51 that the generic keyword parser doesn't handle:
//! "Modular—Sunburst" (CR 702.44c), "Affinity for outlaws" (CR 702.41a).

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::keywords::compile_keyword;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern, StaticPattern, TriggerPattern};
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
    // CR 702.49d: "Commander ninjutsu [cost]".
    if let Some(r) = lower.strip_prefix("commander ninjutsu") {
        let cost = crate::oracle::keywords::parse_keyword_cost(&t[t.len() - r.len()..])?;
        let kw = Keyword::with_cost(KeywordKind::Ninjutsu, cost).text(t);
        return Some(compile_keyword(kw, t));
    }
    // CR 702.48a: "[Quality] offering" ("Fox offering", "Artifact offering").
    if let Some(q) = lower.strip_suffix(" offering") {
        if !q.contains(' ') {
            let f = crate::oracle::keywords::quality_phrase(q)?;
            let kw = Keyword::with_filter(KeywordKind::Offering, f).text(t);
            return Some(compile_keyword(kw, t));
        }
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

/// "Ninjutsu abilities you activate cost {1} less to activate." (CR 702.49a: the
/// abilities ninjutsu stands for are ninjutsu abilities.)
fn ninjutsu_cost_reduction(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.trim_end_matches('.').to_lowercase();
    let r = lower
        .strip_prefix("ninjutsu abilities you activate cost {")?
        .strip_suffix("} less to activate")?;
    let n: i32 = r.parse().ok()?;
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Keyword(KeywordKind::Ninjutsu),
        who: PlayerRel::You,
        change: CostChange::ReduceGeneric(Value::c(n)),
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "ninjutsu abilities cost less", priority: 100, parse: ninjutsu_cost_reduction } }

/// "[Quality] spells you cast have [keyword]" for keywords of spells in CR 702.38–51
/// ("Artifact spells you cast have convoke", "Instant and sorcery spells you cast have
/// storm", "Spells you cast have affinity for artifacts"): a static ability affecting
/// spells on the stack, which have the keyword as they're cast.
fn spells_you_cast_have(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, kw_text) = if let Some(r) = l.strip_prefix("spells you cast have ") {
        ("", r)
    } else {
        l.split_once(" spells you cast have ")?
    };
    let kws = spell_keywords(kw_text, ctx, false)?;
    let mut parts = spell_quality(subject)?;
    parts.push(Filter::Spell);
    parts.push(Filter::ControlledBy(PlayerRel::You));
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(parts),
        mods: kws.into_iter().map(Modification::AddKeyword).collect(),
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spells you cast have [convoke/storm/affinity]", priority: 100, parse: spells_you_cast_have } }

/// Keywords of CR 702.38–51 that can be given to spells: convoke, storm, affinity (and
/// sunburst for the next spell, CR 702.44a, which works as the spell's permanent enters).
fn spell_keywords(s: &str, ctx: &CompileContext, sunburst: bool) -> Option<Vec<Keyword>> {
    let mut kws = Vec::new();
    for a in crate::oracle::keywords::parse_keyword_line(s, ctx)? {
        match &a.kind {
            AbilityKind::Keyword(k)
                if matches!(
                    k.kind,
                    KeywordKind::Convoke | KeywordKind::Storm | KeywordKind::Affinity
                ) || (sunburst && k.kind == KeywordKind::Sunburst) =>
            {
                kws.push(k.clone())
            }
            _ => return None,
        }
    }
    (!kws.is_empty()).then_some(kws)
}

/// "The next [quality] spell you cast this turn has [keyword]" (CR 611.2f): the spell has
/// it as it's cast, so a cost-changing keyword (convoke, affinity) applies to its cost.
fn next_spell_has(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("the next ")?;
    let (subject, kw_text) = match r.strip_prefix("spell you cast this turn has ") {
        Some(k) => ("", k),
        None => r.split_once(" spell you cast this turn has ")?,
    };
    let kws = spell_keywords(kw_text, b.ctx, true)?;
    let mut parts = spell_quality(subject)?;
    parts.push(Filter::Spell);
    Some(Effect::NextSpell {
        filter: Filter::And(parts),
        mods: kws.into_iter().map(Modification::AddKeyword).collect(),
        expires: Duration::EndOfTurn,
    })
}

/// "When you next cast [a quality] spell this turn, that spell gains [keyword]" (Solar
/// Array): a delayed triggered ability (CR 603.7) that triggers once, when such a spell
/// becomes cast (CR 601.2i) — including a spell whose costs were being paid as it was
/// created — and gives the spell the keyword as it resolves.
fn when_you_next_cast_gains(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("when you next cast ")?;
    let r = r
        .strip_prefix("an ")
        .or_else(|| r.strip_prefix("a "))
        .unwrap_or(r);
    let (subject, kw_text) = r.split_once(" spell this turn, that spell gains ")?;
    let subject = if subject == "spell" { "" } else { subject };
    let kws = spell_keywords(kw_text, b.ctx, true)?;
    let mut parts = spell_quality(subject)?;
    parts.push(Filter::Spell);
    Some(Effect::DelayedTrigger {
        // "This turn": it ends with the turn (CR 603.7b).
        trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::And(parts),
        })),
        body: Box::new(Body::effect(Effect::Modify {
            what: Sel::TriggerObject,
            mods: kws.into_iter().map(Modification::AddKeyword).collect(),
            duration: Duration::Permanent,
        })),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "when you next cast a spell this turn, that spell gains [keyword]", priority: 100, parse: when_you_next_cast_gains } }

inventory::submit! { EffectPattern { name: "the next spell you cast this turn has [keyword]", priority: 100, parse: next_spell_has } }

/// The qualities of "[quality] spells": "instant and sorcery" (either type), "artifact
/// creature" (both), "noncreature", "legendary creature", "multicolored", "red".
fn spell_quality(subject: &str) -> Option<Vec<Filter>> {
    // "Instant and sorcery" is either type; "artifact creature" is both.
    let words: Vec<&str> = subject.split_whitespace().collect();
    let either = words.iter().any(|w| *w == "and" || *w == "or");
    let mut types = Vec::new();
    let mut adjectives = Vec::new();
    for w in words {
        if w == "and" || w == "or" {
            continue;
        }
        if let Some(c) = crate::types::Color::from_word(w) {
            adjectives.push(Filter::Color(c));
            continue;
        }
        let adjective = match w {
            "legendary" => Some(Filter::Supertype(crate::types::Supertype::Legendary)),
            "multicolored" => Some(Filter::Multicolored),
            "monocolored" => Some(Filter::Monocolored),
            "colorless" => Some(Filter::Colorless),
            _ => w
                .strip_prefix("non")
                .and_then(CardType::from_word)
                .map(|t| Filter::not(Filter::Type(t))),
        };
        if let Some(f) = adjective {
            adjectives.push(f);
            continue;
        }
        let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(w)?;
        if !crate::oracle::phrases::end(tail).is_empty() {
            return None;
        }
        match f {
            Filter::Type(_) => types.push(f),
            other => adjectives.push(other),
        }
    }
    let mut parts = adjectives;
    if either && types.len() > 1 {
        parts.push(Filter::Or(types));
    } else {
        parts.extend(types);
    }
    Some(parts)
}

/// "That creature gains bushido 1 and becomes a Samurai in addition to its other creature
/// types" (Sensei Golden-Tail): an effect with no duration lasts indefinitely (CR 611.2a).
fn gains_keyword_and_becomes_type(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l
        .strip_prefix("that creature gains ")
        .or_else(|| l.strip_prefix("it gains "))?;
    let (kw_text, rest) = r.split_once(" and becomes a ")?;
    let subtype = rest.strip_suffix(" in addition to its other creature types")?;
    if subtype.contains(' ') {
        return None;
    }
    let mut mods = Vec::new();
    for a in crate::oracle::keywords::parse_keyword_line(kw_text, b.ctx)? {
        match &a.kind {
            AbilityKind::Keyword(k) if k.kind == KeywordKind::Bushido => {
                mods.push(Modification::AddKeyword(k.clone()))
            }
            _ => return None,
        }
    }
    let mut word = subtype.chars();
    let first = word.next()?.to_uppercase().collect::<String>();
    mods.push(Modification::AddSubtypes(vec![SmolStr::new(format!(
        "{first}{}",
        word.as_str()
    ))]));
    Some(Effect::Modify {
        what: b.it.clone(),
        mods,
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "gains bushido and becomes a creature type", priority: 100, parse: gains_keyword_and_becomes_type } }

/// "Each creature card in your hand has ninjutsu {1}{U}{B}" (CR 702.49a: ninjutsu
/// functions while the card is in a hand).
fn cards_in_hand_have_ninjutsu(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("each ")?;
    let (subject, _) = r.split_once(" card in your hand has ninjutsu ")?;
    let t = text.trim().trim_end_matches('.');
    let kw_text = &t[t.to_lowercase().find(" has ninjutsu ")? + " has ".len()..];
    let mut kws = Vec::new();
    for a in crate::oracle::keywords::parse_keyword_line(kw_text, ctx)? {
        match &a.kind {
            AbilityKind::Keyword(k) if k.kind == KeywordKind::Ninjutsu => kws.push(k.clone()),
            _ => return None,
        }
    }
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(subject)?;
    if !crate::oracle::phrases::end(tail).is_empty() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(vec![
            f,
            Filter::Card,
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        mods: kws.into_iter().map(Modification::AddKeyword).collect(),
    });
    s.zone = FunctionZone::Battlefield;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "cards in your hand have ninjutsu", priority: 100, parse: cards_in_hand_have_ninjutsu } }

/// "Each other Samurai creature you control gets +1/+1 for each point of bushido it has."
fn per_point_of_bushido(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("each other ")?;
    let (subject, rest) = r.split_once(" creature you control gets +")?;
    let (a, b) = rest
        .strip_suffix(" for each point of bushido it has")?
        .split_once("/+")?;
    let (a, b): (i32, i32) = (a.parse().ok()?, b.parse().ok()?);
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(subject)?;
    if !crate::oracle::phrases::end(tail).is_empty() {
        return None;
    }
    let points = || Value::Custom(SmolStr::new(crate::kw::bushido::BUSHIDO_POINTS));
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(vec![
            f,
            Filter::Type(CardType::Creature),
            Filter::ControlledBy(PlayerRel::You),
            Filter::Other,
        ]),
        mods: vec![Modification::ModifyPT(
            Value::Mul(Box::new(Value::c(a)), Box::new(points())),
            Value::Mul(Box::new(Value::c(b)), Box::new(points())),
        )],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "gets +1/+1 for each point of bushido it has", priority: 100, parse: per_point_of_bushido } }

/// "Whenever you cast a spell that has convoke", "Whenever you cast another spell that
/// has convoke".
fn cast_spell_with_keyword(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let rest = r.strip_prefix("you cast ")?;
    let (other, rest) = match rest.strip_prefix("another ") {
        Some(x) => (true, x),
        None => (false, rest.strip_prefix("a ")?),
    };
    let kw = rest.strip_prefix("spell that has ")?;
    let kind = match kw {
        "convoke" => KeywordKind::Convoke,
        "storm" => KeywordKind::Storm,
        "affinity" => KeywordKind::Affinity,
        "splice" => KeywordKind::Splice,
        "entwine" => KeywordKind::Entwine,
        _ => return None,
    };
    let mut parts = vec![Filter::HasKeyword(kind)];
    if other {
        parts.push(Filter::Other);
    }
    Some((
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::And(parts),
        },
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you cast a spell that has [keyword]", priority: 100, parse: cast_spell_with_keyword } }

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
