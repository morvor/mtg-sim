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

// ---------------------------------------------------------------------------------------
// Characteristics from what was sacrificed as the permanent entered
// ---------------------------------------------------------------------------------------

std::thread_local! {
    static NOTING_SACRIFICE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// "As ~ enters, sacrifice any number of [permanents].", parsed by the core, with the
/// sacrificed permanents noted for the linked ability that refers to them (CR 607.2a; the
/// notes move to the permanent as it enters, see `linked_notes::carry`).
fn as_enters_sacrifice_noted(sentence: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = sentence.trim().to_lowercase();
    if NOTING_SACRIFICE.with(|n| n.get())
        || !lower.starts_with("as ~ enters, sacrifice any number of ")
        || end(&lower).contains(". ")
    {
        return None;
    }
    NOTING_SACRIFICE.with(|n| n.set(true));
    let parsed = crate::oracle::parse_ability(sentence, ctx);
    NOTING_SACRIFICE.with(|n| n.set(false));
    let mut parsed = parsed?;
    let [a] = parsed.as_mut_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &mut std::sync::Arc::make_mut(a).kind else {
        return None;
    };
    let StaticEffect::Replacement(def) = &mut s.effect else {
        return None;
    };
    let ReplacementAction::AsEnters(e) = &mut def.action else {
        return None;
    };
    let old = std::mem::take(&mut **e);
    **e = Effect::seq(vec![
        old,
        Effect::NoteLinked {
            what: Sel::Var(vars::SACRIFICED),
            replace: true,
        },
    ]);
    Some(parsed)
}

/// Whether the card has an ability that sacrifices permanents as it enters.
fn sacrifices_as_it_enters() -> bool {
    crate::oracle::raw_text()
        .to_lowercase()
        .lines()
        .any(|l| {
            l.contains(" enters, sacrifice any number of ")
                && l.starts_with("as ")
        })
}

/// "As ~ enters, sacrifice any number of untapped Forests." on a card whose
/// characteristic-defining ability counts them ("~'s power and toughness are each equal
/// to the number of Forests sacrificed as it entered", Wood Elemental).
fn as_enters_sacrifice_for_cda(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !crate::oracle::raw_text()
        .to_lowercase()
        .contains(" sacrificed as it entered")
    {
        return None;
    }
    as_enters_sacrifice_noted(block, ctx)
}

inventory::submit! { AbilityPattern { name: "paper statics: as ~ enters, sacrifice any number of [permanents] (noted)", priority: 5, parse: as_enters_sacrifice_for_cda } }

/// "~'s power and toughness are each equal to the number of Forests sacrificed as it
/// entered." (Wood Elemental): a characteristic-defining ability (CR 604.3) counting the
/// permanents its as-enters ability sacrificed (none, outside the battlefield).
fn pt_sacrificed_as_it_entered(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let noun = end(l.trim())
        .strip_prefix("~'s power and toughness are each equal to the number of ")?
        .strip_suffix(" sacrificed as it entered")?;
    if !ctx.power.is_some_and(|p| p.contains('*'))
        || !ctx.toughness.is_some_and(|t| t.contains('*'))
        || !sacrifices_as_it_enters()
    {
        return None;
    }
    let (f, plural) = whole_object_phrase(noun)?;
    if !plural {
        return None;
    }
    let n = Value::CountSel(Box::new(Sel::Matching(Box::new(Sel::LinkedNotedLastKnown), f)));
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::CdaPT(Some(n.clone()), Some(n))],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "paper statics: P/T equal to the number of [permanents] sacrificed as it entered", priority: 100, parse: pt_sacrificed_as_it_entered } }

/// "As ~ enters, sacrifice any number of creatures. ~'s power becomes the total power of
/// those creatures and its toughness becomes their total toughness." (Dracoplasm): the
/// power and toughness it has on the battlefield, set as it enters (CR 614.1c, layer 7b,
/// CR 613.4b), from the creatures as they last existed on the battlefield.
fn as_enters_sacrifice_pt_becomes(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let i = lower.find(". ~'s power becomes the total power of those creatures and its toughness becomes their total toughness")?;
    if !end(&lower[i..]).ends_with("their total toughness") {
        return None;
    }
    let first = &block.trim()[..i + 1];
    if !lower.starts_with("as ~ enters, sacrifice any number of ") || !lower[..i].contains("creatures") {
        return None;
    }
    let mut out = as_enters_sacrifice_noted(first, ctx)?;
    let total = |s: Stat| Value::Aggregate(AggOp::Sum, s, Box::new(Sel::LinkedNotedLastKnown));
    out = out
        .into_iter()
        .map(|a| AbilityDef::with_link(a.kind.clone(), block, a.link))
        .collect();
    out.push(AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::SetPT(
                Some(total(Stat::Power)),
                Some(total(Stat::Toughness)),
            )],
        })),
        block,
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "paper statics: as ~ enters, sacrifice creatures; its P/T becomes their total", priority: 100, parse: as_enters_sacrifice_pt_becomes } }

/// "During your turn, ~'s power and toughness are each equal to 2 plus the number of
/// Swamps your opponents control. During turns other than yours, ~'s power and toughness
/// are each 2." (Angry Mob): each sentence sets them only if a condition is met, so
/// neither is a characteristic-defining ability (CR 604.3a(5)); like Gaea's Liege's
/// "As long as ..." (`statics_cda::conditional_cda`), each is a static ability that sets
/// the permanent's power and toughness (layer 7b, CR 613.4b) on the battlefield. Elsewhere
/// the card has its printed 2+* with * as 0 (CR 208.2a).
fn pt_during_your_turn(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("during your turn, ~'s power and toughness are each equal to ")?;
    let (yours, others) = r.split_once(
        ". during turns other than yours, ~'s power and toughness are each ",
    )?;
    if !ctx.power.is_some_and(|p| p.contains('*')) || !ctx.toughness.is_some_and(|t| t.contains('*')) {
        return None;
    }
    let a = super::statics::parse_amount(yours, Some(&Sel::This))?;
    let (b, rest) = crate::oracle::phrases::parse_number(others)?;
    if !rest.trim().is_empty() || b.as_const().is_none() {
        return None;
    }
    let set = |v: Value, cond: Condition| {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::SetPT(Some(v.clone()), Some(v))],
        });
        s.condition = Some(cond);
        AbilityDef::new(AbilityKind::Static(s), block)
    };
    Some(vec![
        set(a, Condition::YourTurn),
        set(b, Condition::Not(Box::new(Condition::YourTurn))),
    ])
}

inventory::submit! { AbilityPattern { name: "paper statics: during your turn, ~'s P/T are each equal to ...; during turns other than yours, ...", priority: 100, parse: pt_during_your_turn } }

std::thread_local! {
    static NAME_AS_TYPE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// "for each other attacking ~" on a card whose name is also one of its creature types
/// (Aurochs: "Whenever ~ attacks, it gets +1/+0 until end of turn for each other attacking
/// Aurochs."): the name is the creature type there (Aurochs' ruling: it counts attacking
/// creatures with the creature type Aurochs, not just those named Aurochs).
fn name_as_creature_type(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if NAME_AS_TYPE.with(|n| n.get()) || !block.contains(" other attacking ~") {
        return None;
    }
    let name = ctx.card_name;
    if crate::types::subtype_kind(name) != Some(crate::types::SubtypeKind::Creature)
        || !ctx.type_line.subtypes.iter().any(|s| s.as_str() == name)
    {
        return None;
    }
    let rewritten = block.replace(" other attacking ~", &format!(" other attacking {name}"));
    NAME_AS_TYPE.with(|n| n.set(true));
    let parsed = crate::oracle::parse_ability(&rewritten, ctx);
    NAME_AS_TYPE.with(|n| n.set(false));
    Some(
        parsed?
            .into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), block, a.link))
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "paper statics: other attacking ~ (the card's name as its creature type)", priority: 100, parse: name_as_creature_type } }

/// "~'s power is equal to the last chosen number and its toughness is equal to 7 minus
/// that number." (Shapeshifter, printed */7-*): a characteristic-defining ability (CR
/// 604.3, 208.2a) from the number chosen for it (CR 607.2d; the last one chosen, as each
/// new choice replaces the old). Outside the battlefield no number was chosen: 0/7.
fn pt_from_chosen_number(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l.trim()).strip_prefix("~'s power is equal to ")?;
    let (p, t) = r.split_once(" and its toughness is equal to ")?;
    if !matches!(p, "the last chosen number" | "the chosen number")
        || !ctx.power.is_some_and(|p| p == "*")
        || !ctx.toughness.is_some_and(|t| t.contains('*'))
    {
        return None;
    }
    let n = t.strip_suffix(" minus that number")?;
    let (n, rest) = crate::oracle::phrases::parse_number(n)?;
    let n = n.as_const()?;
    if !rest.trim().is_empty() || ctx.toughness != Some(format!("{n}-*").as_str()) {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::CdaPT(
            Some(Value::Chosen),
            Some(Value::Diff(Box::new(Value::c(n)), Box::new(Value::Chosen))),
        )],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "paper statics: power equal to the last chosen number, toughness N minus that number", priority: 40, parse: pt_from_chosen_number } }
