//! Oracle text of the keywords of CR 702.140–702.152 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Boast — [cost]: [effect]" (CR 702.142a), "Whenever you activate a boast ability",
//!   "Boast abilities you activate cost {1} less to activate [for each ...]", "Creatures
//!   you control can boast twice during each of your turns rather than once"
//!   (CR 702.142b);
//! * "Whenever ~ trains" (CR 702.149c), and "if you control a creature with a +1/+1
//!   counter on it that attacked this turn" (Warrior's Resolve);
//! * "You may cast ~ from your graveyard using its blitz ability" (CR 702.152a) and
//!   "... using its mutate ability" (CR 702.140a).

use super::{AbilityPattern, ConditionPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "Boast — [cost]: [effect]" (CR 702.142a): the activated ability, which can be activated
/// only if the creature attacked this turn and only once each turn (see `kw/boast.rs`).
fn boast(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest = t.strip_prefix("Boast — ")?;
    let abilities = crate::oracle::parse_ability(rest, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    let act = crate::kw::boast::boast_ability(act.clone());
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), t)])
}

inventory::submit! { AbilityPattern { name: "boast", priority: 100, parse: boast } }

/// "Whenever you activate a boast ability" (Frenzied Raider; CR 702.142b).
fn boast_activated(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (r == "you activate a boast ability").then(|| {
        (
            TriggerCond::Custom(crate::kw::boast::BOAST_ACTIVATED.into()),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "you activate a boast ability", priority: 100, parse: boast_activated } }

/// "Boast abilities you activate cost {1} less to activate [for each Dragon you control]"
/// (Dragonkin Berserker; CR 702.142b).
fn boast_cost_modifier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rest = l.strip_prefix("boast abilities you activate cost {")?;
    let (n, rest) = rest.split_once('}')?;
    let n: i32 = n.parse().ok()?;
    let rest = rest.strip_prefix(" less to activate")?;
    let amount = match end(rest).trim() {
        "" => Value::c(n),
        r => {
            let each = crate::oracle::patterns::statics::parse_for_each(
                r.strip_prefix("for each ")?,
                None,
            )?;
            if n == 1 {
                each
            } else {
                Value::Mul(Box::new(Value::c(n)), Box::new(each))
            }
        }
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Keyword(KeywordKind::Boast),
                who: PlayerRel::You,
                change: CostChange::ReduceGeneric(amount),
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "boast abilities you activate cost less", priority: 100, parse: boast_cost_modifier } }

/// "Creatures you control can boast twice during each of your turns rather than once."
/// (Birgi, God of Storytelling; CR 702.142b).
fn boast_twice(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    (end(l) == "creatures you control can boast twice during each of your turns rather than once")
        .then(|| {
            vec![AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
                    crate::kw::boast::BOAST_TWICE.into(),
                ))),
                text,
            )]
        })
}

inventory::submit! { StaticPattern { name: "creatures you control can boast twice", priority: 100, parse: boast_twice } }

/// "Creature spells you cast have demonstrate." (Silverquill Lecturer), "Artifact spells
/// you cast have demonstrate." (The Sixth Seraph): the spells have demonstrate as they're
/// cast, so it triggers (CR 702.144a).
fn spells_have_demonstrate(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_suffix(" have demonstrate")?;
    let subject = match r {
        "spells you cast" => None,
        _ => Some(r.strip_suffix(" spells you cast")?),
    };
    let mut parts: Vec<Filter> = Vec::new();
    if let Some(subject) = subject {
        let mut alts = Vec::new();
        for word in subject.split(" and ") {
            let phrase = format!("{word} card");
            let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(&phrase)?;
            if !end(tail).is_empty() {
                return None;
            }
            alts.push(f);
        }
        parts.push(if alts.len() == 1 {
            alts.pop()?
        } else {
            Filter::Or(alts)
        });
    }
    parts.push(Filter::Spell);
    parts.push(Filter::ControlledBy(PlayerRel::You));
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(parts),
        mods: vec![Modification::AddKeyword(
            crate::keywords::Keyword::new(KeywordKind::Demonstrate).text("demonstrate"),
        )],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "[quality] spells you cast have demonstrate", priority: 100, parse: spells_have_demonstrate } }

/// "Whenever ~ trains" (Savior of Ollenbock; CR 702.149c).
fn trains(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (r == "~ trains").then(|| {
        (
            TriggerCond::Custom(crate::kw::training::TRAINS_SELF.into()),
            Sel::This,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "~ trains", priority: 100, parse: trains } }

/// "if you control a creature with a +1/+1 counter on it that attacked this turn"
/// (Warrior's Resolve, which gives creatures training).
fn creature_with_counter_that_attacked(l: &str) -> Option<Condition> {
    (end(l) == "you control a creature with a +1/+1 counter on it that attacked this turn").then(
        || {
            Condition::Exists(Filter::And(vec![
                Filter::Type(crate::types::CardType::Creature),
                Filter::ControlledBy(PlayerRel::You),
                Filter::HasCounter(Some(crate::types::counters::PLUS1.into())),
                Filter::AttackedThisTurn,
            ]))
        },
    )
}

inventory::submit! { ConditionPattern { name: "you control a creature with a +1/+1 counter on it that attacked this turn", priority: 100, parse: creature_with_counter_that_attacked } }

/// "You may cast ~ from your graveyard using its blitz ability." (Tenacious Underdog),
/// "... using its mutate ability." (Brokkos, Apex of Forever): a static ability
/// functioning in the graveyard that lets its owner cast it from there, only for that
/// keyword's cost (see `kw/blitz.rs`, `kw/mutate.rs`).
fn cast_from_graveyard_using(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let kw = end(l)
        .strip_prefix("you may cast ~ from your graveyard using its ")?
        .strip_suffix(" ability")?;
    let kind = match kw {
        "blitz" => KeywordKind::Blitz,
        "mutate" => KeywordKind::Mutate,
        _ => return None,
    };
    let mut s = StaticAbility::new(StaticEffect::Custom(
        crate::kw::cast_using_from_graveyard::permission_name(kind).into(),
    ));
    s.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "you may cast ~ from your graveyard using its [keyword] ability", priority: 100, parse: cast_from_graveyard_using } }
