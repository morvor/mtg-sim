//! Abilities that refer to a choice made as the permanent entered ("the chosen name",
//! "the chosen type", "the chosen player"), linked by CR 607.2d.

use super::{AbilityPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

/// "sources with the chosen name", "spells with the chosen name", "creature spells of the
/// chosen type": an object phrase referring to a choice, whose head may be "source(s)"
/// (any object).
fn chosen_object_phrase(s: &str) -> Option<Filter> {
    let s = s.trim();
    let probe;
    let s = match s
        .strip_prefix("sources ")
        .or_else(|| s.strip_prefix("source "))
    {
        // "card" parses as a head noun with no type restriction.
        Some(r) => {
            probe = format!("card {r}");
            probe.as_str()
        }
        None => s,
    };
    let (f, _, tail) = parse_object_phrase(s)?;
    if !end(tail).is_empty() || !crate::choices::filter_mentions_choice(&f) {
        return None;
    }
    Some(f)
}

/// "Spells [...] can't be cast" restricts casting cards (and copies) with those
/// characteristics; the restriction is checked before the card becomes a spell, so drop
/// the "is a spell on the stack" part of the filter.
fn castable_card(f: Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(castable_card).collect()),
        other => other,
    }
}

/// Restrictions referring to the chosen name:
/// "Spells with the chosen name can't be cast.",
/// "Your opponents can't cast spells with the chosen name.",
/// "Activated abilities of sources with the chosen name can't be activated [unless
/// they're mana abilities]."
fn chosen_restrictions(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let r = if let Some(what) = l.strip_suffix(" can't be cast") {
        Restriction::CantCast {
            who: PlayerFilter::Any,
            what: castable_card(chosen_object_phrase(what)?),
        }
    } else if let Some(what) = l.strip_prefix("your opponents can't cast ") {
        Restriction::CantCast {
            who: PlayerFilter::Opponent,
            what: castable_card(chosen_object_phrase(what)?),
        }
    } else if let Some(r) = l.strip_prefix("activated abilities of ") {
        let (srcs, include_mana) =
            if let Some(x) = r.strip_suffix(" can't be activated unless they're mana abilities") {
                (x, false)
            } else if let Some(x) = r.strip_suffix(" can't be activated") {
                (x, true)
            } else {
                return None;
            };
        Restriction::CantActivate {
            who: PlayerFilter::Any,
            sources: chosen_object_phrase(srcs)?,
            include_mana,
        }
    } else {
        return None;
    };
    Some(vec![static_ability(StaticEffect::Restriction(r), block)])
}

/// Cost modifiers with a chosen-value suffix: "Spells you cast of the chosen type cost
/// {1} less to cast.", "Creature spells you cast of the chosen type cost {1} less to
/// cast.", "Spells of the chosen type cost {1} more to cast." (CR 601.2f).
fn chosen_cost_modifier(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let (spells, rest) = l.split_once(" cost ")?;
    // Move "you cast"/"your opponents cast" out of the middle of the phrase.
    let (who, spells) = if let Some((a, b)) = spells.split_once(" you cast ") {
        (PlayerRel::You, format!("{a} {b}"))
    } else if let Some((a, b)) = spells.split_once(" your opponents cast ") {
        (PlayerRel::Opponent, format!("{a} {b}"))
    } else {
        (PlayerRel::Any, spells.to_string())
    };
    let filter = chosen_object_phrase(&spells)?;
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.strip_prefix('{')?.parse().ok()?;
    let change = match end(tail) {
        "more to cast" => CostChange::IncreaseGeneric(Value::c(n)),
        "less to cast" => CostChange::ReduceGeneric(Value::c(n)),
        _ => return None,
    };
    Some(vec![static_ability(
        StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(filter),
            who,
            change,
        }),
        block,
    )])
}

/// "you cast a spell of the chosen color", "an opponent casts a spell with the chosen
/// name": cast triggers whose spell phrase has suffixes after "spell".
fn cast_spell_suffix_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (who, rest) = if let Some(x) = r.strip_prefix("you cast ") {
        (PlayerRel::You, x)
    } else if let Some(x) = r.strip_prefix("an opponent casts ") {
        (PlayerRel::Opponent, x)
    } else if let Some(x) = r.strip_prefix("a player casts ") {
        (PlayerRel::Any, x)
    } else {
        return None;
    };
    let rest = rest
        .strip_prefix("a ")
        .or_else(|| rest.strip_prefix("an "))?;
    if !rest.contains("spell") {
        return None;
    }
    let f = chosen_object_phrase(rest)?;
    Some((
        TriggerCond::CastSpell { who, filter: f },
        Sel::TriggerSpell,
        PlayerRef::TriggerPlayer,
    ))
}

/// "Enchanted creature has protection from the chosen color. This effect doesn't remove
/// ~." (CR 702.16n): the granted protection doesn't make this Aura fall off.
fn doesnt_remove_self(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let suffix = ". this effect doesn't remove ~";
    let lower = block.to_lowercase();
    let l = end(&lower);
    if !l.ends_with(suffix) {
        return None;
    }
    let head = &block[..l.len() - suffix.len()];
    let mut marked = false;
    let mut out = Vec::new();
    for a in crate::oracle::statics::parse_static(head, ctx)? {
        let AbilityKind::Static(st) = &a.kind else {
            return None;
        };
        let mut st = st.clone();
        if let StaticEffect::Continuous { mods, .. } = &mut st.effect {
            for m in mods.iter_mut() {
                if let Modification::AddKeyword(k) = m {
                    if k.kind == crate::keywords::KeywordKind::Protection {
                        k.text = Some(crate::choices::DOESNT_REMOVE_SOURCE.into());
                        marked = true;
                    }
                }
            }
        }
        out.push(AbilityDef::new(AbilityKind::Static(st), block));
    }
    marked.then_some(out)
}

/// The subject of a clause: "~", "it", "target creature you control", "another target
/// creature you control", "any number of target creatures", "enchanted creature".
fn subject(s: &str, b: &mut Builder) -> Option<Sel> {
    let s = s.trim();
    match s {
        "~" => return Some(Sel::This),
        "it" | "that creature" => return Some(b.it.clone()),
        "enchanted creature" | "equipped creature" => return Some(Sel::AttachedTo),
        _ => {}
    }
    let (spec, tail) = parse_target(s)?;
    // "target spell or permanent" would be restricted to spells by the target phrase
    // parser; leave such mixed targets unsupported.
    let ok = match &spec.what {
        TargetKind::Object(_) => true,
        TargetKind::Spell(f) => !matches!(f, Filter::Or(_)),
        _ => false,
    };
    if !end(tail).is_empty() || !ok {
        return None;
    }
    Some(Sel::Target(b.add_target(spec, s)))
}

/// "[X] gains protection from the color of your choice until end of turn", "[X] becomes
/// the color of your choice until end of turn": a color is chosen as the effect resolves
/// and locked into the effect (CR 608.2h).
fn color_of_your_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let (l, duration) = match l.strip_suffix(" until end of turn") {
        Some(x) => (x, Duration::EndOfTurn),
        None => (l, Duration::Permanent),
    };
    let protection = |f: Filter| {
        let mut k = crate::keywords::Keyword::new(crate::keywords::KeywordKind::Protection);
        k.filter = Some(f);
        Modification::AddKeyword(k)
    };
    let forms: [(&str, ChoiceKind, Vec<Modification>); 5] = [
        (
            "protection from the color of your choice",
            ChoiceKind::Color,
            vec![protection(Filter::ChosenColor)],
        ),
        (
            "protection from the card type of your choice",
            ChoiceKind::CardType,
            vec![protection(Filter::ChosenCardType)],
        ),
        (
            "the color of your choice",
            ChoiceKind::Color,
            vec![Modification::SetChosenColor],
        ),
        // CR 205.1a: the new creature type replaces its other creature types.
        (
            "the creature type of your choice",
            ChoiceKind::CreatureType,
            vec![
                Modification::RemoveAllCreatureTypes,
                Modification::AddChosenType,
            ],
        ),
        // CR 305.7.
        (
            "the basic land type of your choice",
            ChoiceKind::BasicLandType,
            vec![Modification::SetChosenBasicLandType],
        ),
    ];
    let mut found = None;
    for (phrase, kind, mods) in forms {
        let is_protection = phrase.starts_with("protection");
        let verbs: &[&str] = if is_protection {
            &[" gains ", " gain "]
        } else {
            &[" becomes ", " become "]
        };
        for v in verbs {
            if let Some(x) = l.strip_suffix(&format!("{v}{phrase}")) {
                found = Some((x, kind.clone(), mods.clone()));
                break;
            }
        }
        if found.is_some() {
            break;
        }
    }
    let (subj, kind, mods) = match found {
        Some(f) => f,
        None => {
            // "~ becomes the chosen color" (a choice made earlier in the ability).
            let x = l
                .strip_suffix(" becomes the chosen color")
                .or_else(|| l.strip_suffix(" become the chosen color"))?;
            let what = subject(x, b)?;
            return Some(Effect::Modify {
                what,
                mods: vec![Modification::SetChosenColor],
                duration,
            });
        }
    };
    let what = subject(subj, b)?;
    Some(Effect::seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind,
        },
        Effect::Modify {
            what,
            mods,
            duration,
        },
    ]))
}

/// "All Slivers have protection from the chosen color", "All creatures of the chosen type
/// get -1/-1": "all [objects] ..." in a static ability means the same as "[objects] ...".
fn all_objects_static(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let rest = block
        .strip_prefix("All ")
        .or_else(|| block.strip_prefix("all "))?;
    // Only statics about objects ("All creatures get ...", "All Slivers have ...").
    let lower = rest.to_lowercase();
    let (_, plural, tail) = parse_object_phrase(&lower)?;
    let tail = tail.trim_start();
    if !plural
        || !(tail.starts_with("get ")
            || tail.starts_with("have ")
            || tail.starts_with("gain ")
            || tail.starts_with("are "))
    {
        return None;
    }
    let abilities = crate::oracle::statics::parse_static(rest, ctx)?;
    Some(
        abilities
            .into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), block))
            .collect(),
    )
}

inventory::submit! {
    AbilityPattern { name: "chosen name restrictions", priority: 50, parse: chosen_restrictions }
}
inventory::submit! {
    AbilityPattern { name: "all [objects] statics", priority: 60, parse: all_objects_static }
}
inventory::submit! {
    EffectPattern { name: "color of your choice", priority: 100, parse: color_of_your_choice }
}
inventory::submit! {
    AbilityPattern { name: "this effect doesn't remove ~", priority: 50, parse: doesnt_remove_self }
}
inventory::submit! {
    AbilityPattern { name: "chosen type cost modifiers", priority: 50, parse: chosen_cost_modifier }
}
inventory::submit! {
    TriggerPattern { name: "cast a spell of the chosen ...", priority: 100, parse: cast_spell_suffix_trigger }
}
