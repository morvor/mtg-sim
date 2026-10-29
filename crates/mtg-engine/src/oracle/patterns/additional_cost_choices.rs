//! Choices made as additional costs to cast a spell (CR 601.2b) and what its effect says
//! about them (cards5-costs-sets):
//!
//! - "As an additional cost to cast ~, you may pay {1}{R} and/or {1}{G} any number of
//!   times." (Primitive Justice): two repeatable additional costs, each counted on its own
//!   as the spell is cast (CR 601.2b, 601.2f), and "For each additional {1}{R} you paid,
//!   destroy another target artifact.": a target group whose size is that count (chosen
//!   after the costs are announced, CR 601.2c; the same artifact can't be targeted twice,
//!   its ruling), with "and you gain 1 life" once for each.

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

/// The name a repeatable mana additional cost is announced under ("pay {1}{r} any number
/// of times", as `spell_cost_grammar` names it; lowercase mana).
fn repeat_name(mana: &str) -> String {
    format!("pay {mana} any number of times")
}

/// A mana cost written exactly as Oracle text writes it (lowercase), with no X.
fn plain_mana(m: &str) -> Option<ManaCost> {
    let mana = ManaCost::parse(&m.to_uppercase())?;
    (!mana.has_x() && format!("{mana}").to_lowercase() == m && !m.is_empty()).then_some(mana)
}

/// "As an additional cost to cast ~, you may pay {A} and/or {B} any number of times."
fn two_repeatable_mana_costs(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower)
        .strip_prefix("as an additional cost to cast ~, you may pay ")?
        .strip_suffix(" any number of times")?;
    let (a, b) = r.split_once(" and/or ")?;
    if a == b {
        return None;
    }
    let mut out = Vec::new();
    for m in [a, b] {
        let mana = plain_mana(m)?;
        let name = repeat_name(m);
        let times = Value::Custom(crate::kw::spell_cost_grammar::paid_times(&name).into());
        let cost = Cost::free().with(CostPart::Repeated {
            cost: Box::new(Cost::mana(mana)),
            times,
        });
        out.push(super::costs_casting_self::this_spell_cost_ability(
            CostChange::AdditionalCost(cost),
            None,
            text,
        ));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "additional cost choices: you may pay {A} and/or {B} any number of times", priority: 78, parse: two_repeatable_mana_costs } }

/// Whether the card's text has a repeatable additional mana cost `mana` ("you may pay
/// {1}{r} any number of times", alone or in an "and/or" pair).
fn has_repeatable_cost(mana: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    let raw = raw.replace("as an additional cost to cast this spell,", "as an additional cost to cast ~,");
    raw.lines().any(|l| {
        let Some(r) = l
            .trim_end_matches('.')
            .strip_prefix("as an additional cost to cast ~, you may pay ")
            .and_then(|r| r.strip_suffix(" any number of times"))
        else {
            return false;
        };
        r == mana || r.split(" and/or ").any(|m| m == mana)
    })
}

/// "For each additional {1}{R} you paid, destroy another target artifact[, and you gain
/// N life]": the body's last target instance is chosen as many times as that cost was
/// paid (and none if it wasn't), and the life is gained once for each.
fn for_each_additional_paid(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each additional ")?;
    let (mana, body) = r.split_once(" you paid, ")?;
    plain_mana(mana)?;
    if !has_repeatable_cost(mana) {
        return None;
    }
    let times = Value::Custom(crate::kw::spell_cost_grammar::paid_times(&repeat_name(mana)).into());
    let (body, life) = match body.rsplit_once(", and you gain ") {
        Some((head, tail)) => {
            let (n, rest) = parse_number(tail.strip_suffix(" life")?)?;
            if !rest.trim().is_empty() || n.as_const().is_none() {
                return None;
            }
            (head, Some(n))
        }
        None => (body, None),
    };
    let saved = b.targets.len();
    let fail = |b: &mut Builder| {
        b.targets.truncate(saved);
        None
    };
    let Some(e) = parse_clause(body, b) else {
        return fail(b);
    };
    // Exactly one new instance of "target", of one object, which the count sets.
    if b.targets.len() != saved + 1 {
        return fail(b);
    }
    let t = &mut b.targets[saved];
    if t.min.as_const() != Some(1) || t.max.as_const() != Some(1) || t.per_player.is_some() {
        return fail(b);
    }
    // Only an effect on that target ("destroy another target artifact").
    let slot = saved as u8;
    let on_target = match &e {
        Effect::Destroy { what, .. } | Effect::Exile { what, .. } => {
            matches!(what, Sel::Target(s) if *s == slot)
        }
        _ => false,
    };
    if !on_target {
        return fail(b);
    }
    t.min = times.clone();
    t.max = times.clone();
    let mut v = vec![e];
    if let Some(Value::Const(n)) = life {
        v.push(Effect::GainLife {
            who: PlayerRef::You,
            n: match n {
                1 => times,
                n => Value::Mul(Box::new(Value::c(n)), Box::new(times)),
            },
        });
    }
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "additional cost choices: for each additional {M} you paid, [instruction on another target]", priority: 60, parse: for_each_additional_paid } }

// ---------------------------------------------------------------------------
// "choose a creature you control or a warped creature card you own in exile"
// ---------------------------------------------------------------------------

/// One alternative of an object chosen as a cost: "a creature you control" (on the
/// battlefield), "a warped creature card you own in exile" (CR 702.185b), "a creature card
/// in your graveyard".
fn chosen_object_alternative(s: &str) -> Option<Filter> {
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (warped, r) = match r.strip_prefix("warped ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let dbg = format!("{f:?}");
    let in_zone = dbg.contains("InZone(");
    let card = r.contains(" card");
    if warped && !(card && dbg.contains("InZone(Exile)")) {
        return None;
    }
    let mut parts = vec![f];
    if !in_zone {
        if card {
            return None;
        }
        parts.push(Filter::InZone(ZoneKind::Battlefield));
    }
    if warped {
        parts.push(Filter::Custom(crate::kw::warp::WARPED_CARD.into()));
    }
    Some(Filter::and(parts))
}

/// The cost of choosing one object matching `filter`, remembered for the spell
/// (`choices::REMEMBER_CHOSEN_OBJECTS`; "the chosen creature or card").
fn choose_object_cost(filter: Filter) -> Cost {
    Cost::free().with(CostPart::Effect(Box::new(Effect::seq(vec![
        Effect::Store {
            var: vars::IT,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter,
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
        Effect::Custom(crate::choices::REMEMBER_CHOSEN_OBJECTS.into()),
    ]))))
}

/// "As an additional cost to cast ~, choose a creature you control[ or a warped creature
/// card you own in exile]." (Close Encounter): the object is chosen as the spell's costs
/// are paid (CR 601.2b, 601.2h; a cost that can't be paid without an object to choose),
/// and remembered for the spell.
fn choose_object_as_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, choose ")?;
    let mut alts = Vec::new();
    for a in r.split(" or ") {
        alts.push(chosen_object_alternative(a)?);
    }
    let filter = match alts.len() {
        1 => alts.pop()?,
        _ => Filter::Or(alts),
    };
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::AdditionalCost(choose_object_cost(filter)),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "additional cost choices: choose a creature you control or a warped creature card", priority: 74, parse: choose_object_as_cost } }

/// "~ deals damage equal to the power of the chosen creature or card to target creature"
/// after an object chosen as an additional cost: that object's power, or its last known
/// power if it has left the battlefield (Close Encounter's ruling).
fn power_of_object_chosen_as_cost(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let phrase = [
        "damage equal to the power of the chosen creature or card",
        "damage equal to the power of the chosen creature",
    ]
    .into_iter()
    .find(|p| l.contains(p))?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("as an additional cost to cast this spell, choose a ")
        && !raw.contains("as an additional cost to cast ~, choose a ")
    {
        return None;
    }
    let saved = b.targets.len();
    let Some(e) = parse_clause(&l.replacen(phrase, "x damage", 1), b) else {
        b.targets.truncate(saved);
        return None;
    };
    Some(Effect::seq(vec![
        Effect::SetX {
            value: Value::Custom(crate::choices::CHOSEN_OBJECT_POWER.into()),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "additional cost choices: the power of the chosen creature or card", priority: 59, parse: power_of_object_chosen_as_cost } }

// ---------------------------------------------------------------------------
// "choose a creature type and behold two creatures of that type"
// ---------------------------------------------------------------------------

/// "[you may ]choose a creature type and behold N creatures of that type" as an
/// additional cost (Celestial Reunion; see `cost_sets::CHOOSE_TYPE_AND_BEHOLD`).
fn choose_type_and_behold_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !ctx.is_spell() {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, ")?;
    let (optional, r) = match r.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let n = r
        .strip_prefix("choose a creature type and behold ")?
        .strip_suffix(" creatures of that type")
        .or_else(|| {
            r.strip_prefix("choose a creature type and behold ")?
                .strip_suffix(" creature of that type")
        })?;
    let (Value::Const(n), rest) = parse_number(n)? else {
        return None;
    };
    if !rest.trim().is_empty() || n < 1 {
        return None;
    }
    let cost = Cost::free().with(CostPart::Effect(Box::new(Effect::Custom(
        format!("{}{n}", crate::cost_sets::CHOOSE_TYPE_AND_BEHOLD).into(),
    ))));
    let change = if optional {
        CostChange::OptionalAdditionalCost {
            name: smol_str::SmolStr::new(r),
            cost,
        }
    } else {
        CostChange::AdditionalCost(cost)
    };
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        change, None, text,
    )])
}

inventory::submit! { AbilityPattern { name: "additional cost choices: choose a creature type and behold N creatures of that type", priority: 74, parse: choose_type_and_behold_cost } }

/// A condition of "If [condition], put that card onto the battlefield instead of putting
/// it into your hand", judged once the card is found: "the revealed card is the chosen
/// type" (the type chosen for the spell), or any condition of the game.
fn found_card_condition(c: &str, b: &Builder) -> Option<Condition> {
    let mut parts = Vec::new();
    for p in c.split(" and ") {
        let cond = match p {
            "the revealed card is the chosen type" | "it's the chosen type" | "that card is the chosen type" => {
                Condition::SelMatches(Sel::Var(vars::IT), Filter::ChosenType)
            }
            p => crate::oracle::statics::parse_condition(p, b.ctx)?,
        };
        parts.push(cond);
    }
    Some(match parts.len() {
        1 => parts.pop()?,
        _ => Condition::And(parts),
    })
}

/// "Search your library for a creature card ..., reveal it, put it into your hand, then
/// shuffle. If [condition], put that card onto the battlefield instead of putting it into
/// your hand." (Celestial Reunion): the card is found and revealed, then put onto the
/// battlefield if the condition is true of it, or into the hand, then the library is
/// shuffled (CR 614.1a: "instead"; 701.23).
fn found_card_onto_battlefield_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(c) = end(l).strip_prefix("if ").and_then(|r| {
        r.strip_suffix(", put that card onto the battlefield instead of putting it into your hand")
    }) else {
        return false;
    };
    let Some(cond) = found_card_condition(c, b) else {
        return false;
    };
    let last = super::referents_values_5::last_mut_pub(prev);
    let (filter, reveal) = match &*last {
        Effect::Search {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            filter,
            count,
            to,
            reveal,
            shuffle: true,
            ..
        } if count.as_const() == Some(1) && to.zone == ZoneKind::Hand => (
            match filter {
                Filter::And(v) => Filter::and(
                    v.iter()
                        .filter(|f| !matches!(f, Filter::InZone(ZoneKind::Library)))
                        .cloned()
                        .collect(),
                ),
                f => f.clone(),
            },
            *reveal,
        ),
        _ => return false,
    };
    let search = Effect::SearchCards(Box::new(SearchSpec {
        who: PlayerRef::You,
        whose: PlayerRef::You,
        zones: vec![ZoneKind::Library],
        zones_optional: false,
        parts: vec![SearchPart {
            filter,
            count: Value::c(1),
            up_to: false,
            all: false,
        }],
        distinct_names: false,
        optional: false,
        reveal,
        dests: vec![],
        shuffle: SearchShuffle::No,
    }));
    let it = Sel::Var(vars::IT);
    *last = Effect::seq(vec![
        search,
        Effect::If {
            cond,
            then: Box::new(Effect::Move {
                what: it.clone(),
                to: Destination::battlefield(),
            }),
            otherwise: Box::new(Effect::Move {
                what: it,
                to: Destination::zone(ZoneKind::Hand),
            }),
        },
        Effect::Shuffle {
            who: PlayerRef::You,
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "additional cost choices: if [condition of the found card], put that card onto the battlefield instead of putting it into your hand", priority: 99, apply: found_card_onto_battlefield_instead } }
