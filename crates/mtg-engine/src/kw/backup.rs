//! CR 702.165 Backup: "Backup N" means "When this creature enters, put N +1/+1 counters on
//! target creature. If that's another creature, it also gains the non-backup abilities of
//! this creature printed below this one until end of turn." (CR 702.165a).
//!
//! * The abilities printed below this backup ability are found on the card whose printed
//!   characteristics the creature has (`Characteristics::printed`), which a copy of the
//!   creature has too, so the order of abilities printed on it is kept (CR 702.165b).
//!   Abilities the creature gained otherwise — from a copy effect's exceptions, an effect
//!   granting abilities, or an effect creating a token with abilities — aren't printed on
//!   it and aren't granted (CR 702.165c).
//! * They're determined as the ability is put on the stack (CR 702.165d): from the
//!   creature's characteristics as they were then (the stack object's `source_lki`), so
//!   abilities it loses afterwards are still granted, and a printed ability it didn't
//!   have then isn't.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Characteristics;
use crate::types::*;

/// `Effect::Custom`: "If that's another creature, it also gains the non-backup abilities of
/// this creature printed below this one until end of turn."
pub const GRANT: &str = "backup:grant the abilities printed below it";

pub struct Backup;

/// The abilities the backup triggered ability `uid` of a creature with the
/// characteristics `chars` grants: the non-backup abilities printed below the backup
/// ability it comes from that the creature has.
pub fn granted_abilities(chars: &Characteristics, uid: u64) -> Vec<Ability> {
    // The keyword instance the triggered ability comes from, and which of the creature's
    // backup abilities that is.
    let Some(kw_uid) = crate::keyword_impls::derived_by_keyword(chars)
        .into_iter()
        .find(|(_, d)| d.uid == uid)
        .map(|(k, _)| k)
    else {
        return vec![];
    };
    let is_backup = |a: &AbilityDef| {
        matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == KeywordKind::Backup)
    };
    let Some(nth) = chars
        .abilities
        .iter()
        .filter(|a| is_backup(a))
        .position(|a| a.uid == kw_uid)
    else {
        return vec![];
    };
    let Some(printed) = &chars.printed else {
        return vec![];
    };
    let def = &printed.0;
    let face = def
        .faces
        .iter()
        .find(|f| f.chars.name == chars.name)
        .unwrap_or_else(|| def.front());
    let abilities = &face.chars.abilities;
    let Some(at) = abilities
        .iter()
        .enumerate()
        .filter(|(_, a)| is_backup(a))
        .nth(nth)
        .map(|(i, _)| i)
    else {
        return vec![];
    };
    let has = |a: &AbilityDef| {
        chars
            .abilities
            .iter()
            .any(|b| b.uid == a.uid || b.text.eq_ignore_ascii_case(&a.text))
    };
    abilities[at + 1..]
        .iter()
        .filter(|a| !is_backup(a) && has(a))
        .cloned()
        .collect()
}

impl KeywordRules for Backup {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Backup]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::simple(
                    vec![TargetSpec::object(Filter::creature(), "target creature")],
                    Effect::Seq(vec![
                        Effect::AddCounters {
                            what: Sel::Target(0),
                            kind: counters::PLUS1.into(),
                            n: Value::c(n),
                        },
                        Effect::Custom(GRANT.into()),
                    ]),
                ),
            )),
            KeywordKind::Backup.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != GRANT {
            return false;
        }
        let Some(Entity::Object(target)) = ctx.targets.first().and_then(|v| v.first()).copied()
        else {
            return true;
        };
        // Only another creature gains the abilities.
        if ctx.source.is_some_and(|s| g.current(s) == g.current(target)) {
            return true;
        }
        let chars = match (&ctx.source_lki, ctx.source) {
            (Some(c), _) => (**c).clone(),
            (None, Some(s)) => g.obj(s).chars.clone(),
            (None, None) => return true,
        };
        let granted = granted_abilities(&chars, ctx.ability_uid);
        if granted.is_empty() {
            return true;
        }
        g.exec(
            &Effect::Modify {
                what: Sel::Target(0),
                mods: granted.into_iter().map(Modification::AddAbility).collect(),
                duration: Duration::EndOfTurn,
            },
            ctx,
        );
        true
    }
}

inventory::submit! { KeywordRegistration(&Backup) }
