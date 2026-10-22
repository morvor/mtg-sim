//! CR 702.165 Backup: "Backup N" means "When this creature enters, put N +1/+1 counters on
//! target creature. If that's another creature, it also gains the non-backup abilities of
//! this creature printed below this one until end of turn." (CR 702.165a).
//!
//! What the ability grants depends on the card's printed text, so it's compiled from the
//! text rather than derived from the keyword: the backup line is grouped with the lines
//! printed below it, and each instance of backup becomes a triggered ability (labeled
//! "Backup N") that carries the abilities those lines compile to, alongside them. So:
//!
//! * only abilities printed on the object are granted — abilities it gains from other
//!   effects, including a copy effect's exceptions, aren't part of the list (CR 702.165c);
//! * a permanent that's a copy of it copies the triggered ability with its list, so the
//!   order of the printed abilities is maintained (CR 702.165b);
//! * the list is part of the ability on the stack, fixed as it's put there: it doesn't
//!   change if the creature loses abilities before the ability resolves (CR 702.165d);
//! * if the target is the creature with backup itself, it only gets the counters.

use super::{AbilityPattern, BlockGroupPattern};
use crate::ability::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// Marks a grouped backup block: "{BACKUP} Backup 1" followed by the lines printed below it,
/// each after [`BELOW`].
const MARK: &str = "{BACKUP} ";
/// Separates the lines of a grouped backup block.
const BELOW: &str = "\n{BELOW} ";

/// "Backup 1" → [1]; "Backup 1, backup 1, backup 1" → [1, 1, 1] (CR 702.165a: each
/// instance triggers separately).
fn backup_line(block: &str) -> Option<Vec<i32>> {
    block
        .trim()
        .trim_end_matches('.')
        .split(", ")
        .map(|part| {
            let lower = part.trim().to_lowercase();
            lower.strip_prefix("backup ")?.trim().parse::<i32>().ok()
        })
        .collect()
}

/// Groups the backup line with the ability blocks printed below it, if they all compile
/// (otherwise the card isn't supported anyway, and each line is reported on its own).
fn group_backup(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    let Some(i) = blocks.iter().position(|b| backup_line(b).is_some()) else {
        return blocks;
    };
    let below = &blocks[i + 1..];
    if below.is_empty()
        || below
            .iter()
            .any(|b| crate::oracle::parse_ability(b, ctx).is_none())
    {
        return blocks;
    }
    let mut out: Vec<String> = blocks[..i].to_vec();
    let mut group = format!("{MARK}{}", blocks[i].trim());
    for b in below {
        group.push_str(BELOW);
        group.push_str(b.trim());
    }
    out.push(group);
    out
}

inventory::submit! { BlockGroupPattern { name: "k702.165 backup and the abilities below it", priority: 80, group: group_backup } }

/// Whether a compiled ability is itself a backup ability (not granted by another one).
fn is_backup(a: &Ability) -> bool {
    backup_line(&a.text).is_some()
}

/// The triggered ability "Backup N" granting `granted`.
fn backup_trigger(n: i32, granted: &[Ability], text: &str) -> Ability {
    let mut effects = vec![Effect::AddCounters {
        what: Sel::Target(0),
        kind: counters::PLUS1.into(),
        n: Value::c(n),
    }];
    let mods: Vec<Modification> = granted
        .iter()
        .map(|a| match &a.kind {
            AbilityKind::Keyword(k) => Modification::AddKeyword(k.clone()),
            _ => Modification::AddAbility(a.clone()),
        })
        .collect();
    if !mods.is_empty() {
        // "If that's another creature, it also gains [them] until end of turn."
        effects.push(Effect::If {
            cond: Condition::SelMatches(Sel::Target(0), Filter::Other),
            then: Box::new(Effect::Modify {
                what: Sel::Target(0),
                mods,
                duration: Duration::EndOfTurn,
            }),
            otherwise: Box::new(Effect::Noop),
        });
    }
    let t = TriggeredAbility::new(
        TriggerCond::EntersBattlefield(Filter::Source),
        Body::simple(
            vec![TargetSpec::object(Filter::creature(), "target creature")],
            Effect::Seq(effects),
        ),
    );
    AbilityDef::new(AbilityKind::Triggered(t), text)
}

/// A grouped backup block (see [`group_backup`]): the backup abilities, then the abilities
/// printed below them.
fn backup_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.strip_prefix(MARK)?;
    let mut parts = r.split(BELOW);
    let head = parts.next()?.trim();
    let instances = backup_line(head)?;
    let mut below: Vec<Ability> = Vec::new();
    for p in parts {
        below.extend(crate::oracle::parse_ability(p.trim(), ctx)?);
    }
    // CR 702.165a: the non-backup abilities printed below it.
    let granted: Vec<Ability> = below.iter().filter(|a| !is_backup(a)).cloned().collect();
    let mut out: Vec<Ability> = Vec::new();
    for (i, n) in instances.iter().enumerate() {
        let text = if instances.len() == 1 {
            head.trim_end_matches('.').to_string()
        } else {
            let part = head.split(", ").nth(i).unwrap_or(head).trim();
            let mut t = part.to_string();
            if let Some(first) = t.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            t
        };
        out.push(backup_trigger(*n, &granted, &text));
    }
    out.extend(below);
    Some(out)
}

inventory::submit! { AbilityPattern { name: "k702.165 backup", priority: 80, parse: backup_block } }
