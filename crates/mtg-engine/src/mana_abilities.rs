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
                let c = ctx
                    .source
                    .and_then(|s| g.obj(s).choices.color)
                    .map(ManaType::from_color)
                    .unwrap_or(ManaType::C);
                vec![vec![c]; g.eval_value(n, ctx).max(0) as usize]
            }
            ManaProduction::CouldProduce(f) => {
                let t = types_could_produce(g, f, ctx);
                if t.is_empty() {
                    vec![]
                } else {
                    vec![t]
                }
            }
            ManaProduction::CouldProduceColor(f) => {
                let t: Vec<ManaType> = types_could_produce(g, f, ctx)
                    .into_iter()
                    .filter(|t| *t != ManaType::C)
                    .collect();
                if t.is_empty() {
                    vec![]
                } else {
                    vec![t]
                }
            }
            ManaProduction::AnyTypeProduced => {
                let t = mask_types(ctx.event.as_ref().map_or(0, |e| e.amount));
                if t.is_empty() {
                    vec![]
                } else {
                    vec![t]
                }
            }
            ManaProduction::ManaCostOf(sel) => {
                let mut units = Vec::new();
                if let Some(o) = g.eval_sel_objects(sel, ctx).first() {
                    if let Some(mc) = &g.obj(*o).chars.mana_cost {
                        for s in &mc.symbols {
                            units.extend(symbol_units(*s));
                        }
                    }
                }
                units
            }
            ManaProduction::DoubleUnspent => {
                let pool = &g.player(ctx.controller).mana_pool;
                ManaType::ALL
                    .iter()
                    .flat_map(|t| std::iter::repeat_n(vec![*t], pool.count(*t)))
                    .collect()
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
        Effect::AddManaWithSpentTrigger { add, .. } => production_units(g, add, ctx),
        _ => None,
    }
}

/// Mana types that permanents matching `f` could produce (CR 106.7): any type an ability
/// of that permanent would produce if it resolved now, taking replacement effects into
/// account in any order and ignoring whether its costs could be paid.
pub fn types_could_produce(g: &Game, f: &Filter, ctx: &Ctx) -> Vec<ManaType> {
    let mut path = Vec::new();
    let mut out: Vec<ManaType> = Vec::new();
    for o in g.objects_matching(f, ctx) {
        for t in could_produce_of(g, o, &mut path) {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    out.sort();
    out
}

/// Types one permanent could produce. `path` holds the permanents whose "could produce"
/// abilities are being evaluated, so that mutually referential abilities (two Exotic
/// Orchards) don't recurse forever: a permanent can't help itself produce mana.
fn could_produce_of(g: &Game, o: ObjectId, path: &mut Vec<ObjectId>) -> Vec<ManaType> {
    if path.contains(&o) {
        return vec![];
    }
    path.push(o);
    let mut out: Vec<ManaType> = Vec::new();
    for a in &g.obj(o).chars.abilities {
        let AbilityKind::Activated(act) = &a.kind else {
            continue;
        };
        if !act.is_mana_ability {
            continue;
        }
        let c = Ctx::new(Some(o), g.obj(o).controller);
        let mut types: Vec<ManaType> = Vec::new();
        collect_could_produce(g, &act.body.effect, &c, path, &mut types);
        if act.cost.has_tap() && !types.is_empty() {
            // Replacement effects that apply when it's tapped for mana (CR 106.7, 106.12b).
            types = replaced_types_any_order(g, o, &types);
        }
        for t in types {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    path.pop();
    out
}

fn collect_could_produce(
    g: &Game,
    e: &Effect,
    c: &Ctx,
    path: &mut Vec<ObjectId>,
    out: &mut Vec<ManaType>,
) {
    let push = |t: ManaType, out: &mut Vec<ManaType>| {
        if !out.contains(&t) {
            out.push(t);
        }
    };
    match e {
        Effect::AddMana {
            mana: ManaProduction::CouldProduce(f),
            ..
        }
        | Effect::AddMana {
            mana: ManaProduction::CouldProduceColor(f),
            ..
        } => {
            let colors_only = matches!(
                e,
                Effect::AddMana {
                    mana: ManaProduction::CouldProduceColor(_),
                    ..
                }
            );
            for x in g.objects_matching(f, c) {
                for t in could_produce_of(g, x, path) {
                    if !(colors_only && t == ManaType::C) {
                        push(t, out);
                    }
                }
            }
        }
        Effect::Seq(v) => {
            for x in v {
                collect_could_produce(g, x, c, path, out);
            }
        }
        Effect::ChooseOne { options, .. } => {
            for (_, x) in options {
                collect_could_produce(g, x, c, path, out);
            }
        }
        Effect::If {
            then, otherwise, ..
        } => {
            collect_could_produce(g, then, c, path, out);
            collect_could_produce(g, otherwise, c, path, out);
        }
        other => {
            if let Some(units) = production_units(g, other, c) {
                for u in units {
                    for t in u {
                        push(t, out);
                    }
                }
            }
        }
    }
}

/// Bitmask of mana types (bit i = `ManaType::ALL[i]`), used to carry the types a
/// permanent produced in a tapped-for-mana trigger's event info.
pub fn mana_type_mask(types: &[ManaType]) -> i32 {
    let mut m = 0;
    for t in types {
        if let Some(i) = ManaType::ALL.iter().position(|x| x == t) {
            m |= 1 << i;
        }
    }
    m
}

pub fn mask_types(mask: i32) -> Vec<ManaType> {
    ManaType::ALL
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, t)| *t)
        .collect()
}

/// The ways one mana symbol could be added to a mana pool (CR 106.8–106.11): each inner
/// vector is one unit of mana and the types it could be.
pub fn symbol_units(s: ManaSymbol) -> Vec<Vec<ManaType>> {
    let col = ManaType::from_color;
    match s {
        ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => vec![vec![col(c)]],
        ManaSymbol::Generic(n) => vec![vec![ManaType::C]; n as usize],
        ManaSymbol::Colorless | ManaSymbol::Snow => vec![vec![ManaType::C]],
        ManaSymbol::Hybrid(a, b) | ManaSymbol::PhyrexianHybrid(a, b) => {
            vec![vec![col(a), col(b)]]
        }
        ManaSymbol::ColorlessHybrid(c) => vec![vec![ManaType::C, col(c)]],
        ManaSymbol::TwoHybrid(c) => vec![vec![col(c), ManaType::C]],
        // X is 0 off the stack (CR 107.3g); other variable/unusual symbols add nothing.
        _ => vec![],
    }
}

/// The mana replacement effects (CR 106.12b) that apply when `perm` is tapped for mana:
/// (source, controller, definition), in timestamp order.
fn produce_mana_replacements(
    g: &Game,
    perm: ObjectId,
) -> Vec<(ObjectId, PlayerId, ReplacementDef)> {
    let mut v: Vec<(u64, ObjectId, PlayerId, ReplacementDef)> = g
        .statics
        .replacements
        .iter()
        .filter_map(|(s, c, ts, _, d)| match &d.event {
            ReplacementEvent::ProduceMana(f) if g.matches(perm, f, &Ctx::new(Some(*s), *c)) => {
                Some((*ts, *s, *c, d.clone()))
            }
            _ => None,
        })
        .collect();
    v.sort_by_key(|x| x.0);
    v.into_iter().map(|(_, s, c, d)| (s, c, d)).collect()
}

/// Applies one mana replacement to the produced types.
fn apply_mana_replacement(d: &ReplacementDef, types: &[ManaType]) -> Vec<ManaType> {
    match &d.action {
        // "it produces twice/three times as much of that mana instead".
        ReplacementAction::Multiply(k) => types
            .iter()
            .flat_map(|t| std::iter::repeat_n(*t, (*k).max(0) as usize))
            .collect(),
        // "it produces {B} instead of any other type and amount".
        ReplacementAction::Instead(e) => match &**e {
            Effect::AddMana {
                mana: ManaProduction::Fixed(v),
                ..
            } => v.clone(),
            _ => types.to_vec(),
        },
        _ => types.to_vec(),
    }
}

/// Union of the types produced after applying the replacements in every possible order
/// (for "could produce", CR 106.7).
fn replaced_types_any_order(g: &Game, perm: ObjectId, types: &[ManaType]) -> Vec<ManaType> {
    let reps = produce_mana_replacements(g, perm);
    if reps.is_empty() {
        return types.to_vec();
    }
    let n = reps.len().min(4);
    let mut out: Vec<ManaType> = Vec::new();
    let mut idx: Vec<usize> = (0..n).collect();
    permute(&mut idx, 0, &mut |order| {
        let mut ts = types.to_vec();
        for i in order {
            ts = apply_mana_replacement(&reps[*i].2, &ts);
        }
        for t in ts {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    });
    out
}

fn permute(v: &mut Vec<usize>, k: usize, f: &mut dyn FnMut(&[usize])) {
    if k == v.len() {
        f(v);
        return;
    }
    for i in k..v.len() {
        v.swap(k, i);
        permute(v, k + 1, f);
        v.swap(k, i);
    }
}

/// "[Player] activates a mana ability of each [filter] they control" (Drain Power): for
/// each such permanent with a mana ability that can be activated, the player chooses one
/// and activates it.
pub fn activate_mana_abilities_of_each(g: &mut Game, who: &PlayerRef, filter: &Filter, ctx: &Ctx) {
    for p in g.eval_players(who, ctx) {
        let perms: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.controller == p)
            .map(|o| o.id)
            .filter(|id| g.matches(*id, filter, ctx))
            .collect();
        for perm in perms {
            if !g.is_live(perm) || g.obj(perm).zone != Zone::Battlefield {
                continue;
            }
            let usable: Vec<Ability> = g
                .obj(perm)
                .chars
                .abilities
                .iter()
                .filter(|a| match &a.kind {
                    AbilityKind::Activated(act) => {
                        act.is_mana_ability && g.can_activate(p, perm, a, act)
                    }
                    _ => false,
                })
                .cloned()
                .collect();
            if usable.is_empty() {
                continue;
            }
            let i = g.ask_option(
                p,
                Some(perm),
                "Choose a mana ability to activate",
                usable.iter().map(|a| a.text.clone()).collect(),
            );
            let _ = g.activate_ability(p, perm, usable[i.min(usable.len() - 1)].uid);
        }
    }
}

/// "[Player] loses all unspent mana [and you add the mana lost this way]" (CR 106.13).
/// The mana moves with its sources, restrictions, and riders unchanged.
pub fn lose_unspent_mana(g: &mut Game, who: &PlayerRef, to: Option<&PlayerRef>, ctx: &Ctx) {
    let players = g.eval_players(who, ctx);
    let dest = to.and_then(|r| g.eval_player(r, ctx));
    let mut lost: Vec<Mana> = Vec::new();
    for p in players {
        lost.extend(std::mem::take(&mut g.players[p.idx()].mana_pool.mana));
    }
    if let Some(d) = dest {
        if !lost.is_empty() {
            for m in lost {
                g.players[d.idx()].mana_pool.add(m);
            }
            g.emit(crate::events::Event::ManaAdded {
                player: d,
                source: ctx.source,
            });
        }
    }
}

/// Resolves an `Effect::AddMana` (CR 106.3–106.12): determines the mana produced, applies
/// replacement effects if a permanent is being tapped for mana (CR 106.12b; restrictions
/// apply to all the mana produced, CR 106.6a), adds it to the player's mana pool
/// (CR 106.4), and reports the permanent as tapped for mana (CR 106.12a).
pub fn resolve_add_mana(
    g: &mut Game,
    who: &PlayerRef,
    mana: &ManaProduction,
    restriction: &Option<ManaRestriction>,
    ctx: &Ctx,
) {
    add_mana_with(g, who, mana, restriction, None, ctx);
}

/// Resolves `Effect::AddManaWithSpentTrigger` (CR 106.6): each unit of mana produced by
/// the inner `AddMana` carries its own delayed triggered ability (CR 106.6a).
pub fn resolve_add_mana_with_rider(
    g: &mut Game,
    add: &Effect,
    spell_filter: &Filter,
    body: &Body,
    ctx: &mut Ctx,
) {
    match add {
        Effect::AddMana {
            who,
            mana,
            restriction,
        } => {
            let rider = ManaRider {
                id: 0,
                spell_filter: spell_filter.clone(),
                body: body.clone(),
                controller: ctx.controller,
                source: ctx.source,
            };
            add_mana_with(g, who, mana, restriction, Some(rider), ctx);
        }
        other => g.exec(other, ctx),
    }
}

fn add_mana_with(
    g: &mut Game,
    who: &PlayerRef,
    mana: &ManaProduction,
    restriction: &Option<ManaRestriction>,
    rider: Option<ManaRider>,
    ctx: &Ctx,
) {
    let p = g.eval_player(who, ctx).unwrap_or(ctx.controller);
    let mut produced = g.produce_mana(p, mana, ctx);
    // Tapped for mana: a mana ability of this permanent with {T} in its cost is resolving.
    let tapped = g
        .mana_ability_resolving
        .filter(|s| Some(*s) == ctx.source && g.obj(*s).zone == Zone::Battlefield);
    if let (Some(perm), false) = (tapped, produced.is_empty()) {
        let mut reps = produce_mana_replacements(g, perm);
        // CR 616.1: the affected player chooses the order.
        while !reps.is_empty() {
            let i = if reps.len() == 1 {
                0
            } else {
                let options = reps
                    .iter()
                    .map(|(s, _, _)| g.obj(*s).chars.name.to_string())
                    .collect();
                match g.ask(p, crate::decision::Decision::ChooseReplacement { options }) {
                    crate::decision::Answer::Index(i) if i < reps.len() => i,
                    _ => 0,
                }
            };
            let (_, _, d) = reps.remove(i);
            produced = apply_mana_replacement(&d, &produced);
        }
    }
    let snow = ctx
        .source
        .is_some_and(|s| g.obj(s).chars.has_supertype(Supertype::Snow));
    let units: Vec<Mana> = produced
        .iter()
        .map(|t| Mana {
            ty: *t,
            snow,
            source: ctx.source,
            restriction: restriction.clone(),
            persistent: false,
            // A separate delayed triggered ability for each mana (CR 106.6a).
            rider: rider.as_ref().map(|r| {
                Box::new(ManaRider {
                    id: crate::ability::next_ability_uid(),
                    ..r.clone()
                })
            }),
        })
        .collect();
    if units.is_empty() {
        // CR 106.5: mana of an undefined type isn't produced.
        return;
    }
    g.add_mana(p, units, ctx.source);
    if let Some(perm) = tapped {
        g.emit(crate::events::Event::TappedForMana {
            obj: perm,
            player: ctx.controller,
            types: produced,
        });
    }
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
            let ctx = Ctx::new(Some(o.id), p);
            if let Some(units) = production_units(g, &act.body.effect, &ctx) {
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
    let mut units: Vec<Unit> = Vec::new();
    for (i, m) in g.player(p).mana_pool.mana.iter().enumerate() {
        units.push(Unit {
            types: vec![m.ty],
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
                types: u.clone(),
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
    let mut riders = Vec::new();
    for i in idxs {
        let m = g.players[p.idx()].mana_pool.mana.remove(i);
        spent.push(m.ty);
        if let Some(r) = m.rider {
            riders.push(r);
        }
    }
    // "When that mana is spent to cast ..." (CR 106.6): the delayed triggers trigger now
    // and are put on the stack the next time a player would receive priority.
    if let (true, Some(spell)) = (spend.is_spell, spend.source) {
        for r in riders.into_iter().rev() {
            let rctx = Ctx::new(r.source, r.controller);
            if !g.matches(spell, &r.spell_filter, &rctx) {
                continue;
            }
            g.trigger_order += 1;
            let ability = AbilityDef::new(
                AbilityKind::Triggered(TriggeredAbility::new(
                    TriggerCond::Custom("mana spent".into()),
                    r.body.clone(),
                )),
                "When that mana is spent",
            );
            let order = g.trigger_order;
            g.pending_triggers.push(crate::game::PendingTrigger {
                source: r.source.unwrap_or(spell),
                controller: r.controller,
                ability,
                event: crate::object::EventInfo {
                    object: Some(spell),
                    spell: Some(spell),
                    player: Some(p),
                    ..Default::default()
                },
                source_lki: None,
                saved: None,
                body: None,
                order,
            });
        }
    }
    Some(spent)
}
