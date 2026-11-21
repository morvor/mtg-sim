//! Combat history V (cards4-referents-statics, family `combat-history`).
//!
//! - "At the beginning of that player's next end step, you draw a card if they didn't
//!   attack you that turn. Otherwise, ..." (Faramir, Prince of Ithilien): whether the
//!   player the text names declared creatures attacking you during the turn the ability
//!   resolves in (CR 508.1, 508.6; attacking a planeswalker you control doesn't count,
//!   Faramir rulings).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// "[instruction] if they didn't attack you that turn" / "... this turn".
fn unless_they_attacked_you(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, attacked) = [
        (" if they didn't attack you that turn", false),
        (" if they didn't attack you this turn", false),
        (" if that player didn't attack you that turn", false),
        (" if that player didn't attack you this turn", false),
        (" if they attacked you that turn", true),
        (" if that player attacked you that turn", true),
    ]
    .iter()
    .find_map(|(p, a)| l.strip_suffix(p).map(|c| (c, *a)))?;
    // One instruction (a delayed trigger's "At the beginning of ..., " is its own).
    if clause.starts_with("at ") || clause.contains(", ") {
        return None;
    }
    let who = b.it_player.clone();
    if matches!(who, PlayerRef::You)
        || super::oracle_hardening_referents::is_no_player_referent(&who)
    {
        return None;
    }
    let e = parse_clause(clause, b)?;
    let did = Condition::PlayerMatches(
        who,
        PlayerFilter::AttackRelation {
            attacking: true,
            other: Box::new(PlayerFilter::You),
            past: true,
            planeswalkers: false,
        },
    );
    Some(Effect::If {
        cond: if attacked {
            did
        } else {
            Condition::Not(Box::new(did))
        },
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "combat history 5: [instruction] if they didn't attack you that turn", priority: 100, parse: unless_they_attacked_you } }

/// The creature "one of those creatures" chose.
const ONE_IN_BLOCK: Var = vars::USER + 51_001;

/// "Whenever equipped creature blocks or becomes blocked by one or more creatures, you may
/// exile one of those creatures." (Godsend): one of the creatures in a block with the
/// equipped creature this combat (CR 509.1h, 506.4; chosen as the ability resolves, it
/// doesn't target, Godsend rulings).
fn one_of_those_in_the_block(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, after) = l.split_once(" one of those creatures")?;
    if head.contains(' ') || head.is_empty() || after.contains("those") {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let other = if raw.contains("whenever equipped creature blocks or becomes blocked by one or more creatures,")
        || raw.contains("whenever enchanted creature blocks or becomes blocked by one or more creatures,")
    {
        Sel::AttachedTo
    } else if raw.contains("whenever ~ blocks or becomes blocked by one or more creatures,") {
        Sel::This
    } else {
        return None;
    };
    let choose = Effect::Store {
        var: ONE_IN_BLOCK,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::creature(),
                Filter::InZone(ZoneKind::Battlefield),
                Filter::BlockRelation {
                    rel: BlockRel::BlockingOrBlockedBy,
                    other: Box::new(other),
                    when: BlockWhen::ThisCombat,
                },
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    let saved = b.it.clone();
    b.it = Sel::Var(ONE_IN_BLOCK);
    let e = parse_clause(&format!("{head} it{after}"), b);
    if matches!(b.it, Sel::Var(ONE_IN_BLOCK)) {
        b.it = saved;
    }
    Some(Effect::seq(vec![choose, e?]))
}

inventory::submit! { EffectPattern { name: "combat history 5: [verb] one of those creatures (in a block with it)", priority: 100, parse: one_of_those_in_the_block } }

/// "At the beginning of that player's next end step, you draw a card if they didn't attack
/// you that turn. Otherwise, create three 1/1 ... tokens." (Faramir): the alternative of
/// the delayed triggered ability's conditional instruction (CR 603.7), part of its effect.
/// Only an instruction without referents or targets of its own (which would need the
/// delayed ability's captured context).
fn otherwise_in_delayed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("otherwise, ") else {
        return false;
    };
    let last = match prev {
        Effect::Seq(v) => match v.last_mut() {
            Some(x) => x,
            None => return false,
        },
        e => e,
    };
    let Effect::DelayedTrigger { body, .. } = last else {
        return false;
    };
    let Effect::If { otherwise, .. } = &mut body.effect else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let before = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = parse_clause(rest, b) else {
        return false;
    };
    let json = serde_json::to_string(&e).unwrap_or_default();
    if b.targets.len() != before.0
        || ["Trigger", "\"It\"", "Var", "Iterated"]
            .iter()
            .any(|w| json.contains(w))
    {
        return false;
    }
    **otherwise = e;
    true
}

inventory::submit! { super::FollowupPattern { name: "combat history 5: otherwise, [instruction] (in a delayed ability)", priority: 100, apply: otherwise_in_delayed } }
