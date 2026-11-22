//! Combat and untap grammar of old (paper) card wordings:
//!
//! * "Each attacking creature and each blocking creature doesn't untap during its
//!   controller's next untap step." (Spore Cloud, Tangle): the creatures in those groups as
//!   the effect begins (Tangle's rulings: "the creatures that are not going to untap are
//!   determined at the time this spell resolves"), through each one's controller's next
//!   untap step (CR 502.3; Spore Cloud's rulings: whoever controls it then);
//! * "It doesn't untap during its controller's next two untap steps." (Telekinesis): the
//!   same for a count of its controllers' untap steps
//!   ([`Duration::ThroughNextUntapSteps`]);
//! * "~ deals 2 damage to any target and doesn't untap during your next untap step."
//!   (Reveka, Wizard Savant): an instruction for the source and the untap restriction on it
//!   through your next untap step (Reveka's ruling: another controller untaps it as usual).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::patterns::statics::whole_object_phrase;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "each attacking creature and each blocking creature doesn't untap during its
/// controller's next untap step".
fn each_group_doesnt_untap(l: &str, _b: &mut Builder) -> Option<Effect> {
    let subject =
        end(l.trim()).strip_suffix(" doesn't untap during its controller's next untap step")?;
    let mut groups = Vec::new();
    for part in subject.split(" and ") {
        let (f, plural) = whole_object_phrase(part.strip_prefix("each ")?)?;
        if plural {
            return None;
        }
        groups.push(f);
    }
    let f = if groups.len() == 1 {
        groups.pop()?
    } else {
        Filter::Or(groups)
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::DoesntUntap(Filter::In(Box::new(Sel::All(f)))),
        duration: Duration::ThroughNextUntapStep,
    })
}

inventory::submit! { EffectPattern { name: "paper combat: each [group] doesn't untap during its controller's next untap step", priority: 105, parse: each_group_doesnt_untap } }

/// "it doesn't untap during its controller's next two untap steps".
fn doesnt_untap_next_n_steps(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (head, tail) = l.split_once(" doesn't untap during its controller's next ")?;
    let n = match tail.strip_suffix(" untap steps")? {
        "two" => 2,
        "three" => 3,
        _ => return None,
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(
        &format!("{head} doesn't untap during its controller's next untap step"),
        b,
    );
    match e {
        Some(Effect::AddRestriction {
            restriction: r @ Restriction::DoesntUntap(Filter::In(_)),
            duration: Duration::ThroughNextUntapStep,
        }) => Some(Effect::AddRestriction {
            restriction: r,
            duration: Duration::ThroughNextUntapSteps(n),
        }),
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "paper combat: doesn't untap during its controller's next two untap steps", priority: 105, parse: doesnt_untap_next_n_steps } }

/// "~ deals 2 damage to any target and doesn't untap during your next untap step".
fn source_acts_and_doesnt_untap(l: &str, b: &mut Builder) -> Option<Effect> {
    let head = end(l.trim()).strip_suffix(" and doesn't untap during your next untap step")?;
    if !head.starts_with("~ ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(first) = parse_clause(head, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    };
    Some(Effect::seq(vec![
        first,
        Effect::AddRestriction {
            restriction: Restriction::DoesntUntap(Filter::In(Box::new(Sel::This))),
            duration: Duration::ThroughYourNextUntapStep,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "paper combat: ~ [does something] and doesn't untap during your next untap step", priority: 115, parse: source_acts_and_doesnt_untap } }

std::thread_local! {
    /// Set while a block whose trigger is an attack ("Whenever enchanted creature attacks
    /// and isn't blocked, ...") is parsed again by [`attacking_creature_trigger`].
    static IN_ATTACK_TRIGGER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether a trigger is about one creature attacking (its trigger object is "the attacking
/// creature").
fn attack_trigger(t: &TriggerCond) -> bool {
    matches!(
        t,
        TriggerCond::AttacksUnblocked(_) | TriggerCond::Attacks(_)
    )
}

/// "Whenever enchanted creature attacks and isn't blocked, its controller may have it
/// deal damage equal to its power plus 2 to another target creature. If that player does,
/// the attacking creature assigns no combat damage this turn." (Farrel's Mantle): "the
/// attacking creature" is the creature of the trigger event. "Another target creature"
/// after "have it deal damage" is a creature other than "it" (Farrel's Mantle's ruling:
/// the enchanted creature can't deal the damage to itself).
fn attacking_creature_trigger(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    if IN_ATTACK_TRIGGER.with(|f| f.get())
        || !lower.starts_with("whenever ")
        || !lower.contains(" the attacking creature ")
    {
        return None;
    }
    IN_ATTACK_TRIGGER.with(|f| f.set(true));
    let parsed = crate::oracle::parse_ability(block, ctx);
    IN_ATTACK_TRIGGER.with(|f| f.set(false));
    let mut parsed = parsed?;
    let [a] = parsed.as_mut_slice() else {
        return None;
    };
    let AbilityKind::Triggered(t) = &mut std::sync::Arc::make_mut(a).kind else {
        return None;
    };
    if !attack_trigger(&t.trigger) {
        return None;
    }
    if lower.contains(" have it deal ") && lower.contains(" to another target creature") {
        let [spec] = t.body.targets.as_mut_slice() else {
            return None;
        };
        let TargetKind::Object(Filter::And(v)) = &mut spec.what else {
            return None;
        };
        let i = v.iter().position(|f| matches!(f, Filter::Other))?;
        v[i] = Filter::not(Filter::In(Box::new(Sel::TriggerObject)));
    }
    Some(parsed)
}

inventory::submit! { super::AbilityPattern { name: "paper combat: the attacking creature (in an attack trigger)", priority: 100, parse: attacking_creature_trigger } }

/// "the attacking creature [instruction]" in an attack trigger: the creature of the
/// trigger event.
fn the_attacking_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger || !IN_ATTACK_TRIGGER.with(|f| f.get()) {
        return None;
    }
    let rest = end(l.trim()).strip_prefix("the attacking creature ")?;
    if rest.contains(" it ") || rest.contains(" its ") || rest.ends_with(" it") {
        return None;
    }
    let saved = std::mem::replace(&mut b.it, Sel::TriggerObject);
    let e = parse_clause(&format!("it {rest}"), b);
    b.it = saved;
    e
}

inventory::submit! { EffectPattern { name: "paper combat: the attacking creature (the trigger event's)", priority: 100, parse: the_attacking_creature } }

/// "that blocked ~ this turn", "that were blocked by ~ this combat", "that blocked or were
/// blocked by ~ this turn" (Joven's Ferrets: "tap all creatures that blocked ~ this
/// turn"): block history with the source (see `kw::block_history`).
fn blocked_source_suffix<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    for (head, rel) in [
        ("that blocked or were blocked by ~", BlockRel::BlockingOrBlockedBy),
        ("that blocked ~", BlockRel::Blocking),
        ("that were blocked by ~", BlockRel::BlockedBy),
    ] {
        let Some(r) = s.strip_prefix(head) else {
            continue;
        };
        for (w, when) in [
            (" this turn", BlockWhen::ThisTurn),
            (" this combat", BlockWhen::ThisCombat),
        ] {
            if let Some(rest) = r.strip_prefix(w) {
                if rest.starts_with(|c: char| c.is_alphanumeric()) {
                    return None;
                }
                return Some((
                    Filter::BlockRelation {
                        rel,
                        other: Box::new(Sel::This),
                        when,
                    },
                    rest,
                ));
            }
        }
    }
    None
}

inventory::submit! { super::FilterSuffixPattern { name: "paper combat: that blocked ~ this turn", priority: 100, parse: blocked_source_suffix } }

/// The coin flip's "if you lose" instructions, when they remove the source from combat.
fn lose_branch_removing_source(e: &mut Effect) -> Option<&mut Effect> {
    fn removes_source(e: &Effect) -> bool {
        match e {
            Effect::RemoveFromCombat { what: Sel::This } => true,
            Effect::Seq(v) => v.iter().any(removes_source),
            _ => false,
        }
    }
    match e {
        Effect::Seq(v) => v.last_mut().and_then(lose_branch_removing_source),
        Effect::FlipCoins(f) if removes_source(&f.on_lose) => Some(&mut f.on_lose),
        _ => None,
    }
}

/// "Whenever ~ blocks, flip a coin. If you lose the flip, remove ~ from combat and it
/// can't block this turn. Creatures it was blocking that had become blocked by only ~
/// this combat become unblocked." (Ydwen Efreet): the creatures become unblocked when ~
/// is removed from combat, as part of the lost flip's instructions (if the flip is won, ~
/// is still blocking them). Like False Orders' "only that creature": attacking creatures
/// whose only blocker this combat was ~ (CR 509.1h, 506.4).
fn f_unblock_after_removed(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l.trim())
        != "creatures it was blocking that had become blocked by only ~ this combat become unblocked"
    {
        return false;
    }
    let Some(branch) = lose_branch_removing_source(prev) else {
        return false;
    };
    let unblock = Effect::BecomeUnblocked {
        what: Sel::All(Filter::and(vec![
            Filter::creature(),
            Filter::Attacking,
            Filter::BlockRelation {
                rel: BlockRel::BlockedOnlyBy,
                other: Box::new(Sel::This),
                when: BlockWhen::ThisCombat,
            },
        ])),
    };
    let old = std::mem::take(branch);
    *branch = Effect::seq(vec![old, unblock]);
    true
}

inventory::submit! { super::FollowupPattern { name: "paper combat: creatures it was blocking ... only ~ become unblocked (after removing ~ from combat)", priority: 50, apply: f_unblock_after_removed } }

/// A static ability's subject: "~" or a group.
fn static_subject(s: &str) -> Option<Filter> {
    if s == "~" {
        return Some(Filter::Source);
    }
    whole_object_phrase(s).map(|(f, _)| f)
}

/// "~ can't attack unless at least two other creatures attack", "~ can't block unless at
/// least two other creatures block" (Orcish Conscripts; CR 508.1c, 509.1b).
fn cant_unless_others(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (subject, rest, attack) = if let Some((s, r)) = l.split_once(" can't attack unless ") {
        (s, r.strip_suffix(" attack")?, true)
    } else {
        let (s, r) = l.split_once(" can't block unless ")?;
        (s, r.strip_suffix(" block")?, false)
    };
    let (n, noun) = if let Some(r) = rest.strip_prefix("at least ") {
        let (n, r) = crate::oracle::phrases::parse_number(r)?;
        (n.as_const()?, r.trim())
    } else {
        let (n, r) = crate::oracle::phrases::parse_number(rest)?;
        (n.as_const()?, r.trim().strip_prefix("or more ")?)
    };
    if n < 1 || !matches!(noun, "other creatures" | "other creature") {
        return None;
    }
    let what = static_subject(subject)?;
    let r = if attack {
        Restriction::CantAttackUnlessOthersAttack { what, n: n as u32 }
    } else {
        Restriction::CantBlockUnlessOthersBlock { what, n: n as u32 }
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "paper combat: can't attack/block unless at least N other creatures do", priority: 100, parse: cant_unless_others } }

/// "Except for creatures named ~ and artifact creatures, creatures you control can't
/// attack." (Akron Legionnaire): the restriction's subject without the excepted groups
/// (Akron Legionnaire's ruling: every creature named ~ can attack).
fn except_for_groups(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (except, main) = end(l.trim()).strip_prefix("except for ")?.split_once(", ")?;
    let mut excluded = Vec::new();
    for part in except.replace(", and ", ", ").replace(" and ", ", ").split(", ") {
        let (f, plural) = whole_object_phrase(part)?;
        if !plural {
            return None;
        }
        excluded.push(f);
    }
    let mut abilities = crate::oracle::statics::parse_static(&format!("{main}."), ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &mut std::sync::Arc::make_mut(a).kind else {
        return None;
    };
    let StaticEffect::Restriction(
        Restriction::CantAttack(f) | Restriction::CantBlock(f) | Restriction::CantAttackOrBlock(f),
    ) = &mut s.effect
    else {
        return None;
    };
    // The excepted objects must be a part of the group the sentence restricts.
    if matches!(f, Filter::Source | Filter::In(_)) {
        return None;
    }
    let none_of = Filter::not(if excluded.len() == 1 {
        excluded.pop()?
    } else {
        Filter::Or(excluded)
    });
    *f = Filter::and(vec![std::mem::replace(f, Filter::Any), none_of]);
    let s = s.clone();
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { super::StaticPattern { name: "paper combat: except for [groups], [group] can't attack", priority: 100, parse: except_for_groups } }
