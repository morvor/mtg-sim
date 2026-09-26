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

use super::{
    AbilityPattern, ConditionPattern, EffectPattern, FollowupPattern, StaticPattern,
    TriggerPattern,
};
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::Builder;
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

/// An ability of a card with cleave with text in square brackets (CR 702.148a): compiled
/// with the bracketed words in it; the ability keeps the bracketed text, from which the
/// cleaved ability (the words removed) is compiled when the spell is cast for its cleave
/// cost (see `kw/cleave.rs`). Both versions must be understood.
fn cleave_brackets(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if !t.contains('[') || !ctx.keywords.iter().any(|k| k == "Cleave") {
        return None;
    }
    let full = crate::kw::cleave::without_brackets(t);
    let cleaved = crate::kw::cleave::cleaved(t);
    if !cleaved.is_empty() {
        crate::oracle::parse_ability(&cleaved, ctx)?;
    }
    let abilities = crate::oracle::parse_ability(&full, ctx)?;
    Some(
        abilities
            .into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), t))
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "cleave: bracketed text", priority: 50, parse: cleave_brackets } }

/// "Counter target spell that wasn't cast from its owner's hand." (Wash Away): a copy of a
/// spell was never cast, so it can be countered (Wash Away rulings).
fn counter_spell_not_cast_from_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let text = "target spell that wasn't cast from its owner's hand";
    if end(l).strip_prefix("counter ")? != text {
        return None;
    }
    let spec = TargetSpec::object(
        Filter::And(vec![
            Filter::Spell,
            Filter::Not(Box::new(Filter::CastFrom(ZoneKind::Hand))),
        ]),
        text,
    );
    let slot = b.add_target(spec, text);
    Some(Effect::CounterSpell {
        what: Sel::Target(slot),
    })
}

inventory::submit! { EffectPattern { name: "counter target spell that wasn't cast from its owner's hand", priority: 100, parse: counter_spell_not_cast_from_hand } }

/// "Your maximum hand size is reduced by three for the rest of the game." (Inspired Idea):
/// an effect lasting for the rest of the game (CR 402.2).
fn hand_size_for_rest_of_game(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("your maximum hand size is ")?
        .strip_suffix(" for the rest of the game")?;
    let (sign, n) = if let Some(n) = r.strip_prefix("reduced by ") {
        (-1, n)
    } else {
        (1, r.strip_prefix("increased by ")?)
    };
    let (n, tail) = crate::oracle::phrases::parse_number(n)?;
    let Value::Const(n) = n else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::HandSizeDelta(sign * n),
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "your maximum hand size is reduced by N for the rest of the game", priority: 100, parse: hand_size_for_rest_of_game } }

/// "X is the number of creatures you control." after a sentence using X (Lantern Flare):
/// the value of X for the rest of the resolution.
fn x_is(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(s).strip_prefix("x is ") else {
        return false;
    };
    let Some((v, rest)) = crate::oracle::statics::parse_value_phrase(r, b) else {
        return false;
    };
    if !end(&rest).is_empty() || !format!("{prev:?}").contains("X") {
        return false;
    }
    let p = std::mem::take(prev);
    *prev = Effect::Seq(vec![Effect::SetX { value: v }, p]);
    true
}

inventory::submit! { FollowupPattern { name: "x is [value]", priority: 100, apply: x_is } }

/// "Whenever you foretell a card" (Dream Devourer; CR 702.143c).
fn you_foretell(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (r == "you foretell a card").then(|| {
        (
            TriggerCond::Custom(crate::kw::foretell::YOU_FORETELL.into()),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "you foretell a card", priority: 100, parse: you_foretell } }

/// "if this spell was foretold" (Poison the Cup; CR 702.143c).
fn was_foretold(l: &str) -> Option<Condition> {
    matches!(end(l), "~ was foretold" | "this spell was foretold")
        .then(|| Condition::Custom(crate::kw::foretell::WAS_FORETOLD.into()))
}

inventory::submit! { ConditionPattern { name: "~ was foretold", priority: 100, parse: was_foretold } }

/// "exile it face down" (The Foretold Soldier), "exile a card from your hand face down"
/// (Ethereal Valkyrie): face-down exiled cards (CR 406.3), "it" afterward.
fn exile_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let what = match end(l).strip_prefix("exile ")?.strip_suffix(" face down")? {
        "it" => b.it.clone(),
        "~" => Sel::This,
        "a card from your hand" => Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::And(vec![
                Filter::Card,
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::You),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        _ => return None,
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what,
        face_down: true,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "exile it / a card from your hand face down", priority: 100, parse: exile_face_down } }

/// "It becomes foretold." after exiling a card face down (The Foretold Soldier, Ethereal
/// Valkyrie; CR 702.143d).
fn becomes_foretold(l: &str, _b: &mut Builder) -> Option<Effect> {
    matches!(end(l), "it becomes foretold" | "they become foretold")
        .then(|| Effect::Custom(crate::kw::foretell::BECOMES_FORETOLD.into()))
}

inventory::submit! { EffectPattern { name: "it becomes foretold", priority: 100, parse: becomes_foretold } }

/// "Its foretell cost is its mana cost reduced by {2}." after "It becomes foretold."
/// (Ethereal Valkyrie; CR 702.143d): the effect gives the card a foretell cost.
fn foretell_cost_reduced(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(n) = end(s)
        .strip_prefix("its foretell cost is its mana cost reduced by {")
        .and_then(|r| r.strip_suffix('}'))
        .and_then(|n| n.parse::<u32>().ok())
    else {
        return false;
    };
    fn set(e: &mut Effect, n: u32) -> bool {
        match e {
            Effect::Custom(c) if c.as_str() == crate::kw::foretell::BECOMES_FORETOLD => {
                *c = format!("{}{n}", crate::kw::foretell::BECOMES_FORETOLD_REDUCED).into();
                true
            }
            Effect::Seq(v) => v.last_mut().is_some_and(|l| set(l, n)),
            _ => false,
        }
    }
    set(prev, n)
}

inventory::submit! { FollowupPattern { name: "its foretell cost is its mana cost reduced by {N}", priority: 100, apply: foretell_cost_reduced } }

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
