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

/// The untapped artifacts and creatures `p` controls.
fn tappable(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| {
            o.controller == p && !o.tapped && (o.is_creature() || o.is(CardType::Artifact))
        })
        .map(|o| o.id)
        .collect()
}

/// Whether `p` could pay "waterbend {n}".
pub fn can_waterbend(g: &Game, p: PlayerId, n: u32, ctx: &Ctx) -> bool {
    let rest = n.saturating_sub(tappable(g, p).len() as u32);
    rest == 0 || g.can_pay_cost(p, &Cost::mana(ManaCost::generic(rest)), ctx.source, ctx)
}

/// `p` pays "waterbend {n}" (CR 701.67a). Returns false if they couldn't.
pub fn waterbend(g: &mut Game, p: PlayerId, n: u32, ctx: &Ctx) -> bool {
    if g.dirty {
        g.recompute();
    }
    let cands = tappable(g, p);
    let max = (n as usize).min(cands.len());
    let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
    let chosen: Vec<ObjectId> = if max == 0 {
        vec![]
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
                v.iter().filter_map(|e| e.object()).collect()
            }
            // By default, tap only what the mana available can't pay, sparing the source.
            _ => {
                let mut need = 0;
                while need < max
                    && !g.can_pay_cost(
                        p,
                        &Cost::mana(ManaCost::generic(n - need as u32)),
                        ctx.source,
                        ctx,
                    )
                {
                    need += 1;
                }
                let mut order = cands.clone();
                order.sort_by_key(|c| Some(*c) == ctx.source);
                order.into_iter().take(need).collect()
            }
        }
    };
    let mut tapped = 0;
    for c in chosen {
        if g.tap(c) {
            tapped += 1;
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
