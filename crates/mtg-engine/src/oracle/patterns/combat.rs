//! Oracle patterns for combat (CR 506–511): combat triggers ("attacks alone", "attacks
//! you", "blocks a creature", "becomes blocked by a creature", ...), combat restrictions
//! ("can't attack alone", Propaganda-style attack costs, lure), combat timing windows
//! ("Cast this spell only during combat ...", CR 506.8), and optional costs to attack
//! ("You may exert this creature as it attacks", CR 508.1g).

use super::{AbilityPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn object_filter(x: &str) -> Option<Filter> {
    let x = x
        .strip_prefix("a ")
        .or_else(|| x.strip_prefix("an "))
        .or_else(|| x.strip_prefix("one or more "))
        .unwrap_or(x);
    if x == "~" {
        return Some(Filter::Source);
    }
    let (f, _, tail) = parse_object_phrase(x)?;
    end(tail).is_empty().then_some(f)
}

/// "you", "you or a planeswalker you control", "one of your opponents", ...
fn attack_recipient(s: &str) -> Option<DamageRecipient> {
    Some(match s {
        "you" => DamageRecipient::Player(PlayerRel::You),
        "you or a planeswalker you control" => {
            DamageRecipient::PlayerOrPlaneswalker(PlayerRel::You)
        }
        "an opponent" | "one of your opponents" => DamageRecipient::Player(PlayerRel::Opponent),
        "a player" => DamageRecipient::Player(PlayerRel::Any),
        "a planeswalker you control" => DamageRecipient::Object(Filter::and(vec![
            Filter::Type(CardType::Planeswalker),
            Filter::ControlledBy(PlayerRel::You),
        ])),
        "~" => DamageRecipient::Object(Filter::Source),
        _ => return None,
    })
}

fn combat_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let obj = Sel::TriggerObject;
    // CR 506.6: "[filter] attacks a player alone".
    if let Some(x) = r.strip_suffix(" attacks a player alone") {
        return Some((
            TriggerCond::AttacksPlayerAlone(object_filter(x)?),
            obj,
            PlayerRef::TriggerPlayer,
        ));
    }
    // CR 506.5: "[filter] attacks alone".
    if let Some(x) = r.strip_suffix(" attacks alone") {
        return Some((
            TriggerCond::AttacksAlone(object_filter(x)?),
            obj,
            PlayerRef::TriggerPlayer,
        ));
    }
    // CR 509.3b: "[filter] blocks a creature".
    if let Some(x) = r.strip_suffix(" blocks a creature") {
        return Some((
            TriggerCond::BlocksCreature {
                blocker: object_filter(x)?,
                attacker: Filter::Any,
            },
            obj,
            PlayerRef::TriggerPlayer,
        ));
    }
    // CR 509.3e: "[filter] becomes blocked by two or more creatures".
    for (suffix, n) in [
        (" becomes blocked by two or more creatures", 2),
        (" becomes blocked by three or more creatures", 3),
    ] {
        if let Some(x) = r.strip_suffix(suffix) {
            return Some((
                TriggerCond::BlockedByN {
                    attacker: object_filter(x)?,
                    n,
                },
                obj,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    // CR 509.3d: "[filter] becomes blocked by a [blocker filter]".
    if let Some((x, b)) = r.split_once(" becomes blocked by ") {
        let attacker = object_filter(x)?;
        let blocker = object_filter(b)?;
        return Some((
            TriggerCond::BlockedByCreature { attacker, blocker },
            obj,
            PlayerRef::TriggerPlayer,
        ));
    }
    // CR 508.3c: "[player] attacks with [N or more / one or more] [creatures]".
    for (who_s, who) in [
        ("a player", PlayerRel::Any),
        ("you", PlayerRel::You),
        ("an opponent", PlayerRel::Opponent),
    ] {
        if let Some(x) = r.strip_prefix(&format!("{who_s} attacks with ")) {
            let (min, rest) = if let Some(t) = x.strip_prefix("one or more ") {
                (1, t)
            } else {
                let (n, t) = parse_number(x)?;
                let t = t.trim().strip_prefix("or more ")?;
                (n.as_const()? as u32, t)
            };
            let rest = rest.strip_suffix('s').unwrap_or(rest);
            return Some((
                TriggerCond::PlayerAttacksWith {
                    who,
                    filter: object_filter(rest)?,
                    min,
                },
                obj,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    // CR 508.3e: "[player] attacks [another player]".
    for (a, d, attacker, defender) in [
        ("you", "a player", PlayerRel::You, PlayerRel::Any),
        ("an opponent", "you", PlayerRel::Opponent, PlayerRel::You),
        (
            "a player",
            "one or more of your opponents",
            PlayerRel::Any,
            PlayerRel::Opponent,
        ),
        (
            "a player",
            "one of your opponents",
            PlayerRel::Any,
            PlayerRel::Opponent,
        ),
        (
            "an opponent",
            "another one of your opponents",
            PlayerRel::Opponent,
            PlayerRel::Opponent,
        ),
    ] {
        if r == format!("{a} attack {d}") || r == format!("{a} attacks {d}") {
            return Some((
                TriggerCond::PlayerAttacksPlayer { attacker, defender },
                Sel::None,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    // CR 508.3b: "[player or permanent] is attacked".
    if let Some(x) = r.strip_suffix(" is attacked") {
        let rec = match x {
            "~" => DamageRecipient::Object(Filter::Source),
            "a planeswalker you control" => attack_recipient(x)?,
            _ => return None,
        };
        return Some((
            TriggerCond::IsAttacked(rec),
            Sel::None,
            PlayerRef::TriggerPlayer,
        ));
    }
    if r == "you're attacked" || r == "you are attacked" {
        return Some((
            TriggerCond::IsAttacked(DamageRecipient::Player(PlayerRel::You)),
            Sel::None,
            PlayerRef::TriggerPlayer,
        ));
    }
    // CR 508.3a: "[filter] attacks [you / a planeswalker you control / ...]".
    if let Some((x, rec)) = r.split_once(" attacks ") {
        if let (Some(attacker), Some(recipient)) = (object_filter(x), attack_recipient(rec)) {
            return Some((
                TriggerCond::AttacksRecipient {
                    attacker,
                    recipient,
                },
                obj,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    None
}

inventory::submit! { TriggerPattern { name: "combat triggers", priority: 100, parse: combat_trigger } }

fn combat_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let st = |e: StaticEffect| AbilityDef::new(AbilityKind::Static(StaticAbility::new(e)), text);
    let r = |x: Restriction| st(StaticEffect::Restriction(x));
    match l {
        "~ can't attack alone" => return Some(vec![r(Restriction::CantAttackAlone(Filter::Source))]),
        "~ can't block alone" => return Some(vec![r(Restriction::CantBlockAlone(Filter::Source))]),
        "~ can't attack or block alone" => {
            return Some(vec![
                r(Restriction::CantAttackAlone(Filter::Source)),
                r(Restriction::CantBlockAlone(Filter::Source)),
            ])
        }
        "~ must be blocked if able" | "~ must be blocked each combat if able" => {
            return Some(vec![r(Restriction::MustBeBlocked(Filter::Source))])
        }
        "all creatures able to block ~ do so" => {
            return Some(vec![r(Restriction::MustBeBlockedByAll(Filter::Source))])
        }
        "all creatures able to block enchanted creature do so" => {
            return Some(vec![r(Restriction::MustBeBlockedByAll(Filter::AttachedToSource))])
        }
        "no more than one creature can attack each combat" => {
            return Some(vec![r(Restriction::MaxAttackers(1))])
        }
        "no more than one creature can block each combat" => {
            return Some(vec![r(Restriction::MaxBlockers(1))])
        }
        "no more than one creature can attack each combat. no more than one creature can block each combat" => {
            return Some(vec![r(Restriction::MaxAttackers(1)), r(Restriction::MaxBlockers(1))])
        }
        _ => {}
    }
    // Propaganda / Ghostly Prison (CR 508.1d, 508.1h).
    for (prefix, planeswalkers) in [
        (
            "creatures can't attack you unless their controller pays ",
            false,
        ),
        (
            "creatures can't attack you or planeswalkers you control unless their controller pays ",
            true,
        ),
    ] {
        if let Some(rest) = l.strip_prefix(prefix) {
            let cost_s = rest
                .strip_suffix(" for each creature they control that's attacking you")
                .or_else(|| rest.strip_suffix(" for each of those creatures"))?;
            let (cost, _) = crate::oracle::costs::parse_cost(cost_s)?;
            return Some(vec![r(Restriction::AttackCost {
                attackers: Filter::creature(),
                defender: PlayerFilter::You,
                planeswalkers,
                cost,
            })]);
        }
    }
    // Block costs (CR 509.1d).
    if let Some(rest) = l.strip_prefix("creatures can't block unless their controller pays ") {
        let cost_s = rest
            .strip_suffix(" for each blocking creature they control")
            .or_else(|| rest.strip_suffix(" for each of those creatures"))?;
        let (cost, _) = crate::oracle::costs::parse_cost(cost_s)?;
        return Some(vec![r(Restriction::BlockCost {
            blockers: Filter::creature(),
            cost,
        })]);
    }
    None
}

inventory::submit! { StaticPattern { name: "combat statics", priority: 100, parse: combat_static } }

/// "during combat before blockers are declared" etc. → a combat timing condition.
fn timing_condition(s: &str) -> Option<Condition> {
    let mut conds: Vec<Condition> = Vec::new();
    let mut s = s.trim();
    let mut during_combat = false;
    for (p, c) in [
        ("during combat on your turn", Some(Condition::YourTurn)),
        (
            "during combat on an opponent's turn",
            Some(Condition::NotYourTurn),
        ),
        ("during combat", None),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            during_combat = true;
            if let Some(c) = c {
                conds.push(c);
            }
            s = r.trim();
            break;
        }
    }
    let point = |w: &str| -> Option<CombatPoint> {
        Some(match w {
            "attackers are declared" => CombatPoint::AttackersDeclared,
            "blockers are declared" => CombatPoint::BlockersDeclared,
            "the combat damage step" => CombatPoint::CombatDamageStep,
            "the end of combat step" => CombatPoint::EndOfCombatStep,
            "combat" | "the combat phase" => CombatPoint::Combat,
            _ => return None,
        })
    };
    if let Some(r) = s.strip_prefix("before ") {
        conds.push(Condition::CombatTiming(CombatTiming {
            point: point(r)?,
            after: false,
            during_combat,
        }));
    } else if let Some(r) = s.strip_prefix("after ") {
        conds.push(Condition::CombatTiming(CombatTiming {
            point: point(r)?,
            after: true,
            during_combat,
        }));
    } else if s.is_empty() && during_combat {
        conds.push(Condition::Phase(PhaseCond::Combat));
    } else {
        return None;
    }
    Some(if conds.len() == 1 {
        conds.pop().unwrap()
    } else {
        Condition::And(conds)
    })
}

/// "Cast ~ only during combat before blockers are declared." (CR 506.8)
fn cast_only(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let l = end(lower.trim());
    let r = l
        .strip_prefix("cast ~ only ")
        .or_else(|| l.strip_prefix("cast this spell only "))?;
    let cond = timing_condition(r)?;
    let mut s = StaticAbility::new(StaticEffect::CastOnlyIf(cond));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block)])
}

inventory::submit! { AbilityPattern { name: "cast only (combat timing)", priority: 100, parse: cast_only } }

/// "You may exert ~ as it attacks. When you do, [effect]." (CR 508.1g)
fn exert_as_attacks(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let l = lower.trim();
    let rest = l.strip_prefix("you may exert ~ as it attacks.")?;
    let then = if rest.trim().is_empty() {
        None
    } else {
        let orig_rest = block.trim()[block.trim().len() - rest.len()..].trim();
        let eff = orig_rest
            .strip_prefix("When you do, ")
            .or_else(|| orig_rest.strip_prefix("when you do, "))?;
        let body = crate::oracle::effects::parse_trigger_body(eff, ctx, Sel::This, PlayerRef::You)?;
        Some(Box::new(body))
    };
    let s = StaticAbility::new(StaticEffect::OptionalAttackCost {
        cost: Cost::default().with(CostPart::ExertSelf),
        then,
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block)])
}

inventory::submit! { AbilityPattern { name: "exert as it attacks", priority: 100, parse: exert_as_attacks } }
