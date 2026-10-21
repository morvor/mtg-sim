//! Mana grammar IV (cards3-costs-mana-replacements): where the mana spent came from
//! (CR 106.3, 601.2h).
//!
//! - "for each mana from a Treasure spent to cast ~", "for each mana from a Cave spent to
//!   cast it", "for each mana from a Desert spent to cast ~", "for each mana from an
//!   artifact source spent to cast it", "for each mana from a Treasure that was spent to
//!   cast ~": the number of mana from sources of a kind spent on the spell, counted by the
//!   value grammar (`value_grammar::count`), so any instruction taking "for each" reads it.
//!   A source is looked at as it last existed (a sacrificed Treasure is still known to have
//!   been a Treasure, CR 608.2h; see `kw/cast_history.rs`).
//! - "[a spell] using mana produced by ~" (Tecutlan, the Searing Rift; The Myriad Pools;
//!   Thousand Moons Smithy's back face): a spell some of whose mana was produced by the
//!   source (the mana remembers the object that produced it, CR 106.3).
//! - "for each unspent green mana you have", "as long as you have six or more unspent
//!   mana": the mana in your mana pool (CR 106.4), of one type or all of it.
//! - "for each {E} you've paid or lost this turn", "if you've paid or lost four or more
//!   {E} this turn": the energy counters removed from you this turn (paying {E} is losing
//!   energy counters, CR 107.14; see `kw/energy_paid.rs`).
//! - Changes to the mana a permanent is tapped for (CR 106.12b, see `mana_change.rs`):
//!   "If a land is tapped for two or more mana, it produces {C} instead of any other type
//!   and amount.", "If a basic land you control is tapped for mana, it produces mana of a
//!   color of your choice instead of any other type.", "if you tap a land for mana, it
//!   produces one mana of a color of your choice instead of any other type and amount",
//!   "If target Plains is tapped for mana, it produces colorless mana instead of white
//!   mana." (an effect on that Plains from then on). Wordings the older grammars read
//!   (a fixed type for any amount, "twice as much") are left to them.
//! - "Whenever a player taps a Mountain, Forest, or Plains for mana" (Keeper of
//!   Progenitus): a tapped-for-mana trigger (CR 106.12a) on a list of land types.
//! - "If they do, they add {C} for each counter on it." (Cheering Crowd): "they" is the
//!   player the trigger is about, as "that player adds".
//! - "For each artifact or creature card in target opponent's graveyard, add {C} and you
//!   gain 1 life." (Spoils of Evil): both instructions scaled by the one count (its target
//!   chosen once).
//! - "Whenever a basic land is tapped for mana of the chosen color" (Gauntlet of Power): a
//!   tapped-for-mana trigger (CR 106.12a) whose mana included the color chosen for the
//!   source (`kw/mana_produced_chosen_color.rs`).
//! - "Choose a color. Add an amount of mana of that color equal to [value]." (Three Tree
//!   City): that much mana of one color the player chooses, as "add X mana of any one
//!   color" (CR 106.1b).
//! - "If at least seven red mana was spent to cast ~, instead ~ deals 7 damage to each of
//!   those permanents and/or players." (Sundering Stroke): after damage divided among
//!   targets, the same targets are each dealt that damage instead when the condition
//!   holds (the division announced as it was cast, CR 601.2d, is then not used).
//! - Spending restrictions (CR 106.6): "This mana can't be spent to cast spells." (Thran
//!   Turbine: it may pay for anything else, such as activating abilities), "Spend this
//!   mana only to cast spells you don't own." (Thieving Varmint: "you" is the player
//!   spending the mana, checked against the spell being paid for).

use crate::ability::*;
use crate::kw::cast_history as ch;
use crate::oracle::effects::Builder;
use crate::kw::unspent_mana_value::{self, UNSPENT_MANA_YOU_HAVE};
use crate::mana::{ManaRestriction, ManaType, SpendFilter};
use crate::oracle::phrases::{end, parse_number, subtype_word};
use crate::oracle::phrases::parse_object_phrase;
use crate::oracle::CompileContext;
use crate::types::Color;

/// The kind of mana source in "mana from [kind]": a `cast history:spent` amount key
/// (`source subtype:Treasure`, `source type:Artifact`).
pub(crate) fn mana_from_kind(kind: &str) -> Option<String> {
    let kind = kind.trim();
    for (p, t) in [
        ("an artifact source", "Artifact"),
        ("artifact sources", "Artifact"),
        ("a creature source", "Creature"),
        ("creature sources", "Creature"),
        ("a creature", "Creature"),
        ("creatures", "Creature"),
        ("a land", "Land"),
        ("lands", "Land"),
    ] {
        if kind == p {
            return Some(format!("source type:{t}"));
        }
    }
    let w = kind
        .strip_prefix("a ")
        .or_else(|| kind.strip_prefix("an "))
        .unwrap_or(kind);
    if w.contains(' ') {
        return None;
    }
    let st = subtype_word(w)?;
    Some(format!("source subtype:{st}"))
}

/// "mana from [kind] [that was] spent to cast [spell]" at the start of a count ("the
/// number of ..." / "for each ..."): the amount, and the rest of the text.
pub(crate) fn count_phrase(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some((v, rest)) = unspent_mana(r) {
        return Some((v, rest.to_string()));
    }
    if let Some(rest) = r.strip_prefix("{e} you've paid or lost this turn") {
        if rest.is_empty() || rest.starts_with([' ', ',', '.']) {
            return Some((energy_paid_value(), rest.to_string()));
        }
    }
    let r = r.strip_prefix("mana from ")?;
    let (kind, rest) = r
        .split_once(" that was spent to ")
        .or_else(|| r.split_once(" spent to "))?;
    let what = mana_from_kind(kind)?;
    // "cast ~" / "cast it": the spell is one word after "cast".
    let rest = rest.strip_prefix("cast ")?;
    let (word, tail) = match rest.find([' ', ',', '.']) {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let key = super::history_conditions::payment_key(&format!("cast {word}"), Some(b))?;
    Some((
        super::history_conditions::spent_value(key, &what),
        tail.to_string(),
    ))
}

/// "unspent mana", "unspent green mana" at the start of `r`.
fn unspent_kind(r: &str) -> Option<(Value, &str)> {
    let r = r.strip_prefix("unspent ")?;
    let (v, rest) = if let Some(rest) = r.strip_prefix("mana") {
        (UNSPENT_MANA_YOU_HAVE.to_string(), rest)
    } else {
        let (c, rest) = r.split_once(' ')?;
        let t = ManaType::from_color(Color::from_word(c)?);
        (unspent_mana_value::of_type(t), rest.strip_prefix("mana")?)
    };
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((Value::Custom(v.into()), rest))
}

/// "unspent mana you have", "unspent green mana you have" at the start of `r`.
fn unspent_mana(r: &str) -> Option<(Value, &str)> {
    let (v, rest) = unspent_kind(r)?;
    let rest = rest.strip_prefix(" you have")?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    Some((v, rest))
}

/// "you have six or more unspent mana", "you have at least three unspent red mana".
fn unspent_condition(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you have ")?;
    let (n, r) = if let Some(r) = r.strip_prefix("at least ") {
        let (n, r) = parse_number(r)?;
        (n, r.trim_start())
    } else {
        let (n, r) = parse_number(r)?;
        (n, r.trim_start().strip_prefix("or more ")?)
    };
    let Value::Const(_) = n else { return None };
    let (v, rest) = unspent_kind(r)?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(v, Cmp::Ge, n))
}

inventory::submit! { super::ConditionPattern { name: "mana 4: you have N or more unspent mana", priority: 100, parse: unspent_condition } }

fn energy_paid_value() -> Value {
    Value::Custom(crate::kw::energy_paid::ENERGY_PAID_OR_LOST_THIS_TURN.into())
}

/// "you've paid or lost four or more {E} this turn".
fn energy_paid_condition(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("you've paid or lost ")?
        .strip_suffix(" {e} this turn")?;
    let (n, r) = parse_number(r)?;
    if r.trim() != "or more" {
        return None;
    }
    let Value::Const(_) = n else { return None };
    Some(Condition::Compare(energy_paid_value(), Cmp::Ge, n))
}

inventory::submit! { super::ConditionPattern { name: "mana 4: you've paid or lost N or more {E} this turn", priority: 100, parse: energy_paid_condition } }

/// "~ has another activated ability" (an activation restriction, see
/// `kw/has_another_activated_ability.rs`).
fn another_activated_ability(c: &str) -> Option<Condition> {
    (end(c) == "~ has another activated ability").then(|| {
        Condition::Custom(
            crate::kw::has_another_activated_ability::HAS_ANOTHER_ACTIVATED_ABILITY.into(),
        )
    })
}

inventory::submit! { super::ConditionPattern { name: "mana 4: ~ has another activated ability", priority: 100, parse: another_activated_ability } }

/// "for each mana from [kind] spent to cast it/them" about a permanent entering with
/// counters (Kalain, Reclusive Painter; Coin of Mastery): the mana spent to cast the
/// entering permanent's spell. A permanent that wasn't cast had none (CR 107.3m-style:
/// it didn't come from a spell).
pub(crate) fn entering_mana_from(each: &str) -> Option<Value> {
    let r = each.strip_prefix("mana from ")?;
    let (kind, whose) = r
        .split_once(" that was spent to cast ")
        .or_else(|| r.split_once(" spent to cast "))?;
    if !matches!(whose.trim(), "it" | "them") {
        return None;
    }
    let what = mana_from_kind(kind)?;
    Some(Value::Custom(ch::spent(ch::ENTERING, &what).into()))
}

/// "[spell] using mana produced by ~": a filter qualifier on spells.
fn using_mana_produced_by<'a>(t: &'a str, f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("using mana produced by ~")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    // Only spells are cast using mana.
    if !serde_json::to_string(f).is_ok_and(|s| s.contains("\"Spell\"")) {
        return None;
    }
    Some((Filter::Custom(ch::USING_SOURCE_MANA.into()), r))
}

inventory::submit! { super::FilterSuffixPattern { name: "mana 4: spell using mana produced by ~", priority: 100, parse: using_mana_produced_by } }

/// "This mana can't be spent to cast spells.", "Spend this mana only to cast spells you
/// don't own.": a restriction on the mana the previous sentence adds.
fn spend_restriction(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let r = match end(l).trim() {
        "this mana can't be spent to cast spells" => {
            ManaRestriction::NotCastSpell(SpendFilter::new(Filter::Spell))
        }
        "spend this mana only to cast spells you don't own"
        | "spend this mana only to cast a spell you don't own" => {
            ManaRestriction::CastSpell(SpendFilter::new(Filter::and(vec![
                Filter::Spell,
                Filter::Not(Box::new(Filter::OwnedBy(PlayerRel::You))),
            ])))
        }
        _ => return false,
    };
    let mut e = prev.clone();
    if !super::mana_restrictions::restrict(&mut e, &r) {
        return false;
    }
    *prev = e;
    true
}

inventory::submit! { super::FollowupPattern { name: "mana 4: can't be spent to cast spells; only spells you don't own", priority: 95, apply: spend_restriction } }

/// "it produces [to] instead of [of]" → the change (without its minimum amount).
fn mana_change_action(action: &str) -> Option<ManaChange> {
    let (to, of) = end(action)
        .strip_prefix("it produces ")?
        .split_once(" instead of ")?;
    let (to, one) = match to {
        "{c}" => (ManaChangeTo::Type(ManaType::C), true),
        "colorless mana" => (ManaChangeTo::Type(ManaType::C), false),
        "one mana of a color of your choice" => (ManaChangeTo::ColorOfChoice, true),
        "mana of a color of your choice" => (ManaChangeTo::ColorOfChoice, false),
        _ => return None,
    };
    let (only, amount) = match of {
        // "{C} instead of any other type and amount": exactly one mana.
        "any other type and amount" if one => (None, Some(1)),
        "any other type" if !one => (None, None),
        "any other color" if !one => (
            Some(vec![
                ManaType::W,
                ManaType::U,
                ManaType::B,
                ManaType::R,
                ManaType::G,
            ]),
            None,
        ),
        _ if !one => {
            let c = of.strip_suffix(" mana")?;
            (Some(vec![ManaType::from_color(Color::from_word(c)?)]), None)
        }
        _ => return None,
    };
    Some(ManaChange {
        min_amount: 0,
        only,
        to,
        amount,
    })
}

/// "[subject] is tapped for [two or more] mana" / "you tap [subject] for mana": the
/// subject, whether "you" tap it, and the minimum amount.
fn tapped_for(cond: &str) -> Option<(&str, bool, u32)> {
    if let Some(x) = cond.strip_prefix("you tap ") {
        return Some((x.strip_suffix(" for mana")?, true, 0));
    }
    let (subject, tail) = cond.split_once(" is tapped for ")?;
    let min = match tail {
        "mana" => 0,
        t => {
            let (n, r) = parse_number(t)?;
            if r.trim() != "or more mana" {
                return None;
            }
            n.as_const()? as u32
        }
    };
    Some((subject, false, min))
}

/// Whether the change is one the older mana replacement grammars read
/// (`r106_mana::produce_mana_replacement`, `mana_type_instead`): a fixed type for any
/// amount of mana.
fn read_elsewhere(c: &ManaChange) -> bool {
    c.min_amount == 0 && c.only.is_none() && matches!(c.to, ManaChangeTo::Type(_))
}

/// "If [a land] is tapped for [two or more] mana, it produces [mana] instead of [...]".
fn mana_change_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cond, action) = end(l).strip_prefix("if ")?.split_once(", ")?;
    let mut change = mana_change_action(action)?;
    let (subject, you, min) = tapped_for(cond)?;
    change.min_amount = min;
    if read_elsewhere(&change) {
        return None;
    }
    let s = subject
        .strip_prefix("a ")
        .or_else(|| subject.strip_prefix("an "))?;
    let (f, plural, rest) = parse_object_phrase(s)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    // Only a permanent's controller can activate its mana abilities (CR 602.2).
    let f = if you { f.you_control() } else { f };
    let def = ReplacementDef {
        event: ReplacementEvent::ProduceMana(f),
        action: ReplacementAction::ManaChange(change),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "mana 4: if a land is tapped for mana, it produces [mana] instead of [...]", priority: 61, parse: mana_change_static } }

/// "If target Plains is tapped for mana, it produces colorless mana instead of white
/// mana." (an instruction): from then on, for that permanent (CR 611.2a).
fn mana_change_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let (cond, action) = end(l).strip_prefix("if ")?.split_once(", ")?;
    let mut change = mana_change_action(action)?;
    let (subject, you, min) = tapped_for(cond)?;
    change.min_amount = min;
    if you || !subject.starts_with("target ") {
        return None;
    }
    let saved = b.targets.len();
    let (sel, rest) = crate::oracle::effects::object_ref(subject, b)?;
    if !rest.trim().is_empty() || !matches!(sel, Sel::Target(_)) || b.targets.len() != saved + 1 {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ProduceMana(Filter::In(Box::new(sel))),
            action: ReplacementAction::ManaChange(change),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::Permanent,
        uses: None,
    })
}

inventory::submit! { super::EffectPattern { name: "mana 4: if target land is tapped for mana, it produces [mana] instead", priority: 61, parse: mana_change_target } }

/// "a Mountain, Forest, or Plains": land subtypes in a list.
fn subtype_list(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (head, last) = s.rsplit_once(", or ")?;
    let mut v = Vec::new();
    for w in head.split(", ").chain([last]) {
        let st = subtype_word(w.trim())?;
        if crate::types::subtype_kind(&st) != Some(crate::types::SubtypeKind::Land) {
            return None;
        }
        v.push(Filter::Subtype(st));
    }
    (v.len() >= 3).then(|| Filter::Or(v))
}

/// "[a player | you | an opponent] taps a Mountain, Forest, or Plains for mana".
fn taps_listed_lands(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r.trim());
    let (who, rest) = [
        ("a player taps ", PlayerRel::Any),
        ("you tap ", PlayerRel::You),
        ("an opponent taps ", PlayerRel::Opponent),
    ]
    .into_iter()
    .find_map(|(p, w)| r.strip_prefix(p).map(|x| (w, x)))?;
    let filter = subtype_list(rest.strip_suffix(" for mana")?)?;
    Some((
        TriggerCond::TappedForMana { who, filter },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { super::TriggerPattern { name: "mana 4: a player taps a Mountain, Forest, or Plains for mana", priority: 100, parse: taps_listed_lands } }

/// "they add [mana]": "that player adds [mana]" when "they" is the trigger's player.
fn they_add(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("they add ")?;
    if !matches!(b.it_player, PlayerRef::TriggerPlayer) {
        return None;
    }
    let e = crate::oracle::effects::parse_clause(&format!("that player adds {r}"), b)?;
    matches!(&e, Effect::AddMana { who: PlayerRef::TriggerPlayer, .. }).then_some(e)
}

inventory::submit! { super::EffectPattern { name: "mana 4: they add [mana]", priority: 100, parse: they_add } }

/// "for each [counted thing], add [mana] [and you gain N life]".
fn for_each_add(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (counted, rest) = r.split_once(", ")?;
    if !rest.starts_with("add ") || counted.contains(',') || rest.contains(" for each ") {
        return None;
    }
    let saved = b.targets.len();
    let parsed = (|| {
        let (v, tail) = crate::oracle::statics::parse_value_phrase(
            &format!("the number of {counted}"),
            b,
        )?;
        if !tail.trim().is_empty() {
            return None;
        }
        let before = b.targets.len();
        let mut out = Vec::new();
        for part in rest.split(" and ") {
            let e = crate::oracle::effects::parse_clause(part, b)?;
            out.push(match e {
                Effect::AddMana {
                    who,
                    mana: ManaProduction::Fixed(ts),
                    restriction,
                } if !ts.is_empty() && ts.iter().all(|t| *t == ts[0]) => Effect::AddMana {
                    who,
                    mana: ManaProduction::Amount(
                        ts[0],
                        match ts.len() {
                            1 => v.clone(),
                            n => Value::Mul(Box::new(Value::c(n as i32)), Box::new(v.clone())),
                        },
                    ),
                    restriction,
                },
                e @ Effect::GainLife { .. } => {
                    super::iteration_grammar::times_over(e, v.clone())
                }
                _ => return None,
            });
        }
        (b.targets.len() == before && matches!(out.first(), Some(Effect::AddMana { .. })))
            .then(|| Effect::seq(out))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { super::EffectPattern { name: "mana 4: for each [x], add [mana] and you gain N life", priority: 120, parse: for_each_add } }

/// "[a basic land] is tapped for mana of the chosen color".
fn tapped_for_chosen_color(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let subject = end(r.trim()).strip_suffix(" is tapped for mana of the chosen color")?;
    let s = subject
        .strip_prefix("a ")
        .or_else(|| subject.strip_prefix("an "))?;
    let (f, plural, rest) = parse_object_phrase(s)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::TappedForMana {
                who: PlayerRel::Any,
                filter: f,
            }),
            cond: Condition::Custom(
                crate::kw::mana_produced_chosen_color::PRODUCED_CHOSEN_COLOR.into(),
            ),
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

/// "its controller adds an additional one mana of that color" after "is tapped for mana of
/// the chosen color": the chosen color (CR 607.2d).
fn adds_one_of_that_color(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger
        || end(l) != "its controller adds an additional one mana of that color"
        || !matches!(b.it, Sel::TriggerObject)
        || !crate::oracle::raw_text()
            .to_lowercase()
            .contains("is tapped for mana of the chosen color, its controller adds")
    {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
        mana: ManaProduction::ChosenColor(Value::c(1)),
        restriction: None,
    })
}

inventory::submit! { super::EffectPattern { name: "mana 4: its controller adds an additional one mana of that color", priority: 100, parse: adds_one_of_that_color } }

inventory::submit! { super::TriggerPattern { name: "mana 4: [land] is tapped for mana of the chosen color", priority: 100, parse: tapped_for_chosen_color } }

/// "add an amount of mana of that color equal to [value]" after "choose a color".
fn amount_of_chosen_color(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("add an amount of mana of that color equal to ") else {
        return false;
    };
    if !matches!(
        prev,
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Color,
        }
    ) {
        return false;
    }
    let saved = b.targets.len();
    let Some((v, tail)) = crate::oracle::statics::parse_value_phrase(r, b) else {
        return false;
    };
    // Only a count of things (the rendering words it "for each"); devotion is
    // `game_rules_mana`'s.
    if !tail.trim().is_empty() || b.targets.len() != saved || !matches!(v, Value::Count(_)) {
        b.targets.truncate(saved);
        return false;
    }
    *prev = Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::AnyOneColor(v),
        restriction: None,
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "mana 4: choose a color. add an amount of mana of that color equal to [count]", priority: 100, apply: amount_of_chosen_color } }

/// "if [condition], instead ~ deals N damage to each of those permanents and/or players"
/// after "~ deals N damage divided as you choose among ... targets".
fn instead_damage_to_each_target(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::DealDividedDamage { source, slot } = prev.clone() else {
        return false;
    };
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((cond_s, rest)) = r.split_once(", instead ~ deals ") else {
        return false;
    };
    let Some((n, tail)) = parse_number(rest) else {
        return false;
    };
    if !matches!(n, Value::Const(_))
        || !matches!(
            tail.trim(),
            "damage to each of those permanents and/or players" | "damage to each of them"
        )
    {
        return false;
    }
    let saved = b.targets.len();
    let Some(cond) = super::conditions_referents::parse_condition_with(cond_s, b) else {
        return false;
    };
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return false;
    }
    *prev = Effect::If {
        cond,
        then: Box::new(Effect::DealDamage {
            source,
            amount: n,
            to: Sel::Target(slot),
        }),
        otherwise: Box::new(prev.clone()),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "mana 4: if [condition], instead ~ deals N damage to each of those targets", priority: 100, apply: instead_damage_to_each_target } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        assert_eq!(
            mana_from_kind("a treasure").as_deref(),
            Some("source subtype:Treasure")
        );
        assert_eq!(
            mana_from_kind("a cave").as_deref(),
            Some("source subtype:Cave")
        );
        assert_eq!(
            mana_from_kind("an artifact source").as_deref(),
            Some("source type:Artifact")
        );
        assert_eq!(mana_from_kind("a blue source"), None);
    }
}
