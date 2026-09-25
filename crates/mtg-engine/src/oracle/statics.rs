//! Parsing static abilities (CR 604), conditions, and value phrases.

use super::effects::{parse_pt_mod, Builder};
use super::phrases::*;
use super::CompileContext;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::types::*;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

fn keyword_list_mods(s: &str) -> Option<Vec<Modification>> {
    // A quoted keyword ability (`has "cumulative upkeep {1}."`) grants that keyword.
    let parts = match end(s)
        .strip_prefix('"')
        .and_then(|r| r.strip_suffix('"'))
        .filter(|r| !r.contains('"'))
    {
        Some(inner) => vec![inner.trim_end_matches('.').to_string()],
        None => super::keywords::split_keyword_phrases(end(s)),
    };
    let mut out = Vec::new();
    for p in &parts {
        let tl = TypeLine::default();
        let ctx = CompileContext {
            card_name: "",
            full_name: "",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        for a in super::keywords::parse_keyword_line(p, &ctx)? {
            if let AbilityKind::Keyword(k) = &a.kind {
                out.push(Modification::AddKeyword(k.clone()));
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Parses a static ability line on a permanent.
pub fn parse_static(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = text.to_lowercase();
    let l = end(&lower);
    // "As long as [condition], [static]". Lines the built-in forms don't understand fall
    // through whole to the pluggable static patterns.
    if let Some(r) = l.strip_prefix("as long as ") {
        let parsed = r.split_once(", ").and_then(|(c, rest)| {
            let cond = parse_condition(c, ctx)?;
            let mut abilities = parse_static_inner(rest, text, ctx)?;
            for a in abilities.iter_mut() {
                if let AbilityKind::Static(s) = &a.kind {
                    let mut s2 = s.clone();
                    s2.condition = Some(cond.clone());
                    *a = AbilityDef::new(AbilityKind::Static(s2), text);
                }
            }
            Some(abilities)
        });
        if parsed.is_some() {
            return parsed;
        }
    }
    parse_static_inner(l, text, ctx)
}

fn parse_static_inner(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    // ETB replacements.
    if l == "~ enters tapped" || l == "~ enters the battlefield tapped" {
        return Some(vec![static_ability(
            StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::EnterTapped,
                self_replacement: false,
                optional: false,
            }),
            text,
        )]);
    }
    if let Some(r) = l
        .strip_prefix("~ enters with ")
        .or_else(|| l.strip_prefix("~ enters the battlefield with "))
    {
        let (n, r2) = parse_number(r)?;
        let (kind, r3) = super::costs::counter_kind(r2)?;
        let r3 = strip(r3, "counters").or_else(|| strip(r3, "counter"))?;
        if end(r3) != "on it" {
            return None;
        }
        return Some(vec![static_ability(
            StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::EnterWithCounters(kind, n),
                self_replacement: false,
                optional: false,
            }),
            text,
        )]);
    }
    // Self restrictions.
    let self_restr: [(&str, Restriction); 9] = [
        ("~ can't block", Restriction::CantBlock(Filter::Source)),
        ("~ can't attack", Restriction::CantAttack(Filter::Source)),
        (
            "~ can't attack or block",
            Restriction::CantAttackOrBlock(Filter::Source),
        ),
        (
            "~ can't be blocked",
            Restriction::CantBeBlocked(Filter::Source),
        ),
        (
            "~ attacks each combat if able",
            Restriction::MustAttack(Filter::Source),
        ),
        (
            "~ blocks each combat if able",
            Restriction::MustBlock(Filter::Source),
        ),
        (
            "~ can't be countered",
            Restriction::CantBeCountered(Filter::Source),
        ),
        (
            "~ doesn't untap during your untap step",
            Restriction::DoesntUntap(Filter::Source),
        ),
        (
            "~ can block only creatures with flying",
            Restriction::CanBlockOnly {
                blocker: Filter::Source,
                attackers: Filter::HasKeyword(KeywordKind::Flying),
            },
        ),
    ];
    for (p, r) in self_restr {
        if l == p {
            let mut s = StaticAbility::new(StaticEffect::Restriction(r));
            if p == "~ can't be countered" {
                s.zone = FunctionZone::Stack;
            }
            return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
        }
    }
    if let Some((f, _, _)) = l
        .strip_prefix("~ can't be blocked by ")
        .and_then(parse_object_phrase)
        .filter(|(_, _, tail)| end(tail).is_empty())
    {
        return Some(vec![static_ability(
            StaticEffect::Restriction(Restriction::CantBeBlockedBy {
                attacker: Filter::Source,
                blocker: f,
            }),
            text,
        )]);
    }
    if l == "~ can't be blocked except by two or more creatures" {
        return Some(vec![static_ability(
            StaticEffect::Restriction(Restriction::MinBlockers {
                attacker: Filter::Source,
                n: 2,
            }),
            text,
        )]);
    }
    if l == "~ can block an additional creature each combat" {
        return Some(vec![static_ability(
            StaticEffect::Restriction(Restriction::ExtraBlocks {
                blocker: Filter::Source,
                n: Some(1),
            }),
            text,
        )]);
    }
    if l == "~ can block any number of creatures" {
        return Some(vec![static_ability(
            StaticEffect::Restriction(Restriction::ExtraBlocks {
                blocker: Filter::Source,
                n: None,
            }),
            text,
        )]);
    }
    // Enchanted/equipped creature statics.
    for (prefix, affected) in [
        ("enchanted creature ", Filter::AttachedToSource),
        ("equipped creature ", Filter::AttachedToSource),
        ("enchanted permanent ", Filter::AttachedToSource),
        ("enchanted land ", Filter::AttachedToSource),
        ("enchanted artifact ", Filter::AttachedToSource),
    ] {
        if let Some(r) = l.strip_prefix(prefix) {
            if let Some(v) = anthem(r, affected, text).or_else(|| attached_restriction(r, text)) {
                return Some(v);
            }
            break;
        }
    }
    // "[filter] get +N/+N [and have ...]" / "[filter] have [keywords]"
    // (Spells on the stack are left to the registry's patterns, which grant only the
    // keywords the engine applies to a spell.)
    if let Some((f, _, rest)) =
        parse_object_phrase(l).filter(|(f, _, _)| f.zone() != Some(ZoneKind::Stack))
    {
        let rest = rest.trim();
        if rest.starts_with("get ")
            || rest.starts_with("gets ")
            || rest.starts_with("have ")
            || rest.starts_with("has ")
        {
            if let Some(v) = anthem(rest, f, text) {
                return Some(v);
            }
        }
    }
    if let Some(r) = l
        .strip_prefix("~ gets ")
        .map(|r| format!("gets {r}"))
        .or_else(|| l.strip_prefix("~ has ").map(|r| format!("has {r}")))
    {
        if let Some(v) = anthem(&r, Filter::Source, text) {
            return Some(v);
        }
    }
    // Cost modifiers.
    if let Some(a) = parse_cost_modifier(l, text) {
        return Some(vec![a]);
    }
    // Player effects.
    let player_pairs: [(&str, StaticEffect); 6] = [
        (
            "you have hexproof",
            StaticEffect::PlayerEffect {
                affected: PlayerFilter::You,
                effect: PlayerModification::Hexproof,
            },
        ),
        (
            "you have shroud",
            StaticEffect::PlayerEffect {
                affected: PlayerFilter::You,
                effect: PlayerModification::Shroud,
            },
        ),
        (
            "you have no maximum hand size",
            StaticEffect::PlayerEffect {
                affected: PlayerFilter::You,
                effect: PlayerModification::MaxHandSize(None),
            },
        ),
        (
            "you may play an additional land on each of your turns",
            StaticEffect::AdditionalLandPlays(PlayerRel::You, 1),
        ),
        (
            "players can't gain life",
            StaticEffect::Restriction(Restriction::CantGainLife(PlayerFilter::Any)),
        ),
        (
            "your opponents can't gain life",
            StaticEffect::Restriction(Restriction::CantGainLife(PlayerFilter::Opponent)),
        ),
    ];
    for (p, e) in player_pairs {
        if l == p {
            return Some(vec![static_ability(e, text)]);
        }
    }
    if l == "you can't lose the game and your opponents can't win the game" {
        return Some(vec![
            static_ability(
                StaticEffect::Restriction(Restriction::CantLoseGame(PlayerFilter::You)),
                text,
            ),
            static_ability(
                StaticEffect::Restriction(Restriction::CantWinGame(PlayerFilter::Opponent)),
                text,
            ),
        ]);
    }
    if let Some(r) = l.strip_prefix("you may look at the top card of your library any time") {
        if end(r).is_empty() {
            return Some(vec![static_ability(
                StaticEffect::LookAtTopCard(PlayerRel::You),
                text,
            )]);
        }
    }
    // CDA: "~'s power and toughness are each equal to [value]".
    // (Phrases this doesn't understand fall through to the pattern registry.)
    if let Some(v) = l
        .strip_prefix("~'s power and toughness are each equal to ")
        .and_then(|r| parse_value_phrase(r, &mut Builder::new(ctx)))
        .and_then(|(v, tail)| end(&tail).is_empty().then_some(v))
    {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::CdaPT(Some(v.clone()), Some(v))],
        });
        s.is_cda = true;
        s.zone = FunctionZone::Anywhere;
        return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
    }
    if let Some(v) = l
        .strip_prefix("~'s power is equal to ")
        .and_then(|r| parse_value_phrase(r, &mut Builder::new(ctx)))
        .and_then(|(v, tail)| end(&tail).is_empty().then_some(v))
    {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::CdaPT(Some(v), None)],
        });
        s.is_cda = true;
        s.zone = FunctionZone::Anywhere;
        return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
    }
    crate::oracle_ext::parse_static_ext(l, text, ctx)
}

/// "gets +1/+1", "get +1/+1 and have flying", "have flying and haste", "has trample".
fn anthem(r: &str, affected: Filter, text: &str) -> Option<Vec<Ability>> {
    let r = end(r);
    let mut mods = Vec::new();
    let rest = if let Some(x) = r.strip_prefix("gets ").or_else(|| r.strip_prefix("get ")) {
        let (p, t, tail) = parse_pt_mod(x)?;
        mods.push(Modification::ModifyPT(p, t));
        tail.trim().to_string()
    } else {
        r.to_string()
    };
    let rest = rest.trim();
    if !rest.is_empty() {
        let k = rest
            .strip_prefix("and have ")
            .or_else(|| rest.strip_prefix("and has "))
            .or_else(|| rest.strip_prefix("have "))
            .or_else(|| rest.strip_prefix("has "))?;
        mods.extend(keyword_list_mods(k)?);
    }
    if mods.is_empty() {
        return None;
    }
    Some(vec![static_ability(
        StaticEffect::Continuous { affected, mods },
        text,
    )])
}

fn attached_restriction(r: &str, text: &str) -> Option<Vec<Ability>> {
    let r = end(r);
    let f = Filter::AttachedToSource;
    let restr = match r {
        "can't attack or block" => Restriction::CantAttackOrBlock(f),
        "can't block" => Restriction::CantBlock(f),
        "can't attack" => Restriction::CantAttack(f),
        "doesn't untap during its controller's untap step" => Restriction::DoesntUntap(f),
        _ => return None,
    };
    Some(vec![static_ability(StaticEffect::Restriction(restr), text)])
}

/// "spells your opponents cast cost {1} more to cast", "creature spells you cast cost {1} less to cast".
fn parse_cost_modifier(l: &str, text: &str) -> Option<Ability> {
    let (spells, rest) = l.split_once(" cost ")?;
    let (who, spells) = if let Some(s) = spells.strip_suffix(" you cast") {
        (PlayerRel::You, s)
    } else if let Some(s) = spells.strip_suffix(" your opponents cast") {
        (PlayerRel::Opponent, s)
    } else if let Some(s) = spells.strip_suffix(" cast") {
        (PlayerRel::Any, s)
    } else {
        (PlayerRel::Any, spells)
    };
    let filter = if spells == "spells" || spells == "noncreature spells" && false {
        Filter::Any
    } else {
        let (f, _, tail) = parse_object_phrase(spells)?;
        if !end(tail).is_empty() {
            return None;
        }
        f
    };
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.trim_start_matches('{').parse().ok()?;
    let change = match end(tail) {
        "more to cast" => CostChange::IncreaseGeneric(Value::c(n)),
        "less to cast" => CostChange::ReduceGeneric(Value::c(n)),
        _ => return None,
    };
    Some(static_ability(
        StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(filter),
            who,
            change,
        }),
        text,
    ))
}

/// Statics that appear on instants/sorceries: additional costs, "can't be countered".
pub fn parse_spell_static(text: &str, _ctx: &CompileContext) -> Option<Ability> {
    let lower = text.to_lowercase();
    let l = end(&lower);
    if l == "~ can't be countered" {
        let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantBeCountered(
            Filter::Source,
        )));
        s.zone = FunctionZone::Stack;
        return Some(AbilityDef::new(AbilityKind::Static(s), text));
    }
    if let Some(r) = l.strip_prefix("as an additional cost to cast ~, ") {
        let (c, _) = super::costs::parse_cost(r)?;
        let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            who: PlayerRel::You,
            change: CostChange::AdditionalCost(c),
        }));
        s.zone = FunctionZone::Anywhere;
        return Some(AbilityDef::new(AbilityKind::Static(s), text));
    }
    None
}

/// Conditions: "you control an artifact", "you have 10 or less life", "it's your turn".
/// Falls back to the pluggable [`crate::oracle::patterns::ConditionPattern`]s when the
/// built-in phrases don't match.
pub fn parse_condition(c: &str, ctx: &CompileContext) -> Option<Condition> {
    parse_condition_core(c, ctx).or_else(|| crate::oracle_ext::parse_condition_ext(end(c)))
}

fn parse_condition_core(c: &str, _ctx: &CompileContext) -> Option<Condition> {
    let c = end(c);
    match c {
        "it's your turn" => return Some(Condition::YourTurn),
        "it's not your turn" => return Some(Condition::NotYourTurn),
        "you're the monarch" => return Some(Condition::IsMonarch),
        "you have the city's blessing" => return Some(Condition::CitysBlessing),
        "it's night" => return Some(Condition::IsNight),
        "it's day" => return Some(Condition::IsDay),
        "~ was kicked" | "it was kicked" => return Some(Condition::CostPaid("kicker".into())),
        "you cast it" | "you cast ~" => return Some(Condition::WasCast),
        _ => {}
    }
    if let Some(r) = c.strip_prefix("you control ") {
        let r2 = r
            .strip_prefix("a ")
            .or_else(|| r.strip_prefix("an "))
            .unwrap_or(r);
        if let Some((n, rest)) = parse_number(r) {
            // "you control three or more artifacts"
            if let Some(rest2) = strip(rest, "or more") {
                let (f, _, tail) = parse_object_phrase(rest2)?;
                if !end(tail).is_empty() {
                    return None;
                }
                return Some(Condition::Compare(
                    Value::Count(f.you_control()),
                    Cmp::Ge,
                    n,
                ));
            }
        }
        let (f, _, tail) = parse_object_phrase(r2)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Exists(f.you_control()));
    }
    if let Some(r) = c
        .strip_prefix("you have ")
        .filter(|r| parse_number(r).is_some())
    {
        let (n, rest) = parse_number(r)?;
        let rest = end(rest);
        let cmp = if let Some(x) = rest
            .strip_suffix(" or less life")
            .or_else(|| rest.strip_prefix("or less life"))
        {
            let _ = x;
            Cmp::Le
        } else if rest == "or more life" {
            Cmp::Ge
        } else if rest == "or more cards in hand" {
            return Some(Condition::Compare(
                Value::HandSize(PlayerRef::You),
                Cmp::Ge,
                n,
            ));
        } else if rest == "or more cards in your graveyard" {
            return Some(Condition::Compare(
                Value::GraveyardSize(PlayerRef::You),
                Cmp::Ge,
                n,
            ));
        } else if rest == "or fewer cards in hand" {
            return Some(Condition::Compare(
                Value::HandSize(PlayerRef::You),
                Cmp::Le,
                n,
            ));
        } else {
            return None;
        };
        return Some(Condition::Compare(Value::LifeTotal(PlayerRef::You), cmp, n));
    }
    if c == "you have no cards in hand" {
        return Some(Condition::Compare(
            Value::HandSize(PlayerRef::You),
            Cmp::Eq,
            Value::c(0),
        ));
    }
    None
}

/// Value phrases: "the number of creatures you control", "its power", "X", "twice X".
pub fn parse_value_phrase(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("the number of ") {
        // "the number of cards in your hand"
        if let Some(rest) = r.strip_prefix("cards in your hand") {
            return Some((Value::HandSize(PlayerRef::You), rest.to_string()));
        }
        if let Some(rest) = r.strip_prefix("cards in your graveyard") {
            return Some((Value::GraveyardSize(PlayerRef::You), rest.to_string()));
        }
        if let Some(rest) = r.strip_prefix("creature cards in your graveyard") {
            return Some((
                Value::CardsInGraveyard(PlayerRef::You, Filter::creature()),
                rest.to_string(),
            ));
        }
        let (f, _, rest) = parse_object_phrase(r)?;
        return Some((Value::Count(f), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("the sacrificed ") {
        return sacrificed_value(r);
    }
    if let Some(r) = s.strip_prefix("the greatest power among ") {
        let (f, _, rest) = parse_object_phrase(r)?;
        return Some((Value::GreatestPower(f), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("the greatest mana value among ") {
        let (f, _, rest) = parse_object_phrase(r)?;
        return Some((Value::GreatestManaValue(f), rest.to_string()));
    }
    for (p, v) in [
        ("its power", Value::PowerOf(Box::new(b.it.clone()))),
        ("its toughness", Value::ToughnessOf(Box::new(b.it.clone()))),
        ("~'s power", Value::PowerOf(Box::new(Sel::This))),
        ("~'s toughness", Value::ToughnessOf(Box::new(Sel::This))),
        ("its mana value", Value::ManaValueOf(Box::new(b.it.clone()))),
        ("that much", Value::EventAmount),
        ("the damage dealt this way", Value::Prev),
        ("your life total", Value::LifeTotal(PlayerRef::You)),
        ("x", Value::X),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            return Some((v, rest.to_string()));
        }
    }
    let (n, rest) = parse_number(s)?;
    Some((n, rest.to_string()))
}

/// "[the sacrificed] creature's power", "artifact's mana value": a characteristic of the
/// permanent sacrificed to pay the cost (its last known information).
fn sacrificed_value(r: &str) -> Option<(Value, String)> {
    let (noun, r) = r.split_once("'s ")?;
    if !matches!(
        noun,
        "creature" | "artifact" | "permanent" | "land" | "enchantment"
    ) {
        return None;
    }
    let what = Box::new(Sel::Var(vars::SACRIFICED));
    for (p, v) in [
        ("power", Value::PowerOf(what.clone())),
        ("toughness", Value::ToughnessOf(what.clone())),
        ("mana value", Value::ManaValueOf(what.clone())),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with([' ', ',', '.']) {
                return Some((v, rest.to_string()));
            }
        }
    }
    None
}

/// "*/*" P/T with a CDA line that the compiler didn't catch: nothing to add by default.
pub fn star_pt_cda(_norm: &str, _ctx: &CompileContext) -> Option<Ability> {
    None
}
