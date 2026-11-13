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

/// "choose a nonland card name and a number greater than 0", "choose a number greater than
/// 0 and a color": two choices, made in order.
fn choose_two_kinds(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (a, c) = r.split_once(" and ")?;
    let one = |s: &str, b: &mut Builder| -> Option<Effect> {
        if let Some(n) = s.strip_prefix("a number greater than ") {
            let (Value::Const(n), rest) = parse_number(n)? else {
                return None;
            };
            if !end(rest).is_empty() {
                return None;
            }
            return Some(Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Number {
                    min: n + 1,
                    max: crate::choices::MAX_CHOSEN_NUMBER,
                },
            });
        }
        match parse_clause(&format!("choose {s}"), b)? {
            e @ Effect::Choose { .. } => Some(e),
            _ => None,
        }
    };
    if !a.contains("number greater than") && !c.contains("number greater than") {
        return None;
    }
    Some(Effect::seq(vec![one(a, b)?, one(c, b)?]))
}

inventory::submit! { EffectPattern { name: "choose a number greater than N and a [value]", priority: 60, parse: choose_two_kinds } }

/// "with mana value equal to that number" / "... equal to the number" after "Choose a
/// number.": the chosen number.
fn that_number(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !(l.contains("equal to that number") || l.contains("equal to the number")) {
        return None;
    }
    if !crate::oracle::raw_text()
        .to_lowercase()
        .contains("choose a number")
    {
        return None;
    }
    let text = l
        .replace("equal to that number", "equal to the chosen number")
        .replace("equal to the number", "equal to the chosen number");
    parse_clause(&text, b)
}

inventory::submit! { EffectPattern { name: "that number (the chosen number)", priority: 130, parse: that_number } }

/// "If that library contains exactly the chosen number of cards with the chosen name,
/// [instruction]" (Mindblaze), "If that opponent reveals exactly the chosen number of cards
/// of the chosen color, [instruction]" (Scrying Glass): the cards in that player's library
/// or hand with the chosen quality, counted against the chosen number.
fn exactly_the_chosen_number(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if that ")?;
    let (c, rest) = r.split_once(", ")?;
    let (zone, quality) = if let Some(q) = c.strip_prefix("library contains exactly the chosen number of cards ") {
        (ZoneKind::Library, q)
    } else if let Some(q) = c
        .strip_prefix("opponent reveals exactly the chosen number of cards ")
        .or_else(|| c.strip_prefix("player reveals exactly the chosen number of cards "))
    {
        (ZoneKind::Hand, q)
    } else {
        return None;
    };
    let quality = match quality {
        "with the chosen name" => Filter::ChosenName,
        "of the chosen color" => Filter::ChosenColor,
        _ => return None,
    };
    let who = b.it_player.clone();
    if super::oracle_hardening_referents::is_no_player_referent(&who) {
        return None;
    }
    let e = parse_clause(rest, b)?;
    let count = Value::Count(Filter::and(vec![
        quality,
        Filter::Card,
        Filter::InZone(zone),
        Filter::OwnedByPlayer(Box::new(who)),
    ]));
    Some(Effect::If {
        cond: Condition::Compare(count, Cmp::Eq, Value::Chosen),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if that library contains exactly the chosen number of cards ...", priority: 60, parse: exactly_the_chosen_number } }

/// "it's at least one of the chosen colors" (of the triggering spell or object).
fn at_least_one_chosen_color(c: &str) -> Option<Condition> {
    // About the triggering object (it exists, and its colors are counted).
    (end(c) == "it's at least one of the chosen colors").then(|| {
        Condition::And(vec![
            Condition::SelMatches(Sel::TriggerObject, Filter::Any),
            Condition::Compare(
                Value::Custom(crate::choices::CHOSEN_COLORS_IT_IS.into()),
                Cmp::Ge,
                Value::c(1),
            ),
        ])
    })
}

inventory::submit! { super::ConditionPattern { name: "it's at least one of the chosen colors", priority: 60, parse: at_least_one_chosen_color } }

/// "you gain 1 life for each of the chosen colors it is".
fn for_each_chosen_color_it_is(l: &str, b: &mut Builder) -> Option<Effect> {
    let head = end(l).strip_suffix(" life for each of the chosen colors it is")?;
    let (pre, n) = head.rsplit_once(' ')?;
    let (Value::Const(n), rest) = parse_number(n)? else {
        return None;
    };
    if !rest.trim().is_empty() {
        return None;
    }
    let e = parse_clause(&format!("{pre} x life"), b)?;
    Some(Effect::seq(vec![
        Effect::SetX {
            value: Value::Mul(
                Box::new(Value::c(n)),
                Box::new(Value::Custom(crate::choices::CHOSEN_COLORS_IT_IS.into())),
            ),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "N life for each of the chosen colors it is", priority: 60, parse: for_each_chosen_color_it_is } }

/// The value a choice made as a cost chooses: "a creature type", "a number between 0 and
/// 5", "a color".
fn cost_choice_kind(s: &str) -> Option<ChoiceKind> {
    Some(match s {
        "a creature type" => ChoiceKind::CreatureType,
        "a color" => ChoiceKind::Color,
        "a card name" => ChoiceKind::CardName,
        "a basic land type" => ChoiceKind::BasicLandType,
        "a card type" => ChoiceKind::CardType,
        _ => {
            let r = s.strip_prefix("a number between ")?;
            let (a, b) = r.split_once(" and ")?;
            let (Value::Const(min), ra) = parse_number(a)? else {
                return None;
            };
            let (Value::Const(max), rb) = parse_number(b)? else {
                return None;
            };
            if !ra.trim().is_empty() || !end(rb).is_empty() || max < min {
                return None;
            }
            ChoiceKind::Number { min, max }
        }
    })
}

/// "choose a creature type" / "choose a number between 0 and 5" as a cost ("As an
/// additional cost to cast this spell, choose a creature type."): the choice is made as
/// the spell is cast (CR 601.2b, 601.2f-h) and kept for its resolution and, for a
/// permanent spell, by the permanent (CR 607.2d).
fn choose_as_cost(s: &str) -> Option<CostPart> {
    let r = end(s).strip_prefix("choose ")?;
    let kind = cost_choice_kind(r)?;
    Some(CostPart::Effect(Box::new(Effect::Choose {
        who: PlayerRef::You,
        kind,
    })))
}

inventory::submit! { super::CostPattern { name: "choose [a value] as a cost", priority: 60, parse: choose_as_cost } }

/// "[instruction with X], where X is the chosen number" (Liquid Fire): X is the number
/// chosen for the spell (as it was cast); "N minus X damage" is N minus that.
fn x_is_the_chosen_number(l: &str, b: &mut Builder) -> Option<Effect> {
    let x = end(l).strip_suffix(", where x is the chosen number")?;
    // "5 minus x damage": a sentinel amount replaced afterwards.
    const SENTINEL: i32 = 987_654;
    let mut minus: Option<i32> = None;
    let mut text = x.to_string();
    if let Some(at) = text.find(" minus x damage") {
        let before = &text[..at];
        let (head, n) = before.rsplit_once(' ')?;
        let (Value::Const(n), rest) = parse_number(n)? else {
            return None;
        };
        if !rest.trim().is_empty() {
            return None;
        }
        minus = Some(n);
        text = format!("{head} {SENTINEL} damage{}", &text[at + " minus x damage".len()..]);
    }
    let targets = b.targets.len();
    let Some(mut e) = parse_clause(&text, b) else {
        b.targets.truncate(targets);
        return None;
    };
    if let Some(n) = minus {
        let from = serde_json::to_value(Value::c(SENTINEL)).ok()?;
        let to = serde_json::to_value(Value::Diff(Box::new(Value::c(n)), Box::new(Value::X))).ok()?;
        let mut k = 0;
        let v = replace_value(serde_json::to_value(&e).ok()?, &from, &to, &mut k);
        if k != 1 {
            b.targets.truncate(targets);
            return None;
        }
        e = serde_json::from_value(v).ok()?;
    }
    Some(Effect::seq(vec![Effect::SetX { value: Value::Chosen }, e]))
}

inventory::submit! { EffectPattern { name: "where X is the chosen number", priority: 60, parse: x_is_the_chosen_number } }

fn replace_value(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
    n: &mut usize,
) -> serde_json::Value {
    use serde_json::Value as J;
    if v == *from {
        *n += 1;
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| replace_value(x, from, to, n)).collect()),
        J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, replace_value(x, from, to, n))).collect()),
        x => x,
    }
}

/// Behold a [quality], remembering what was beheld for the spell.
fn behold_and_remember(quality: Filter) -> Cost {
    Cost::free().with(CostPart::Effect(Box::new(Effect::seq(vec![
        Effect::KeywordAction {
            action: KeywordAction::Behold,
            who: PlayerRef::You,
            what: Sel::All(quality),
            n: Value::c(1),
        },
        Effect::Custom(crate::choices::REMEMBER_CHOSEN_OBJECTS.into()),
    ]))))
}

/// "As an additional cost to cast ~, choose a creature you control or reveal a creature
/// card from your hand." / "..., you may reveal a Dragon card from your hand or choose a
/// Dragon you control.": beholding (CR 701.4a), with what was chosen or revealed
/// remembered for the spell ("the power of the creature you chose or the card you
/// revealed").
fn choose_or_reveal_cost(text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, ")?;
    let (optional, r) = match r.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    // "choose a creature you control or reveal a creature card from your hand", "reveal a
    // Dragon card from your hand or choose a Dragon you control".
    let (a, b) = r.split_once(" or ")?;
    let (choose, reveal) = if a.starts_with("choose ") {
        (a, b)
    } else {
        (b, a)
    };
    let perm = choose
        .strip_prefix("choose a ")
        .or_else(|| choose.strip_prefix("choose an "))?
        .strip_suffix(" you control")?;
    let card = reveal
        .strip_prefix("reveal a ")
        .or_else(|| reveal.strip_prefix("reveal an "))?
        .strip_suffix(" card from your hand")?;
    let (f, false, tail) = parse_object_phrase(perm)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    // The same quality both ways ("a creature" / "a creature card", "a Dragon" / "a
    // Dragon card").
    let (g, false, tail) = parse_object_phrase(card)? else {
        return None;
    };
    if !end(tail).is_empty() || format!("{f:?}") != format!("{g:?}") {
        return None;
    }
    let cost = behold_and_remember(f);
    let change = if optional {
        CostChange::OptionalAdditionalCost {
            name: crate::behold::BEHOLD.into(),
            cost,
        }
    } else {
        CostChange::AdditionalCost(cost)
    };
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change,
    }));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { super::AbilityPattern { name: "choose a creature you control or reveal a creature card (cost)", priority: 75, parse: choose_or_reveal_cost } }

/// "~ deals damage equal to the power of the creature you chose or the card you revealed
/// to target creature".
fn power_of_chosen_or_revealed(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let phrase = [
        "damage equal to the power of the creature you chose or the card you revealed",
        "damage equal to the power of that card or creature",
    ]
    .into_iter()
    .find(|p| l.contains(p))?;
    let text = l.replacen(phrase, "x damage", 1);
    let targets = b.targets.len();
    let Some(e) = parse_clause(&text, b) else {
        b.targets.truncate(targets);
        return None;
    };
    Some(Effect::seq(vec![
        Effect::SetX {
            value: Value::Custom(crate::choices::CHOSEN_OBJECT_POWER.into()),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "the power of the creature you chose or the card you revealed", priority: 60, parse: power_of_chosen_or_revealed } }

/// "If you revealed a Dragon card or chose a Dragon as you cast ~, ~ deals damage equal
/// to the power of that card or creature instead." after "~ deals 3 damage to [target]".
fn if_revealed_or_chose_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you revealed a ") else {
        return false;
    };
    let Some((_, rest)) = r.split_once(" as you cast ~, ") else {
        return false;
    };
    let Some(x) = rest.strip_suffix(" instead") else {
        return false;
    };
    let Effect::DealDamage { source, to, .. } = &*prev else {
        return false;
    };
    // "deals damage equal to the power of that card or creature" to the same recipient.
    let Some(e) = power_of_chosen_or_revealed(&format!("{x} to it"), b) else {
        return false;
    };
    let Effect::Seq(v) = &e else {
        return false;
    };
    let [set_x, Effect::DealDamage { source: s2, .. }] = v.as_slice() else {
        return false;
    };
    if format!("{s2:?}") != format!("{source:?}") {
        return false;
    }
    let instead = Effect::seq(vec![
        set_x.clone(),
        Effect::DealDamage {
            source: source.clone(),
            amount: Value::X,
            to: to.clone(),
        },
    ]);
    let first = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond: Condition::CostPaid(crate::behold::BEHOLD.into()),
        then: Box::new(instead),
        otherwise: Box::new(first),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "if you revealed or chose [a card] as you cast ~, ... instead", priority: 60, apply: if_revealed_or_chose_instead } }
