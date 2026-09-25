//! Mana abilities (CR 605) and automatic mana payment.
//!
//! When a cost requires mana that isn't already in the pool, the engine plans which
//! mana abilities to activate (CR 601.2g) using a small backtracking search over the
//! player's available mana sources, then activates them and pays from the pool.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::*;
use crate::object::Zone;
use crate::types::*;

/// A mana ability the player could activate to help pay a cost.
#[derive(Clone, Debug)]
pub struct ManaSource {
    pub obj: ObjectId,
    pub ability: Ability,
    /// Each unit this source produces: the set of types that unit could be.
    pub units: Vec<Vec<ManaType>>,
    /// Lower is preferred (tapping lands before sacrificing Treasures, etc.).
    pub cost_rank: u8,
}

const ALL_COLORS: [ManaType; 5] = [
    ManaType::W,
    ManaType::U,
    ManaType::B,
    ManaType::R,
    ManaType::G,
];

/// Mana types an AddMana effect could produce, per unit.
fn production_units(g: &Game, e: &Effect, ctx: &Ctx) -> Option<Vec<Vec<ManaType>>> {
    match e {
        Effect::AddMana { mana, .. } => Some(match mana {
            ManaProduction::Fixed(v) => v.iter().map(|t| vec![*t]).collect(),
            ManaProduction::Amount(t, n) => vec![vec![*t]; g.eval_value(n, ctx).max(0) as usize],
            ManaProduction::AnyOneColor(n) | ManaProduction::AnyCombination(n) => {
                vec![ALL_COLORS.to_vec(); g.eval_value(n, ctx).max(0) as usize]
            }
            ManaProduction::OneOf(opts) => vec![opts.clone()],
            ManaProduction::ChosenColor(n) => {
                // CR 607.2d: the color chosen by the linked ability; CR 607.5a: no mana if
                // no color was chosen.
                match g
                    .source_choices(ctx)
                    .and_then(|c| c.color)
                    .map(ManaType::from_color)
                {
                    Some(c) => vec![vec![c]; g.eval_value(n, ctx).max(0) as usize],
                    None => vec![],
                }
            }
            ManaProduction::OneOfOrChosenColor(opts) => {
                let mut u = opts.clone();
                if let Some(c) = g
                    .source_choices(ctx)
                    .and_then(|c| c.color)
                    .map(ManaType::from_color)
                {
                    if !u.contains(&c) {
                        u.push(c);
                    }
                }
                vec![u]
            }
            ManaProduction::CouldProduce(f) => {
                let t = types_could_produce(g, f, ctx);
                if t.is_empty() {
                    vec![]
                } else {
                    vec![t]
                }
            }
            ManaProduction::AnyColorAmong(f) => {
                let mut cs = ColorSet::NONE;
                for o in g.objects_matching(f, ctx) {
                    cs = cs.union(g.obj(o).chars.colors);
                }
                let t: Vec<ManaType> = cs.iter().map(ManaType::from_color).collect();
                if t.is_empty() {
                    vec![]
                } else {
                    vec![t]
                }
            }
            // CR 106.12a: the types of mana the triggering mana ability produced.
            ManaProduction::AnyTypeProduced | ManaProduction::TypeProduced => {
                let t = crate::resolve::produced_types(ctx);
                if t.is_empty() {
                    vec![]
                } else {
                    vec![t]
                }
            }
        }),
        Effect::Seq(v) => {
            let mut out: Vec<Vec<ManaType>> = Vec::new();
            let mut any = false;
            for x in v {
                if let Some(u) = production_units(g, x, ctx) {
                    any = true;
                    out.extend(u);
                }
            }
            any.then_some(out)
        }
        Effect::ChooseOne { options, .. } => {
            // e.g. "Add {R} or {G}": union per unit position of the first option size.
            let mut merged: Option<Vec<Vec<ManaType>>> = None;
            for (_, o) in options {
                if let Some(u) = production_units(g, o, ctx) {
                    merged = Some(match merged {
                        None => u,
                        Some(mut m) => {
                            for (i, unit) in u.into_iter().enumerate() {
                                if i < m.len() {
                                    for t in unit {
                                        if !m[i].contains(&t) {
                                            m[i].push(t);
                                        }
                                    }
                                }
                            }
                            m
                        }
                    });
                }
            }
            merged
        }
        Effect::If { then, .. } => production_units(g, then, ctx),
        _ => None,
    }
}

/// Mana types that permanents matching `f` could produce (CR 106.7).
pub fn types_could_produce(g: &Game, f: &Filter, ctx: &Ctx) -> Vec<ManaType> {
    let mut out: Vec<ManaType> = Vec::new();
    for o in g.objects_matching(f, ctx) {
        for a in &g.obj(o).chars.abilities {
            if let AbilityKind::Activated(act) = &a.kind {
                if act.is_mana_ability {
                    let c = Ctx::new(Some(o), g.obj(o).controller);
                    // Avoid infinite recursion through CouldProduce chains.
                    if let Effect::AddMana {
                        mana: ManaProduction::CouldProduce(_),
                        ..
                    } = &act.body.effect
                    {
                        continue;
                    }
                    if let Some(units) = production_units(g, &act.body.effect, &c) {
                        for u in units {
                            for t in u {
                                if !out.contains(&t) {
                                    out.push(t);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Extra mana units that triggered mana abilities would add to `p`'s pool when `obj`
/// (producing `units`) is tapped for mana (CR 605.1b, 605.4a).
fn triggered_mana_units(
    g: &Game,
    p: PlayerId,
    obj: ObjectId,
    units: &[Vec<ManaType>],
) -> Vec<Vec<ManaType>> {
    let mut produced: Vec<ManaType> = Vec::new();
    for u in units {
        for t in u {
            if !produced.contains(t) {
                produced.push(*t);
            }
        }
    }
    let mut out = Vec::new();
    for s in g.permanents() {
        for a in &s.chars.abilities {
            let AbilityKind::Triggered(t) = &a.kind else {
                continue;
            };
            if !t.is_mana_ability {
                continue;
            }
            let TriggerCond::TappedForMana {
                who: tapper,
                filter: f,
            } = &t.trigger
            else {
                continue;
            };
            let mut ctx = Ctx::new(Some(s.id), s.controller);
            ctx.link = a.link;
            ctx.event = Some(crate::object::EventInfo {
                object: Some(obj),
                player: Some(p),
                amount: produced.len() as i32,
                mana: produced.clone(),
                ..Default::default()
            });
            if !g.player_rel_matches(*tapper, p, &ctx) || !g.matches(obj, f, &ctx) {
                continue;
            }
            // Only mana that goes to the paying player helps.
            let who = match &t.body.effect {
                Effect::AddMana { who, .. } => g.eval_player(who, &ctx),
                _ => None,
            };
            if who != Some(p) {
                continue;
            }
            if let Some(extra) = production_units(g, &t.body.effect, &ctx) {
                out.extend(extra);
            }
        }
    }
    out
}

/// Mana abilities the player could activate right now to pay a cost.
pub fn mana_sources(g: &Game, p: PlayerId, reserve: Option<ObjectId>) -> Vec<ManaSource> {
    let mut out = Vec::new();
    for o in g.permanents() {
        if Some(o.id) == reserve {
            continue;
        }
        for a in &o.chars.abilities {
            let AbilityKind::Activated(act) = &a.kind else {
                continue;
            };
            if !act.is_mana_ability {
                continue;
            }
            if o.controller != p && !act.any_player {
                continue;
            }
            // Only plan with abilities whose costs are simple to pay automatically.
            let mut rank = 0u8;
            let mut ok = true;
            for part in &act.cost.parts {
                match part {
                    CostPart::Tap => {
                        if o.tapped
                            || (o.is_creature()
                                && o.summoning_sick
                                && !o.has_keyword(KeywordKind::Haste))
                        {
                            ok = false;
                        }
                    }
                    CostPart::SacrificeSelf => rank = rank.max(3),
                    CostPart::PayLife(_) => rank = rank.max(2),
                    CostPart::RemoveCounters { kind, count } => {
                        let n = g.eval_value(count, &Ctx::new(Some(o.id), p)).max(0) as u32;
                        if o.counter(kind) < n {
                            ok = false;
                        }
                        rank = rank.max(2);
                    }
                    _ => ok = false,
                }
            }
            if act.cost.mana.as_ref().is_some_and(|m| !m.is_zero()) {
                // Mana abilities that cost mana (filters) aren't auto-planned.
                ok = false;
            }
            if let Some(c) = &act.condition {
                if !g.eval_cond(c, &Ctx::new(Some(o.id), p)) {
                    ok = false;
                }
            }
            if let Some(max) = act.max_per_turn {
                if o.activations_this_turn.get(&a.uid).copied().unwrap_or(0) >= max {
                    ok = false;
                }
            }
            if !ok {
                continue;
            }
            let mut ctx = Ctx::new(Some(o.id), p);
            ctx.link = a.link;
            if let Some(mut units) = production_units(g, &act.body.effect, &ctx) {
                // CR 605.4a: triggered mana abilities that trigger on tapping it for mana
                // add their mana right away, so they help pay too.
                if act.cost.has_tap() && !units.is_empty() {
                    let extra = triggered_mana_units(g, p, o.id, &units);
                    units.extend(extra);
                }
                if !units.is_empty() {
                    if o.is_creature() {
                        rank = rank.max(1);
                    }
                    out.push(ManaSource {
                        obj: o.id,
                        ability: a.clone(),
                        units,
                        cost_rank: rank,
                    });
                }
            }
        }
    }
    // Prefer cheap, less flexible sources first.
    out.sort_by_key(|s| {
        (
            s.cost_rank,
            s.units.iter().map(|u| u.len()).sum::<usize>(),
            s.obj,
        )
    });
    // One ability per permanent (tapping uses the permanent).
    let mut seen = Vec::new();
    out.retain(|s| {
        let taps = s.ability.kind_cost_has_tap();
        if taps {
            if seen.contains(&s.obj) {
                return false;
            }
            seen.push(s.obj);
        }
        true
    });
    out
}

trait CostTap {
    fn kind_cost_has_tap(&self) -> bool;
}
impl CostTap for AbilityDef {
    fn kind_cost_has_tap(&self) -> bool {
        match &self.kind {
            AbilityKind::Activated(a) => a.cost.has_tap(),
            _ => false,
        }
    }
}

/// Number of mana units the player could produce (pool excluded).
pub fn potential_mana_count(g: &Game, p: PlayerId, reserve: Option<ObjectId>) -> u32 {
    mana_sources(g, p, reserve)
        .iter()
        .map(|s| s.units.len() as u32)
        .sum()
}

#[derive(Clone, Copy, Debug)]
enum Req {
    Colored(Color),
    Colorless,
    Snow,
    Generic,
    Hybrid(Color, Color),
    TwoHybrid(Color),
    ColorlessHybrid(Color),
    Phyrexian(Color),
    PhyrexianHybrid(Color, Color),
}

fn expand(cost: &ManaCost) -> Option<Vec<Req>> {
    let mut v = Vec::new();
    for s in &cost.symbols {
        match *s {
            ManaSymbol::Generic(n) => v.extend(std::iter::repeat_n(Req::Generic, n as usize)),
            ManaSymbol::Colored(c) => v.push(Req::Colored(c)),
            ManaSymbol::Colorless => v.push(Req::Colorless),
            ManaSymbol::Snow => v.push(Req::Snow),
            ManaSymbol::Hybrid(a, b) => v.push(Req::Hybrid(a, b)),
            ManaSymbol::TwoHybrid(c) => v.push(Req::TwoHybrid(c)),
            ManaSymbol::ColorlessHybrid(c) => v.push(Req::ColorlessHybrid(c)),
            ManaSymbol::Phyrexian(c) => v.push(Req::Phyrexian(c)),
            ManaSymbol::PhyrexianHybrid(a, b) => v.push(Req::PhyrexianHybrid(a, b)),
            ManaSymbol::X | ManaSymbol::Y | ManaSymbol::Z | ManaSymbol::Half(_) => {}
            ManaSymbol::Infinity => return None,
        }
    }
    v.sort_by_key(|r| match r {
        Req::Colored(_) | Req::Colorless => 0,
        Req::Hybrid(..) | Req::ColorlessHybrid(_) => 1,
        Req::Snow => 2,
        Req::TwoHybrid(_) | Req::Phyrexian(_) | Req::PhyrexianHybrid(..) => 3,
        Req::Generic => 4,
    });
    Some(v)
}

/// A unit of mana available for planning: from the pool or from a source.
#[derive(Clone, Debug)]
struct Unit {
    types: Vec<ManaType>,
    snow: bool,
    /// Index into sources, or None for pool mana (with pool index).
    source: Option<usize>,
    pool_index: Option<usize>,
    restriction_ok: bool,
}

/// Plans which sources to activate. Returns (source index, chosen types per unit) for
/// each source to activate, or None if the cost can't be paid.
pub fn plan_payment(
    g: &Game,
    p: PlayerId,
    cost: &ManaCost,
    spend: &SpendContext,
    reserve: Option<ObjectId>,
) -> Option<Vec<(ManaSource, Vec<ManaType>)>> {
    let reqs = expand(cost)?;
    let sources = mana_sources(g, p, reserve);
    // Mana that may be spent as though it were mana of any color (CR 602.1e) can meet
    // any colored requirement.
    let widen = |mut types: Vec<ManaType>| {
        if types.iter().any(|t| spend.any_color.contains(t)) {
            for c in ALL_COLORS {
                if !types.contains(&c) {
                    types.push(c);
                }
            }
        }
        types
    };
    let mut units: Vec<Unit> = Vec::new();
    for (i, m) in g.player(p).mana_pool.mana.iter().enumerate() {
        units.push(Unit {
            types: widen(vec![m.ty]),
            snow: m.snow,
            source: None,
            pool_index: Some(i),
            restriction_ok: m.can_spend(spend),
        });
    }
    for (si, s) in sources.iter().enumerate() {
        let snow = g.obj(s.obj).chars.has_supertype(Supertype::Snow);
        for u in &s.units {
            units.push(Unit {
                types: widen(u.clone()),
                snow,
                source: Some(si),
                pool_index: None,
                restriction_ok: true,
            });
        }
    }
    let life = g.player(p).life.max(0) as u32;
    let mut assign: Vec<Option<(usize, ManaType)>> = vec![None; reqs.len()];
    let mut used = vec![false; units.len()];
    let mut extra_generic = 0usize;
    let mut life_used = 0u32;
    if !plan_rec(
        &reqs,
        0,
        &units,
        &mut used,
        &mut assign,
        &mut extra_generic,
        &mut life_used,
        life,
    ) {
        return None;
    }
    // Which sources are used, and what each unit should produce.
    let mut out: Vec<(ManaSource, Vec<ManaType>)> = Vec::new();
    for (si, s) in sources.iter().enumerate() {
        let unit_idxs: Vec<usize> = (0..units.len())
            .filter(|&u| units[u].source == Some(si))
            .collect();
        if unit_idxs.iter().any(|u| used[*u]) {
            let mut types = Vec::new();
            for (ri, a) in assign.iter().enumerate() {
                if let Some((u, t)) = a {
                    if unit_idxs.contains(u) {
                        types.push(*t);
                    }
                    let _ = ri;
                }
            }
            out.push((s.clone(), types));
        }
    }
    Some(out)
}

#[allow(clippy::too_many_arguments)]
fn plan_rec(
    reqs: &[Req],
    i: usize,
    units: &[Unit],
    used: &mut Vec<bool>,
    assign: &mut Vec<Option<(usize, ManaType)>>,
    extra_generic: &mut usize,
    life_used: &mut u32,
    life: u32,
) -> bool {
    if i == reqs.len() {
        // Extra generic from {2/X} paid generically.
        if *extra_generic == 0 {
            return true;
        }
        let free: Vec<usize> = (0..units.len())
            .filter(|u| !used[*u] && units[*u].restriction_ok)
            .collect();
        if free.len() < *extra_generic {
            return false;
        }
        for u in free.into_iter().take(*extra_generic) {
            used[u] = true;
        }
        return true;
    }
    let try_pred = |pred: &dyn Fn(&Unit) -> Option<ManaType>,
                    used: &mut Vec<bool>,
                    assign: &mut Vec<Option<(usize, ManaType)>>,
                    extra_generic: &mut usize,
                    life_used: &mut u32|
     -> bool {
        // Prefer pool units, then already-used sources' other units, then new sources.
        let mut cands: Vec<usize> = (0..units.len())
            .filter(|u| !used[*u] && units[*u].restriction_ok)
            .collect();
        cands.sort_by_key(|u| {
            let unit = &units[*u];
            let source_in_use = unit
                .source
                .is_some_and(|s| (0..units.len()).any(|x| used[x] && units[x].source == Some(s)));
            (unit.pool_index.is_none(), !source_in_use, unit.types.len())
        });
        for u in cands {
            if let Some(t) = pred(&units[u]) {
                used[u] = true;
                assign[i] = Some((u, t));
                if plan_rec(
                    reqs,
                    i + 1,
                    units,
                    used,
                    assign,
                    extra_generic,
                    life_used,
                    life,
                ) {
                    return true;
                }
                used[u] = false;
                assign[i] = None;
            }
        }
        false
    };
    let has = |u: &Unit, t: ManaType| u.types.contains(&t);
    let col = |c: Color| ManaType::from_color(c);
    match reqs[i] {
        Req::Colored(c) => try_pred(
            &|u| has(u, col(c)).then_some(col(c)),
            used,
            assign,
            extra_generic,
            life_used,
        ),
        Req::Colorless => try_pred(
            &|u| has(u, ManaType::C).then_some(ManaType::C),
            used,
            assign,
            extra_generic,
            life_used,
        ),
        Req::Snow => try_pred(
            &|u| {
                if u.snow {
                    u.types.first().copied()
                } else {
                    None
                }
            },
            used,
            assign,
            extra_generic,
            life_used,
        ),
        Req::Generic => try_pred(
            &|u| u.types.first().copied(),
            used,
            assign,
            extra_generic,
            life_used,
        ),
        Req::Hybrid(a, b) => try_pred(
            &|u| {
                if has(u, col(a)) {
                    Some(col(a))
                } else if has(u, col(b)) {
                    Some(col(b))
                } else {
                    None
                }
            },
            used,
            assign,
            extra_generic,
            life_used,
        ),
        Req::ColorlessHybrid(c) => try_pred(
            &|u| {
                if has(u, ManaType::C) {
                    Some(ManaType::C)
                } else if has(u, col(c)) {
                    Some(col(c))
                } else {
                    None
                }
            },
            used,
            assign,
            extra_generic,
            life_used,
        ),
        Req::TwoHybrid(c) => {
            if try_pred(
                &|u| has(u, col(c)).then_some(col(c)),
                used,
                assign,
                extra_generic,
                life_used,
            ) {
                return true;
            }
            *extra_generic += 2;
            if plan_rec(
                reqs,
                i + 1,
                units,
                used,
                assign,
                extra_generic,
                life_used,
                life,
            ) {
                return true;
            }
            *extra_generic -= 2;
            false
        }
        Req::Phyrexian(c) => {
            if try_pred(
                &|u| has(u, col(c)).then_some(col(c)),
                used,
                assign,
                extra_generic,
                life_used,
            ) {
                return true;
            }
            if *life_used + 2 <= life {
                *life_used += 2;
                if plan_rec(
                    reqs,
                    i + 1,
                    units,
                    used,
                    assign,
                    extra_generic,
                    life_used,
                    life,
                ) {
                    return true;
                }
                *life_used -= 2;
            }
            false
        }
        Req::PhyrexianHybrid(a, b) => {
            if try_pred(
                &|u| {
                    if has(u, col(a)) {
                        Some(col(a))
                    } else if has(u, col(b)) {
                        Some(col(b))
                    } else {
                        None
                    }
                },
                used,
                assign,
                extra_generic,
                life_used,
            ) {
                return true;
            }
            if *life_used + 2 <= life {
                *life_used += 2;
                if plan_rec(
                    reqs,
                    i + 1,
                    units,
                    used,
                    assign,
                    extra_generic,
                    life_used,
                    life,
                ) {
                    return true;
                }
                *life_used -= 2;
            }
            false
        }
    }
}

/// Pays a mana cost: activates planned mana abilities, then spends mana from the pool.
/// Returns the types of mana spent.
pub fn pay_mana(
    g: &mut Game,
    p: PlayerId,
    cost: &ManaCost,
    spend: &SpendContext,
    reserve: Option<ObjectId>,
) -> Option<Vec<ManaType>> {
    let life = g.player(p).life.max(0) as u32;
    let cant_pay_life = g.cant_lose_life(p);
    let max_life = if cant_pay_life { 0 } else { life };
    // Prefer paying from the pool if possible (without life for Phyrexian if mana suffices).
    let try_pool = |g: &Game, allow_life: u32| {
        find_payment(&g.player(p).mana_pool.mana, cost, spend, allow_life)
    };
    let plan_now = try_pool(g, 0);
    if plan_now.is_none() {
        let plan = plan_payment(g, p, cost, spend, reserve)?;
        for (src, types) in plan {
            // Triggered mana abilities (CR 605.4a, "whenever enchanted land is tapped for
            // mana, ... adds an additional {G}") may already have added enough.
            if try_pool(g, 0).is_some() {
                break;
            }
            if !g.is_live(src.obj) || g.obj(src.obj).zone != Zone::Battlefield {
                continue;
            }
            g.mana_hint = Some(types);
            let r = g.activate_ability(p, src.obj, src.ability.uid);
            g.mana_hint = None;
            if r.is_err() {
                return None;
            }
        }
    }
    let plan = try_pool(g, 0).or_else(|| try_pool(g, max_life))?;
    if plan.life > 0 && !g.pay_life(p, plan.life) {
        return None;
    }
    let mut spent = Vec::new();
    let mut idxs = plan.pool_indices.clone();
    idxs.sort_unstable_by(|a, b| b.cmp(a));
    for i in idxs {
        let m = g.players[p.idx()].mana_pool.mana.remove(i);
        spent.push(m.ty);
    }
    Some(spent)
}
