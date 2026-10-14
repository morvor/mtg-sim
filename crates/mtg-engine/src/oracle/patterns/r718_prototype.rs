//! Prototype cards (CR 718, 702.160): "Prototype [mana cost] — [power]/[toughness]".

use super::AbilityPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

fn prototype(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.trim().strip_prefix("Prototype ")?;
    let (cost, pt) = r.split_once(" — ")?;
    let mana = ManaCost::parse(cost.trim())?;
    let (p, t) = pt.trim().split_once('/')?;
    let (p, t): (i32, i32) = (p.trim().parse().ok()?, t.trim().parse().ok()?);
    let mut kw = Keyword::new(KeywordKind::Prototype);
    kw.cost = Some(Cost::mana(mana));
    kw.text = Some(SmolStr::new(format!("{p}/{t}")));
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), block.trim())])
}

inventory::submit! { AbilityPattern { name: "r718 prototype", priority: 70, parse: prototype } }
