//! CR 702.122 Crew.
//!
//! * "Crew N" means "Tap any number of other untapped creatures you control with total
//!   power N or greater: This permanent becomes an artifact creature until end of turn."
//!   (CR 702.122a). The cost is a [`CostPart::TapTotalPower`], paid here
//!   ([`total_power_payable`], [`pay_total_power`]). "Crew N. Activate only once each
//!   turn." is the same keyword with that restriction (kept in its text).
//! * A creature "crews a Vehicle" when it's tapped to pay the cost of the Vehicle's crew
//!   ability (CR 702.122b); the Vehicle is then "crewed by" it (CR 702.122c). Those
//!   creatures are recorded for the turn with their characteristics as they crewed
//!   ([`CrewRecord`] in `TurnHistory::crewed`): "creature that crewed it this turn" (on
//!   the battlefield), "for each creature that crewed it this turn" (counting those that
//!   have left since, [`CREATURES_THAT_CREWED_IT`]), "if an Assassin crewed it this turn"
//!   (one that was an Assassin as it crewed, [`crewed_by_type`]). They're also the
//!   objects the crew ability's cost tapped (its saved context), which
//!   [`CREWS_A_VEHICLE`] triggers look at.
//! * "Whenever [this Vehicle] becomes crewed" means "whenever a crew ability of [this
//!   Vehicle] resolves" (CR 702.122e): the ability reports a [`CREWED`] event as it
//!   resolves, whose amount is the number of creatures tapped to pay for that ability.
//! * Effects that change what can be tapped for crew are static abilities named by
//!   `StaticEffect::Custom` (see [`cant_tap_for`], [`power_bonus`], [`uses_toughness`]):
//!   "Enchanted creature can't crew Vehicles" (CR 702.122d), "~ crews Vehicles as though
//!   its power were 2 greater", "~ crews Vehicles using its toughness rather than its
//!   power". The names carry the keyword they apply to, so the same statics can speak of
//!   saddling (CR 702.171a) too.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::Illegal;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{Characteristics, EventInfo, StackKind, Zone};
use crate::types::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A creature tapped to pay the cost of a Vehicle's crew ability: it crewed the Vehicle
/// (CR 702.122b–c).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewRecord {
    pub vehicle: ObjectId,
    pub creature: ObjectId,
    /// The creature's characteristics as it was tapped to pay that cost.
    pub chars: Box<Characteristics>,
}

/// Whether `creature` crewed `vehicle` this turn (CR 702.122c).
pub fn crewed_this_turn(g: &Game, vehicle: ObjectId, creature: ObjectId) -> bool {
    g.history
        .crewed
        .iter()
        .any(|r| r.vehicle == vehicle && r.creature == creature)
}

/// `Event::Custom` name: a crew ability of the Vehicle (`obj`) resolved — it "becomes
/// crewed" (CR 702.122e). The amount is the number of creatures that crewed it with that
/// ability.
pub const CREWED: &str = "crewed";
/// `Effect::Custom`: reports [`CREWED`] for the resolving crew ability's Vehicle.
const BECOME_CREWED: &str = "crew:becomes crewed";
/// `TriggerCond::Custom`: "whenever ~ crews a Vehicle" (CR 702.122b). The trigger object
/// is the Vehicle ("that Vehicle").
pub const CREWS_A_VEHICLE: &str = "crew:~ crews a vehicle";
/// `Filter::Custom`: "creature that crewed it this turn" — a creature tapped this turn to
/// pay for a crew ability of the source (CR 702.122c).
pub const CREWED_IT_THIS_TURN: &str = "crew:crewed it this turn";
/// `Filter::Custom`: "Vehicle crewed by ~ this turn" — a Vehicle the source crewed this
/// turn (CR 702.122c).
pub const CREWED_BY_IT_THIS_TURN: &str = "crew:crewed by it this turn";
/// `Value::Custom`: the number of creatures that crewed the source this turn, including
/// those that have left the battlefield since.
pub const CREATURES_THAT_CREWED_IT: &str = "crew:creatures that crewed it this turn";
/// Prefix of a `Condition::Custom` followed by a subtype ("if an Assassin crewed it this
/// turn"): a creature that had that subtype as it crewed the source this turn crewed it,
/// whether or not it's still on the battlefield or still has that subtype.
pub const CREWED_BY_TYPE: &str = "crew:crewed it this turn, a creature of subtype ";

/// The [`CREWED_BY_TYPE`] condition for `subtype`.
pub fn crewed_by_type(subtype: &str) -> SmolStr {
    SmolStr::new(format!("{CREWED_BY_TYPE}{subtype}"))
}

/// The variable of an activated ability's saved context holding the objects its cost
/// tapped, sacrificed, etc. (see `Game::activate_ability`).
const COST_OBJECTS: Var = vars::USER + 91;

/// `StaticEffect::Custom` name: the permanent this static ability's source is attached to
/// ("enchanted creature") can't be tapped to pay for `kw` ("can't crew Vehicles",
/// CR 702.122d).
pub fn cant_tap_for(kw: KeywordKind) -> SmolStr {
    SmolStr::new(format!("tap_total_power:{}:attached can't", kw.name()))
}

/// `StaticEffect::Custom` name: the source is tapped to pay for `kw` as though its power
/// were `n` greater ("~ crews Vehicles as though its power were 2 greater").
pub fn power_bonus(kw: KeywordKind, n: i32) -> SmolStr {
    SmolStr::new(format!("tap_total_power:{}:+{n}", kw.name()))
}

/// `StaticEffect::Custom` name: the source is tapped to pay for `kw` using its toughness
/// rather than its power.
pub fn uses_toughness(kw: KeywordKind) -> SmolStr {
    SmolStr::new(format!("tap_total_power:{}:toughness", kw.name()))
}

/// Whether `creature` may be tapped to pay a [`CostPart::TapTotalPower`] cost for `kw`.
pub fn can_be_tapped_for(g: &Game, kw: KeywordKind, creature: ObjectId) -> bool {
    let cant = cant_tap_for(kw);
    !g.statics.customs.iter().any(|(src, _, name)| {
        *name == cant && g.obj(*src).attached_to == Some(Entity::Object(creature))
    })
}

/// The power `creature` counts for when tapped to pay for `kw`.
pub fn power_for(g: &Game, kw: KeywordKind, creature: ObjectId) -> i64 {
    let o = g.obj(creature);
    let toughness = uses_toughness(kw);
    let prefix = format!("tap_total_power:{}:+", kw.name());
    let mut base = o.power() as i64;
    let mut bonus = 0i64;
    for (src, _, name) in &g.statics.customs {
        if *src != creature {
            continue;
        }
        if *name == toughness {
            base = o.toughness() as i64;
        } else if let Some(n) = name.strip_prefix(prefix.as_str()) {
            bonus += n.parse::<i64>().unwrap_or(0);
        }
    }
    base + bonus
}

/// Creatures `p` could tap to pay a total-power cost for `kw` with `filter`.
fn candidates(g: &Game, p: PlayerId, filter: &Filter, kw: KeywordKind, ctx: &Ctx) -> Vec<ObjectId> {
    g.objects_matching(filter, ctx)
        .into_iter()
        .filter(|o| {
            let ob = g.obj(*o);
            ob.controller == p
                && ob.zone == Zone::Battlefield
                && !ob.tapped
                && can_be_tapped_for(g, kw, *o)
        })
        .collect()
}

/// Whether `p` could pay "tap any number of [filter] with total power N or greater".
pub fn total_power_payable(
    g: &Game,
    p: PlayerId,
    _src: Option<ObjectId>,
    filter: &Filter,
    power: &Value,
    kw: KeywordKind,
    ctx: &Ctx,
) -> bool {
    let need = g.eval_value(power, ctx);
    let available: i64 = candidates(g, p, filter, kw, ctx)
        .into_iter()
        .map(|c| power_for(g, kw, c).max(0))
        .sum();
    available >= need
}

/// Pays "tap any number of [filter] with total power N or greater": the player chooses
/// which to tap (by default, the most powerful ones until the total is reached). Returns
/// the creatures tapped; for a crew ability, they crewed its source (CR 702.122b–c).
pub fn pay_total_power(
    g: &mut Game,
    p: PlayerId,
    src: Option<ObjectId>,
    filter: &Filter,
    power: &Value,
    kw: KeywordKind,
    ctx: &Ctx,
) -> Result<Vec<ObjectId>, Illegal> {
    let need = g.eval_value(power, ctx);
    let cands = candidates(g, p, filter, kw, ctx);
    let total = |g: &Game, v: &[ObjectId]| -> i64 { v.iter().map(|c| power_for(g, kw, *c)).sum() };
    let n = cands.len() as u32;
    let entities: Vec<Entity> = cands.iter().map(|c| Entity::Object(*c)).collect();
    let prompt = format!("Choose creatures to tap with total power {need} or greater");
    let answer: Vec<ObjectId> = g
        .ask_entities(p, src, &prompt, entities, 0, n)
        .into_iter()
        .filter_map(|e| e.object())
        .filter(|o| cands.contains(o))
        .collect();
    let chosen = if total(g, &answer) >= need {
        answer
    } else {
        let mut sorted = cands.clone();
        sorted.sort_by_key(|c| std::cmp::Reverse(power_for(g, kw, *c)));
        let mut pick = Vec::new();
        for c in sorted {
            if total(g, &pick) >= need {
                break;
            }
            pick.push(c);
        }
        pick
    };
    if total(g, &chosen) < need {
        return Err(Illegal(format!("not enough total power to tap ({need})")));
    }
    for c in &chosen {
        g.tap(*c);
    }
    if kw == KeywordKind::Crew {
        if let Some(v) = src {
            for c in &chosen {
                let chars = Box::new(g.obj(*c).chars.clone());
                g.history.crewed.push(CrewRecord {
                    vehicle: v,
                    creature: *c,
                    chars,
                });
            }
            g.log(|g| {
                let names: Vec<String> = chosen.iter().map(|c| g.describe(*c)).collect();
                format!("{} crew {}", names.join(", "), g.describe(v))
            });
        }
    }
    Ok(chosen)
}

/// The crew ability "Crew N" stands for.
fn crew_ability(n: i32, once_each_turn: bool) -> Ability {
    let cost = Cost::free().with(CostPart::TapTotalPower {
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::Other,
            Filter::ControlledBy(PlayerRel::You),
        ]),
        power: Value::c(n),
        keyword: KeywordKind::Crew,
    });
    let mut act = ActivatedAbility::new(
        cost,
        Body::effect(Effect::Seq(vec![
            Effect::Modify {
                what: Sel::This,
                mods: vec![Modification::AddTypes(vec![
                    CardType::Artifact,
                    CardType::Creature,
                ])],
                duration: Duration::EndOfTurn,
            },
            Effect::Custom(BECOME_CREWED.into()),
        ])),
    );
    if once_each_turn {
        act.max_per_turn = Some(1);
    }
    AbilityDef::new(AbilityKind::Activated(act), KeywordKind::Crew.name())
}

/// Whether the stack object `id` is a crew ability.
fn is_crew_ability(g: &Game, id: ObjectId) -> bool {
    g.obj(id).stack.as_deref().is_some_and(|si| {
        matches!(&si.kind, StackKind::Activated { ability, .. } if ability.text == KeywordKind::Crew.name())
    })
}

pub struct Crew;

impl KeywordRules for Crew {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Crew]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let once = kw
            .text
            .as_ref()
            .is_some_and(|t| t.to_lowercase().contains("activate only once each turn"));
        Some(vec![crew_ability(kw.n.unwrap_or(0), once)])
    }

    /// CR 702.122e: the Vehicle "becomes crewed" as its crew ability resolves.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != BECOME_CREWED {
            return false;
        }
        let Some(vehicle) = ctx.source else {
            return true;
        };
        let crew = ctx.vars.get(&COST_OBJECTS).map_or(0, Vec::len);
        g.emit(Event::Custom {
            name: SmolStr::new(CREWED),
            player: Some(ctx.controller),
            obj: Some(vehicle),
            amount: crew as i32,
        });
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let src = ctx.source?;
        match name {
            CREWED_IT_THIS_TURN => Some(crewed_this_turn(g, src, id)),
            CREWED_BY_IT_THIS_TURN => Some(crewed_this_turn(g, id, src)),
            _ => None,
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != CREATURES_THAT_CREWED_IT {
            return None;
        }
        let mut crew: Vec<ObjectId> = Vec::new();
        for r in &g.history.crewed {
            if Some(r.vehicle) == ctx.source && !crew.contains(&r.creature) {
                crew.push(r.creature);
            }
        }
        Some(crew.len() as i64)
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        let subtype = name.strip_prefix(CREWED_BY_TYPE)?;
        Some(g.history.crewed.iter().any(|r| {
            Some(r.vehicle) == ctx.source && r.chars.has_subtype(subtype)
        }))
    }

    /// CR 702.122b: "whenever ~ crews a Vehicle": the source was tapped to pay for a
    /// Vehicle's crew ability that was just activated.
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        if name != CREWS_A_VEHICLE {
            return None;
        }
        let Event::AbilityActivated {
            ability: Some(a),
            source: vehicle,
            player,
            ..
        } = ev
        else {
            return Some(vec![]);
        };
        let crewed = is_crew_ability(g, *a)
            && g.saved_ctx
                .get(a)
                .and_then(|c| c.vars.get(&COST_OBJECTS))
                .is_some_and(|v| v.contains(&Entity::Object(src)));
        Some(if crewed {
            vec![EventInfo {
                object: Some(*vehicle),
                other: Some(src),
                player: Some(*player),
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

inventory::submit! { KeywordRegistration(&Crew) }
