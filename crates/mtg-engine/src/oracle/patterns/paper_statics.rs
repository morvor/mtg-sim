//! Static abilities and permissions of old (paper) card wordings:
//!
//! * "Other players can't gain control of [objects]" ([`ObjectAction::ControlGainedByOthers`]);
//! * "As long as ~ is untapped, noncreature artifacts you control can't be enchanted, they
//!   have indestructible, and other players can't gain control of them. This effect doesn't
//!   remove Auras already attached to those artifacts." (Guardian Beast): a list of
//!   predicates about one subject, "they" and "them" being that subject; "can't be
//!   enchanted" with that last sentence stops only new attachments
//!   ([`ObjectAction::NewlyEnchanted`]);
//! * "Enchanted creature can't attack or block, and its activated abilities with {T} in
//!   their costs can't be activated." (Serra Bestiary; see `kw::tap_abilities_prohibited`);
//! * "Until end of turn, ~ can be the target of spells and abilities controlled by target
//!   player as though it didn't have shroud." (Autumn Willow;
//!   [`Restriction::TargetableAsThoughNo`]);
//! * "You may play any number of lands on each of your turns." (Fastbond, CR 305.2;
//!   [`ANY_NUMBER_OF_LANDS`]).

use super::{AbilityPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::patterns::statics::whole_object_phrase;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn static_ability(e: StaticEffect, cond: Option<Condition>, text: &str) -> Ability {
    let mut s = StaticAbility::new(e);
    s.condition = cond;
    AbilityDef::new(AbilityKind::Static(s), text)
}

fn control_prohibition(what: Filter) -> StaticEffect {
    StaticEffect::Restriction(Restriction::CantBe {
        what,
        action: ObjectAction::ControlGainedByOthers,
    })
}

/// "Other players can't gain control of [objects]".
fn other_players_cant_gain_control(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let what = end(l.trim()).strip_prefix("other players can't gain control of ")?;
    let f = if what == "~" {
        Filter::Source
    } else {
        whole_object_phrase(what)?.0
    };
    Some(vec![static_ability(control_prohibition(f), None, text)])
}

inventory::submit! { StaticPattern { name: "paper statics: other players can't gain control of [objects]", priority: 100, parse: other_players_cant_gain_control } }

/// "[as long as C, ]SUBJECT P1, they P2, and other players can't gain control of them.[
/// This effect doesn't remove Auras already attached to those PERMANENTS.]".
fn predicate_list(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.trim().to_lowercase();
    let l = end(&lower);
    let (first, keep_auras) = match l.split_once(". this effect doesn't remove auras already attached to ") {
        Some((a, b)) => {
            if !matches!(b, "those artifacts" | "those permanents" | "those creatures" | "them" | "it") {
                return None;
            }
            (a, true)
        }
        None => (l, false),
    };
    if first.contains(". ") {
        return None;
    }
    let (cond_text, rest) = match first.strip_prefix("as long as ") {
        Some(r) => {
            let (c, rest) = r.split_once(", ")?;
            (Some(c), rest)
        }
        None => (None, first),
    };
    let parts: Vec<&str> = rest.split(", ").collect();
    let [head, middle @ .., last] = parts.as_slice() else {
        return None;
    };
    let last = last.strip_prefix("and ")?;
    // The subject: the words of the first part before its verb.
    let verb_at = head.find(" can't ").or_else(|| head.find(" have "))?;
    let subject = &head[..verb_at];
    let (filter, plural) = whole_object_phrase(subject)?;
    if !plural {
        return None;
    }
    let cond = match cond_text {
        Some(c) => Some(
            super::statics_conditions::parse_static_condition(c, Some(&Sel::This), ctx)?.0,
        ),
        None => None,
    };
    let mut out = Vec::new();
    let mut control = false;
    let mut enchanted = false;
    let all = std::iter::once(*head).chain(middle.iter().copied()).chain([last]);
    for (i, p) in all.enumerate() {
        if p == "other players can't gain control of them" {
            out.push(static_ability(control_prohibition(filter.clone()), cond.clone(), block));
            control = true;
            continue;
        }
        let sentence = match p.strip_prefix("they ") {
            Some(r) => format!("{subject} {r}"),
            None if i == 0 => p.to_string(),
            None => return None,
        };
        let parsed = crate::oracle::statics::parse_static(&format!("{sentence}."), ctx)?;
        for a in parsed {
            let AbilityKind::Static(s) = &a.kind else {
                return None;
            };
            if s.condition.is_some() || s.zone != FunctionZone::Battlefield {
                return None;
            }
            let mut s = s.clone();
            if let StaticEffect::Restriction(Restriction::CantBe { action, .. }) = &mut s.effect {
                if *action == ObjectAction::Enchanted {
                    enchanted = true;
                    if keep_auras {
                        *action = ObjectAction::NewlyEnchanted;
                    }
                }
            }
            s.condition = cond.clone();
            out.push(AbilityDef::new(AbilityKind::Static(s), block));
        }
    }
    // A list this pattern reads, not the core's; the last sentence needs "can't be enchanted".
    (control && (enchanted || !keep_auras)).then_some(out)
}

inventory::submit! { AbilityPattern { name: "paper statics: [subject] P1, they P2, and other players can't gain control of them", priority: 100, parse: predicate_list } }

/// "Enchanted creature can't attack or block, and its activated abilities with {T} in
/// their costs can't be activated."
fn its_tap_abilities(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let head = end(l.trim()).strip_suffix(
        ", and its activated abilities with {t} in their costs can't be activated",
    )?;
    let (subj, pred) = head.split_once(" can't ")?;
    let f = match subj {
        "~" => Filter::Source,
        "enchanted creature" | "enchanted permanent" | "equipped creature" => {
            whole_object_phrase(subj)?.0
        }
        _ => return None,
    };
    let restriction = match pred {
        "attack or block" => Restriction::CantAttackOrBlock(f.clone()),
        "attack" => Restriction::CantAttack(f.clone()),
        "block" => Restriction::CantBlock(f.clone()),
        _ => return None,
    };
    Some(vec![
        static_ability(StaticEffect::Restriction(restriction), None, text),
        static_ability(crate::kw::tap_abilities_prohibited::effect(&f)?, None, text),
    ])
}

inventory::submit! { StaticPattern { name: "paper statics: its activated abilities with {T} in their costs can't be activated", priority: 100, parse: its_tap_abilities } }

/// "until end of turn, ~ can be the target of spells and abilities controlled by target
/// player as though it didn't have shroud".
fn targetable_as_though_no(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("until end of turn, ")?;
    let (subject, r) = r.split_once(" can be the target of spells and abilities controlled by ")?;
    let (player, keyword) = if let Some(p) = r.strip_suffix(" as though it didn't have shroud") {
        (p, crate::keywords::KeywordKind::Shroud)
    } else {
        (
            r.strip_suffix(" as though it didn't have hexproof")?,
            crate::keywords::KeywordKind::Hexproof,
        )
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let (what, tail) = object_ref(subject, b)?;
    if !end(&tail).is_empty() || !(matches!(what, Sel::This) && subject == "~" || matches!(what, Sel::Target(_))) {
        restore(b);
        return None;
    }
    let by = match player {
        "you" => PlayerFilter::You,
        "target player" | "target opponent" => {
            let pf = if player == "target player" {
                PlayerFilter::Any
            } else {
                PlayerFilter::Opponent
            };
            let slot = b.add_target(TargetSpec::player(pf, player), player);
            PlayerFilter::Ref(Box::new(PlayerRef::Target(slot)))
        }
        _ => {
            restore(b);
            return None;
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::TargetableAsThoughNo {
            what: Filter::In(Box::new(what)),
            by,
            keyword,
        },
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "paper statics: until end of turn, ~ can be the target of spells and abilities controlled by [player] as though it didn't have shroud", priority: 100, parse: targetable_as_though_no } }

/// "You may play any number of lands on each of your turns."
fn any_number_of_lands(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l.trim()) != "you may play any number of lands on each of your turns" {
        return None;
    }
    Some(vec![static_ability(
        StaticEffect::AdditionalLandPlays(PlayerRel::You, ANY_NUMBER_OF_LANDS),
        None,
        text,
    )])
}

inventory::submit! { StaticPattern { name: "paper statics: you may play any number of lands on each of your turns", priority: 100, parse: any_number_of_lands } }
