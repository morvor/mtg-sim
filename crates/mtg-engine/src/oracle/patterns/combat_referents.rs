//! Combat referents and player phrases of combat:
//!
//! * object phrases: "creatures the active player controls", "non-Wall creature the
//!   active player has controlled continuously since the beginning of the turn" (CR
//!   302.6), "creatures blocking or blocked by target creature" (CR 509.1g);
//! * a player subject shared by two instructions: "the attacking player gains control of
//!   ~ and untaps it".

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, player_ref, Builder};
use crate::oracle::phrases::end;

/// "[creatures] the active player controls", "[creature] the active player has
/// controlled continuously since the beginning of the turn".
fn active_player_suffix<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    if let Some(rest) = r.strip_prefix(
        "the active player has controlled continuously since the beginning of the turn",
    ) {
        return Some((
            Filter::and(vec![
                Filter::ControlledByPlayer(Box::new(PlayerRef::ActivePlayer)),
                Filter::ControlledSinceTurnBegan,
            ]),
            rest,
        ));
    }
    let rest = r.strip_prefix("the active player controls")?;
    Some((
        Filter::ControlledByPlayer(Box::new(PlayerRef::ActivePlayer)),
        rest,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "combat referents: the active player controls", priority: 100, parse: active_player_suffix } }

/// "[creatures that player controls] that didn't attack" (Total War, as attackers are
/// declared): not declared as attackers this combat (CR 508.1).
fn didnt_attack_suffix<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = r.strip_prefix("that didn't attack")?;
    if rest.starts_with(" this turn") || rest.starts_with(" during") {
        return None;
    }
    Some((Filter::not(Filter::AttackedThisCombat), rest))
}

inventory::submit! { FilterSuffixPattern { name: "combat referents: that didn't attack", priority: 100, parse: didnt_attack_suffix } }

/// Verbs (third person) that can continue a player subject after "and".
const PLAYER_VERBS: &[&str] = &[
    "untaps",
    "taps",
    "draws",
    "discards",
    "loses",
    "gains",
    "sacrifices",
    "mills",
    "creates",
    "shuffles",
    "exiles",
    "returns",
    "puts",
    "reveals",
];

/// "[player] gains control of ~ and untaps it": both instructions have the player as
/// their subject.
fn shared_player_subject(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (_who, rest) = player_ref(l, b)?;
    let subject = l[..l.len() - rest.len()].trim();
    if subject.is_empty() || subject.contains("target") {
        return None;
    }
    let (first, second) = rest
        .match_indices(" and ")
        .map(|(i, _)| (&rest[..i], &rest[i + " and ".len()..]))
        .find(|(_, s)| {
            s.split(' ')
                .next()
                .is_some_and(|w| PLAYER_VERBS.contains(&w))
        })?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let a = parse_clause(&format!("{subject} {}", first.trim()), b);
    // Tapping and untapping don't depend on who does it: "untap it".
    let second = match second.split_once(' ') {
        Some(("untaps", r)) => format!("untap {r}"),
        Some(("taps", r)) => format!("tap {r}"),
        _ => format!("{subject} {second}"),
    };
    // "gains control of ~ and untaps it": "it" is the object just named.
    if first.trim().ends_with(" ~")
        && super::oracle_hardening_referents::is_no_referent(&super::pronoun_groups::singular_it(b))
    {
        b.it = Sel::This;
    }
    let c = a.as_ref().and_then(|_| parse_clause(&second, b));
    match (a, c) {
        (Some(a), Some(c)) => Some(Effect::seq(vec![a, c])),
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "combat referents: a player subject shared by two instructions", priority: 990, parse: shared_player_subject } }

/// "[all creatures] blocking or blocked by target creature [rest]": the creatures blocking
/// that creature or blocked by it as the effect happens (CR 509.1g).
fn blocking_or_blocked_by_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    const PHRASE: &str = " blocking or blocked by target creature";
    let at = l.find(PHRASE)?;
    let (head, tail) = (&l[..at], &l[at + PHRASE.len()..]);
    if head.contains("target") || head.contains('~') || tail.contains("target") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (t, rest) = object_ref("target creature", b)?;
    let Sel::Target(slot) = t else {
        b.targets.truncate(saved.0);
        return None;
    };
    if !rest.trim().is_empty() {
        b.targets.truncate(saved.0);
        return None;
    }
    // Parse with "~" standing in for the target, then point the relation at the target.
    let e = parse_clause(&format!("{head} blocking or blocked by ~{tail}"), b);
    let ok = e.and_then(|mut e| {
        let mut n = 0;
        retarget(&mut e, slot, &mut n);
        (n == 1 && !mentions_source(&e)).then_some(e)
    });
    if ok.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    ok
}

/// Replaces "blocking or blocked by ~" in an effect's selections with the relation to the
/// target in `slot`, counting the replacements.
fn retarget(e: &mut Effect, slot: u8, n: &mut u32) {
    let mut v = serde_json::to_value(&*e).unwrap_or_default();
    fn walk(v: &mut serde_json::Value, slot: u8, n: &mut u32) {
        let either = serde_json::json!({"Or": ["BlockingSource", "BlockedBySource"]});
        if *v == either {
            *v = serde_json::to_value(Filter::BlockRelation {
                rel: BlockRel::BlockingOrBlockedBy,
                other: Box::new(Sel::Target(slot)),
                when: BlockWhen::Now,
            })
            .unwrap_or_default();
            *n += 1;
            return;
        }
        match v {
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, slot, n)),
            serde_json::Value::Object(o) => o.values_mut().for_each(|x| walk(x, slot, n)),
            _ => {}
        }
    }
    walk(&mut v, slot, n);
    if let Ok(new) = serde_json::from_value(v) {
        *e = new;
    } else {
        *n = 0;
    }
}

/// Whether the effect still refers to the source.
fn mentions_source(e: &Effect) -> bool {
    let s = serde_json::to_string(e).unwrap_or_default();
    s.contains("\"Source\"") || s.contains("BlockingSource") || s.contains("BlockedBySource")
}

inventory::submit! { EffectPattern { name: "combat referents: blocking or blocked by target creature", priority: 80, parse: blocking_or_blocked_by_target } }

/// "creatures attacking the last chosen player" (Beckoning Will-o'-Wisp, Triarch
/// Stalker): attacking the player most recently chosen for the source (CR 607.2d).
fn last_chosen_player_suffix<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = r.strip_prefix("attacking the last chosen player")?;
    Some((Filter::AttackingPlayer(PlayerRel::Chosen), rest))
}

inventory::submit! { FilterSuffixPattern { name: "combat referents: attacking the last chosen player", priority: 100, parse: last_chosen_player_suffix } }

/// "~ deals 3 damage to you and attacks this turn if able" (Kookus): the source is the
/// subject of both instructions.
fn source_and_attacks(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let head = l.strip_suffix(" and attacks this turn if able")?;
    if !head.starts_with("~ ") || head.contains(" and ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let a = parse_clause(head, b);
    let c = a
        .as_ref()
        .and_then(|_| parse_clause("~ attacks this turn if able", b));
    match (a, c) {
        (Some(a), Some(c)) => Some(Effect::seq(vec![a, c])),
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "combat referents: ~ [does something] and attacks this turn if able", priority: 90, parse: source_and_attacks } }

/// "it can't be blocked by creatures with greater power this combat" (Tadeas): greater
/// than the attacking creature's power as blockers are declared.
fn cant_be_blocked_by_greater_power(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (subject, duration) = if let Some(s) =
        l.strip_suffix(" can't be blocked by creatures with greater power this combat")
    {
        (s, Duration::EndOfCombat)
    } else {
        let s = l.strip_suffix(" can't be blocked by creatures with greater power this turn")?;
        (s, Duration::EndOfTurn)
    };
    if !matches!(subject, "it" | "that creature") {
        return None;
    }
    let (what, rest) = object_ref(subject, b)?;
    if !rest.trim().is_empty()
        || !matches!(what, Sel::TriggerObject | Sel::Target(_) | Sel::This)
        || super::oracle_hardening_referents::is_no_referent(&what)
    {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::CantBeBlockedBy {
            attacker: Filter::In(Box::new(what)),
            blocker: Filter::and(vec![
                Filter::creature(),
                Filter::Power(
                    Cmp::Gt,
                    Box::new(Value::PowerOf(Box::new(Sel::Var(vars::AFFECTED)))),
                ),
            ]),
        },
        duration,
    })
}

inventory::submit! { EffectPattern { name: "combat referents: can't be blocked by creatures with greater power this combat", priority: 90, parse: cant_be_blocked_by_greater_power } }

/// "Equipped creature has first strike as long as it's blocking or blocked by a Goblin or
/// Orc." (Sting): while a creature matching the noun blocks it or is blocked by it.
fn as_long_as_blocking_or_blocked_by(
    l: &str,
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (body, noun) = l.split_once(" as long as it's blocking or blocked by ")?;
    let it = if body.starts_with("equipped creature ") || body.starts_with("enchanted creature ") {
        Sel::AttachedTo
    } else if body.starts_with("~ ") {
        Sel::This
    } else {
        return None;
    };
    let noun = noun
        .strip_prefix("a ")
        .or_else(|| noun.strip_prefix("an "))?;
    let (f, plural) = super::statics::whole_object_phrase(noun)?;
    if plural {
        return None;
    }
    let cond = Condition::Exists(Filter::and(vec![
        f,
        Filter::BlockRelation {
            rel: BlockRel::BlockingOrBlockedBy,
            other: Box::new(it),
            when: BlockWhen::Now,
        },
    ]));
    let cut = text.to_lowercase().find(" as long as ")?;
    let mut abilities = crate::oracle::statics::parse_static(&format!("{}.", &text[..cut]), ctx)?;
    if abilities.is_empty() {
        return None;
    }
    for a in abilities.iter_mut() {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let mut s = s.clone();
        if s.condition.is_some() {
            return None;
        }
        s.condition = Some(cond.clone());
        *a = AbilityDef::new(AbilityKind::Static(s), text);
    }
    Some(abilities)
}

inventory::submit! { super::StaticPattern { name: "combat referents: as long as it's blocking or blocked by", priority: 110, parse: as_long_as_blocking_or_blocked_by } }

/// "at least one other Wall creature is blocking that creature and no non-Wall creatures
/// are blocking that creature" (Wall of Caltrops, after "whenever ~ blocks a creature":
/// that creature is the blocked one): counts of the creatures blocking it.
fn blocking_that_creature(c: &str) -> Option<Condition> {
    fn part(c: &str) -> Option<Condition> {
        let blocking = Filter::BlockRelation {
            rel: BlockRel::Blocking,
            other: Box::new(Sel::TriggerObject),
            when: BlockWhen::Now,
        };
        let (cmp, n, noun) = if let Some(r) = c.strip_prefix("at least one other ") {
            (
                Cmp::Ge,
                1,
                format!("other {}", r.strip_suffix(" is blocking that creature")?),
            )
        } else if let Some(r) = c.strip_prefix("no ") {
            (
                Cmp::Eq,
                0,
                r.strip_suffix(" are blocking that creature")?.to_string(),
            )
        } else {
            return None;
        };
        let (f, _) = super::statics::whole_object_phrase(&noun)?;
        Some(Condition::Compare(
            Value::Count(Filter::and(vec![f, blocking])),
            cmp,
            Value::c(n),
        ))
    }
    let c = end(c.trim());
    match c.split_once(" and ") {
        Some((a, b)) => Some(Condition::And(vec![part(a)?, part(b)?])),
        None => part(c),
    }
}

inventory::submit! { super::ConditionPattern { name: "combat referents: creatures blocking that creature", priority: 100, parse: blocking_that_creature } }

/// The internal form of "attacking that player" when "that player" is the trigger's
/// player (see `attacking_that_player`).
const ATTACKING_TRIGGER_PLAYER: &str = "attacking the-trigger-player";

fn attacking_trigger_player_suffix<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = r.strip_prefix(ATTACKING_TRIGGER_PLAYER)?;
    Some((Filter::AttackingPlayer(PlayerRel::TriggerPlayer), rest))
}

inventory::submit! { FilterSuffixPattern { name: "combat referents: attacking the trigger's player", priority: 100, parse: attacking_trigger_player_suffix } }

/// "[creatures you control] attacking that player get +2/+0 until end of turn" in a
/// trigger whose player is the attacked one (Namor: "whenever ~ attacks a player ...").
fn attacking_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    if !l.contains(" attacking that player ") || !matches!(b.it_player, PlayerRef::TriggerPlayer) {
        return None;
    }
    let text = l.replacen(
        " attacking that player ",
        &format!(" {ATTACKING_TRIGGER_PLAYER} "),
        1,
    );
    parse_clause(&text, b)
}

inventory::submit! { EffectPattern { name: "combat referents: creatures attacking that player", priority: 90, parse: attacking_that_player } }
