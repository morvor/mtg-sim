//! Filters on a creature's base power and toughness (CR 208.4b): "with base power and
//! toughness 2/2" (Duskana, the Rage Mother), "with base power or toughness 1" (Sword of
//! the Squeak), "each with base power or toughness 1 or less" (Angelic Aberration).
//!
//! Base power and toughness are the values after characteristic-defining abilities and
//! effects that set them (layers 7a and 7b), ignoring effects and counters that modify
//! them (`GameObject::base_pt`). A creature whose printed power and toughness are 0/0 and
//! that gets a bonus from its own ability (which isn't a characteristic-defining ability)
//! has base power and toughness 0/0.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Cmp, Filter};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom` prefix: "base:" then `p` (power) or `t` (toughness), a comparison
/// (`=`, `<=`, `>=`) and a number ("base:t<=1").
const PREFIX: &str = "base:";

/// A filter comparing base power (`power`) or base toughness with `n`.
pub fn base_filter(power: bool, cmp: Cmp, n: i32) -> Filter {
    let op = match cmp {
        Cmp::Le => "<=",
        Cmp::Ge => ">=",
        _ => "=",
    };
    let stat = if power { 'p' } else { 't' };
    Filter::Custom(SmolStr::new(format!("{PREFIX}{stat}{op}{n}")))
}

pub struct BasePt;

impl KeywordRules for BasePt {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let r = name.strip_prefix(PREFIX)?;
        let power = match r.chars().next()? {
            'p' => true,
            't' => false,
            _ => return None,
        };
        let r = &r[1..];
        let (cmp, n) = if let Some(n) = r.strip_prefix("<=") {
            (Cmp::Le, n)
        } else if let Some(n) = r.strip_prefix(">=") {
            (Cmp::Ge, n)
        } else {
            (Cmp::Eq, r.strip_prefix('=')?)
        };
        let n: i64 = n.parse().ok()?;
        let (p, t) = g.obj(id).base_pt;
        let v = if power { p } else { t };
        Some(v.is_some_and(|v| cmp.eval(v as i64, n)))
    }
}

inventory::submit! { KeywordRegistration(&BasePt) }
