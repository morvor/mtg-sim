//! Double-faced cards (CR 712): "As this permanent transforms into [this face], [effect]"
//! is applied while it transforms into that face (CR 712.20).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn s_as_transforms(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let r = ["as ~ ", "as this creature ", "as this permanent "]
        .iter()
        .find_map(|p| l.strip_prefix(p))?;
    let r = r.strip_prefix("transforms into ~, ")?;
    let mut b = Builder::new(ctx);
    b.it = Sel::This;
    let e = parse_clause(r, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Transforms,
                action: ReplacementAction::AsEnters(Box::new(e)),
                self_replacement: true,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r712 as this transforms", priority: 0, parse: s_as_transforms } }
