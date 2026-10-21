//! Choices among listed values made as an instruction is performed:
//!
//! - "[object] gains your choice of flying, vigilance, deathtouch, or haste until end of
//!   turn", "... both gain your choice of first strike or lifelink until end of turn"
//!   (Argivian Avenger, Jodah's Avenger, Ballroom Brawlers): the controller chooses one of
//!   the listed abilities as the instruction is performed (CR 608.2d), and the
//!   instruction gives that one.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// The listed options of "A, B, or C" / "A or B".
fn options(s: &str) -> Option<Vec<String>> {
    let v: Vec<String> = if let Some((a, b)) = s.split_once(", or ") {
        let mut v: Vec<String> = a.split(", ").map(str::to_string).collect();
        v.push(b.to_string());
        v
    } else {
        let (a, b) = s.split_once(" or ")?;
        vec![a.to_string(), b.to_string()]
    };
    (v.len() >= 2 && v.iter().all(|o| !o.is_empty())).then_some(v)
}

/// "... gains your choice of A, B, or C [until end of turn]".
fn gains_your_choice_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (verb, at) = ["gains your choice of ", "gain your choice of "]
        .iter()
        .find_map(|v| l.find(v).map(|i| (*v, i)))?;
    let head = &l[..at];
    let r = &l[at + verb.len()..];
    let (list, tail) = match r.find(" until ") {
        Some(i) => (&r[..i], &r[i..]),
        None => (r, ""),
    };
    let opts = options(list)?;
    let plain_verb = verb.trim_end_matches("your choice of ");
    let first = b.targets.len();
    let mut out = Vec::new();
    for o in &opts {
        b.targets.truncate(first);
        let text = format!("{head}{plain_verb}{o}{tail}");
        let e = parse_clause(&text, b)?;
        // Each option grants only what it names: a keyword ability.
        out.push((o.clone(), e));
    }
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options: out,
    })
}

inventory::submit! { EffectPattern { name: "gains your choice of [abilities]", priority: 60, parse: gains_your_choice_of } }

/// "As ~ becomes attached to a creature, choose a color." (Sanctuary Blade): the choice is
/// made as the Equipment becomes attached, each time it does (`choices.rs`,
/// [`crate::choices::AS_ATTACHED_CHOOSE_COLOR`]).
fn as_attached_choose(l: &str, text: &str, _ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "as ~ becomes attached to a creature, choose a color" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            crate::choices::AS_ATTACHED_CHOOSE_COLOR.into(),
        ))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "as ~ becomes attached to a creature, choose a color", priority: 60, parse: as_attached_choose } }

/// "the last chosen color" in a static ability: the color chosen most recently for the
/// permanent, which is the one its later abilities see ("Equipped creature gets +2/+0 and
/// has protection from the last chosen color", "Prevent all damage that would be dealt to
/// enchanted creature by sources of the last chosen color"; CR 607.2d).
fn last_chosen_color(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if !l.contains("the last chosen color") {
        return None;
    }
    let plain = text.replace("the last chosen color", "the chosen color");
    let mut v = crate::oracle::statics::parse_static(&plain, ctx)?;
    for a in &mut v {
        std::sync::Arc::make_mut(a).text = text.to_string();
    }
    Some(v)
}

inventory::submit! { super::StaticPattern { name: "the last chosen color", priority: 60, parse: last_chosen_color } }

const BASIC_TYPES: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

/// "choose two basic land types", "choose two abilities from among first strike,
/// vigilance, and lifelink": that many different ones, in order.
fn choose_several(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (n, r) = parse_number(r)?;
    let Value::Const(n @ 2..=5) = n else {
        return None;
    };
    let r = r.trim_start();
    let words: Vec<String> = if r == "basic land types" {
        BASIC_TYPES.iter().map(|s| s.to_string()).collect()
    } else {
        let list = r.strip_prefix("abilities from among ")?;
        let list = list.replace(", and ", ", ").replace(" and ", ", ");
        let v: Vec<String> = list.split(", ").map(|s| s.trim().to_string()).collect();
        if v.len() <= n as usize
            || v
                .iter()
                .any(|k| crate::keywords::KeywordKind::from_name(k).is_none())
        {
            return None;
        }
        v
    };
    Some(Effect::Choose {
        who: PlayerRef::You,
        kind: ChoiceKind::Distinct(n as u8, words),
    })
}

inventory::submit! { EffectPattern { name: "choose two [values]", priority: 60, parse: choose_several } }

/// "Basic lands of the first chosen type are the second chosen type."
fn first_type_are_second(l: &str, text: &str, _ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let subj = end(l).strip_suffix(" of the first chosen type are the second chosen type")?;
    let (f, true, tail) = parse_object_phrase(subj)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let effect = StaticEffect::Continuous {
        affected: Filter::and(vec![
            f,
            Filter::Custom(crate::choices::FIRST_CHOSEN_TYPE.into()),
        ]),
        mods: vec![Modification::Custom {
            name: crate::choices::SECOND_CHOSEN_TYPE.into(),
            layer: Layer::L4Type,
        }],
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "[lands] of the first chosen type are the second chosen type", priority: 60, parse: first_type_are_second } }

/// "Each land of the first chosen type becomes the second chosen type until end of turn."
fn first_type_becomes_second(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each ")?;
    let (noun, rest) = r.split_once(" of the first chosen type becomes the second chosen type")?;
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    if end(rest) != "until end of turn" {
        return None;
    }
    let duration = Duration::EndOfTurn;
    Some(Effect::Modify {
        what: Sel::All(Filter::and(vec![
            f,
            Filter::Custom(crate::choices::FIRST_CHOSEN_TYPE.into()),
        ])),
        mods: vec![Modification::Custom {
            name: crate::choices::SECOND_CHOSEN_TYPE.into(),
            layer: Layer::L4Type,
        }],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "each [land] of the first chosen type becomes the second chosen type", priority: 60, parse: first_type_becomes_second } }

/// "Humans you control have each of the chosen abilities."
fn have_each_chosen_ability(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if !end(l).ends_with(" have each of the chosen abilities") {
        return None;
    }
    let probe = text.replace("each of the chosen abilities", "flying");
    let v = crate::oracle::statics::parse_static(&probe, ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let StaticEffect::Continuous { affected, mods } = &s.effect else {
        return None;
    };
    if !matches!(mods.as_slice(), [Modification::AddKeyword(k)] if k.kind == crate::keywords::KeywordKind::Flying)
    {
        return None;
    }
    let mut st = s.clone();
    st.effect = StaticEffect::Continuous {
        affected: affected.clone(),
        mods: vec![Modification::Custom {
            name: crate::choices::CHOSEN_ABILITIES.into(),
            layer: Layer::L6Ability,
        }],
    };
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { super::StaticPattern { name: "have each of the chosen abilities", priority: 60, parse: have_each_chosen_ability } }
