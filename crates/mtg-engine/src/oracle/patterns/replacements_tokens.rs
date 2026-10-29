//! "If you would create one or more Treasure tokens, instead create those tokens plus an
//! additional Treasure token." (Xorn; CR 614.1a, 111.1): the creation event is replaced
//! with one that creates one more token of that kind. Several such effects each add one
//! (Xorn ruling), and each applies once to the event (CR 614.5).
//!
//! Only the same kind of token is accepted: "plus an additional Food token" after "one or
//! more tokens" creates a different token, which isn't the same event.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn s_plus_additional(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if you would create one or more ")?;
    let (kind, r) = r.split_once(" tokens, instead create those tokens plus an additional ")?;
    if r.strip_suffix(" token")? != kind {
        return None;
    }
    let (tokens, plural, tail) = parse_object_phrase(kind)?;
    if plural || !end(tail).is_empty() || matches!(tokens, Filter::Any) {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::CreateTokensMatching {
                    who: PlayerFilter::You,
                    tokens,
                },
                action: ReplacementAction::Add(Value::Const(1)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements: create those tokens plus an additional one of that kind", priority: 70, parse: s_plus_additional } }
