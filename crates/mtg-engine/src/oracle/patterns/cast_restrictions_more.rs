//! Restrictions on casting spells and playing lands (CR 101.2, 601.3, 305.2):
//!
//! * "Players can cast spells only during their own turns." (Dosan the Falling Leaf),
//!   "Players can cast spells and activate abilities only during their own turns." (City of
//!   Solitude; mana abilities too, and abilities of cards in any zone);
//! * "You can't play lands or cast spells from your hand." (Experimental Frenzy), "Players
//!   can't cast spells from graveyards or activate abilities of cards in graveyards."
//!   (Ashes of the Abhorrent);
//! * "Each player can't cast more than one noncreature spell each turn." (Deafening
//!   Silence), "Each player who has cast a nonartifact spell this turn can't cast
//!   additional nonartifact spells." (Ethersworn Canonist): counting the spells cast this
//!   turn before the ability began to apply;
//! * "Each opponent who controls more creatures than you can't cast creature spells. The
//!   same is true for artifacts and enchantments." / "Each opponent who controls more
//!   lands than you can't play lands." (Ward of Bones);
//! * "You can't cast ~ unless an opponent lost life this turn." (Rakdos, Lord of Riots),
//!   "You can't cast ~ if you've played a land this turn." — a restriction on casting the
//!   card that functions wherever it could be cast from (CR 601.3).

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::statics::parse_condition;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn stat(e: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(e)), text)
}

/// "[quality] spells" as a filter of the cards cast ("noncreature spells", "non-Phyrexian
/// spells", "creature spells").
fn spells(s: &str) -> Option<Filter> {
    let q = s
        .strip_suffix(" spells")
        .or_else(|| s.strip_suffix(" spell"))?;
    let phrase = format!("{q} cards");
    let (f, _, tail) = parse_object_phrase(&phrase)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    Some(f)
}

fn restrictions(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l).trim();
    let not_their_turn = PlayerFilter::Not(Box::new(PlayerFilter::Active));
    let r = |r: Restriction| stat(StaticEffect::Restriction(r), text);
    match l {
        "players can cast spells only during their own turns" => {
            return Some(vec![r(Restriction::CantCast {
                who: not_their_turn,
                what: Filter::Any,
            })]);
        }
        "players can cast spells and activate abilities only during their own turns" => {
            return Some(vec![
                r(Restriction::CantCast {
                    who: not_their_turn.clone(),
                    what: Filter::Any,
                }),
                r(Restriction::CantActivate {
                    who: not_their_turn,
                    sources: Filter::Any,
                    include_mana: true,
                }),
            ]);
        }
        "you can't play lands or cast spells from your hand" => {
            return Some(vec![
                r(Restriction::CantCast {
                    who: PlayerFilter::You,
                    what: Filter::InZone(ZoneKind::Hand),
                }),
                r(Restriction::CantPlayLandCards {
                    who: PlayerFilter::You,
                    what: Filter::InZone(ZoneKind::Hand),
                }),
            ]);
        }
        "players can't cast spells from graveyards or activate abilities of cards in graveyards" => {
            return Some(vec![
                r(Restriction::CantCast {
                    who: PlayerFilter::Any,
                    what: Filter::InZone(ZoneKind::Graveyard),
                }),
                r(Restriction::CantActivate {
                    who: PlayerFilter::Any,
                    sources: Filter::InZone(ZoneKind::Graveyard),
                    include_mana: true,
                }),
            ]);
        }
        _ => {}
    }
    // "each player can't cast more than one noncreature spell each turn"
    if let Some(q) = l
        .strip_prefix("each player can't cast more than one ")
        .and_then(|r| r.strip_suffix(" spell each turn"))
    {
        let what = spells(&format!("{q} spells"))?;
        return Some(vec![r(Restriction::MaxSpellsOfKindPerTurn {
            who: PlayerFilter::Any,
            what,
            n: 1,
        })]);
    }
    // "each player who has cast a nonartifact spell this turn can't cast additional
    // nonartifact spells"
    if let Some((a, b)) = l
        .strip_prefix("each player who has cast a ")
        .and_then(|r| r.split_once(" this turn can't cast additional "))
    {
        let what = spells(a)?;
        if format!("{:?}", spells(b)?) != format!("{what:?}") {
            return None;
        }
        return Some(vec![r(Restriction::MaxSpellsOfKindPerTurn {
            who: PlayerFilter::Any,
            what,
            n: 1,
        })]);
    }
    // "each opponent who controls more creatures than you can't cast creature spells. the
    // same is true for artifacts and enchantments."
    if let Some(rest) = l.strip_prefix("each opponent who controls more ") {
        let (kind, rest) = rest.split_once(" than you can't ")?;
        let (head, also) = match rest.split_once(". the same is true for ") {
            Some((h, a)) => (h, Some(a)),
            None => (rest, None),
        };
        let t = CardType::from_word(kind.strip_suffix('s')?)?;
        let mut kinds = vec![t];
        if let Some(a) = also {
            for w in a.split(" and ").flat_map(|p| p.split(", ")) {
                kinds.push(CardType::from_word(w.trim().strip_suffix('s')?)?);
            }
        }
        let lands = match head {
            "play lands" if t == CardType::Land && also.is_none() => true,
            _ => {
                if head != format!("cast {} spells", t.word()) {
                    return None;
                }
                false
            }
        };
        let who = |t: CardType| {
            PlayerFilter::And(vec![
                PlayerFilter::Opponent,
                PlayerFilter::Controls(
                    Box::new(Filter::Type(t)),
                    Cmp::Gt,
                    Box::new(Value::Count(Filter::and(vec![
                        Filter::Type(t),
                        Filter::ControlledBy(PlayerRel::You),
                    ]))),
                ),
            ])
        };
        return Some(
            kinds
                .into_iter()
                .map(|t| {
                    if lands {
                        r(Restriction::CantPlayLands(who(t)))
                    } else {
                        r(Restriction::CantCast {
                            who: who(t),
                            what: Filter::Type(t),
                        })
                    }
                })
                .collect(),
        );
    }
    None
}

/// "You can't cast ~ unless [condition]." / "You can't cast ~ if [condition]." (on
/// permanents and on instants and sorceries alike).
fn cant_cast_self(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let lower = crate::oracle::strip_ability_word(block.trim()).to_lowercase();
    let l = end(lower.trim());
    let r = l.strip_prefix("you can't cast ~ ")?;
    let condition = if let Some(c) = r.strip_prefix("unless ") {
        Condition::Not(Box::new(parse_condition(c, ctx)?))
    } else {
        parse_condition(r.strip_prefix("if ")?, ctx)?
    };
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantCast {
        who: PlayerFilter::Any,
        what: Filter::Source,
    }));
    s.zone = FunctionZone::Anywhere;
    s.condition = Some(condition);
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block.trim())])
}

inventory::submit! { StaticPattern { name: "cast restrictions: only during their own turns, from zones, one of a kind each turn, more than you", priority: 120, parse: restrictions } }
inventory::submit! { AbilityPattern { name: "cast restrictions: you can't cast ~ unless/if", priority: 120, parse: cant_cast_self } }
