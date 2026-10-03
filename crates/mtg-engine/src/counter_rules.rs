//! Counters (CR 122): the effects that shield, stun, and finality counters create
//! (CR 122.1c, 122.1d, 122.1h), the rad counter triggered ability (CR 122.1i, 728),
//! counter limits (CR 122.4), moving counters (CR 122.5), counters an object enters with
//! (CR 122.6), and putting an object's counters on another object (CR 122.8, 122.9).

use crate::ability::*;
use crate::game::{Game, PendingTrigger};
use crate::object::*;
use crate::replacement::{ReplEvent, ReplKey};
use crate::types::*;
use smol_str::SmolStr;
use std::collections::BTreeMap;

/// Pseudo ability uids identifying the effects counters create, for the "each effect
/// applies once to an event" rule (CR 614.5).
const SHIELD_DESTROY_UID: u64 = u64::MAX - 1;
const SHIELD_DAMAGE_UID: u64 = u64::MAX - 2;
const FINALITY_UID: u64 = u64::MAX - 3;

/// Whether counters of `kind` can't be put on `obj`: an ability of its own or another
/// object's stops them ("~ can't have counters put on it", "creatures your opponents
/// control can't have +1/+1 counters put on them", CR 113.6i, 614.17). A cost that
/// includes putting such counters can't be paid (CR 614.17b), and riot's choice of a
/// counter isn't available.
pub fn counters_prevented(g: &Game, obj: ObjectId, kind: &str) -> bool {
    let prevents = |r: &ReplacementDef| -> Option<Filter> {
        match (&r.event, &r.action) {
            (
                ReplacementEvent::PutCounters {
                    on_objects: Some(f),
                    kind: k,
                    ..
                },
                ReplacementAction::Prevent,
            ) if k.as_deref().is_none_or(|k| k == kind) => Some(f.clone()),
            _ => None,
        }
    };
    let own = g.obj(obj).chars.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Static(s) => match &s.effect {
            StaticEffect::Replacement(r) => {
                prevents(r).is_some_and(|f| matches!(f, Filter::Source))
            }
            _ => false,
        },
        _ => false,
    });
    own || g.statics.replacements.iter().any(|(src, ctl, _, _, r)| {
        *src != obj
            && prevents(r).is_some_and(|f| {
                !matches!(f, Filter::Source)
                    && g.matches(obj, &f, &crate::eval::Ctx::new(Some(*src), *ctl))
            })
    })
}

/// Whether a replacement key is the prevention effect shield counters create (CR 122.1c),
/// a single effect for all the damage that would be dealt to the permanent at once.
pub fn is_shield_prevention(key: &ReplKey) -> bool {
    matches!(key, ReplKey::Static(_, u) if *u == SHIELD_DAMAGE_UID)
}

/// Hone counters (CR 122.1j).
pub const HONE: &str = "hone";

/// A replacement or prevention effect created by counters on a permanent: (key, the
/// permanent, its controller, the effect, a description).
pub type CounterReplacement = (ReplKey, ObjectId, PlayerId, ReplacementDef, String);

fn def(action: ReplacementAction) -> ReplacementDef {
    ReplacementDef {
        // Matching is done here, not from this pattern.
        event: ReplacementEvent::Destroy(Filter::Source),
        action,
        self_replacement: false,
        optional: false,
    }
}

fn remove_a_shield_counter() -> Effect {
    Effect::RemoveCounters {
        what: Sel::This,
        kind: Some(counters::SHIELD.into()),
        n: Value::c(1),
    }
}

fn remove_shield() -> ReplacementAction {
    ReplacementAction::Instead(Box::new(remove_a_shield_counter()))
}

/// "Prevent that damage and remove a shield counter from it": a prevention effect, so if
/// the damage can't be prevented, the counter is still removed (CR 615.12).
fn prevent_and_remove_shield() -> ReplacementAction {
    ReplacementAction::PreventAndThen(None, Box::new(remove_a_shield_counter()))
}

/// The replacement and prevention effects counters create that apply to a proposed event:
/// * shield counters (CR 122.1c): "If this permanent would be destroyed as the result of
///   an effect, instead remove a shield counter from it" and "If damage would be dealt to
///   this permanent, prevent that damage and remove a shield counter from it" — one of
///   each, however many shield counters it has;
/// * finality counters (CR 122.1h): "If this permanent would be put into a graveyard from
///   the battlefield, exile it instead."
pub fn counter_replacements(g: &Game, ev: &ReplEvent) -> Vec<CounterReplacement> {
    let mut out = Vec::new();
    let on_bf = |o: ObjectId| g.is_live(o) && g.obj(o).zone == Zone::Battlefield;
    match ev {
        // Destruction by a state-based action (no source) isn't the result of an effect.
        ReplEvent::Destroy {
            obj,
            source: Some(_),
            ..
        } if on_bf(*obj) && g.obj(*obj).counter(counters::SHIELD) > 0 => {
            out.push((
                ReplKey::Static(*obj, SHIELD_DESTROY_UID),
                *obj,
                g.obj(*obj).controller,
                def(remove_shield()),
                "Shield counter: remove it instead of being destroyed".into(),
            ));
        }
        ReplEvent::Damage {
            target: Entity::Object(o),
            amount,
            ..
        } if *amount > 0 && on_bf(*o) && g.obj(*o).counter(counters::SHIELD) > 0 => {
            out.push((
                ReplKey::Static(*o, SHIELD_DAMAGE_UID),
                *o,
                g.obj(*o).controller,
                def(prevent_and_remove_shield()),
                "Shield counter: prevent the damage and remove it".into(),
            ));
        }
        ReplEvent::Move(m)
            if on_bf(m.obj)
                && matches!(m.to, Zone::Graveyard(_))
                && g.obj(m.obj).counter(counters::FINALITY) > 0 =>
        {
            out.push((
                ReplKey::Static(m.obj, FINALITY_UID),
                m.obj,
                g.obj(m.obj).controller,
                def(ReplacementAction::MoveInstead(Destination::zone(
                    ZoneKind::Exile,
                ))),
                "Finality counter: exile it instead".into(),
            ));
        }
        _ => {}
    }
    out
}

/// CR 122.1d: "If a permanent with a stun counter on it would become untapped, instead
/// remove a stun counter from it." Returns true if untapping was replaced.
pub fn stun_instead_of_untap(g: &mut Game, obj: ObjectId) -> bool {
    if g.obj(obj).counter(counters::STUN) == 0 {
        return false;
    }
    g.remove_counters(Entity::Object(obj), counters::STUN, 1);
    true
}

/// Hone counters on Equipment attached to a creature give it +1/+0 each (CR 122.1j).
pub fn hone_bonus(g: &Game, creature: ObjectId) -> i32 {
    g.battlefield
        .iter()
        .map(|id| g.obj(*id))
        .filter(|o| {
            o.attached_to == Some(Entity::Object(creature)) && o.chars.has_subtype("Equipment")
        })
        .map(|o| o.counter(HONE) as i32)
        .sum()
}

/// CR 122.1i, 728.1: the inherent triggered ability associated with rad counters. It has
/// no source and is controlled by the active player: "At the beginning of each player's
/// precombat main phase, if that player has one or more rad counters, that player mills a
/// number of cards equal to the number of rad counters they have. For each nonland card
/// milled this way, that player loses 1 life and removes one rad counter from themselves."
/// With shared team turns, it's each active player's precombat main phase: the ability
/// triggers for each of them that has rad counters.
pub fn rad_trigger(g: &mut Game) {
    for p in g.active_players() {
        rad_trigger_for(g, p);
    }
}

fn rad_trigger_for(g: &mut Game, p: PlayerId) {
    if g.player(p).counter(counters::RAD) == 0 {
        return;
    }
    let has_rad = Condition::Compare(
        Value::PlayerCounters(PlayerRef::TriggerPlayer, counters::RAD.into()),
        Cmp::Gt,
        Value::c(0),
    );
    let card: Var = vars::USER + 80;
    let body = Body::effect(Effect::seq(vec![
        Effect::Mill {
            who: PlayerRef::TriggerPlayer,
            n: Value::PlayerCounters(PlayerRef::TriggerPlayer, counters::RAD.into()),
        },
        Effect::ForEach {
            sel: Sel::Var(vars::IT),
            var: card,
            effect: Box::new(Effect::If {
                cond: Condition::SelMatches(
                    Sel::Var(card),
                    Filter::not(Filter::Type(CardType::Land)),
                ),
                then: Box::new(Effect::seq(vec![
                    // Life lost "from radiation" (CR 728.1a).
                    crate::radiation::lose_life_from_radiation(),
                    Effect::RemoveCounters {
                        what: Sel::Players(PlayerRef::TriggerPlayer),
                        kind: Some(counters::RAD.into()),
                        n: Value::c(1),
                    },
                ])),
                otherwise: Box::new(Effect::Noop),
            }),
        },
    ]));
    let mut tr = TriggeredAbility::new(
        TriggerCond::BeginningOf {
            step: TriggerStep::PrecombatMain,
            whose: PlayerRel::Any,
        },
        body,
    );
    tr.intervening_if = Some(has_rad);
    let ability = AbilityDef::new(
        AbilityKind::Triggered(tr),
        "Radiation (rad counters, CR 728.1)",
    );
    // The ability has no source (CR 728.1); a placeholder object that is in no zone stands
    // in for it.
    let chars = Characteristics {
        name: SmolStr::new("Radiation"),
        rules_text: std::sync::Arc::from(""),
        ..Default::default()
    };
    let mut obj = GameObject::new(ObjectId(0), ObjKind::Emblem, p, Zone::Nowhere, chars);
    obj.controller = p;
    let src = ObjectId(g.objects.len() as u32);
    obj.id = src;
    obj.timestamp = g.new_timestamp();
    g.objects.push(obj);
    let order = g.trigger_order;
    g.trigger_order += 1;
    g.pending_triggers.push(PendingTrigger {
        source: src,
        controller: p,
        ability,
        event: EventInfo {
            player: Some(p),
            ..Default::default()
        },
        source_lki: None,
        saved: None,
        body: None,
        order,
    });
}

/// The name of the static ability "[This] can't have more than N [kind] counters on it".
pub fn counter_limit_name(kind: &str, n: u32) -> SmolStr {
    SmolStr::new(format!("counter limit:{n}:{kind}"))
}

/// CR 122.4, 704.5r: limits from abilities of a permanent that say it can't have more
/// than N counters of a kind on it.
pub fn counter_limits(g: &Game, id: ObjectId) -> Vec<(CounterKind, u32)> {
    let mut out = Vec::new();
    for a in &g.obj(id).chars.abilities {
        let AbilityKind::Static(s) = &a.kind else {
            continue;
        };
        let StaticEffect::Custom(name) = &s.effect else {
            continue;
        };
        if let Some(rest) = name.strip_prefix("counter limit:") {
            if let Some((n, kind)) = rest.split_once(':') {
                if let Ok(n) = n.parse::<u32>() {
                    out.push((CounterKind::from(kind), n));
                }
            }
        }
    }
    out
}

/// CR 122.5: moves up to `n` counters of a kind from one object to another: removes them
/// from the first and puts them on the second. If either isn't possible — the objects are
/// the same object, the first doesn't have such a counter, or either is no longer on the
/// battlefield — no counter is moved. Returns the number moved.
pub fn move_counters(g: &mut Game, from: ObjectId, to: Entity, kind: &str, n: u32) -> u32 {
    move_counters_by(g, from, to, kind, n, None)
}

/// [`move_counters`] as instructed by the spell or ability of `source`: its controller
/// puts the counters on the second object (CR 122.5, 122.6), which replacement effects
/// that care who puts counters see (Halving Season, Vorinclex, Monstrous Raider).
pub fn move_counters_by(
    g: &mut Game,
    from: ObjectId,
    to: Entity,
    kind: &str,
    n: u32,
    source: Option<ObjectId>,
) -> u32 {
    if to == Entity::Object(from) || !g.is_live(from) || g.obj(from).zone != Zone::Battlefield {
        return 0;
    }
    if let Entity::Object(t) = to {
        if !g.is_live(t) || g.obj(t).zone != Zone::Battlefield {
            return 0;
        }
    }
    let k = g.obj(from).counter(kind).min(n);
    if k == 0 {
        return 0;
    }
    let removed = g.remove_counters(Entity::Object(from), kind, k);
    g.add_counters(to, kind, removed, source);
    removed
}

/// CR 122.8, 122.9: "put [its] counters on [another object]" when the object with the
/// counters has left the battlefield (it was sacrificed to pay the cost, or the trigger
/// checks that it left): the same number of each kind of counter it had (or only of the
/// listed kind) is put on the other object, by the controller of `source` (the spell or
/// ability doing it). Nothing is moved.
pub fn put_counters_of(
    g: &mut Game,
    from: ObjectId,
    to: Entity,
    kind: Option<&str>,
    source: Option<ObjectId>,
) -> u32 {
    let counters: BTreeMap<CounterKind, u32> = g.obj(from).counters.clone();
    let mut total = 0;
    for (k, n) in counters {
        if n == 0 || kind.is_some_and(|x| x != k.as_str()) {
            continue;
        }
        total += g.add_counters(to, &k, n, source);
    }
    total
}

/// Custom effect name prefix for "distribute N [kind] counters among [targets]":
/// `divided-counters:<target slot>:<kind>`.
pub const DIVIDED_COUNTERS: &str = "divided-counters:";

/// The custom effect name for distributing counters of `kind` among the targets in `slot`.
pub fn divided_counters_effect(slot: u8, kind: &str) -> SmolStr {
    SmolStr::from(format!("{DIVIDED_COUNTERS}{slot}:{kind}"))
}

/// "Distribute two +1/+1 counters among one or two target creatures" (CR 601.2d): each
/// target gets the number of counters assigned to it as the spell or ability was put on
/// the stack. Counters assigned to a target that became illegal aren't put on anything
/// (CR 608.2b).
pub fn custom_effect(g: &mut Game, name: &str, ctx: &crate::eval::Ctx) -> bool {
    let Some(spec) = name.strip_prefix(DIVIDED_COUNTERS) else {
        return false;
    };
    let Some((slot, kind)) = spec.split_once(':') else {
        return true;
    };
    let Ok(slot) = slot.parse::<usize>() else {
        return true;
    };
    let targets = ctx.targets.get(slot).cloned().unwrap_or_default();
    let div = ctx.divided.get(slot).cloned().unwrap_or_default();
    for (i, t) in targets.into_iter().enumerate() {
        let n = div.get(i).copied().unwrap_or(0);
        if n > 0 {
            g.put_counters(t, kind, n, crate::event_causes::CounterPut::of(ctx));
        }
    }
    true
}

/// The counters on an object or player: (kind, number), in a fixed order.
fn counters_of(g: &Game, target: Entity) -> Vec<(CounterKind, u32)> {
    let map = match target {
        // Counters can't be removed from an object's last known information (CR 400.7).
        Entity::Object(o) if !g.is_live(o) => return vec![],
        Entity::Object(o) => &g.obj(o).counters,
        Entity::Player(p) => &g.player(p).counters,
    };
    map.iter()
        .filter(|(_, n)| **n > 0)
        .map(|(k, n)| (k.clone(), *n))
        .collect()
}

/// "Remove N counters from [target]" with no kind named, or "remove N [kind] counters"
/// (`kind`): N counters in all (every one if it has N or fewer). Where it has counters of
/// several kinds and N is fewer than it has, `chooser` (the controller of the spell or
/// ability) chooses which, one counter at a time. Returns how many were removed.
pub fn remove_chosen_counters(
    g: &mut Game,
    target: Entity,
    kind: Option<&CounterKind>,
    n: u32,
    chooser: PlayerId,
    source: Option<ObjectId>,
) -> u32 {
    let mut present: Vec<(CounterKind, u32)> = counters_of(g, target)
        .into_iter()
        .filter(|(k, _)| kind.is_none_or(|x| x == k))
        .collect();
    let have: u32 = present.iter().map(|(_, c)| c).sum();
    let mut plan: Vec<(CounterKind, u32)> = Vec::new();
    if have <= n || present.len() <= 1 {
        let mut left = n;
        for (k, c) in present {
            let take = c.min(left);
            left -= take;
            plan.push((k, take));
        }
    } else {
        for _ in 0..n {
            let labels: Vec<String> = present
                .iter()
                .map(|(k, _)| format!("{k} counter"))
                .collect();
            let i = g.ask_option(chooser, source, "Choose a counter to remove", labels);
            let i = i.min(present.len() - 1);
            let k = present[i].0.clone();
            present[i].1 -= 1;
            if present[i].1 == 0 {
                present.remove(i);
            }
            match plan.iter_mut().find(|(pk, _)| *pk == k) {
                Some(e) => e.1 += 1,
                None => plan.push((k, 1)),
            }
        }
    }
    let mut total = 0;
    for (k, c) in plan {
        if c > 0 {
            total += g.remove_counters_by(target, &k, c, Some(chooser));
        }
    }
    total
}

/// "Remove up to N [kind] counters from [target]" (`max`), "remove any number of counters
/// from [target]" (`max: None`): `chooser` chooses how many (CR 107.1c), then which.
pub fn remove_up_to_counters(
    g: &mut Game,
    target: Entity,
    kind: Option<&CounterKind>,
    max: Option<u32>,
    chooser: PlayerId,
    source: Option<ObjectId>,
) -> u32 {
    let have: u32 = counters_of(g, target)
        .iter()
        .filter(|(k, _)| kind.is_none_or(|x| x == k))
        .map(|(_, c)| c)
        .sum();
    let most = max.map_or(have, |m| m.min(have));
    if most == 0 {
        return 0;
    }
    let n = g.ask_number(
        chooser,
        source,
        "Choose how many counters to remove",
        0,
        most as i64,
    );
    remove_chosen_counters(g, target, kind, n.max(0) as u32, chooser, source)
}

/// [`Effect::ChooseCounterKind`]: the controller chooses a kind of counter among those on
/// `from`; returns `then` with that kind in place of [`CHOSEN_COUNTER_KIND`], or None if
/// there's no counter to choose.
/// The instruction `then` for each kind of counter on `from` ([`Effect::ForEachCounterKind`]).
pub fn for_each_counter_kind(
    g: &mut Game,
    from: &Sel,
    then: &Effect,
    ctx: &mut crate::eval::Ctx,
) -> Vec<Effect> {
    let mut kinds: Vec<CounterKind> = Vec::new();
    for e in g.resolve_sel(from, ctx) {
        for (k, _) in counters_of(g, e) {
            if !kinds.contains(&k) {
                kinds.push(k);
            }
        }
    }
    let (Ok(json), Ok(placeholder)) = (
        serde_json::to_string(then),
        serde_json::to_string(CHOSEN_COUNTER_KIND),
    ) else {
        return vec![];
    };
    kinds
        .iter()
        .filter_map(|k| {
            let chosen = serde_json::to_string(k.as_str()).ok()?;
            serde_json::from_str(&json.replace(&placeholder, &chosen)).ok()
        })
        .collect()
}

pub fn with_chosen_counter_kind(
    g: &mut Game,
    from: &Sel,
    then: &Effect,
    ctx: &mut crate::eval::Ctx,
) -> Option<Effect> {
    let mut kinds: Vec<CounterKind> = Vec::new();
    for e in g.resolve_sel(from, ctx) {
        for (k, _) in counters_of(g, e) {
            if !kinds.contains(&k) {
                kinds.push(k);
            }
        }
    }
    if kinds.is_empty() {
        return None;
    }
    let labels = kinds.iter().map(|k| format!("{k} counter")).collect();
    let i = g.ask_option(
        ctx.controller,
        ctx.source,
        "Choose a kind of counter",
        labels,
    );
    let kind = kinds.get(i).unwrap_or(&kinds[0]);
    let json = serde_json::to_string(then).ok()?;
    let placeholder = serde_json::to_string(CHOSEN_COUNTER_KIND).ok()?;
    let chosen = serde_json::to_string(kind.as_str()).ok()?;
    serde_json::from_str(&json.replace(&placeholder, &chosen)).ok()
}
