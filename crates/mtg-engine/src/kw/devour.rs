//! CR 702.82 Devour: "Devour N" means "As this object enters, you may sacrifice any number
//! of creatures. This permanent enters with N +1/+1 counters on it for each creature
//! sacrificed this way." (CR 702.82a). "Devour [quality] N" sacrifices [quality]
//! permanents instead (CR 702.82c; the quality is the keyword's filter). "It devoured"
//! means "sacrificed as a result of its devour ability as it entered" (CR 702.82b): the
//! devoured permanents (as they last existed) are linked to the permanent under
//! [`DEVOUR_LINK`].
//!
//! "Devour X, where X is the number of creatures devoured this way" (Thromok the
//! Insatiable) has N = -1: it enters with X +1/+1 counters for each of the X devoured.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The link under which a permanent keeps the permanents it devoured.
pub const DEVOUR_LINK: u16 = 0x7ffd;
/// `Effect::Custom` (inside the "as enters" replacement): "you may sacrifice any number of
/// the candidates in [`CANDIDATES`]; it enters with N counters for each", followed by N.
const DEVOUR: &str = "devour:";
/// `Effect::Custom` performed as the permanent enters: links the permanents it devoured
/// (in [`DEVOURED`]) to it.
const RECORD: &str = "devour:record devoured";
/// `Value::Custom`: the number of permanents the source devoured ("for each creature it
/// devoured").
pub const DEVOURED_COUNT: &str = "devour:number devoured";
/// `Value::Custom` prefix: the number of permanents of a subtype the source devoured ("the
/// number of Goblins it devoured"), followed by the subtype.
pub const DEVOURED_OF_TYPE: &str = "devour:number devoured of type:";

const CANDIDATES: Var = vars::USER + 71;
const DEVOURED: Var = vars::USER + 72;

pub struct Devour;

impl KeywordRules for Devour {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Devour]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        let quality = kw.filter.clone().unwrap_or_else(Filter::creature);
        // What can be devoured: [quality] permanents its controller controls. (It can't
        // devour itself or anything entering at the same time, see `custom_effect`.)
        let candidates = Filter::and(vec![
            quality,
            Filter::Permanent,
            Filter::ControlledBy(PlayerRel::You),
        ]);
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Seq(vec![
                Effect::Store {
                    var: CANDIDATES,
                    sel: Sel::All(candidates),
                },
                Effect::Custom(SmolStr::new(format!("{DEVOUR}{n}"))),
            ]))),
            self_replacement: false,
            optional: false,
        }));
        let text = kw
            .text
            .clone()
            .map(|t| t.to_string())
            .unwrap_or_else(|| format!("Devour {n}"));
        Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name == RECORD {
            if let Some(this) = ctx.source {
                let devoured = ctx.var_objects(DEVOURED);
                g.objects[this.0 as usize]
                    .linked
                    .entry(DEVOUR_LINK)
                    .or_default()
                    .extend(devoured);
            }
            return true;
        }
        let Some(n) = name.strip_prefix(DEVOUR) else {
            return false;
        };
        let Ok(n) = n.parse::<i32>() else {
            return false;
        };
        let this = ctx.source;
        let p = ctx.controller;
        let cands: Vec<ObjectId> = ctx
            .var_objects(CANDIDATES)
            .into_iter()
            .filter(|c| Some(*c) != this && !g.entering.contains(c))
            .collect();
        let chosen = if cands.is_empty() {
            vec![]
        } else {
            let max = cands.len() as u32;
            g.ask_objects(
                p,
                this,
                "Sacrifice any number of permanents (devour)",
                cands,
                0,
                max,
            )
        };
        let what: Vec<(ObjectId, PlayerId)> = chosen.iter().map(|o| (*o, p)).collect();
        let sacrificed: Vec<ObjectId> = g
            .sacrifice_simultaneously(&what)
            .into_iter()
            .map(|(old, _)| old)
            .collect();
        let k = sacrificed.len() as u32;
        // "Devour X, where X is the number of creatures devoured this way".
        let per = if n < 0 { k } else { n as u32 };
        let total = per * k;
        ctx.set_var(
            DEVOURED,
            sacrificed.iter().map(|o| Entity::Object(*o)).collect(),
        );
        if let Some(em) = ctx.entering.as_mut() {
            if total > 0 {
                em.counters.push((SmolStr::new(counters::PLUS1), total));
            }
            em.on_entry.push(Effect::Custom(RECORD.into()));
        }
        true
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let devoured = |g: &Game| -> Vec<ObjectId> {
            ctx.source
                .and_then(|s| g.obj(s).linked.get(&DEVOUR_LINK).cloned())
                .unwrap_or_default()
        };
        if name == DEVOURED_COUNT {
            return Some(devoured(g).len() as i64);
        }
        let ty = name.strip_prefix(DEVOURED_OF_TYPE)?;
        let f = Filter::Subtype(SmolStr::new(ty));
        Some(
            devoured(g)
                .into_iter()
                .filter(|o| g.matches(*o, &f, ctx))
                .count() as i64,
        )
    }
}

inventory::submit! { KeywordRegistration(&Devour) }
