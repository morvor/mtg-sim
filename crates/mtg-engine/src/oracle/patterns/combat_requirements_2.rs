//! Attack requirements and their follow-ups (CR 508.1d):
//!
//! * "Destroy it at the beginning of the next end step if it didn't attack this turn."
//!   (Norritt, Nettling Imp), "At the beginning of the next end step, destroy each of
//!   those creatures that didn't attack this turn." (Maddening Imp): a delayed triggered
//!   ability (CR 603.7) about the objects named now, checking whether each attacked
//!   (was declared as an attacker, CR 508.1) this turn as it resolves.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;

/// The objects a delayed "destroy ... if it didn't attack this turn" is about.
const NOT_ATTACKED: Var = vars::USER + 7710;

fn destroy_unless_attacked(what: Sel) -> Effect {
    let stored = Sel::Var(NOT_ATTACKED);
    Effect::seq(vec![
        Effect::Store {
            var: NOT_ATTACKED,
            sel: what,
        },
        Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(Effect::Destroy {
                what: Sel::All(Filter::and(vec![
                    Filter::In(Box::new(stored)),
                    Filter::not(Filter::AttackedThisTurn),
                ])),
                no_regen: false,
            }),
        },
    ])
}

/// "destroy it at the beginning of the next end step if it didn't attack this turn",
/// "at the beginning of the next end step, destroy it if it didn't attack this turn".
fn p_destroy_if_didnt_attack(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let what = l
        .strip_prefix("destroy ")
        .and_then(|r| {
            r.strip_suffix(" at the beginning of the next end step if it didn't attack this turn")
        })
        .or_else(|| {
            l.strip_prefix("at the beginning of the next end step, destroy ")
                .and_then(|r| r.strip_suffix(" if it didn't attack this turn"))
        })?;
    if !matches!(what, "it" | "that creature" | "the creature") {
        return None;
    }
    let n = b.targets.len();
    let (sel, rest) = object_ref(what, b)?;
    if !rest.trim().is_empty()
        || b.targets.len() != n
        || !matches!(sel, Sel::Target(_) | Sel::TriggerObject | Sel::This)
    {
        return None;
    }
    Some(destroy_unless_attacked(sel))
}

inventory::submit! { EffectPattern { name: "combat requirements: destroy it at the next end step if it didn't attack", priority: 90, parse: p_destroy_if_didnt_attack } }

/// "[Creatures ...] attack this turn if able. At the beginning of the next end step,
/// destroy each of those creatures that didn't attack this turn.": the creatures the
/// requirement applied to as it began (CR 611.2c).
fn f_destroy_those_that_didnt_attack(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l.trim())
        != "at the beginning of the next end step, destroy each of those creatures that didn't attack this turn"
    {
        return false;
    }
    let Effect::AddRestriction {
        restriction: Restriction::MustAttack(f),
        duration: Duration::EndOfTurn,
    } = &*prev
    else {
        return false;
    };
    if matches!(f, Filter::In(_)) {
        return false;
    }
    let what = Sel::All(f.clone());
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, destroy_unless_attacked(what)]);
    true
}

inventory::submit! { FollowupPattern { name: "combat requirements: destroy those creatures that didn't attack", priority: 40, apply: f_destroy_those_that_didnt_attack } }

/// Adds "controlled continuously since the beginning of the turn" to the objects of a
/// sacrifice or destroy instruction.
fn add_since_turn_began(e: &mut Effect) -> bool {
    match e {
        Effect::Sacrifice { filter, .. } => {
            *filter = Filter::and(vec![filter.clone(), Filter::ControlledSinceTurnBegan]);
            true
        }
        Effect::Destroy {
            what: Sel::All(f), ..
        } => {
            *f = Filter::and(vec![f.clone(), Filter::ControlledSinceTurnBegan]);
            true
        }
        _ => false,
    }
}

/// "[player] sacrifices a creature of their choice that they controlled since the
/// beginning of the turn" (Keldon Twilight), "destroy all [creatures that player
/// controls], except for creatures the player hasn't controlled continuously since the
/// beginning of the turn" (Total War): the player is the active player, so these are the
/// creatures CR 302.6 doesn't call summoning sick.
fn p_since_turn_began(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let body = if let Some(x) = l
        .strip_suffix(" that they controlled since the beginning of the turn")
        .or_else(|| l.strip_suffix(" that they've controlled continuously since the beginning of the turn"))
    {
        if !x.contains(" sacrifices ") {
            return None;
        }
        x
    } else if let Some(x) = l.strip_suffix(
        ", except for creatures the player hasn't controlled continuously since the beginning of the turn",
    ) {
        if !x.starts_with("destroy all ") {
            return None;
        }
        x
    } else {
        return None;
    };
    let mut e = crate::oracle::effects::parse_clause(body, b)?;
    add_since_turn_began(&mut e).then_some(e)
}

inventory::submit! { EffectPattern { name: "combat requirements: controlled since the beginning of the turn", priority: 90, parse: p_since_turn_began } }

fn restriction(r: Restriction, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )
}

/// "a black or green creature", "a creature with greater power" (than ~): one creature,
/// relative to the source.
fn companion(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    // "with greater power": greater than the source's.
    if let Some(noun) = s.strip_suffix(" with greater power") {
        let (f, plural) = super::statics::whole_object_phrase(noun)?;
        if plural {
            return None;
        }
        return Some(Filter::and(vec![
            f,
            Filter::Power(Cmp::Gt, Box::new(Value::PowerOf(Box::new(Sel::This)))),
        ]));
    }
    let (f, plural) = super::statics::whole_object_phrase(s)?;
    if plural
        || serde_json::to_string(&f)
            .unwrap_or_default()
            .contains("Source")
    {
        return None;
    }
    Some(f)
}

/// Static combat restrictions on the source: "~ can't attack unless [a creature] also
/// attacks", "~ can't block unless [a creature] also blocks", "~ can't be blocked unless
/// all creatures defending player controls block it", "~ can't attack a player it has
/// already attacked this turn", "~ can't block creatures with power 2 or greater unless
/// you pay {1}".
fn source_restrictions(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    if let Some(c) = l
        .strip_prefix("~ can't attack unless ")
        .and_then(|r| r.strip_suffix(" also attacks"))
    {
        return Some(vec![restriction(
            Restriction::AttackOnlyWith {
                attackers: Filter::Source,
                companion: companion(c)?,
            },
            text,
        )]);
    }
    if let Some(c) = l
        .strip_prefix("~ can't block unless ")
        .and_then(|r| r.strip_suffix(" also blocks"))
    {
        return Some(vec![restriction(
            Restriction::BlockOnlyWith {
                blockers: Filter::Source,
                companion: companion(c)?,
            },
            text,
        )]);
    }
    if l == "~ can't be blocked unless all creatures defending player controls block it" {
        return Some(vec![restriction(
            Restriction::MinBlockersCounted {
                attacker: Filter::Source,
                n: Value::Count(Filter::and(vec![
                    Filter::creature(),
                    Filter::ControlledBy(PlayerRel::Defending),
                ])),
            },
            text,
        )]);
    }
    if l == "~ can't attack a player it has already attacked this turn" {
        return Some(vec![restriction(
            Restriction::CantAttackPlayerAgain(Filter::Source),
            text,
        )]);
    }
    // "~ can't block creatures with power 2 or greater unless you pay {1}".
    if let Some(r) = l.strip_prefix("~ can't block ") {
        let (attackers, pay) = r.split_once(" unless you pay ")?;
        let (a, plural) = super::statics::whole_object_phrase(attackers)?;
        if !plural {
            return None;
        }
        let cost = super::counters_resources_pay::resolution_cost(pay)?;
        return Some(vec![restriction(
            Restriction::BlockCostAgainst {
                blockers: Filter::Source,
                attackers: a,
                cost,
            },
            text,
        )]);
    }
    None
}

inventory::submit! { super::StaticPattern { name: "combat requirements: restrictions on ~", priority: 120, parse: source_restrictions } }

/// Block taxes keyed to the attacker: "[blockers] can't block [attackers] unless their
/// controller pays [cost] for each blocking creature they control" (Heat Wave),
/// "enchanted creature can't be blocked unless defending player pays [cost] for each
/// creature they control that's blocking it" (Awesome Presence).
fn block_taxes(l: &str, text: &str, _ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    if let Some(r) = l
        .strip_prefix("enchanted creature can't be blocked unless defending player pays ")
        .or_else(|| {
            l.strip_prefix("equipped creature can't be blocked unless defending player pays ")
        })
    {
        let pay = r.strip_suffix(" for each creature they control that's blocking it")?;
        let cost = super::counters_resources_pay::resolution_cost(pay)?;
        return Some(vec![restriction(
            Restriction::BlockCostAgainst {
                blockers: Filter::creature(),
                attackers: Filter::AttachedToSource,
                cost,
            },
            text,
        )]);
    }
    let (subject, r) = l.split_once(" can't block ")?;
    let (attackers, pay) = r.split_once(" unless their controller pays ")?;
    let pay = pay.strip_suffix(" for each blocking creature they control")?;
    let (b, plural) = super::statics::whole_object_phrase(subject)?;
    let (a, plural2) = super::statics::whole_object_phrase(attackers)?;
    if !plural || !plural2 {
        return None;
    }
    let cost = super::counters_resources_pay::resolution_cost(pay)?;
    Some(vec![restriction(
        Restriction::BlockCostAgainst {
            blockers: b,
            attackers: a,
            cost,
        },
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "combat requirements: block taxes keyed to the attacker", priority: 120, parse: block_taxes } }

/// "Each opponent must attack [you or a planeswalker you control] with at least one
/// creature each combat if able."
fn player_must_attack(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let r = l.strip_prefix("each opponent must attack ")?;
    let r = r.strip_suffix("with at least one creature each combat if able")?;
    let (defender, planeswalkers) = match r.trim() {
        "" => (None, false),
        "you" => (Some(PlayerFilter::You), false),
        "you or a planeswalker you control" => (Some(PlayerFilter::You), true),
        _ => return None,
    };
    Some(vec![restriction(
        Restriction::PlayerMustAttack {
            players: PlayerFilter::Opponent,
            defender,
            planeswalkers,
        },
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "combat requirements: each opponent must attack", priority: 120, parse: player_must_attack } }

/// "The attacking player chooses how each creature blocks each combat." (Invasion Plans)
fn attacking_player_chooses_blocks(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    (end(l.trim()) == "the attacking player chooses how each creature blocks each combat").then(
        || {
            vec![restriction(
                Restriction::Custom(crate::block_choice::ATTACKING_PLAYER_CHOOSES_BLOCKS.into()),
                text,
            )]
        },
    )
}

inventory::submit! { super::StaticPattern { name: "combat requirements: the attacking player chooses blocks", priority: 120, parse: attacking_player_chooses_blocks } }
