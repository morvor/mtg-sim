//! Oracle patterns for converting (CR 701.28): "convert ~", "convert it", and "[objects]
//! can't transform" (which also stops them from converting, CR 701.28f).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "convert ~" / "convert it".
fn convert(l: &str, b: &mut Builder) -> Option<Effect> {
    let what = match end(l).strip_prefix("convert ")? {
        "~" => Sel::This,
        "it" => b.it.clone(),
        _ => return None,
    };
    Some(Effect::KeywordAction {
        action: KeywordAction::Convert,
        who: PlayerRef::You,
        what,
        n: Value::c(1),
    })
}

inventory::submit! { EffectPattern { name: "a701 convert", priority: 60, parse: convert } }

/// "[objects] can't transform" / "~ can't transform".
fn cant_transform(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let subject = end(l).strip_suffix(" can't transform")?;
    let f = if subject == "~" {
        Filter::Source
    } else {
        let (f, plural, tail) = parse_object_phrase(subject)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        f
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
            Restriction::CantTransform(f),
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 can't transform", priority: 60, parse: cant_transform } }
