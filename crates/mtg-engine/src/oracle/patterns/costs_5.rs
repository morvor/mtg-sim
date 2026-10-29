//! Cost grammar V (cards4-costs-mana-replacements).
//!
//! - Rules about a spell's cost that apply only when it's cast a particular way
//!   ([`CostRule::CastThisWay`], see `payment_rules.rs`): "You may pay {R}{G} rather than
//!   pay ~'s mana cost. Spend only mana produced by Treasures to cast it this way."
//!   (Security Rhox: an alternative cost of the spell's own text, CR 118.9, whose mana may
//!   come only from such sources, CR 106.3, 601.2h), "Flashback—{3}{R}, Remove X loyalty
//!   counters from among planeswalkers you control. If you cast ~ this way, X can't be 0."
//!   (Light Up the Night: CR 107.3a when cast with flashback).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// The rule of a "this way" sentence (lowercase, no period).
fn this_way_rule(s: &str) -> Option<CostRule> {
    if matches!(
        s,
        "if you cast ~ this way, x can't be 0" | "if you cast this spell this way, x can't be 0"
    ) {
        return Some(CostRule::XAtLeast(1));
    }
    let r = s
        .strip_prefix("spend only mana produced by ")?
        .strip_suffix(" to cast it this way")
        .or_else(|| {
            s.strip_prefix("spend only mana produced by ")?
                .strip_suffix(" to cast ~ this way")
        })?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if !plural || !end(tail).trim().is_empty() {
        return None;
    }
    let f = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]);
    Some(CostRule::OnlyManaFrom(crate::mana::SpendFilter::new(f)))
}

/// "[alternative cost or keyword alternative cost]. [rule that applies when cast this
/// way]."
fn cast_this_way_rule(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let block = block.trim_end();
    let (head, tail) = ["Spend only mana produced by ", "If you cast ~ this way, "]
        .iter()
        .find_map(|p| {
            let i = block.find(p)?;
            (i > 0 && block[..i].trim_end().ends_with('.')).then(|| (&block[..i], &block[i..]))
        })?;
    let rule = this_way_rule(end(&tail.to_lowercase()))?;
    let out = crate::oracle::parse_ability(head.trim_end(), ctx)?;
    let [a] = out.as_slice() else {
        return None;
    };
    let way = match &a.kind {
        // A keyword's alternative cost (flashback, ...).
        AbilityKind::Keyword(k) if k.cost.is_some() => CastWay::Keyword(k.kind),
        AbilityKind::Static(s)
            if matches!(
                &s.effect,
                StaticEffect::CostModifier(CostModifier {
                    applies_to: CostTarget::ThisSpell,
                    change: CostChange::AlternativeCost(_)
                        | CostChange::AlternativeCostWithFlash(_),
                    ..
                })
            ) =>
        {
            CastWay::OwnAlternativeCost
        }
        _ => return None,
    };
    let rule = CostRule::CastThisWay {
        way,
        rule: Box::new(rule),
    };
    let mut out = out.clone();
    out.push(super::costs_casting_self::this_spell_cost_ability(
        CostChange::Rule(rule),
        None,
        tail.trim(),
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "costs 5: [alternative cost]. [rule] to cast it this way", priority: 80, parse: cast_this_way_rule } }

/// "[instruction granting "[cost]: [effect]."] Only you may activate this ability."
/// (Martyrdom): the granted activated ability may be activated only by the player who
/// controlled the granting effect (CR 602.2), fixed as the effect is created
/// (`granted_captures::CAPTURED_YOU`), whoever controls the creature later.
fn granted_only_you_may_activate(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let head = block
        .trim_end()
        .strip_suffix(" Only you may activate this ability.")?;
    if !head.ends_with("\"") {
        return None;
    }
    let out = crate::oracle::parse_ability(head, ctx)?;
    let mut granted = 0;
    let mut fixed = Vec::new();
    for a in out {
        let mut v = serde_json::to_value(&a.kind).ok()?;
        granted += set_granted_activator(&mut v);
        let kind: AbilityKind = serde_json::from_value(v).ok()?;
        fixed.push(AbilityDef::with_link(kind, block.trim(), a.link));
    }
    (granted == 1).then_some(fixed)
}

/// Sets the activator of each activated ability granted by an `AddAbility` modification in
/// `v`; returns how many.
fn set_granted_activator(v: &mut serde_json::Value) -> usize {
    use serde_json::Value as J;
    match v {
        J::Object(m) => {
            if let Some(J::Object(def)) = m.get_mut("AddAbility") {
                if let Some(J::Object(kind)) = def.get_mut("kind") {
                    if let Some(J::Object(act)) = kind.get_mut("Activated") {
                        if act.get("activator").is_some_and(|x| x.is_null())
                            && act.get("any_player") == Some(&J::Bool(false))
                        {
                            let you = PlayerFilter::Ref(Box::new(PlayerRef::Var(
                                crate::granted_captures::CAPTURED_YOU,
                            )));
                            act.insert(
                                "activator".into(),
                                serde_json::to_value(you).unwrap_or(J::Null),
                            );
                            return 1;
                        }
                    }
                }
            }
            m.values_mut().map(set_granted_activator).sum()
        }
        J::Array(a) => a.iter_mut().map(set_granted_activator).sum(),
        _ => 0,
    }
}

inventory::submit! { AbilityPattern { name: "costs 5: [grant of an activated ability]. Only you may activate this ability.", priority: 80, parse: granted_only_you_may_activate } }

/// "[effect] unless you pay its mana cost [reduced by {N}]" (Flash: "sacrifice it unless
/// you pay its mana cost reduced by {2}"): the controller may pay that object's mana cost,
/// less N generic mana (CR 118.7d, 118.12a); if they don't, the effect happens.
fn unless_you_pay_its_mana_cost(
    l: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<Effect> {
    let (eff, cost) = end(l).rsplit_once(" unless you pay ")?;
    let (who, rest) = cost.split_once(" mana cost")?;
    let sel = match who {
        "its" => {
            if super::oracle_hardening_referents::is_no_referent(&b.it) {
                return None;
            }
            b.it.clone()
        }
        "~'s" => Sel::This,
        _ => return None,
    };
    let part = match rest.trim() {
        "" => CostPart::PayManaCostOf(Box::new(sel)),
        r => {
            let n = r.strip_prefix("reduced by {")?.strip_suffix('}')?;
            CostPart::PayManaCostOfLess(Box::new(sel), n.parse().ok()?)
        }
    };
    let saved = b.targets.len();
    let Some(effect) = crate::oracle::effects::parse_clause(eff, b) else {
        b.targets.truncate(saved);
        return None;
    };
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![part],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    })
}

inventory::submit! { super::EffectPattern { name: "costs 5: [effect] unless you pay its mana cost reduced by {N}", priority: 100, parse: unless_you_pay_its_mana_cost } }

/// The energy paid by "Pay any amount of {E}" / "you may pay any amount of {E}" (CR
/// 107.14): that instruction defines X as the amount paid, so "if you paid one or more
/// {E} this way" is "if X is 1 or more", "the amount of {E} paid this way" is X, and "if
/// seven or more {E} was paid this way" is "if X is 7 or more" (Vault 112: Sadistic
/// Simulation, Wheel of Potential). Only on a card whose text pays any amount of energy.
fn energy_paid_this_way(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("pay any amount of {e}")
        || !block.contains("{E} this way")
            && !block.contains("{E} paid this way")
            && !block.contains("{E} was paid this way")
    {
        return None;
    }
    let mut t = block.to_string();
    for (from, to) in [
        (
            "If you paid one or more {E} this way, ",
            "If X is 1 or more, ",
        ),
        ("the amount of {E} paid this way", "X"),
        (
            "If seven or more {E} was paid this way, ",
            "If X is 7 or more, ",
        ),
        (
            "exile that many cards from the top.",
            "exile the top X cards of your library.",
        ),
    ] {
        t = t.replace(from, to);
    }
    if t == block || t.contains("{E} this way") || t.contains("paid this way") {
        return None;
    }
    let out = crate::oracle::parse_ability(&t, ctx)?;
    if out
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Unsupported(_)))
    {
        return None;
    }
    Some(
        out.into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), block.trim(), a.link))
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "costs 5: the {E} paid this way (pay any amount of energy)", priority: 80, parse: energy_paid_this_way } }

/// "As an additional cost to cast ~, you may sacrifice one or more creatures. When you do,
/// copy ~ for each creature sacrificed this way." (Plumb the Forbidden): an optional cost
/// paid any number of times (one or more, or not at all: "any number of", CR 601.2b), and a
/// reflexive triggered ability when it's paid (CR 603.12), which triggers as the spell is
/// cast and copies it once for each creature sacrificed.
fn optional_sacrifice_one_or_more(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let (cost_s, reflexive) = block
        .trim_end()
        .split_once(" When you do, copy ~ for each creature sacrificed this way.")
        .map(|(a, b)| (a, b.is_empty()))?;
    if !reflexive {
        return None;
    }
    let cost_s = cost_s.replacen(
        "you may sacrifice one or more creatures.",
        "you may sacrifice any number of creatures.",
        1,
    );
    if !cost_s.ends_with("you may sacrifice any number of creatures.") {
        return None;
    }
    let mut out = crate::oracle::parse_ability(&cost_s, ctx)?;
    let [a] = out.as_slice() else {
        return None;
    };
    // The repeatable cost's count.
    let s = serde_json::to_string(&a.kind).ok()?;
    let start = s.find("spell cost:times paid:")?;
    let name: String = s[start..].chars().take_while(|c| *c != '"').collect();
    let times = Value::Custom(name.into());
    let mut t = TriggeredAbility::new(
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::Source,
        },
        Body {
            targets: vec![],
            effect: Effect::CopySpell {
                what: Sel::This,
                count: times.clone(),
                new_targets: false,
            },
            modal: None,
        },
    );
    t.intervening_if = Some(Condition::Compare(times, Cmp::Ge, Value::c(1)));
    t.zone = FunctionZone::Stack;
    out = out
        .into_iter()
        .map(|a| AbilityDef::with_link(a.kind.clone(), block.trim(), a.link))
        .collect();
    out.push(AbilityDef::new(AbilityKind::Triggered(t), block.trim()));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "costs 5: you may sacrifice one or more creatures. When you do, copy ~ for each", priority: 80, parse: optional_sacrifice_one_or_more } }

/// "Remove a time counter from a permanent you control or suspended card you own" (Rift
/// Elemental): one permanent or one suspended card in exile (a card with suspend and a time
/// counter, CR 702.62b), CR 118.3.
fn remove_counter_from_permanent_or_suspended(p: &str) -> Option<CostPart> {
    let r = crate::oracle::phrases::strip(p, "remove")?;
    let r = r.trim_start().strip_prefix("a ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counter from a ")?;
    let perm = end(r).strip_suffix(" or suspended card you own")?;
    let (f, plural, tail) = parse_object_phrase(perm)?;
    if plural || !end(tail).is_empty() || !format!("{f:?}").contains("ControlledBy(You)") {
        return None;
    }
    let suspended = Filter::and(vec![
        Filter::Card,
        Filter::InZone(ZoneKind::Exile),
        Filter::OwnedBy(PlayerRel::You),
        Filter::HasKeyword(crate::keywords::KeywordKind::Suspend),
        Filter::HasCounter(Some(crate::types::counters::TIME.into())),
    ]);
    Some(CostPart::RemoveCountersFromOne {
        kind: Some(kind),
        filter: Filter::Or(vec![
            Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
            suspended,
        ]),
        count: Value::c(1),
    })
}

inventory::submit! { super::CostPattern { name: "costs 5: remove a counter from a permanent you control or suspended card you own", priority: 60, parse: remove_counter_from_permanent_or_suspended } }

/// "Spells you cast cost {1} less to cast for each card type they share with cards exiled
/// with ~." (Cemetery Prowler; `kw/shared_card_types.rs`).
fn cost_less_per_shared_card_type(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let head = end(l).strip_suffix(" for each card type they share with cards exiled with ~")?;
    if !head.ends_with(" cost {1} less to cast") {
        return None;
    }
    let mut out =
        crate::oracle::statics::parse_static(&format!("{head} for each card exiled with ~"), ctx)?;
    let [a] = out.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Static(s) = &mut a.kind else {
        return None;
    };
    let StaticEffect::CostModifier(CostModifier {
        change: CostChange::ReduceGeneric(v),
        applies_to: CostTarget::Spells(_),
        ..
    }) = &mut s.effect
    else {
        return None;
    };
    *v = Value::Custom(crate::kw::shared_card_types::SHARED_WITH_LINKED_EXILE.into());
    a.text = text.into();
    Some(out)
}

inventory::submit! { super::StaticPattern { name: "costs 5: spells cost {1} less for each card type they share with cards exiled with ~", priority: 100, parse: cost_less_per_shared_card_type } }

/// "As an additional cost to cast ~, collect evidence X, where X is the total mana value of
/// the permanents ~ targets." (Urgent Necropsy, CR 701.59): X is the total of the targets'
/// mana values, determined once they're chosen (CR 601.2c-f).
fn collect_evidence_of_targets(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if t != "As an additional cost to cast ~, collect evidence X, where X is the total mana value of the permanents ~ targets."
    {
        return None;
    }
    let v = Value::Aggregate(AggOp::Sum, Stat::ManaValue, Box::new(Sel::AllTargets));
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::AdditionalCost(Cost {
            mana: None,
            parts: vec![CostPart::CollectEvidenceOf(v)],
        }),
        None,
        t,
    )])
}

inventory::submit! { AbilityPattern { name: "costs 5: collect evidence X, where X is the total mana value of the permanents ~ targets", priority: 80, parse: collect_evidence_of_targets } }
