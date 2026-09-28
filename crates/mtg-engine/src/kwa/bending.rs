//! CR 701.65 airbend, 701.66 earthbend, 701.67 waterbend.
//!
//! * Airbending exiles the objects; for each card exiled this way, for as long as it
//!   remains exiled, its owner may cast it by paying {2} rather than its mana cost
//!   (CR 701.65a): the cards are recorded in `KwaState::airbent`, and [`AirbentCasting`]
//!   offers the casting option. "Whenever [a player] airbends" triggers when the player
//!   exiles one or more objects this way (CR 701.65b).
//! * "Earthbend N": target land you control becomes a 0/0 land creature with haste in
//!   addition to its other types, gets N +1/+1 counters, and a delayed triggered ability
//!   returns it to the battlefield tapped under your control when it dies or is put into
//!   exile (CR 701.66a). "Whenever [a player] earthbends" triggers as that delayed
//!   ability is created (CR 701.66b).
//! * "Waterbend [cost]": pay the cost, tapping untapped artifacts and creatures you
//!   control for any of its generic mana (CR 701.67a) — only the waterbend part of a total
//!   cost (CR 701.67b): a waterbend cost is a cost part of its own
//!   (`CostPart::Effect(KeywordAction::Waterbend)`). "Whenever [a player] waterbends"
//!   triggers whenever they pay a waterbend cost, however they paid it (CR 701.67c).

use super::*;
use crate::casting::CastOption;
use crate::decision::{Answer, Decision};
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::{ManaCost, SpendContext};
use crate::object::{CastMethod, FaceState, ObjKind};
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::counters;

/// `Event::Custom` names reported when a player airbends, earthbends, or waterbends.
pub const AIRBENT_EVENT: &str = "airbend";
pub const EARTHBENT_EVENT: &str = "earthbend";
pub const WATERBENT_EVENT: &str = "waterbend";

/// The `CastMethod::Alternative` id of casting an airbent card for {2}.
pub const AIRBEND_METHOD: u64 = 0x0A1B_E4D0_0701_0065;

// ---------------------------------------------------------------------------
// Airbend
// ---------------------------------------------------------------------------

/// `p` airbends `objs` (permanents and/or spells) (CR 701.65a). Returns the cards
/// exiled.
pub fn airbend(g: &mut Game, p: PlayerId, objs: &[ObjectId], ctx: &mut Ctx) -> Vec<ObjectId> {
    let moves: Vec<MoveEv> = objs
        .iter()
        .filter(|o| {
            g.is_live(**o) && matches!(g.obj(**o).zone, Zone::Battlefield | Zone::Stack)
        })
        .map(|o| MoveEv {
            obj: *o,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: crate::events::MoveCause::Exile,
            by: Some(p),
            etb: EtbInfo::default(),
            source: ctx.source,
        })
        .collect();
    let mut exiled_any = false;
    let mut cards = vec![];
    for new in g.move_objects(moves).into_iter().flatten() {
        if !g.is_live(new) || g.obj(new).zone != Zone::Exile {
            continue;
        }
        exiled_any = true;
        // Tokens and copies of spells cease to exist; only cards may be cast.
        if g.obj(new).kind == ObjKind::Card {
            g.kwa.airbent.push(new);
            cards.push(new);
        }
    }
    ctx.set_var(
        kvars::AIRBENT,
        cards.iter().map(|c| Entity::Object(*c)).collect(),
    );
    // CR 701.65b: one or more objects exiled.
    if exiled_any {
        g.log(|_| format!("{p} airbends"));
        emit(g, AIRBENT_EVENT, p, None, cards.len() as i32);
    }
    cards
}

pub struct Airbend;

impl KeywordActionRules for Airbend {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Airbend]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let objs = g.resolve_objects(a.what, ctx);
        let p = g.eval_player(a.who, ctx).unwrap_or(ctx.controller);
        airbend(g, p, &objs, ctx);
    }
}

inventory::submit! { KeywordActionRegistration(&Airbend) }

/// Casting airbent cards for {2} (CR 701.65a).
pub struct AirbentCasting;

impl crate::kw::KeywordRules for AirbentCasting {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Exile || o.owner != p || !g.kwa.airbent.contains(&card) {
            return vec![];
        }
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Alternative(AIRBEND_METHOD);
        opt.alt_cost = Some(Cost::mana(ManaCost::generic(2)));
        opt.tag = Some("airbend");
        vec![opt]
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&AirbentCasting) }

// ---------------------------------------------------------------------------
// Earthbend
// ---------------------------------------------------------------------------

/// `p` earthbends N targeting `land` (CR 701.66a–b).
pub fn earthbend(g: &mut Game, p: PlayerId, land: ObjectId, n: u32, ctx: &mut Ctx) {
    if !on_battlefield(g, land) {
        return;
    }
    let this = Sel::All(Filter::Objects(vec![land]));
    let mut c = ctx.clone();
    c.controller = p;
    g.exec(
        &Effect::Modify {
            what: this,
            mods: vec![
                Modification::AddTypes(vec![CardType::Creature]),
                Modification::SetPT(Some(Value::c(0)), Some(Value::c(0))),
                Modification::AddKeyword(Keyword::new(KeywordKind::Haste)),
            ],
            duration: Duration::Permanent,
        },
        &mut c,
    );
    g.add_counters(Entity::Object(land), counters::PLUS1, n, ctx.source);
    // "When that land dies or is put into exile, return it to the battlefield tapped
    // under your control."
    let it = Filter::Objects(vec![land]);
    let mut back = Destination::battlefield();
    back.tapped = true;
    back.controller = Some(PlayerRef::You);
    g.exec(
        &Effect::DelayedTrigger {
            trigger: TriggerCond::AnyOf(vec![
                TriggerCond::Dies(it.clone()),
                TriggerCond::ZoneChange {
                    filter: it,
                    from: Some(ZoneKind::Battlefield),
                    to: Some(ZoneKind::Exile),
                },
            ]),
            body: Box::new(Body::effect(Effect::Move {
                what: Sel::TriggerObject,
                to: back,
            })),
            once: true,
        },
        &mut c,
    );
    g.kwa.earthbent.push((land, p));
    g.log(|g| format!("{p} earthbends {n}: {}", g.describe(land)));
    emit(g, EARTHBENT_EVENT, p, Some(land), n as i32);
}

pub struct Earthbend;

impl KeywordActionRules for Earthbend {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Earthbend]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        let p = g.eval_player(a.who, ctx).unwrap_or(ctx.controller);
        for land in g.resolve_objects(a.what, ctx) {
            earthbend(g, p, land, n, ctx);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Earthbend) }

// ---------------------------------------------------------------------------
// Waterbend
// ---------------------------------------------------------------------------

/// The untapped artifacts and creatures `p` controls, split into those that couldn't make
/// mana now (`source` last among them) and those with a mana ability that taps or
/// sacrifices them (fewest mana first). Tapping one of the latter for waterbend pays {1}
/// but gives up the mana it would make: a permanent tapped for mana, or sacrificed for
/// mana, can't also be tapped for waterbend (CR 118.3, 701.67a).
fn tappable(g: &Game, p: PlayerId, source: Option<ObjectId>) -> (Vec<ObjectId>, Vec<ObjectId>) {
    let sources = crate::mana_abilities::mana_sources(g, p, None);
    let mana_made = |id: ObjectId| {
        sources
            .iter()
            .filter(|s| s.obj == id && (s.taps() || s.sacrifices()))
            .map(|s| s.units.len())
            .max()
    };
    let (mut free, mut makers): (Vec<ObjectId>, Vec<ObjectId>) = g
        .permanents()
        .filter(|o| {
            o.controller == p && !o.tapped && (o.is_creature() || o.is(CardType::Artifact))
        })
        .map(|o| o.id)
        .partition(|id| mana_made(*id).is_none());
    free.sort_by_key(|c| Some(*c) == source);
    makers.sort_by_key(|c| mana_made(*c));
    (free, makers)
}

/// Whether `p` could pay {n} with mana.
fn can_pay_generic(g: &Game, p: PlayerId, n: u32, ctx: &Ctx) -> bool {
    n == 0 || g.can_pay_cost(p, &Cost::mana(ManaCost::generic(n)), ctx.source, ctx)
}

/// Whether `p` could pay "waterbend {n}".
pub fn can_waterbend(g: &Game, p: PlayerId, n: u32, ctx: &Ctx) -> bool {
    let (free, makers) = tappable(g, p, ctx.source);
    let rest = n.saturating_sub(free.len() as u32);
    if can_pay_generic(g, p, rest, ctx) {
        return true;
    }
    if makers.is_empty() {
        return false;
    }
    // Tapping a permanent that could make mana helps only if its mana couldn't pay (a
    // restriction on spending it, say): try it on a copy of the game.
    let mut h = g.clone();
    let mut rest = rest;
    for c in makers {
        if can_pay_generic(&h, p, rest, ctx) {
            break;
        }
        if h.tap(c) {
            rest -= 1;
        }
    }
    can_pay_generic(&h, p, rest, ctx)
}

/// `p` pays "waterbend {n}" (CR 701.67a). Returns false if they couldn't.
pub fn waterbend(g: &mut Game, p: PlayerId, n: u32, ctx: &Ctx) -> bool {
    if g.dirty {
        g.recompute();
    }
    let (free, makers) = tappable(g, p, ctx.source);
    let cands: Vec<ObjectId> = free.into_iter().chain(makers).collect();
    let max = (n as usize).min(cands.len());
    let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
    let chosen: Option<Vec<ObjectId>> = if max == 0 {
        Some(vec![])
    } else {
        match g.ask(
            p,
            Decision::ChooseEntities {
                source: ctx.source,
                prompt: format!(
                    "Waterbend {{{n}}}: tap artifacts and creatures to pay its generic mana"
                ),
                candidates: entities.clone(),
                min: 0,
                max: max as u32,
            },
        ) {
            Answer::Entities(v)
                if v.len() <= max
                    && v.iter().all(|e| entities.contains(e))
                    && v.iter().collect::<std::collections::BTreeSet<_>>().len() == v.len() =>
            {
                let v: Vec<ObjectId> = v.iter().filter_map(|e| e.object()).collect();
                // The rest must still be payable with mana once these are tapped.
                let mut h = g.clone();
                for c in &v {
                    h.tap(*c);
                }
                can_pay_generic(&h, p, n - v.len() as u32, ctx).then_some(v)
            }
            _ => None,
        }
    };
    let mut tapped = 0;
    match chosen {
        Some(v) => {
            for c in v {
                if g.tap(c) {
                    tapped += 1;
                }
            }
        }
        // By default, tap only what the mana available can't pay: first artifacts and
        // creatures that couldn't make mana (sparing the source), then the others.
        None => {
            for c in cands {
                if tapped as usize >= max || can_pay_generic(g, p, n - tapped, ctx) {
                    break;
                }
                if g.tap(c) {
                    tapped += 1;
                }
            }
        }
    }
    let rest = n.saturating_sub(tapped);
    if rest > 0 {
        let spend = SpendContext {
            source: ctx.source,
            ..Default::default()
        };
        if crate::mana_abilities::pay_mana(g, p, &ManaCost::generic(rest), &spend, None)
            .is_none()
        {
            return false;
        }
    }
    g.log(|_| format!("{p} waterbends {{{n}}} (tapping {tapped})"));
    emit(g, WATERBENT_EVENT, p, None, n as i32);
    true
}

/// CR 601.2f, 701.67b: the generic mana of a waterbend cost is part of the total cost, so
/// a generic cost reduction that the rest of the total cost can't absorb reduces it.
/// Returns how much of `n` was used.
pub fn reduce_waterbend_generic(cost: &mut Cost, n: u32) -> u32 {
    let mut used = 0;
    for part in cost.parts.iter_mut() {
        let CostPart::Effect(e) = part else {
            continue;
        };
        if let Effect::KeywordAction {
            action: KeywordAction::Waterbend,
            n: Value::Const(k),
            ..
        } = &mut **e
        {
            let take = ((*k).max(0) as u32).min(n - used);
            *k -= take as i32;
            used += take;
        }
    }
    used
}

pub struct Waterbend;

impl KeywordActionRules for Waterbend {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Waterbend]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            waterbend(g, p, n, ctx);
        }
    }

    fn can_choose(&self, g: &Game, a: &Args, ctx: &Ctx) -> bool {
        let n = number(g, a.n, ctx);
        g.eval_players(a.who, ctx)
            .into_iter()
            .all(|p| can_waterbend(g, p, n, ctx))
    }
}

inventory::submit! { KeywordActionRegistration(&Waterbend) }
