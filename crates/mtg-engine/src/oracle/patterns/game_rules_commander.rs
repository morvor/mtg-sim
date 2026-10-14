//! Commander rules (CR 903) in Oracle text: the command zone as a zone objects are
//! described in or come from ("a commander you own from the command zone", "all
//! commanders you own from the command zone and from your graveyard"), an ability that
//! functions in the command zone ("{1}{G}{W}{U}: Put ~ onto the battlefield from the
//! command zone.", CR 113.6), a flashback spell that costs less "to cast this way"
//! (CR 702.34a, 601.2f), and amounts of commander casts for another player.

use super::{AbilityPattern, EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// One zone an object is described as being in or coming from: "the command zone", "your
/// graveyard", "exile", "your hand".
fn zone_filter(s: &str) -> Option<(Filter, &str)> {
    for (p, f) in [
        ("the command zone", Filter::InZone(ZoneKind::Command)),
        (
            "your graveyard",
            Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ]),
        ),
        ("exile", Filter::InZone(ZoneKind::Exile)),
        (
            "your hand",
            Filter::and(vec![
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::You),
            ]),
        ),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((f, r));
            }
        }
    }
    None
}

/// "from the command zone", "in the command zone", "from the command zone and from your
/// graveyard" (either zone).
fn command_zone_suffix<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s
        .strip_prefix("from ")
        .or_else(|| s.strip_prefix("in "))?;
    let (first, mut rest) = zone_filter(r)?;
    let mut zones = vec![first];
    while let Some(r) = rest.strip_prefix(" and from ") {
        let (z, r) = zone_filter(r)?;
        zones.push(z);
        rest = r;
    }
    // Only phrases naming the command zone: the core reads the others.
    if !zones
        .iter()
        .any(|z| matches!(z, Filter::InZone(ZoneKind::Command)))
    {
        return None;
    }
    let f = if zones.len() == 1 {
        zones.pop()?
    } else {
        Filter::Or(zones)
    };
    Some((f, rest))
}

inventory::submit! { FilterSuffixPattern { name: "game rules: from the command zone", priority: 100, parse: command_zone_suffix } }

/// "[cost]: Put ~ onto the battlefield from the command zone." (Derevi): an activated
/// ability that functions only in the command zone (CR 113.6, 113.6b).
fn put_from_command_zone(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (cost_s, eff) = crate::oracle::split_cost(text)?;
    if end(&eff.to_lowercase()) != "put ~ onto the battlefield from the command zone" {
        return None;
    }
    let (cost, false) = crate::oracle::costs::parse_cost(cost_s)? else {
        return None;
    };
    let _ = ctx;
    let mut act = ActivatedAbility::new(
        cost,
        Body::effect(Effect::Move {
            what: Sel::This,
            to: Destination::battlefield(),
        }),
    );
    act.zone = FunctionZone::Command;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), text)])
}

inventory::submit! { AbilityPattern { name: "game rules: put ~ onto the battlefield from the command zone", priority: 100, parse: put_from_command_zone } }

/// "Flashback {8}{B}{B}. ~ costs {X} less to cast this way, where X is ...": the flashback
/// ability, and a cost reduction that applies only when the spell is cast with it
/// (CR 702.34a, 601.2f).
fn flashback_costs_less_this_way(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (kw, rest) = text.split_once(". ")?;
    if !kw.to_lowercase().starts_with("flashback") {
        return None;
    }
    if !rest.starts_with("~ costs ") || !rest.contains(" less to cast this way") {
        return None;
    }
    let reworded = rest.replacen(" less to cast this way", " less to cast", 1);
    let mut out = crate::oracle::keywords::parse_keyword_line(kw, ctx)?;
    let reduction = crate::oracle::parse_ability(&reworded, ctx)?;
    let [a] = reduction.as_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    if !matches!(
        &s.effect,
        StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            ..
        })
    ) || s.condition.is_some()
    {
        return None;
    }
    let mut s = s.clone();
    s.condition = Some(Condition::CostPaid(crate::kw::flashback::FLASHBACK.into()));
    out.push(AbilityDef::new(AbilityKind::Static(s), rest));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "game rules: flashback; costs less to cast this way", priority: 100, parse: flashback_costs_less_this_way } }

/// "Target player draws X cards plus an additional card for each time they've cast a
/// commander from the command zone this game.": the player draws, counting their own
/// commander casts (CR 903.8).
fn draws_plus_commander_casts(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, r) = l.split_once(" draws ")?;
    let (n, r) = parse_number(r)?;
    let r = r
        .strip_prefix("cards plus an additional card for each time ")?
        .strip_suffix(" cast a commander from the command zone this game")?;
    if !matches!(r, "they've" | "you've") {
        return None;
    }
    let (who, rest) = crate::oracle::effects::player_ref(subject, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(Effect::Draw {
            who: PlayerRef::You,
            n: Value::Sum(vec![
                n,
                Value::Custom(crate::kw::partner::COMMANDER_CASTS.into()),
            ]),
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: draws N plus a card for each commander cast", priority: 100, parse: draws_plus_commander_casts } }

/// "put a commander you own from the command zone onto the battlefield" (Hellkite
/// Courser): you choose one and it enters under your control; "it" is the permanent it
/// became (CR 400.7), as for a card put onto the battlefield from a hand.
fn put_from_command_zone_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    use super::sweep_long_tail_hand::PUT;
    let r = end(l).strip_prefix("put ")?;
    let (desc, tail) = r.split_once(" onto the battlefield")?;
    if !end(tail).is_empty() || !desc.contains(" from the command zone") {
        return None;
    }
    let desc = desc
        .strip_prefix("a ")
        .or_else(|| desc.strip_prefix("an "))?;
    let (f, _, rest) = parse_object_phrase(desc)?;
    if !end(rest).is_empty() || f.zone() != Some(ZoneKind::Command) {
        return None;
    }
    b.it = Sel::Var(PUT);
    Some(Effect::seq(vec![
        Effect::Move {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: f,
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            to: Destination::battlefield().under_your_control(),
        },
        Effect::Store {
            var: PUT,
            sel: Sel::Var(vars::IT),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "game rules: put [object] from the command zone onto the battlefield", priority: 100, parse: put_from_command_zone_effect } }

/// "Spend this mana only to cast your commander." (Jeweled Lotus): a spell that is a
/// commander its caster owns (CR 903.3: the commander designation stays with the card on
/// the stack; "your commander" is a commander you own). Checked against the spell on
/// the stack as its costs are paid (CR 106.6, 601.2h).
fn spend_only_on_your_commander(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l).trim() != "spend this mana only to cast your commander" {
        return false;
    }
    let r = crate::mana::ManaRestriction::CastSpell(crate::mana::SpendFilter::new(Filter::and(
        vec![Filter::Commander, Filter::OwnedBy(PlayerRel::You)],
    )));
    let mut e = prev.clone();
    if !super::mana_restrictions::restrict(&mut e, &r) {
        return false;
    }
    *prev = e;
    true
}

inventory::submit! { super::FollowupPattern { name: "game rules: spend this mana only to cast your commander", priority: 95, apply: spend_only_on_your_commander } }

/// "Spells your opponents cast that target one or more commanders you control cost {3}
/// more to cast.": a spell targets one or more of them exactly when it targets one
/// ("that target a commander you control").
fn that_target_one_or_more(l: &str, _text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (before, after) = end(l).split_once(" that target one or more ")?;
    let (noun, rest) = after.split_once(' ')?;
    let singular = noun.strip_suffix('s')?;
    let article = if singular.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    let reworded = format!("{before} that target {article} {singular} {rest}");
    crate::oracle::statics::parse_static(&reworded, ctx)
}

inventory::submit! { super::StaticPattern { name: "game rules: that target one or more [objects]", priority: 100, parse: that_target_one_or_more } }
