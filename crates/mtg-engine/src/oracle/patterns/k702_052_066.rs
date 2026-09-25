//! Oracle text of the keywords of CR 702.52–702.66 that the generic keyword parser
//! doesn't handle, and phrases that go with them.

use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::parse_trigger_body;
use crate::oracle::keywords::compile_keyword;
use crate::oracle::patterns::{AbilityPattern, ConditionPattern, StaticPattern};
use crate::oracle::CompileContext;
use crate::types::counters;

/// Keyword lines with a cost the generic keyword parser doesn't understand.
fn keyword_line(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    // CR 702.59a: "Recover—Pay half your life, rounded up." (Garza's Assassin).
    if let Some(r) = lower.strip_prefix("recover—") {
        let cost = half_life_cost(r)?;
        let kw = Keyword::with_cost(KeywordKind::Recover, cost).text(t);
        return Some(compile_keyword(kw, t));
    }
    // "Suspend X—{X}{B}{B}{B}. X can't be 0." (Roiling Horror): suspend with X time
    // counters, X being paid in the suspend cost (see `kw/suspend.rs`).
    if let Some(r) = lower
        .strip_prefix("suspend x—")
        .and_then(|r| r.strip_suffix(". x can't be 0"))
    {
        let start = "suspend x—".len();
        let cost = crate::oracle::keywords::parse_keyword_cost(&t[start..start + r.len()])?;
        if !cost.mana.as_ref().is_some_and(|m| m.has_x()) {
            return None;
        }
        let mut kw = Keyword::with_cost(KeywordKind::Suspend, cost).text(t);
        kw.n = Some(-1);
        return Some(compile_keyword(kw, t));
    }
    // CR 702.56a: "Replicate—Pay {E}{E}{E}." (Reiterating Bolt).
    if let Some(r) = lower.strip_prefix("replicate—pay ") {
        let symbols = &t[t.len() - r.len()..];
        if !symbols.starts_with('{') {
            return None;
        }
        let (cost, _) = crate::oracle::costs::parse_cost(symbols)?;
        let kw = Keyword::with_cost(KeywordKind::Replicate, cost).text(t);
        return Some(compile_keyword(kw, t));
    }
    None
}

/// "pay half your life, rounded up/down" as a cost (CR 107.1a, 119.4).
fn half_life_cost(s: &str) -> Option<Cost> {
    let up = match s.trim() {
        "pay half your life, rounded up" => true,
        "pay half your life, rounded down" => false,
        _ => return None,
    };
    Some(Cost::free().with(CostPart::PayLife(Value::Div(
        Box::new(Value::LifeTotal(PlayerRef::You)),
        2,
        up,
    ))))
}

inventory::submit! { AbilityPattern { name: "k702_052_066 keywords", priority: 100, parse: keyword_line } }

/// "Each [quality] spell you cast has replicate. The replicate cost is equal to its mana
/// cost." (Hatchery Sliver, Djinn Illuminatus): the spells have replicate as they're cast
/// (CR 702.56a), with a cost that is the spell's own mana cost (see `kw/replicate.rs`).
fn spells_have_replicate(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let subject = l
        .strip_prefix("each ")?
        .strip_suffix(
            " spell you cast has replicate. the replicate cost is equal to its mana cost",
        )?;
    let mut parts = vec![spell_quality(subject)?];
    parts.push(Filter::Spell);
    parts.push(Filter::ControlledBy(PlayerRel::You));
    let kw = Keyword::with_cost(
        KeywordKind::Replicate,
        crate::kw::replicate::its_mana_cost(),
    )
    .text("replicate");
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(parts),
        mods: vec![Modification::AddKeyword(kw)],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// The quality of "each [quality] spell": "instant and sorcery" (either type), or one card
/// type or subtype ("Sliver", "Saga", "creature").
fn spell_quality(subject: &str) -> Option<Filter> {
    let types: Vec<&str> = subject.split(" and ").collect();
    let mut fs = Vec::new();
    for t in &types {
        let t = t.trim();
        if t.contains(' ') {
            return None;
        }
        let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(t)?;
        if !crate::oracle::phrases::end(tail).is_empty() {
            return None;
        }
        fs.push(f);
    }
    Some(if fs.len() == 1 {
        fs.pop()?
    } else {
        Filter::Or(fs)
    })
}

inventory::submit! { StaticPattern { name: "each [quality] spell you cast has replicate", priority: 100, parse: spells_have_replicate } }

/// "Spells you cast have ripple N." (Thrumming Stone): the spells have ripple as they're
/// cast, so it triggers when they're cast (CR 702.60a).
fn spells_have_ripple(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let n: i32 = l.strip_prefix("spells you cast have ripple ")?.parse().ok()?;
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(vec![Filter::Spell, Filter::ControlledBy(PlayerRel::You)]),
        mods: vec![Modification::AddKeyword(
            Keyword::with_n(KeywordKind::Ripple, n).text(format!("ripple {n}")),
        )],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spells you cast have ripple N", priority: 100, parse: spells_have_ripple } }

/// "~ enters with a +1/+1 counter on it for each [quality] card exiled with it" (Murktide
/// Regent): the cards exiled to pay for its spell with delve (CR 702.66a).
fn enters_with_counter_per_delved_card(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let quality = lower
        .trim_end_matches('.')
        .strip_prefix("~ enters with a +1/+1 counter on it for each ")?
        .strip_suffix(" card exiled with it")?;
    let filter = Filter::And(vec![
        Filter::InZone(ZoneKind::Exile),
        spell_quality(quality)?,
        Filter::Custom(crate::kw::delve::EXILED_WITH_IT.into()),
    ]);
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::EnterWithCounters(
            counters::PLUS1.into(),
            Value::Count(filter),
        ),
        self_replacement: false,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "enters with a counter for each card exiled with it", priority: 100, parse: enters_with_counter_per_delved_card } }

/// "Spells you cast have delve." (Teval, Arbiter of Virtue): the spells have delve as
/// they're cast, so it applies to their total cost (CR 702.66a).
fn spells_have_delve(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if l != "spells you cast have delve" {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(vec![Filter::Spell, Filter::ControlledBy(PlayerRel::You)]),
        mods: vec![Modification::AddKeyword(
            Keyword::new(KeywordKind::Delve).text("delve"),
        )],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spells you cast have delve", priority: 100, parse: spells_have_delve } }

/// "Whenever a time counter is removed from ~ while it's exiled, [effect]" (suspend X
/// cards such as Roiling Horror): a triggered ability that functions in exile and
/// triggers once for each time counter removed.
fn time_counter_removed_while_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let rest =
        lower.strip_prefix("whenever a time counter is removed from ~ while it's exiled, ")?;
    let eff = &t[t.len() - rest.len()..];
    let body = parse_trigger_body(eff, ctx, Sel::This, PlayerRef::You)?;
    let mut tr = TriggeredAbility::new(
        TriggerCond::Custom(crate::kw::suspend::TIME_COUNTER_REMOVED.into()),
        body,
    );
    tr.zone = FunctionZone::Exile;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "whenever a time counter is removed from ~ while it's exiled", priority: 100, parse: time_counter_removed_while_exiled } }

/// "When the creature ~ haunts dies, [effect]" and "When ~ enters or the creature it
/// haunts dies, [effect]" (cards with haunt): the "creature it haunts" part functions in
/// exile, where the card haunts that creature (CR 702.55b–c); the "enters" part is an
/// ordinary enters ability of the permanent. They're two abilities with the same effect.
fn haunted_creature_dies(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let (enters, rest) = if let Some(r) = lower.strip_prefix("when the creature ~ haunts dies, ") {
        (false, r)
    } else if let Some(r) = lower.strip_prefix("when ~ enters or the creature it haunts dies, ") {
        (true, r)
    } else {
        return None;
    };
    let eff = &t[t.len() - rest.len()..];
    let body = parse_trigger_body(eff, ctx, Sel::TriggerObject, PlayerRef::TriggerPlayer)?;
    let mut out = Vec::new();
    if enters {
        let etb = TriggeredAbility::new(TriggerCond::EntersBattlefield(Filter::Source), body.clone());
        out.push(AbilityDef::new(AbilityKind::Triggered(etb), t));
    }
    let mut dies = TriggeredAbility::new(
        TriggerCond::Dies(Filter::Custom(crate::kw::haunt::HAUNTED.into())),
        body,
    );
    dies.zone = FunctionZone::Exile;
    out.push(AbilityDef::new(AbilityKind::Triggered(dies), t));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "when the creature ~ haunts dies", priority: 100, parse: haunted_creature_dies } }

/// "When ~ is put into your hand from your graveyard, [effect]" (Golgari Brownscale, a
/// dredge card): a leaves-the-graveyard ability, which functions in the graveyard and
/// looks back in time (CR 603.10a). It triggers however the card gets there.
fn put_into_hand_from_graveyard(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let rest = lower.strip_prefix("when ~ is put into your hand from your graveyard, ")?;
    let eff = &t[t.len() - rest.len()..];
    let body = parse_trigger_body(eff, ctx, Sel::TriggerObject, PlayerRef::You)?;
    let mut tr = TriggeredAbility::new(
        TriggerCond::ZoneChange {
            filter: Filter::Source,
            from: Some(ZoneKind::Graveyard),
            to: Some(ZoneKind::Hand),
        },
        body,
    );
    tr.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "when ~ is put into your hand from your graveyard", priority: 100, parse: put_into_hand_from_graveyard } }

/// "it had no time counters on it" (vanishing creatures' "When ~ dies, if it had no time
/// counters on it, ..."): the permanent as it last existed on the battlefield (the dies
/// trigger's source is that object).
fn had_no_counters(l: &str) -> Option<Condition> {
    let kind = l
        .strip_prefix("it had no ")?
        .strip_suffix(" counters on it")?
        .trim();
    if kind.is_empty() || kind.contains(' ') {
        return None;
    }
    let kind = if kind == "+1/+1" { counters::PLUS1 } else { kind };
    Some(Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(kind.into())),
        Cmp::Eq,
        Value::c(0),
    ))
}

inventory::submit! { ConditionPattern { name: "it had no [kind] counters on it", priority: 100, parse: had_no_counters } }
